use std::process::Command;

use super::detector::{TerminalId, TerminalKind};

#[derive(Debug, thiserror::Error)]
pub enum JumpError {
    #[error("no terminal info stored for this session")]
    NoTerminalInfo,
    #[error("command not found: {0}")]
    CommandNotFound(String),
    #[error("command failed")]
    CommandFailed,
    #[error("focus failed — all methods exhausted")]
    FocusFailed,
}

/// Central dispatcher: focus the terminal pane identified by the session's
/// stored `TerminalKind` and `TerminalId`.
pub fn jump_to_terminal(terminal_kind: &Option<TerminalKind>, terminal_id: &Option<TerminalId>) -> Result<(), JumpError> {
    match (terminal_kind, terminal_id) {
        (
            Some(TerminalKind::Tmux),
            Some(TerminalId::TmuxWindow { session, window, pane }),
        ) => {
            run("tmux", &["select-window", "-t", &format!("{session}:{window}")])?;
            run("tmux", &["select-pane", "-t", &format!("{session}:{window}.{pane}")])?;
            Ok(())
        }

        (Some(TerminalKind::Zellij), Some(TerminalId::ZellijPane { tab, .. })) => {
            run("zellij", &["action", "go-to-tab", &tab.to_string()])
        }

        (Some(TerminalKind::Kitty), Some(TerminalId::Kitty { window_id })) => {
            run("kitty", &["@", "focus-window", "--match", &format!("id:{window_id}")])
        }

        (Some(TerminalKind::WezTerm), Some(TerminalId::WezTerm { pane_id })) => {
            run("wezterm", &["cli", "activate-pane", "--pane-id", &pane_id.to_string()])
        }

        (Some(TerminalKind::Ghostty), Some(TerminalId::Ghostty { pid }))
        | (_, Some(TerminalId::Pid { pid })) => jump_by_pid(*pid),

        _ => Err(JumpError::NoTerminalInfo),
    }
}

fn jump_by_pid(pid: u32) -> Result<(), JumpError> {
    if is_wayland() {
        jump_wayland(pid)
    } else {
        jump_x11(pid)
    }
}

fn jump_x11(pid: u32) -> Result<(), JumpError> {
    if run("xdotool", &["search", "--pid", &pid.to_string(), "windowactivate", "--sync"]).is_ok() {
        return Ok(());
    }
    // wmctrl fallback: list all windows with PIDs, find matching one
    let output = Command::new("wmctrl")
        .args(["-lp"])
        .output()
        .map_err(|_| JumpError::CommandNotFound("wmctrl".to_string()))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            if let Ok(win_pid) = parts[2].parse::<u32>() {
                if win_pid == pid {
                    return run("wmctrl", &["-i", "-a", parts[0]]);
                }
            }
        }
    }
    Err(JumpError::FocusFailed)
}

fn jump_wayland(pid: u32) -> Result<(), JumpError> {
    if run("swaymsg", &[format!("[pid={pid}] focus").as_str()]).is_ok() {
        return Ok(());
    }
    if run(
        "qdbus",
        &[
            "org.kde.KWin",
            "/KWin",
            "activateWindow",
            &pid.to_string(),
        ],
    )
    .is_ok()
    {
        return Ok(());
    }
    if run("ydotool", &["search", "--pid", &pid.to_string()]).is_ok() {
        return Ok(());
    }
    Err(JumpError::FocusFailed)
}

fn is_wayland() -> bool {
    std::env::var("WAYLAND_DISPLAY").is_ok()
}

fn run(cmd: &str, args: &[&str]) -> Result<(), JumpError> {
    Command::new(cmd)
        .args(args)
        .status()
        .map_err(|_| JumpError::CommandNotFound(cmd.to_string()))
        .and_then(|s| {
            if s.success() {
                Ok(())
            } else {
                Err(JumpError::CommandFailed)
            }
        })
}
