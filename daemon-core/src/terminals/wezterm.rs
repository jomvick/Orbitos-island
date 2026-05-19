use std::process::Command;

use super::detector::TerminalPane;

#[derive(Debug, thiserror::Error)]
pub enum WeztermError {
    #[error("wezterm not found")]
    NotInstalled,
    #[error("pane not found: {0}")]
    PaneNotFound(String),
    #[error("command failed: {0}")]
    CommandFailed(String),
    #[error("no wezterm instance running")]
    NoInstance,
}

pub fn is_available() -> bool {
    Command::new("wezterm")
        .args(["cli", "list"])
        .output()
        .is_ok()
}

pub fn is_in_wezterm() -> bool {
    std::env::var("TERM_PROGRAM").is_ok_and(|v| v == "WezTerm")
}

pub fn list_panes() -> Result<Vec<TerminalPane>, WeztermError> {
    let output = Command::new("wezterm")
        .args(["cli", "list", "--format", "json"])
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                WeztermError::NotInstalled
            } else {
                WeztermError::CommandFailed(e.to_string())
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(WeztermError::CommandFailed(stderr.to_string()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let panes: Vec<serde_json::Value> = serde_json::from_str(&stdout).unwrap_or_default();
    let result = panes
        .into_iter()
        .map(|p| TerminalPane {
            pane_id: p
                .get("pane_id")
                .and_then(|v| v.as_i64().map(|n| n.to_string())),
            pid: p.get("pid").and_then(|v| v.as_u64().map(|n| n as u32)),
            command: p.get("title").and_then(|v| v.as_str().map(String::from)),
            cwd: p.get("cwd").and_then(|v| v.as_str().map(String::from)),
            session: p
                .get("workspace")
                .and_then(|v| v.as_str().map(String::from)),
        })
        .collect();

    Ok(result)
}

/// Find the wezterm pane ID that contains the process with the given PID.
pub fn find_pane_id_by_pid(pid: u32) -> Option<u32> {
    let panes = list_panes().ok()?;
    for pane in panes {
        if let Some(pane_pid) = pane.pid {
            if pane_pid == pid {
                return pane.pane_id?.parse().ok();
            }
        }
    }
    None
}

pub fn focus_pane(pane_id: &str) -> Result<(), WeztermError> {
    let status = Command::new("wezterm")
        .args(["cli", "activate-pane", "--pane-id", pane_id])
        .status()
        .map_err(|e| WeztermError::CommandFailed(e.to_string()))?;

    if !status.success() {
        return Err(WeztermError::PaneNotFound(pane_id.to_string()));
    }
    Ok(())
}
