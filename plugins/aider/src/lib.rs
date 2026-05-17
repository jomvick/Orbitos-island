use daemon_core::agents::{AgentPlugin, SimplePlugin, PluginResult};
use daemon_core::state::{AgentKind, EventKind};

pub struct AiderPlugin;

impl SimplePlugin for AiderPlugin {
    fn name(&self) -> &'static str {
        "aider"
    }

    fn agent_kind(&self) -> AgentKind {
        AgentKind::Aider
    }

    fn map_event_type(&self, raw_type: &str) -> Option<EventKind> {
        match raw_type {
            "session_start" | "task_start" | "start" => Some(EventKind::SessionStarted),
            "session_complete" | "task_complete" | "complete" | "done" => {
                Some(EventKind::SessionCompleted)
            }
            "session_failed" | "error" | "fail" => Some(EventKind::SessionFailed),
            "activity" | "progress" | "think" => Some(EventKind::ActivityUpdated),
            "heartbeat" => Some(EventKind::Heartbeat),
            "token_usage" => Some(EventKind::TokenUsage),
            _ => None,
        }
    }
}

impl AgentPlugin for AiderPlugin {
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
        let plugin = AiderPlugin;
        let payload =
            r#"{"type":"session_start","session_id":"aider-1","model":"claude-sonnet-4"}"#;
        let result = plugin.parse(payload).unwrap().unwrap();
        assert_eq!(result.agent, AgentKind::Aider);
        assert_eq!(result.event, EventKind::SessionStarted);
    }

    #[test]
    fn test_parse_task_complete() {
        let plugin = AiderPlugin;
        let payload = r#"{"type":"complete","session_id":"aider-1","tokens_input":3000,"tokens_output":1500}"#;
        let result = plugin.parse(payload).unwrap().unwrap();
        assert_eq!(result.event, EventKind::SessionCompleted);
        assert_eq!(result.tokens_input, Some(3000));
    }

    #[test]
    fn test_parse_invalid_json() {
        let plugin = AiderPlugin;
        let result = plugin.parse("not json");
        assert!(result.is_err());
    }
}
