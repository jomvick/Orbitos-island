use std::process::Command;

use super::detector::TerminalPane;

#[derive(Debug, thiserror::Error)]
pub enum KittyError {
    #[error("kitty not found")]
    NotInstalled,
    #[error("pane not found: {0}")]
    PaneNotFound(String),
    #[error("command failed: {0}")]
    CommandFailed(String),
    #[error("not inside kitty session")]
    NotInKitty,
}

pub fn is_available() -> bool {
    // kitty @ ls lists all windows/tabs — only works inside kitty
    Command::new("kitty")
        .args(["@", "ls"])
        .output()
        .is_ok()
}

pub fn is_in_kitty() -> bool {
    std::env::var("KITTY_WINDOW_ID").is_ok()
        || std::env::var("TERM").is_ok_and(|v| v.contains("kitty"))
}

pub fn list_panes() -> Result<Vec<TerminalPane>, KittyError> {
    let output = Command::new("kitty")
        .args(["@", "ls"])
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                KittyError::NotInstalled
            } else {
                KittyError::CommandFailed(e.to_string())
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(KittyError::CommandFailed(stderr.to_string()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let os_data: Vec<serde_json::Value> = serde_json::from_str(&stdout).unwrap_or_default();
    let mut panes = Vec::new();

    for os in &os_data {
        if let Some(tabs) = os.get("tabs").and_then(|v| v.as_array()) {
            for tab in tabs {
                let session = tab
                    .get("title")
                    .and_then(|v| v.as_str().map(String::from));
                if let Some(windows) = tab.get("windows").and_then(|v| v.as_array()) {
                    for w in windows {
                        panes.push(TerminalPane {
                            pane_id: w
                                .get("id")
                                .and_then(|v| v.as_i64().map(|n| n.to_string())),
                            pid: w
                                .get("pid")
                                .and_then(|v| v.as_u64().map(|n| n as u32)),
                            command: w
                                .get("foreground_processes")
                                .and_then(|v| v.as_array())
                                .and_then(|arr| {
                                    arr.first()
                                        .and_then(|p| p.get("cmdline").and_then(|c| c.as_array()))
                                        .and_then(|cmd| {
                                            cmd.first().and_then(|c| c.as_str().map(String::from))
                                        })
                                }),
                            cwd: w
                                .get("cwd")
                                .and_then(|v| v.as_str().map(|s| s.to_string())),
                            session: session.clone(),
                        });
                    }
                }
            }
        }
    }

    Ok(panes)
}

/// Find the kitty window ID that contains the process with the given PID.
pub fn find_window_by_pid(pid: u32) -> Option<u32> {
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

pub fn focus_pane(pane_id: &str) -> Result<(), KittyError> {
    let status = Command::new("kitty")
        .args(["@", "focus-window", "--match", &format!("id:{}", pane_id)])
        .status()
        .map_err(|e| KittyError::CommandFailed(e.to_string()))?;

    if !status.success() {
        return Err(KittyError::PaneNotFound(pane_id.to_string()));
    }
    Ok(())
}
