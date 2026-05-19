use std::sync::OnceLock;
use std::time::Duration;

use agentos_ipc::{connect_to_daemon, IpcMessage, IpcStatus};

const DAEMON_TIMEOUT: Duration = Duration::from_millis(100);
const PERMISSION_TIMEOUT: Duration = Duration::from_secs(300);

fn runtime() -> &'static tokio::runtime::Runtime {
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to create tokio runtime")
    })
}

pub fn send_event(source: &str, payload: serde_json::Value) -> Result<(), String> {
    runtime().block_on(async {
        let mut codec = connect_to_daemon(DAEMON_TIMEOUT)
            .await
            .map_err(|e| format!("daemon unreachable: {}", e))?;

        let msg = IpcMessage::new_event(source, payload);
        codec
            .send(&msg)
            .await
            .map_err(|e| format!("send failed: {}", e))?;

        Ok::<(), String>(())
    })
}

pub enum PermissionAction {
    Allow,
    Deny,
}

/// Envoie une demande de permission et bloque jusqu'à la réponse du daemon.
/// Timeout 5 min → Deny par défaut.
pub fn send_permission_request(
    source: &str,
    payload: serde_json::Value,
) -> Result<PermissionAction, String> {
    runtime().block_on(async {
        let mut codec = connect_to_daemon(PERMISSION_TIMEOUT)
            .await
            .map_err(|e| format!("daemon unreachable: {}", e))?;

        let msg = IpcMessage::new_event(source, payload);
        codec
            .send(&msg)
            .await
            .map_err(|e| format!("send failed: {}", e))?;

        // Bloque jusqu'à ce que le daemon réponde (ou timeout)
        match codec.recv_timeout(PERMISSION_TIMEOUT).await {
            Ok(IpcMessage::Response {
                status: IpcStatus::Ok,
                data,
                ..
            }) => {
                let action = data
                    .and_then(|d| d.get("action")?.as_str().map(String::from))
                    .unwrap_or_else(|| "deny".to_string());
                match action.as_str() {
                    "allow" => Ok(PermissionAction::Allow),
                    _ => Ok(PermissionAction::Deny),
                }
            }
            _ => Ok(PermissionAction::Deny),
        }
    })
}

/// Envoie une question et bloque jusqu'à la réponse du daemon.
/// Timeout 5 min → réponse "default" par défaut.
pub fn send_question_request(
    source: &str,
    payload: serde_json::Value,
) -> Result<String, String> {
    runtime().block_on(async {
        let mut codec = connect_to_daemon(PERMISSION_TIMEOUT)
            .await
            .map_err(|e| format!("daemon unreachable: {}", e))?;

        let msg = IpcMessage::new_event(source, payload);
        codec
            .send(&msg)
            .await
            .map_err(|e| format!("send failed: {}", e))?;

        match codec.recv_timeout(PERMISSION_TIMEOUT).await {
            Ok(IpcMessage::Response {
                status: IpcStatus::Ok,
                data,
                ..
            }) => {
                let label = data
                    .and_then(|d| d.get("label")?.as_str().map(String::from))
                    .unwrap_or_else(|| "default".to_string());
                Ok(label)
            }
            _ => Ok("default".to_string()),
        }
    })
}
