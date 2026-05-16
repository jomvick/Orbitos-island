use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::net::UnixStream;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use uuid::Uuid;

fn get_socket_path() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    format!("{}/.agentosd.sock", home)
}

#[derive(Clone, Serialize)]
pub struct DaemonEvent {
    pub channel: String,
    pub data: serde_json::Value,
    pub timestamp: String,
}

#[allow(dead_code)]
pub struct DaemonClient {
    pub connected: bool,
}

pub async fn connect_and_listen(app: AppHandle) -> Result<(), String> {
    loop {
        let socket_path = get_socket_path();
        tracing::info!("attempting to connect to daemon at {}", socket_path);
        
        let stream_result = UnixStream::connect(&socket_path).await;
        
        let stream = match stream_result {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("failed to connect to daemon: {}. Retrying in 2s...", e);
                let _ = app.emit("daemon-disconnected", true);
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                continue;
            }
        };

        let (reader, mut writer) = stream.into_split();
        let mut reader = tokio::io::BufReader::new(reader);

        let subscribe_msg = serde_json::json!({
            "type": "subscribe",
            "channel": "sessions",
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        let mut buf = serde_json::to_vec(&subscribe_msg).map_err(|e| e.to_string())?;
        buf.push(b'\n');
        if let Err(e) = writer.write_all(&buf).await {
            tracing::error!("failed to send subscribe message: {}", e);
            continue;
        }
        let _ = writer.flush().await;

        tracing::info!("connected to daemon and subscribed to sessions");
        let _ = app.emit("daemon-connected", true);

        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    tracing::warn!("daemon connection closed by remote");
                    break;
                }
                Ok(_) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    match serde_json::from_str::<serde_json::Value>(trimmed) {
                        Ok(msg) => {
                            let event = DaemonEvent {
                                channel: msg
                                    .get("channel")
                                    .and_then(|c| c.as_str())
                                    .unwrap_or("unknown")
                                    .to_string(),
                                data: msg
                                    .get("event")
                                    .cloned()
                                    .unwrap_or(serde_json::Value::Null),
                                timestamp: chrono::Utc::now().to_rfc3339(),
                            };

                            let _ = app.emit("daemon-event", event);
                        }
                        Err(e) => {
                            tracing::warn!("failed to parse daemon message: {}", e);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("daemon read error: {}", e);
                    break;
                }
            }
        }

        tracing::info!("disconnected from daemon. Retrying in 2s...");
        let _ = app.emit("daemon-disconnected", true);
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
}

async fn send_command(command_action: serde_json::Value) -> Result<serde_json::Value, String> {
    let socket_path = get_socket_path();
    let stream = UnixStream::connect(&socket_path).await.map_err(|e| {
        let err = format!("daemon unreachable at {}: {}", socket_path, e);
        tracing::error!("{}", err);
        err
    })?;

    let (reader, mut writer) = stream.into_split();

    let command = serde_json::json!({
        "type": "command",
        "id": Uuid::new_v4(),
        "command": command_action,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    let mut buf = serde_json::to_vec(&command).map_err(|e| e.to_string())?;
    buf.push(b'\n');
    writer.write_all(&buf).await.map_err(|e| e.to_string())?;
    writer.flush().await.map_err(|e| e.to_string())?;

    let mut reader = tokio::io::BufReader::new(reader);
    let mut line = String::new();
    reader.read_line(&mut line).await.map_err(|e| {
        let err = format!("read error: {}", e);
        tracing::error!("{}", err);
        err
    })?;

    let response: serde_json::Value = serde_json::from_str(&line).map_err(|e| {
        let err = format!("parse error: {} (line: {})", e, line);
        tracing::error!("{}", err);
        err
    })?;

    Ok(response)
}

#[tauri::command]
pub async fn get_sessions(filter: Option<String>) -> Result<serde_json::Value, String> {
    tracing::info!("get_sessions called with filter: {:?}", filter);
    let command_action = serde_json::json!({
        "action": "get_sessions",
        "filter": filter
    });
    send_command(command_action).await
}

#[tauri::command]
pub async fn ping() -> Result<String, String> {
    let command_action = serde_json::json!({
        "action": "ping"
    });
    match send_command(command_action).await {
        Ok(_) => Ok("pong".to_string()),
        Err(e) => Err(format!("ping failed: {}", e)),
    }
}

#[tauri::command]
pub async fn get_session(session_id: String) -> Result<serde_json::Value, String> {
    let command_action = serde_json::json!({
        "action": "get_session",
        "session_id": session_id
    });
    send_command(command_action).await
}

#[tauri::command]
pub async fn get_session_stats() -> Result<serde_json::Value, String> {
    let command_action = serde_json::json!({
        "action": "get_session_stats"
    });
    send_command(command_action).await
}

#[tauri::command]
pub async fn get_agent_analytics() -> Result<serde_json::Value, String> {
    let command_action = serde_json::json!({
        "action": "get_agent_analytics"
    });
    send_command(command_action).await
}

#[tauri::command]
pub async fn get_timeline(limit: u32) -> Result<serde_json::Value, String> {
    let command_action = serde_json::json!({
        "action": "get_timeline",
        "limit": limit
    });
    send_command(command_action).await
}

#[tauri::command]
pub async fn search_sessions(query: String) -> Result<serde_json::Value, String> {
    let command_action = serde_json::json!({
        "action": "search_sessions",
        "query": query
    });
    send_command(command_action).await
}

#[tauri::command]
pub async fn resolve_permission(permission_id: String, approved: bool, response: Option<String>) -> Result<serde_json::Value, String> {
    let command_action = serde_json::json!({
        "action": "resolve_permission",
        "permission_id": permission_id,
        "approved": approved,
        "response": response
    });
    send_command(command_action).await
}

#[tauri::command]
pub async fn answer_question(question_id: String, answer: String) -> Result<serde_json::Value, String> {
    let command_action = serde_json::json!({
        "action": "answer_question",
        "question_id": question_id,
        "answer": answer
    });
    send_command(command_action).await
}

#[tauri::command]
pub async fn jump_to_session(session_id: String) -> Result<serde_json::Value, String> {
    let command_action = serde_json::json!({
        "action": "jump_to_session",
        "session_id": session_id
    });
    send_command(command_action).await
}

#[tauri::command]
pub async fn stop_agent(session_id: String) -> Result<serde_json::Value, String> {
    let command_action = serde_json::json!({
        "action": "stop_agent",
        "session_id": session_id
    });
    send_command(command_action).await
}

#[tauri::command]
pub async fn shutdown() -> Result<serde_json::Value, String> {
    let command_action = serde_json::json!({
        "action": "shutdown"
    });
    send_command(command_action).await
}

#[tauri::command]
pub async fn discover_agents() -> Result<serde_json::Value, String> {
    let command_action = serde_json::json!({
        "action": "discover_agents"
    });
    send_command(command_action).await
}
