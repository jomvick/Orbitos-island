use std::sync::OnceLock;
use std::time::Duration;

use agentos_ipc::{connect_to_daemon, IpcMessage};

const DAEMON_TIMEOUT: Duration = Duration::from_millis(100);

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
