use daemon_core::agents::{AgentPlugin, SimplePlugin, PluginResult};
use daemon_core::state::{AgentKind, EventKind};

pub struct AntigravityPlugin;

impl SimplePlugin for AntigravityPlugin {
    fn name(&self) -> &'static str {
        "antigravity"
    }

    fn agent_kind(&self) -> AgentKind {
        AgentKind::Antigravity
    }

    fn map_event_type(&self, raw_type: &str) -> Option<EventKind> {
        match raw_type {
            "agent_start" | "session_start" => Some(EventKind::SessionStarted),
            "agent_complete" | "session_complete" => Some(EventKind::SessionCompleted),
            "agent_error" | "error" => Some(EventKind::SessionFailed),
            "agent_activity" | "activity" => Some(EventKind::ActivityUpdated),
            "heartbeat" => Some(EventKind::Heartbeat),
            _ => None,
        }
    }
}

impl AgentPlugin for AntigravityPlugin {
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
        let plugin = AntigravityPlugin;
        let payload = r#"{"type":"session_start","session_id":"antigravity-1","model":"test-model"}"#;
        let result = plugin.parse(payload).expect("parse should succeed").expect("parse result should be valid");
        assert_eq!(result.agent, AgentKind::Antigravity);
        assert_eq!(result.event, EventKind::SessionStarted);
    }
}
