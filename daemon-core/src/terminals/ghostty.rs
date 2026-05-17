use std::process::Command;

use super::detector::TerminalPane;

#[derive(Debug, thiserror::Error)]
pub enum GhosttyError {
    #[error("ghostty not found")]
    NotInstalled,
    #[error("pane not found: {0}")]
    PaneNotFound(String),
    #[error("command failed: {0}")]
    CommandFailed(String),
    #[error("no ghostty instance running")]
    NoInstance,
}

pub fn is_available() -> bool {
    Command::new("ghostty").arg("+list-splits").output().is_ok()
}

pub fn is_in_ghostty() -> bool {
    std::env::var("GHOSTTY_RESOURCES_DIR").is_ok()
        || std::env::var("TERM_PROGRAM").is_ok_and(|v| v == "ghostty")
}

pub fn list_panes() -> Result<Vec<TerminalPane>, GhosttyError> {
    let output = Command::new("ghostty")
        .args(["+list-splits", "--json"])
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                GhosttyError::NotInstalled
            } else {
                GhosttyError::CommandFailed(e.to_string())
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GhosttyError::CommandFailed(stderr.to_string()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let panes: Vec<serde_json::Value> = serde_json::from_str(&stdout).unwrap_or_default();
    let result = panes
        .into_iter()
<<<<<<< HEAD
        .map(|p| TerminalPane {
            pane_id: p.get("id").and_then(|v| v.as_str().map(String::from)),
            pid: p.get("pid").and_then(|v| v.as_u64().map(|n| n as u32)),
            command: p.get("command").and_then(|v| v.as_str().map(String::from)),
            cwd: p.get("cwd").and_then(|v| v.as_str().map(String::from)),
            session: p.get("tab").and_then(|v| v.as_str().map(String::from)),
        })
        .collect();

    Ok(result)
}

pub fn focus_pane(pane_id: &str) -> Result<(), GhosttyError> {
    let status = Command::new("ghostty")
        .args(["+focus-split", pane_id])
        .status()
        .map_err(|e| GhosttyError::CommandFailed(e.to_string()))?;

    if !status.success() {
        return Err(GhosttyError::PaneNotFound(pane_id.to_string()));
    }
    Ok(())
}
