use serde::{Deserialize, Serialize};

use super::ghostty;
use super::kitty;
use super::tmux;
use super::wezterm;
use super::zellij;
use super::konsole;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalPane {
    pub pane_id: Option<String>,
    pub pid: Option<u32>,
    pub command: Option<String>,
    pub cwd: Option<String>,
    pub session: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TerminalId {
    TmuxWindow {
        session: String,
        window: u32,
        pane: u32,
    },
    ZellijPane {
        session: String,
        tab: u32,
        pane: u32,
    },
    Kitty {
        window_id: u32,
    },
    WezTerm {
        pane_id: u32,
    },
    Ghostty {
        pid: u32,
    },
    Pid {
        pid: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TerminalKind {
    Tmux,
    Zellij,
    Ghostty,
    WezTerm,
    Kitty,
    Konsole,
    Unknown,
    Other(String),
}

pub fn detect() -> Option<TerminalKind> {
    if tmux::is_available() {
        return Some(TerminalKind::Tmux);
    }
    if zellij::is_available() {
        return Some(TerminalKind::Zellij);
    }
    if ghostty::is_available() {
        return Some(TerminalKind::Ghostty);
    }
    if wezterm::is_available() {
        return Some(TerminalKind::WezTerm);
    }
    if kitty::is_available() {
        return Some(TerminalKind::Kitty);
    }
    if std::env::var("TMUX").is_ok() {
        return Some(TerminalKind::Tmux);
    }
    if std::env::var("ZELLIJ").is_ok() {
        return Some(TerminalKind::Zellij);
    }
    if ghostty::is_in_ghostty() {
        return Some(TerminalKind::Ghostty);
    }
    if wezterm::is_in_wezterm() {
        return Some(TerminalKind::WezTerm);
    }
    if kitty::is_in_kitty() {
        return Some(TerminalKind::Kitty);
    }
    if konsole::is_in_konsole() {
        return Some(TerminalKind::Konsole);
    }
    None
}

pub fn parse_terminal_kind(s: Option<&str>) -> TerminalKind {
    match s {
        Some("tmux") => TerminalKind::Tmux,
        Some("zellij") => TerminalKind::Zellij,
        Some("ghostty") => TerminalKind::Ghostty,
        Some("wezterm" | "WezTerm") => TerminalKind::WezTerm,
        Some("kitty") => TerminalKind::Kitty,
        Some("konsole" | "Konsole") => TerminalKind::Konsole,
        Some(other) => TerminalKind::Other(other.to_string()),
        None => TerminalKind::Tmux,
    }
}

pub fn resolve_jump_target(
    terminal: Option<&str>,
    pid: Option<u32>,
    cwd: Option<&str>,
) -> Result<Option<String>, String> {
    let kind = parse_terminal_kind(terminal);
    match kind {
        TerminalKind::Tmux => {
            if let Some(pid) = pid {
                let pane = tmux::find_pane_by_pid(pid).map_err(|e| e.to_string())?;
                if pane.is_some() {
                    return Ok(pane.and_then(|p| p.pane_id));
                }
            }
            if let Some(cwd) = cwd {
                let pane = tmux::find_pane_by_cwd(cwd).map_err(|e| e.to_string())?;
                return Ok(pane.and_then(|p| p.pane_id));
            }
            Ok(None)
        }
        TerminalKind::Zellij => {
            let panes = zellij::list_panes().map_err(|e| e.to_string())?;
            let pane = panes.into_iter().find(|p| {
                if let (Some(pane_pid), Some(target_pid)) = (p.pid, pid) {
                    pane_pid == target_pid
                } else {
                    false
                }
            });
            Ok(pane.and_then(|p| p.pane_id))
        }
        TerminalKind::Ghostty => {
            let panes = ghostty::list_panes().map_err(|e| e.to_string())?;
            let pane = panes.into_iter().find(|p| {
                if let (Some(pane_pid), Some(target_pid)) = (p.pid, pid) {
                    pane_pid == target_pid
                } else {
                    false
                }
            });
            Ok(pane.and_then(|p| p.pane_id))
        }
        TerminalKind::WezTerm => {
            let panes = wezterm::list_panes().map_err(|e| e.to_string())?;
            let pane = panes.into_iter().find(|p| {
                if let (Some(pane_pid), Some(target_pid)) = (p.pid, pid) {
                    pane_pid == target_pid
                } else {
                    false
                }
            });
            Ok(pane.and_then(|p| p.pane_id))
        }
        TerminalKind::Kitty => {
            let panes = kitty::list_panes().map_err(|e| e.to_string())?;
            let pane = panes.into_iter().find(|p| {
                if let (Some(pane_pid), Some(target_pid)) = (p.pid, pid) {
                    pane_pid == target_pid
                } else {
                    false
                }
            });
            Ok(pane.and_then(|p| p.pane_id))
        }
        TerminalKind::Unknown => Err("unsupported terminal: unknown".to_string()),
        TerminalKind::Konsole => {
            let panes = konsole::list_panes().map_err(|e| e.to_string())?;
            let pane = panes.into_iter().find(|p| {
                if let (Some(pane_pid), Some(target_pid)) = (p.pid, pid) {
                    pane_pid == target_pid
                } else {
                    false
                }
            });
            Ok(pane.and_then(|p| p.pane_id))
        }
        TerminalKind::Other(other) => Err(format!("unsupported terminal: {}", other)),
    }
}

/// Read the process name from /proc/{pid}/comm
pub fn get_process_name(pid: u32) -> Option<String> {
    let path = format!("/proc/{pid}/comm");
    std::fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

/// Read the parent PID from /proc/{pid}/status
pub fn get_parent_pid(pid: u32) -> Option<u32> {
    let path = format!("/proc/{pid}/status");
    let status = std::fs::read_to_string(path).ok()?;
    status
        .lines()
        .find(|l| l.starts_with("PPid:"))?
        .split_whitespace()
        .nth(1)?
        .parse()
        .ok()
}

/// Walk the process tree from `ppid` upward to detect the running terminal.
/// `pid` is the hook's own PID, used to match against terminal panes.
pub fn detect_terminal_from_pid(pid: u32, ppid: u32) -> Option<(TerminalKind, TerminalId)> {
    let mut current = ppid;

    for _ in 0..10 {
        let name = get_process_name(current)?;

        let result = match name.as_str() {
            "tmux: server" | "tmux" => {
                let (session, window, pane) = tmux::locate_pane(pid)?;
                Some((
                    TerminalKind::Tmux,
                    TerminalId::TmuxWindow { session, window, pane },
                ))
            }
            "zellij" | "zellij-server" => {
                let (session, tab, pane) = zellij::locate_pane(pid)?;
                Some((
                    TerminalKind::Zellij,
                    TerminalId::ZellijPane { session, tab, pane },
                ))
            }
            "kitty" => {
                let window_id = kitty::find_window_by_pid(pid)?;
                Some((TerminalKind::Kitty, TerminalId::Kitty { window_id }))
            }
            "wezterm-gui" | "wezterm" => {
                let pane_id = wezterm::find_pane_id_by_pid(pid)?;
                Some((TerminalKind::WezTerm, TerminalId::WezTerm { pane_id }))
            }
            "ghostty" => {
                // Ghostty has no stable IPC API → store the ghostty PID as fallback
                Some((TerminalKind::Ghostty, TerminalId::Ghostty { pid: current }))
            }
            _ => {
                current = get_parent_pid(current)?;
                continue;
            }
        };

        if result.is_some() {
            return result;
        }

        current = get_parent_pid(current)?;
    }

    // Fallback: store the hook PID
    Some((TerminalKind::Unknown, TerminalId::Pid { pid }))
}

pub fn focus_terminal(pane_id: &str, terminal: Option<&str>) -> Result<(), String> {
    match parse_terminal_kind(terminal) {
        TerminalKind::Tmux => tmux::focus_pane(pane_id).map_err(|e| e.to_string()),
        TerminalKind::Zellij => zellij::focus_pane(pane_id).map_err(|e| e.to_string()),
        TerminalKind::Ghostty => ghostty::focus_pane(pane_id).map_err(|e| e.to_string()),
        TerminalKind::WezTerm => wezterm::focus_pane(pane_id).map_err(|e| e.to_string()),
        TerminalKind::Kitty => kitty::focus_pane(pane_id).map_err(|e| e.to_string()),
        TerminalKind::Konsole => konsole::focus_pane(pane_id).map_err(|e| e.to_string()),
        TerminalKind::Unknown => Err("unsupported terminal: unknown".to_string()),
        TerminalKind::Other(other) => Err(format!("unsupported terminal: {}", other)),
    }
}
