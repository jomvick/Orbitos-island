use daemon_core::agents::{AgentPlugin, SimplePlugin, PluginResult};
use daemon_core::state::{AgentKind, EventKind};

pub struct GeminiPlugin;

impl SimplePlugin for GeminiPlugin {
    fn name(&self) -> &'static str {
        "gemini"
    }

    fn agent_kind(&self) -> AgentKind {
        AgentKind::Gemini
    }

    fn map_event_type(&self, raw_type: &str) -> Option<EventKind> {
        match raw_type {
            "session_start" | "task_start" => Some(EventKind::SessionStarted),
            "session_complete" | "task_complete" | "done" => Some(EventKind::SessionCompleted),
            "session_failed" | "error" | "task_failed" => Some(EventKind::SessionFailed),
            "activity" | "progress" | "think" => Some(EventKind::ActivityUpdated),
            "permission" | "permission_request" => Some(EventKind::PermissionRequested),
            "heartbeat" => Some(EventKind::Heartbeat),
            "token_usage" => Some(EventKind::TokenUsage),
            "paused" | "waiting" => Some(EventKind::SessionPaused),
            _ => None,
        }
    }
}

impl AgentPlugin for GeminiPlugin {
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
        let plugin = GeminiPlugin;
        let payload = r#"{"type":"session_start","session_id":"gemini-1","model":"test-model"}"#;
        let result = plugin.parse(payload).expect("parse should succeed").expect("parse result should be valid");
        assert_eq!(result.agent, AgentKind::Gemini);
        assert_eq!(result.event, EventKind::SessionStarted);
    }
}
