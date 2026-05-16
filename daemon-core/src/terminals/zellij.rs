use std::process::Command;

use super::detector::TerminalPane;

#[derive(Debug, thiserror::Error)]
pub enum ZellijError {
    #[error("zellij not found")]
    NotInstalled,
    #[error("pane not found: {0}")]
    PaneNotFound(String),
    #[error("zellij command failed: {0}")]
    CommandFailed(String),
    #[error("no zellij session running")]
    NoSession,
}

pub fn is_available() -> bool {
    Command::new("zellij").arg("list-sessions").output().is_ok()
}

pub fn list_panes() -> Result<Vec<TerminalPane>, ZellijError> {
    let output = Command::new("zellij")
        .args(["list-panes", "--short"])
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ZellijError::NotInstalled
            } else {
                ZellijError::CommandFailed(e.to_string())
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ZellijError::CommandFailed(stderr.to_string()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let panes = stdout
        .lines()
        .filter(|l| !l.is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('\t').collect();
            if !parts.is_empty() {
                Some(TerminalPane {
                    pane_id: Some(parts[0].to_string()),
                    pid: parts.get(1).and_then(|p| p.parse::<u32>().ok()),
                    command: parts.get(2).map(|s| s.to_string()),
                    cwd: parts.get(3).map(|s| s.to_string()),
                    session: parts.get(4).map(|s| s.to_string()),
                })
            } else {
                None
            }
        })
        .collect();

    Ok(panes)
}

pub fn focus_pane(pane_id: &str) -> Result<(), ZellijError> {
    let status = Command::new("zellij")
        .args(["action", "focus-pane", "-p", pane_id])
        .status()
        .map_err(|e| ZellijError::CommandFailed(e.to_string()))?;

    if !status.success() {
        return Err(ZellijError::PaneNotFound(pane_id.to_string()));
    }
    Ok(())
}

pub fn is_in_zellij() -> bool {
    std::env::var("ZELLIJ").is_ok()
}
