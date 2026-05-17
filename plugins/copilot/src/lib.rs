use daemon_core::agents::{AgentPlugin, SimplePlugin, PluginResult};
use daemon_core::state::{AgentKind, EventKind};

pub struct CopilotPlugin;

impl SimplePlugin for CopilotPlugin {
    fn name(&self) -> &'static str {
        "copilot"
    }

    fn agent_kind(&self) -> AgentKind {
        AgentKind::Copilot
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

impl AgentPlugin for CopilotPlugin {
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
        let plugin = CopilotPlugin;
        let payload = r#"{"type":"session_start","session_id":"copilot-1","model":"test-model"}"#;
        let result = plugin.parse(payload).unwrap().unwrap();
        assert_eq!(result.agent, AgentKind::Copilot);
        assert_eq!(result.event, EventKind::SessionStarted);
    }
}
