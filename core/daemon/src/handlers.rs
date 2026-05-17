use std::sync::Arc;
use tracing::{info, error};
use uuid::Uuid;

use agentos_ipc::{BridgeCodec, SessionFilter, IpcMessage};
use daemon_core::state::AgentSession;

use crate::server::DaemonState;

pub async fn handle_get_sessions(codec: &mut BridgeCodec, id: Uuid, filter: Option<SessionFilter>, state: &Arc<DaemonState>) {
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

pub async fn handle_get_session(codec: &mut BridgeCodec, id: Uuid, session_id: String, state: &Arc<DaemonState>) {
    let session_state = state.session_state.read().await;
    let data = session_state
        .sessions
        .get(&session_id)
        .map(|s| serde_json::to_value(s).unwrap_or_default());
    let _ = codec.send(&IpcMessage::new_response(id, data)).await;
}

pub async fn handle_ping(codec: &mut BridgeCodec, id: Uuid) {
    let _ = codec
        .send(&IpcMessage::new_response(
            id,
            Some(serde_json::json!({"pong": true})),
        ))
        .await;
}

pub async fn handle_discover_agents(codec: &mut BridgeCodec, id: Uuid) {
    let result = crate::discover::discover_agents();
    let data = serde_json::to_value(&result).unwrap_or_default();
    let _ = codec.send(&IpcMessage::new_response(id, Some(data))).await;
}

pub async fn handle_shutdown(codec: &mut BridgeCodec, id: Uuid) {
    info!("shutdown requested via IPC");
    let _ = codec.send(&IpcMessage::new_response(id, None)).await;
    std::process::exit(0);
}

pub async fn handle_get_session_stats(codec: &mut BridgeCodec, id: Uuid, state: &Arc<DaemonState>) {
    match state.db.as_ref() {
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
    }
}

pub async fn handle_get_agent_analytics(codec: &mut BridgeCodec, id: Uuid, state: &Arc<DaemonState>) {
    match state.db.as_ref() {
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
    }
}

pub async fn handle_get_timeline(codec: &mut BridgeCodec, id: Uuid, limit: u32, state: &Arc<DaemonState>) {
    match state.db.as_ref() {
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
    }
}

pub async fn handle_search_sessions(codec: &mut BridgeCodec, id: Uuid, query: String, state: &Arc<DaemonState>) {
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

pub async fn handle_jump_to_session(codec: &mut BridgeCodec, id: Uuid, session_id: String, state: &Arc<DaemonState>) {
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

pub async fn handle_resolve_permission(codec: &mut BridgeCodec, id: Uuid, permission_id: Uuid, approved: bool, state: &Arc<DaemonState>) {
    let session_id = {
        let session_state = state.session_state.read().await;
        session_state
            .sessions
            .iter()
            .find(|(_, s)| {
                s.permission
                    .as_ref()
                    .is_some_and(|p| p.id == permission_id)
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
                pid: None,
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

pub async fn handle_answer_question(codec: &mut BridgeCodec, id: Uuid, question_id: Uuid, answer: String, state: &Arc<DaemonState>) {
    let session_id = {
        let session_state = state.session_state.read().await;
        session_state
            .sessions
            .iter()
            .find(|(_, s)| {
                s.question
                    .as_ref()
                    .is_some_and(|q| q.id == question_id)
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
                pid: None,
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

pub async fn handle_stop_agent(codec: &mut BridgeCodec, id: Uuid, session_id: String, state: &Arc<DaemonState>) {
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
            pid: None,
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
