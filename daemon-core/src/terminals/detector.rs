use serde::{Deserialize, Serialize};

use super::ghostty;
use super::kitty;
use super::tmux;
use super::wezterm;
use super::zellij;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalPane {
    pub pane_id: Option<String>,
    pub pid: Option<u32>,
    pub command: Option<String>,
    pub cwd: Option<String>,
    pub session: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TerminalKind {
    Tmux,
    Zellij,
    Ghostty,
    WezTerm,
    Kitty,
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
    None
}

pub fn parse_terminal_kind(s: Option<&str>) -> TerminalKind {
    match s {
        Some("tmux") => TerminalKind::Tmux,
        Some("zellij") => TerminalKind::Zellij,
        Some("ghostty") => TerminalKind::Ghostty,
        Some("wezterm" | "WezTerm") => TerminalKind::WezTerm,
        Some("kitty") => TerminalKind::Kitty,
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
        TerminalKind::Other(other) => Err(format!("unsupported terminal: {}", other)),
    }
}

pub fn focus_terminal(pane_id: &str, terminal: Option<&str>) -> Result<(), String> {
    match parse_terminal_kind(terminal) {
        TerminalKind::Tmux => tmux::focus_pane(pane_id).map_err(|e| e.to_string()),
        TerminalKind::Zellij => zellij::focus_pane(pane_id).map_err(|e| e.to_string()),
        TerminalKind::Ghostty => ghostty::focus_pane(pane_id).map_err(|e| e.to_string()),
        TerminalKind::WezTerm => wezterm::focus_pane(pane_id).map_err(|e| e.to_string()),
        TerminalKind::Kitty => kitty::focus_pane(pane_id).map_err(|e| e.to_string()),
        TerminalKind::Other(other) => Err(format!("unsupported terminal: {}", other)),
    }
}
