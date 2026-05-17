use std::process::Command;
use super::detector::TerminalPane;

#[derive(Debug, thiserror::Error)]
pub enum KonsoleError {
    #[error("konsole not found")]
    NotInstalled,
    #[error("pane not found: {0}")]
    PaneNotFound(String),
    #[error("command failed: {0}")]
    CommandFailed(String),
}

pub fn is_available() -> bool {
    Command::new("konsole").arg("--version").output().is_ok()
}

pub fn is_in_konsole() -> bool {
    std::env::var("KONSOLE_VERSION").is_ok()
        || std::env::var("KONSOLE_DBUS_SERVICE").is_ok()
        || std::env::var("KONSOLE_DBUS_SESSION").is_ok()
}

pub fn list_panes() -> Result<Vec<TerminalPane>, KonsoleError> {
    // Traditional desktop terminals like Konsole do not offer splits
    // info via simple JSON CLI command. We return empty list gracefully
    // which falls back to default PID/CWD detection mechanisms.
    Ok(vec![])
}

pub fn focus_pane(_pane_id: &str) -> Result<(), KonsoleError> {
    // Fallback focus pane logic or D-Bus activation can go here if needed.
    // For now we return Ok to gracefully hand over.
    Ok(())
}
