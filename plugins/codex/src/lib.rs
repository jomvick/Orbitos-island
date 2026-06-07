use daemon_core::agents::{AgentPlugin, SimplePlugin, PluginResult};
use daemon_core::state::{AgentKind, EventKind};

pub struct CodexPlugin;

impl SimplePlugin for CodexPlugin {
    fn name(&self) -> &'static str {
        "codex"
    }

    fn agent_kind(&self) -> AgentKind {
        AgentKind::Codex
    }

    fn map_event_type(&self, raw_type: &str) -> Option<EventKind> {
        match raw_type {
            "session_start" | "task_start" | "prompt_submit" => Some(EventKind::SessionStarted),
            "session_complete" | "task_complete" | "stop" => Some(EventKind::SessionCompleted),
            "session_failed" | "error" => Some(EventKind::SessionFailed),
            "activity" | "progress" | "shell_execution" => Some(EventKind::ActivityUpdated),
            "heartbeat" => Some(EventKind::Heartbeat),
            "token_usage" => Some(EventKind::TokenUsage),
            _ => None,
        }
    }
}

impl AgentPlugin for CodexPlugin {
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
    fn test_parse_prompt_submit() {
        let plugin = CodexPlugin;
        let payload = r#"{"type":"prompt_submit","session_id":"codex-1","model":"gpt-4"}"#;
        let result = plugin.parse(payload).expect("parse should succeed").expect("parse result should be valid");
        assert_eq!(result.agent, AgentKind::Codex);
        assert_eq!(result.event, EventKind::SessionStarted);
    }
}
