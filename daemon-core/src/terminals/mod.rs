pub mod detector;
pub mod ghostty;
pub mod kitty;
pub mod tmux;
pub mod wezterm;
pub mod zellij;

pub use detector::*;

#[cfg(test)]
mod tests {
    use super::detector::*;

    #[test]
    fn test_parse_terminal_kind_tmux() {
        let kind = parse_terminal_kind(Some("tmux"));
        assert_eq!(kind, TerminalKind::Tmux);
    }

    #[test]
    fn test_parse_terminal_kind_zellij() {
        let kind = parse_terminal_kind(Some("zellij"));
        assert_eq!(kind, TerminalKind::Zellij);
    }

    #[test]
    fn test_parse_terminal_kind_ghostty() {
        let kind = parse_terminal_kind(Some("ghostty"));
        assert_eq!(kind, TerminalKind::Ghostty);
    }

    #[test]
    fn test_parse_terminal_kind_wezterm() {
        let kind = parse_terminal_kind(Some("wezterm"));
        assert_eq!(kind, TerminalKind::WezTerm);
    }

    #[test]
    fn test_parse_terminal_kind_kitty() {
        let kind = parse_terminal_kind(Some("kitty"));
        assert_eq!(kind, TerminalKind::Kitty);
    }

    #[test]
    fn test_parse_terminal_kind_other() {
        let kind = parse_terminal_kind(Some("warp"));
        assert_eq!(kind, TerminalKind::Other("warp".to_string()));
    }

    #[test]
    fn test_parse_terminal_kind_none() {
        let kind = parse_terminal_kind(None);
        assert_eq!(kind, TerminalKind::Tmux);
    }

    #[test]
    fn test_terminal_pane_defaults() {
        let pane = TerminalPane {
            pane_id: None,
            pid: None,
            command: None,
            cwd: None,
            session: None,
        };
        assert!(pane.pane_id.is_none());
        assert!(pane.pid.is_none());
    }

    #[test]
    fn test_terminal_pane_with_values() {
        let pane = TerminalPane {
            pane_id: Some("%1".to_string()),
            pid: Some(12345),
            command: Some("zsh".to_string()),
            cwd: Some("/home/user".to_string()),
            session: Some("main".to_string()),
        };
        assert_eq!(pane.pane_id, Some("%1".to_string()));
        assert_eq!(pane.pid, Some(12345));
        assert_eq!(pane.command, Some("zsh".to_string()));
        assert_eq!(pane.cwd, Some("/home/user".to_string()));
        assert_eq!(pane.session, Some("main".to_string()));
    }

    #[test]
    fn test_resolve_jump_target_unsupported_terminal() {
        let result = resolve_jump_target(Some("warp"), None, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unsupported terminal"));
    }

    #[test]
    fn test_focus_terminal_unsupported_terminal() {
        let result = focus_terminal("%1", Some("warp"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unsupported terminal"));
    }
}


