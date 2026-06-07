use daemon_core::agents::{AgentPlugin, SimplePlugin, PluginResult};
use daemon_core::state::{AgentKind, EventKind};

pub struct OpenCodePlugin;

impl SimplePlugin for OpenCodePlugin {
    fn name(&self) -> &'static str {
        "opencode"
    }

    fn agent_kind(&self) -> AgentKind {
        AgentKind::Opencode
    }

    fn map_event_type(&self, raw_type: &str) -> Option<EventKind> {
        match raw_type {
            "session_start" => Some(EventKind::SessionStarted),
            "session_complete" => Some(EventKind::SessionCompleted),
            "session_failed" => Some(EventKind::SessionFailed),
            "activity" => Some(EventKind::ActivityUpdated),
            "permission" => Some(EventKind::PermissionRequested),
            "question" => Some(EventKind::QuestionAsked),
            "heartbeat" => Some(EventKind::Heartbeat),
            "token_usage" => Some(EventKind::TokenUsage),
            _ => None,
        }
    }
}

impl AgentPlugin for OpenCodePlugin {
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
    fn test_parse_session_start() {
        let plugin = OpenCodePlugin;
        let payload = r#"{"type":"session_start","session_id":"abc123","model":"claude-sonnet-4"}"#;
        let result = plugin.parse(payload).expect("parse should succeed").expect("parse result should be valid");
        assert_eq!(result.agent, AgentKind::Opencode);
        assert_eq!(result.event, EventKind::SessionStarted);
        assert_eq!(result.session_id, "abc123");
    }

    #[test]
    fn test_parse_session_complete() {
        let plugin = OpenCodePlugin;
        let payload = r#"{"type":"session_complete","session_id":"abc123","tokens_input":100,"tokens_output":50}"#;
        let result = plugin.parse(payload).expect("parse should succeed").expect("parse result should be valid");
        assert_eq!(result.event, EventKind::SessionCompleted);
        assert_eq!(result.tokens_input, Some(100));
    }

    #[test]
    fn test_parse_invalid_json() {
        let plugin = OpenCodePlugin;
        let result = plugin.parse("not json");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_unknown_fields() {
        let plugin = OpenCodePlugin;
        let payload = r#"{"type":"session_start","unknown_future_field":"x","session_id":"abc"}"#;
        let result = plugin.parse(payload).expect("parse should succeed").expect("parse result should be valid");
        assert_eq!(result.event, EventKind::SessionStarted);
    }

    #[test]
    fn test_parse_missing_optional_fields() {
        let plugin = OpenCodePlugin;
        let payload = r#"{"type":"token_usage","session_id":"abc"}"#;
        let event = plugin.parse(payload).expect("parse should succeed").expect("parse result should be valid");
        assert!(event.tokens_input.is_none());
    }

    #[test]
    fn test_parse_unknown_event_type_returns_none() {
        let plugin = OpenCodePlugin;
        let payload = r#"{"type":"future_event_type","session_id":"abc"}"#;
        let result = plugin.parse(payload);
        assert!(matches!(result, Ok(None)));
    }
}
