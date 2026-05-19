use std::process::Command;

use super::detector::TerminalPane;

#[derive(Debug, thiserror::Error)]
pub enum TmuxError {
    #[error("tmux not found")]
    NotInstalled,
    #[error("pane not found: {0}")]
    PaneNotFound(String),
    #[error("tmux command failed: {0}")]
    CommandFailed(String),
    #[error("no tmux session running")]
    NoSession,
}

pub fn is_available() -> bool {
    Command::new("tmux").arg("list-sessions").output().is_ok()
}

pub fn list_panes() -> Result<Vec<TerminalPane>, TmuxError> {
    let output = Command::new("tmux")
        .args([
            "list-panes",
            "-a",
            "-F",
            "#{pane_id}|#{pane_pid}|#{pane_current_command}|#{pane_current_path}|#{session_name}",
        ])
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                TmuxError::NotInstalled
            } else {
                TmuxError::CommandFailed(e.to_string())
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("no server running") {
            return Err(TmuxError::NoSession);
        }
        return Err(TmuxError::CommandFailed(stderr.to_string()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let panes = stdout
        .lines()
        .filter(|l| !l.is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 4 {
                Some(TerminalPane {
                    pane_id: Some(parts[0].to_string()),
                    pid: parts[1].parse::<u32>().ok(),
                    command: Some(parts[2].to_string()),
                    cwd: {
                        let cwd = parts[3].to_string();
                        if cwd.is_empty() || cwd == "(null)" {
                            None
                        } else {
                            Some(cwd)
                        }
                    },
                    session: parts.get(4).map(|s| s.to_string()),
                })
            } else {
                None
            }
        })
        .collect();

    Ok(panes)
}

pub fn find_pane_by_pid(pid: u32) -> Result<Option<TerminalPane>, TmuxError> {
    let panes = list_panes()?;
    Ok(panes.into_iter().find(|p| p.pid == Some(pid)))
}

pub fn find_pane_by_cwd(cwd: &str) -> Result<Option<TerminalPane>, TmuxError> {
    let panes = list_panes()?;
    Ok(panes
        .into_iter()
        .find(|p| p.cwd.as_ref().is_some_and(|pane_cwd| pane_cwd == cwd)))
}

pub fn focus_pane(pane_id: &str) -> Result<(), TmuxError> {
    let status = Command::new("tmux")
        .args(["select-pane", "-t", pane_id])
        .status()
        .map_err(|e| TmuxError::CommandFailed(e.to_string()))?;

    if !status.success() {
        return Err(TmuxError::PaneNotFound(pane_id.to_string()));
    }
    Ok(())
}

pub fn send_keys(pane_id: &str, keys: &str) -> Result<(), TmuxError> {
    let status = Command::new("tmux")
        .args(["send-keys", "-t", pane_id, keys])
        .status()
        .map_err(|e| TmuxError::CommandFailed(e.to_string()))?;

    if !status.success() {
        return Err(TmuxError::PaneNotFound(pane_id.to_string()));
    }
    Ok(())
}

/// Get the tmux session name, window index, and pane index for the process tree rooted at `pid`.
pub fn locate_pane(pid: u32) -> Option<(String, u32, u32)> {
    let output = Command::new("tmux")
        .args([
            "list-panes",
            "-a",
            "-F",
            "#{pane_pid}|#{session_name}|#{window_index}|#{pane_index}",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    for line in stdout.lines() {
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() == 4 {
            let pane_pid: u32 = parts[0].parse().ok()?;
            if is_descendant(pane_pid, pid) {
                return Some((
                    parts[1].to_string(),
                    parts[2].parse().ok()?,
                    parts[3].parse().ok()?,
                ));
            }
        }
    }
    None
}

/// Check if `descendant` is a child (or deeper) of `ancestor` in the process tree.
fn is_descendant(ancestor: u32, descendant: u32) -> bool {
    let mut current = descendant;
    loop {
        if current == ancestor {
            return true;
        }
        let path = format!("/proc/{current}/status");
        let status = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => return false,
        };
        let ppid: u32 = match status
            .lines()
            .find(|l| l.starts_with("PPid:"))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|s| s.parse().ok())
        {
            Some(p) => p,
            None => return false,
        };
        if ppid == 0 || ppid == 1 {
            return false;
        }
        current = ppid;
    }
}

pub fn current_pane_id() -> Result<Option<String>, TmuxError> {
    let output = Command::new("tmux")
        .args(["display-message", "-p", "#{pane_id}"])
        .output()
        .map_err(|_| TmuxError::NoSession)?;

    if output.status.success() {
        Ok(Some(
            String::from_utf8_lossy(&output.stdout).trim().to_string(),
        ))
    } else {
        Ok(None)
    }
}
