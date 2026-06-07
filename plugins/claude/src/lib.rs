use daemon_core::agents::{AgentPlugin, SimplePlugin, PluginResult};
use daemon_core::state::{AgentKind, EventKind};

pub struct ClaudePlugin;

impl SimplePlugin for ClaudePlugin {
    fn name(&self) -> &'static str {
        "claude"
    }

    fn agent_kind(&self) -> AgentKind {
        AgentKind::Claude
    }

    fn map_event_type(&self, raw_type: &str) -> Option<EventKind> {
        match raw_type {
            "session_start" | "task_start" => Some(EventKind::SessionStarted),
            "session_complete" | "task_complete" => Some(EventKind::SessionCompleted),
            "session_failed" | "task_failed" | "error" => Some(EventKind::SessionFailed),
            "activity" | "progress" => Some(EventKind::ActivityUpdated),
            "permission" | "permission_request" => Some(EventKind::PermissionRequested),
            "question" | "ask" => Some(EventKind::QuestionAsked),
            "heartbeat" => Some(EventKind::Heartbeat),
            "token_usage" => Some(EventKind::TokenUsage),
            "paused" | "waiting" => Some(EventKind::SessionPaused),
            _ => None,
        }
    }
}

impl AgentPlugin for ClaudePlugin {
    fn name(&self) -> &'static str {
        SimplePlugin::name(self)
    }

    fn agent_kind(&self) -> AgentKind {
        SimplePlugin::agent_kind(self)
    }

    fn parse(&self, payload: &str) -> PluginResult {
        self.parse_base(payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_task_complete() {
        let plugin = ClaudePlugin;
        let payload = r#"{"type":"task_complete","session_id":"sess1","tokens_input":5000,"tokens_output":2000}"#;
        let result = plugin.parse(payload).expect("parse should succeed").expect("parse result should be valid");
        assert_eq!(result.agent, AgentKind::Claude);
        assert_eq!(result.event, EventKind::SessionCompleted);
        assert_eq!(result.tokens_input, Some(5000));
    }

    #[test]
    fn test_parse_permission_request() {
        let plugin = ClaudePlugin;
        let payload = r#"{"type":"permission_request","session_id":"sess1","permission":{"command":"sudo apt install","description":"Install package"}}"#;
        let result = plugin.parse(payload).expect("parse should succeed").expect("parse result should be valid");
        assert_eq!(result.event, EventKind::PermissionRequested);
        assert!(result.permission.is_some());
    }

    #[test]
    fn test_parse_unknown_fields() {
        let plugin = ClaudePlugin;
        let payload = r#"{"type":"session_start","unknown_future_field":"x","session_id":"abc"}"#;
        let result = plugin.parse(payload).expect("parse should succeed").expect("parse result should be valid");
        assert_eq!(result.agent, AgentKind::Claude);
        assert_eq!(result.event, EventKind::SessionStarted);
    }

    #[test]
    fn test_parse_missing_optional_fields() {
        let plugin = ClaudePlugin;
        let payload = r#"{"type":"token_usage","session_id":"abc"}"#;
        let event = plugin.parse(payload).expect("parse should succeed").expect("parse result should be valid");
        assert!(event.tokens_input.is_none());
        assert!(event.tokens_output.is_none());
    }

    #[test]
    fn test_parse_unknown_event_type_returns_none() {
        let plugin = ClaudePlugin;
        let payload = r#"{"type":"future_event_type","session_id":"abc"}"#;
        let result = plugin.parse(payload);
        assert!(matches!(result, Ok(None)));
    }
}
