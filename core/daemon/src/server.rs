use std::sync::Arc;

use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use agentos_ipc::{BridgeCodec, IpcCommand, IpcMessage, SessionFilter, MAX_MESSAGE_SIZE};
use agentos_storage::Database;
use daemon_core::agents::AgentRegistry;
use daemon_core::events::EventBus;
use daemon_core::state::{apply_event, AgentSession, SessionState, UniversalEvent};

pub struct DaemonState {
    pub event_bus: EventBus,
    pub session_state: RwLock<SessionState>,
    pub plugin_registry: AgentRegistry,
    pub subscribers: RwLock<Vec<Uuid>>,
    pub db: Option<Arc<Mutex<Database>>>,
}

impl DaemonState {
    pub fn new(
        plugin_registry: AgentRegistry,
        db: Option<Arc<Mutex<Database>>>,
    ) -> Self {
        Self {
            event_bus: EventBus::new(),
            session_state: RwLock::new(SessionState::new()),
            plugin_registry,
            subscribers: RwLock::new(Vec::new()),
            db,
        }
    }

    pub async fn process_event(&self, source: &str, payload: serde_json::Value) {
        let payload_str = payload.to_string();
        if payload_str.len() > MAX_MESSAGE_SIZE {
            warn!(source = %source, size = %payload_str.len(), "event payload too large");
            return;
        }

        let agent_kind: daemon_core::state::AgentKind = match source.parse() {
            Ok(a) => a,
            Err(_) => {
                warn!(source = %source, "unknown agent source");
                return;
            }
        };

        let event_result = self
            .plugin_registry
            .process(&agent_kind, &payload.to_string());

        let event: UniversalEvent = match event_result {
            Ok(Some(event)) => event,
            Ok(None) => return,
            Err(e) => {
                warn!(source = %source, error = %e, "plugin rejected event");
                return;
            }
        };

        let arc_event = Arc::new(event);
        self.event_bus.publish(Arc::clone(&arc_event)).unwrap_or_else(|e| {
            error!("event bus full: {:?}", e);
            0
        });

        let mut session_state = self.session_state.write().await;
        *session_state = apply_event(session_state.clone(), (*arc_event).clone());
    }
}

pub async fn handle_client(mut codec: BridgeCodec, state: Arc<DaemonState>) {
    let client_id = Uuid::new_v4();
    let mut event_subscriber = state.event_bus.subscribe();

    info!(client = %client_id, "client connected");

    loop {
        tokio::select! {
            msg = codec.recv() => {
                match msg {
                    Ok(IpcMessage::Event { source, payload, .. }) => {
                        state.process_event(&source, payload).await;
                    }

                    Ok(IpcMessage::Command { id, command, .. }) => {
                        handle_command(&mut codec, id, command, &state).await;
                    }

                    Ok(IpcMessage::Subscribe { .. }) => {
                        let mut subs = state.subscribers.write().await;
                        subs.push(client_id);
                        info!(client = %client_id, "client subscribed");
                    }

                    Err(e) => {
                        debug!(client = %client_id, error = %e, "client disconnected");
                        break;
                    }

                    _ => {}
                }
            }

            event = event_subscriber.recv() => {
                match event {
                    Ok(event) => {
                        let msg = IpcMessage::SubscriptionEvent {
                            channel: "sessions".to_string(),
                            event: Box::new((*event).clone()),
                            timestamp: chrono::Utc::now(),
                        };
                        if let Err(e) = codec.send(&msg).await {
                            debug!(client = %client_id, error = %e, "failed to send event");
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        debug!(client = %client_id, lag = %n, "subscriber lagged");
                    }
                    Err(_) => break,
                }
            }
        }
    }

    {
        let mut subs = state.subscribers.write().await;
        subs.retain(|id| *id != client_id);
    }

    info!(client = %client_id, "client disconnected");
}

async fn handle_command(
    codec: &mut BridgeCodec,
    id: Uuid,
    command: IpcCommand,
    state: &Arc<DaemonState>,
) {
    info!(command = ?command, id = %id, "received command");
    match command {
        IpcCommand::GetSessions { filter } => {
            let session_state = state.session_state.read().await;
            let sessions: Vec<AgentSession> = match filter {
                Some(SessionFilter::Active) => session_state
                    .sessions
                    .values()
                    .filter(|s| s.is_active())
                    .cloned()
                    .collect(),
                Some(SessionFilter::ByAgent(agent)) => session_state
                    .sessions
                    .values()
                    .filter(|s| s.agent == agent)
                    .cloned()
                    .collect(),
                _ => session_state.sessions.values().cloned().collect(),
            };
            let data = serde_json::to_value(&sessions).unwrap_or_default();
            let _ = codec.send(&IpcMessage::new_response(id, Some(data))).await;
        }

        IpcCommand::GetSession { session_id } => {
            let session_state = state.session_state.read().await;
            let data = session_state
                .sessions
                .get(&session_id)
                .map(|s| serde_json::to_value(s).unwrap_or_default());
            let _ = codec.send(&IpcMessage::new_response(id, data)).await;
        }

        IpcCommand::Ping => {
            let _ = codec
                .send(&IpcMessage::new_response(
                    id,
                    Some(serde_json::json!({"pong": true})),
                ))
                .await;
        }

        IpcCommand::DiscoverAgents => {
            let result = crate::discover::discover_agents();
            let data = serde_json::to_value(&result).unwrap_or_default();
            let _ = codec.send(&IpcMessage::new_response(id, Some(data))).await;
        }

        IpcCommand::Shutdown => {
            info!("shutdown requested via IPC");
            let _ = codec.send(&IpcMessage::new_response(id, None)).await;
            std::process::exit(0);
        }

        IpcCommand::GetSessionStats => match state.db.as_ref() {
            Some(db) => {
                let data = db
                    .lock()
                    .await
                    .get_session_stats()
                    .ok()
                    .and_then(|s| serde_json::to_value(&s).ok());
                match data {
                    Some(v) => {
                        let _ = codec.send(&IpcMessage::new_response(id, Some(v))).await;
                    }
                    None => {
                        let _ = codec
                            .send(&IpcMessage::new_error(id, "query failed".to_string()))
                            .await;
                    }
                }
            }
            None => {
                let _ = codec
                    .send(&IpcMessage::new_error(id, "no database".to_string()))
                    .await;
            }
        },

        IpcCommand::GetAgentAnalytics => match state.db.as_ref() {
            Some(db) => {
                let data = db
                    .lock()
                    .await
                    .get_agent_analytics()
                    .ok()
                    .and_then(|a| serde_json::to_value(&a).ok());
                match data {
                    Some(v) => {
                        let _ = codec.send(&IpcMessage::new_response(id, Some(v))).await;
                    }
                    None => {
                        let _ = codec
                            .send(&IpcMessage::new_error(id, "query failed".to_string()))
                            .await;
                    }
                }
            }
            None => {
                let _ = codec
                    .send(&IpcMessage::new_error(id, "no database".to_string()))
                    .await;
            }
        },

        IpcCommand::GetTimeline { limit } => match state.db.as_ref() {
            Some(db) => {
                let data = db
                    .lock()
                    .await
                    .get_timeline(limit)
                    .ok()
                    .and_then(|e| serde_json::to_value(&e).ok());
                match data {
                    Some(v) => {
                        let _ = codec.send(&IpcMessage::new_response(id, Some(v))).await;
                    }
                    None => {
                        let _ = codec
                            .send(&IpcMessage::new_error(id, "query failed".to_string()))
                            .await;
                    }
                }
            }
            None => {
                let _ = codec
                    .send(&IpcMessage::new_error(id, "no database".to_string()))
                    .await;
            }
        },

        IpcCommand::SearchSessions { query } => {
            let result = match state.db.as_ref() {
                Some(db) => db
                    .lock()
                    .await
                    .search_sessions(&query)
                    .ok()
                    .and_then(|sessions| {
                        let domain: Vec<AgentSession> = sessions
                            .into_iter()
                            .filter_map(|s| s.to_domain().ok())
                            .collect();
                        serde_json::to_value(&domain).ok()
                    }),
                None => None,
            };
            match result {
                Some(data) => {
                    let _ = codec.send(&IpcMessage::new_response(id, Some(data))).await;
                }
                None => {
                    let _ = codec
                        .send(&IpcMessage::new_error(id, "no database".to_string()))
                        .await;
                }
            }
        }

        IpcCommand::JumpToSession { session_id } => {
            let session_state = state.session_state.read().await;
            let session = session_state.sessions.get(&session_id);

            match session {
                Some(s) => {
                    let terminal = s.terminal.as_deref();
                    let pid = s.jump_target.as_ref().and_then(|j| j.pid);
                    let cwd = s.jump_target.as_ref().and_then(|j| j.cwd.as_deref());

                    match daemon_core::terminals::detector::resolve_jump_target(terminal, pid, cwd)
                    {
                        Ok(Some(pane_id)) => {
                            match daemon_core::terminals::detector::focus_terminal(
                                &pane_id, terminal,
                            ) {
                                Ok(_) => {
                                    let _ = codec
                                        .send(&IpcMessage::new_response(
                                            id,
                                            Some(serde_json::json!({"pane_id": pane_id, "status": "focused"})),
                                        ))
                                        .await;
                                }
                                Err(e) => {
                                    let _ = codec
                                        .send(&IpcMessage::new_error(
                                            id,
                                            format!("focus failed: {}", e),
                                        ))
                                        .await;
                                }
                            }
                        }
                        Ok(None) => {
                            let _ = codec
                                .send(&IpcMessage::new_error(
                                    id,
                                    "terminal pane not found".to_string(),
                                ))
                                .await;
                        }
                        Err(e) => {
                            let _ = codec
                                .send(&IpcMessage::new_error(id, format!("terminal error: {}", e)))
                                .await;
                        }
                    }
                }
                None => {
                    let _ = codec
                        .send(&IpcMessage::new_error(
                            id,
                            format!("session not found: {}", session_id),
                        ))
                        .await;
                }
            }
        }

        IpcCommand::ResolvePermission {
            permission_id,
            approved,
            ..
        } => {
            let session_id = {
                let session_state = state.session_state.read().await;
                session_state
                    .sessions
                    .iter()
                    .find(|(_, s)| {
                        s.permission
                            .as_ref()
                            .map_or(false, |p| p.id == permission_id)
                    })
                    .map(|(id, _)| id.clone())
            };

            match session_id {
                Some(sid) => {
                    let event = daemon_core::state::UniversalEvent {
                        id: Uuid::new_v4(),
                        agent: daemon_core::state::AgentKind::Custom("agentos".into()),
                        event: daemon_core::state::EventKind::ActionableStateResolved,
                        session_id: sid.clone(),
                        cwd: None,
                        branch: None,
                        model: None,
                        tokens_input: None,
                        tokens_output: None,
                        duration_ms: None,
                        terminal: None,
                        pane: None,
                        permission: None,
                        question: None,
                        jump_target: None,
                        plan: None,
                        diff: None,
                        error: None,
                        metadata: Some(serde_json::json!({
                            "resolved_by": "user",
                            "approved": approved
                        })),
                        timestamp: chrono::Utc::now(),
                    };

                    let arc_event = Arc::new(event);
                    state
                        .event_bus
                        .publish(Arc::clone(&arc_event))
                        .unwrap_or_else(|e| {
                            error!("event bus full: {:?}", e);
                            0
                        });

                    let mut session_state = state.session_state.write().await;
                    *session_state =
                        daemon_core::state::apply_event(session_state.clone(), (*arc_event).clone());

                    let _ = codec
                        .send(&IpcMessage::new_response(
                            id,
                            Some(serde_json::json!({"status": "resolved", "session_id": sid})),
                        ))
                        .await;
                }
                None => {
                    let _ = codec
                        .send(&IpcMessage::new_error(
                            id,
                            "permission not found".to_string(),
                        ))
                        .await;
                }
            }
        }

        IpcCommand::AnswerQuestion {
            question_id,
            answer,
        } => {
            let session_id = {
                let session_state = state.session_state.read().await;
                session_state
                    .sessions
                    .iter()
                    .find(|(_, s)| {
                        s.question
                            .as_ref()
                            .map_or(false, |q| q.id == question_id)
                    })
                    .map(|(id, _)| id.clone())
            };

            match session_id {
                Some(sid) => {
                    let event = daemon_core::state::UniversalEvent {
                        id: Uuid::new_v4(),
                        agent: daemon_core::state::AgentKind::Custom("agentos".into()),
                        event: daemon_core::state::EventKind::ActionableStateResolved,
                        session_id: sid.clone(),
                        cwd: None,
                        branch: None,
                        model: None,
                        tokens_input: None,
                        tokens_output: None,
                        duration_ms: None,
                        terminal: None,
                        pane: None,
                        permission: None,
                        question: None,
                        jump_target: None,
                        plan: None,
                        diff: None,
                        error: None,
                        metadata: Some(serde_json::json!({
                            "resolved_by": "user",
                            "answer": answer
                        })),
                        timestamp: chrono::Utc::now(),
                    };

                    let arc_event = Arc::new(event);
                    state
                        .event_bus
                        .publish(Arc::clone(&arc_event))
                        .unwrap_or_else(|e| {
                            error!("event bus full: {:?}", e);
                            0
                        });

                    let mut session_state = state.session_state.write().await;
                    *session_state =
                        daemon_core::state::apply_event(session_state.clone(), (*arc_event).clone());

                    let _ = codec
                        .send(&IpcMessage::new_response(
                            id,
                            Some(serde_json::json!({"status": "resolved", "session_id": sid})),
                        ))
                        .await;
                }
                None => {
                    let _ = codec
                        .send(&IpcMessage::new_error(
                            id,
                            "permission not found".to_string(),
                        ))
                        .await;
                }
            }
        }

        IpcCommand::StopAgent { session_id } => {
            let exists = {
                let session_state = state.session_state.read().await;
                session_state.sessions.contains_key(&session_id)
            };

            if exists {
                let event = daemon_core::state::UniversalEvent {
                    id: Uuid::new_v4(),
                    agent: daemon_core::state::AgentKind::Custom("agentos".into()),
                    event: daemon_core::state::EventKind::SessionFailed,
                    session_id: session_id.clone(),
                    cwd: None,
                    branch: None,
                    model: None,
                    tokens_input: None,
                    tokens_output: None,
                    duration_ms: None,
                    terminal: None,
                    pane: None,
                    permission: None,
                    question: None,
                    jump_target: None,
                    plan: None,
                    diff: None,
                    error: Some("stopped by user".to_string()),
                    metadata: Some(serde_json::json!({
                        "stopped_by": "user"
                    })),
                    timestamp: chrono::Utc::now(),
                };

                let arc_event = Arc::new(event);
                state
                    .event_bus
                    .publish(Arc::clone(&arc_event))
                    .unwrap_or_else(|e| {
                        error!("event bus full: {:?}", e);
                        0
                    });

                let mut session_state = state.session_state.write().await;
                *session_state =
                    daemon_core::state::apply_event(session_state.clone(), (*arc_event).clone());

                let _ = codec
                    .send(&IpcMessage::new_response(
                        id,
                        Some(serde_json::json!({"status": "stopped", "session_id": session_id})),
                    ))
                    .await;
            } else {
                let _ = codec
                    .send(&IpcMessage::new_error(
                        id,
                        format!("session not found: {}", session_id),
                    ))
                    .await;
            }
        }
    }
}
