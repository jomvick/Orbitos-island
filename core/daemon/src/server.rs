use std::sync::Arc;

use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use agentos_ipc::{BridgeCodec, IpcCommand, IpcMessage, MAX_MESSAGE_SIZE};
use agentos_storage::Database;
use daemon_core::agents::AgentRegistry;
use daemon_core::events::EventBus;
use daemon_core::state::{apply_event, SessionState, UniversalEvent};

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
                        let session = {
                            let session_state = state.session_state.read().await;
                            session_state.sessions.get(&event.session_id).cloned()
                        };
                        let msg = IpcMessage::SubscriptionEvent {
                            channel: "sessions".to_string(),
                            event: Box::new((*event).clone()),
                            session: session.map(Box::new),
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
            crate::handlers::handle_get_sessions(codec, id, filter, state).await;
        }
        IpcCommand::GetSession { session_id } => {
            crate::handlers::handle_get_session(codec, id, session_id, state).await;
        }
        IpcCommand::Ping => {
            crate::handlers::handle_ping(codec, id).await;
        }
        IpcCommand::DiscoverAgents => {
            crate::handlers::handle_discover_agents(codec, id).await;
        }
        IpcCommand::Shutdown => {
            crate::handlers::handle_shutdown(codec, id).await;
        }
        IpcCommand::GetSessionStats => {
            crate::handlers::handle_get_session_stats(codec, id, state).await;
        }
        IpcCommand::GetAgentAnalytics => {
            crate::handlers::handle_get_agent_analytics(codec, id, state).await;
        }
        IpcCommand::GetTimeline { limit } => {
            crate::handlers::handle_get_timeline(codec, id, limit, state).await;
        }
        IpcCommand::SearchSessions { query } => {
            crate::handlers::handle_search_sessions(codec, id, query, state).await;
        }
        IpcCommand::JumpToSession { session_id } => {
            crate::handlers::handle_jump_to_session(codec, id, session_id, state).await;
        }
        IpcCommand::ResolvePermission {
            permission_id,
            approved,
            ..
        } => {
            crate::handlers::handle_resolve_permission(codec, id, permission_id, approved, state).await;
        }
        IpcCommand::AnswerQuestion {
            question_id,
            answer,
        } => {
            crate::handlers::handle_answer_question(codec, id, question_id, answer, state).await;
        }
        IpcCommand::StopAgent { session_id } => {
            crate::handlers::handle_stop_agent(codec, id, session_id, state).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use tokio::net::UnixStream;
    use tokio::sync::Mutex;

    use agentos_ipc::{BridgeCodec, IpcCommand, IpcMessage, IpcServer, IpcStatus, SocketConfig};
    use agentos_storage::Database;

    use crate::plugin_loader;

    use super::DaemonState;

    #[tokio::test]
    async fn test_e2e_hook_to_session_via_ipc() {
        let dir = std::env::temp_dir().join(format!("agentosd-e2e-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let socket_path = dir.join("e2e.sock");
        let _ = std::fs::remove_file(&socket_path);

        let db = Database::open_in_memory().unwrap();
        let plugin_registry = plugin_loader::load_default_plugins();
        let db_arc = Arc::new(Mutex::new(db));
        let state = Arc::new(DaemonState::new(plugin_registry, Some(db_arc)));

        let config = SocketConfig {
            path: socket_path.clone(),
            max_connections: 8,
        };
        let server = IpcServer::bind(config).unwrap();
        let server_path = server.local_path().to_path_buf();
        let state_clone = state.clone();

        tokio::spawn(async move {
            while let Ok((codec, _fd)) = server.accept().await {
                let s = state_clone.clone();
                tokio::spawn(async move {
                    super::handle_client(codec, s).await;
                });
            }
        });

        tokio::time::sleep(Duration::from_millis(100)).await;

        let stream = UnixStream::connect(&server_path).await.unwrap();
        let (read, write) = stream.into_split();
        let mut client = BridgeCodec::new(read, write);

        let payload = serde_json::json!({
            "type": "session_start",
            "session_id": "e2e-test-session",
            "model": "gpt-4",
            "cwd": "/home/user/project"
        });
        client
            .send(&IpcMessage::new_event("opencode", payload))
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(200)).await;

        let cmd = IpcMessage::new_command(IpcCommand::GetSessions { filter: None });
        client.send(&cmd).await.unwrap();

        let response = loop {
            match client.recv().await.unwrap() {
                IpcMessage::SubscriptionEvent { .. } => continue,
                other => break other,
            }
        };
        match response {
            IpcMessage::Response { status, data, .. } => {
                assert_eq!(status, IpcStatus::Ok);
                let sessions: Vec<serde_json::Value> =
                    serde_json::from_value(data.unwrap()).unwrap();
                assert_eq!(sessions.len(), 1, "expected 1 session");
                assert_eq!(sessions[0]["id"], "e2e-test-session");
                assert_eq!(sessions[0]["agent"], "opencode");
                assert_eq!(sessions[0]["phase"], "running");
                assert_eq!(sessions[0]["model"], "gpt-4");
            }
            other => panic!("expected Response, got {:?}", other),
        }

        let _ = std::fs::remove_file(&socket_path);
    }

    /// 🔴 Critical: The daemon must not crash when it receives garbage over the socket.
    /// It should log the error and continue responding to subsequent valid commands.
    #[tokio::test]
    async fn test_malformed_json_does_not_crash_server() {
        use tokio::io::AsyncWriteExt;

        let dir = std::env::temp_dir()
            .join(format!("agentosd-malformed-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let socket_path = dir.join("malformed.sock");
        let _ = std::fs::remove_file(&socket_path);

        let db = Database::open_in_memory().unwrap();
        let plugin_registry = plugin_loader::load_default_plugins();
        let state = Arc::new(DaemonState::new(plugin_registry, Some(Arc::new(Mutex::new(db)))));

        let config = SocketConfig {
            path: socket_path.clone(),
            max_connections: 4,
        };
        let server = IpcServer::bind(config).unwrap();
        let server_path = server.local_path().to_path_buf();
        let state_clone = state.clone();

        tokio::spawn(async move {
            while let Ok((codec, _fd)) = server.accept().await {
                let s = state_clone.clone();
                tokio::spawn(async move { super::handle_client(codec, s).await });
            }
        });

        tokio::time::sleep(Duration::from_millis(50)).await;

        // Connect a raw stream to inject garbage bytes directly.
        let stream = tokio::net::UnixStream::connect(&server_path).await.unwrap();
        let (_read_half, mut write_half) = stream.into_split();

        // Send malformed JSON — this must not crash the server.
        write_half.write_all(b"not json at all\n").await.unwrap();
        write_half.flush().await.unwrap();

        // Wait briefly for the server to process and drop/ignore the bad message.
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Now open a fresh connection and send a valid Ping — the server must still respond.
        let stream2 = tokio::net::UnixStream::connect(&server_path).await.unwrap();
        let (read2, write2) = stream2.into_split();
        let mut client2 = BridgeCodec::new(read2, write2);

        let ping_msg = IpcMessage::new_command(IpcCommand::Ping);
        client2.send(&ping_msg).await.unwrap();

        let response = tokio::time::timeout(Duration::from_secs(2), async {
            loop {
                match client2.recv().await.unwrap() {
                    IpcMessage::SubscriptionEvent { .. } => continue,
                    other => break other,
                }
            }
        })
        .await
        .expect("server did not respond to Ping after receiving malformed JSON");

        match response {
            IpcMessage::Response { status, .. } => {
                assert_eq!(status, IpcStatus::Ok, "Ping should return Ok");
            }
            other => panic!("expected Ping Response, got {:?}", other),
        }

        let _ = std::fs::remove_file(&socket_path);
    }
}
