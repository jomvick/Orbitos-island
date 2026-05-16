use chrono::Utc;
use daemon_core::agents::{AgentPlugin, PluginError, PluginResult};
use daemon_core::state::{AgentKind, EventKind, UniversalEvent};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
struct AntigravityHookPayload {
    #[serde(rename = "type")]
    event_type: String,
    session_id: Option<String>,
    cwd: Option<String>,
    branch: Option<String>,
    model: Option<String>,
    tokens_input: Option<u64>,
    tokens_output: Option<u64>,
    duration_ms: Option<u64>,
    error: Option<String>,
    metadata: Option<serde_json::Value>,
}

pub struct AntigravityPlugin;

impl AgentPlugin for AntigravityPlugin {
    fn name(&self) -> &'static str {
        "antigravity"
    }

    fn agent_kind(&self) -> AgentKind {
        AgentKind::Antigravity
    }

    fn parse(&self, payload: &str) -> PluginResult {
        let hook: AntigravityHookPayload =
            serde_json::from_str(payload).map_err(|e| PluginError::ParseError(e.to_string()))?;

        let event_kind = match hook.event_type.as_str() {
            "agent_start" | "session_start" => EventKind::SessionStarted,
            "agent_complete" | "session_complete" => EventKind::SessionCompleted,
            "agent_error" | "error" => EventKind::SessionFailed,
            "agent_activity" | "activity" => EventKind::ActivityUpdated,
            "heartbeat" => EventKind::Heartbeat,
            _ => return Err(PluginError::UnsupportedEvent(hook.event_type)),
        };

        Ok(Some(UniversalEvent {
            id: Uuid::new_v4(),
            agent: AgentKind::Antigravity,
            event: event_kind,
            session_id: hook
                .session_id
                .unwrap_or_else(|| Uuid::new_v4().to_string()),
            cwd: hook.cwd,
            branch: hook.branch,
            model: hook.model,
            tokens_input: hook.tokens_input,
            tokens_output: hook.tokens_output,
            duration_ms: hook.duration_ms,
            terminal: None,
            pane: None,
            permission: None,
            question: None,
            jump_target: None,
            error: hook.error,
            metadata: hook.metadata,
            plan: None,
            diff: None,
            timestamp: Utc::now(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_session_start() {
        let plugin = AntigravityPlugin;
        let payload = r#"{"type":"session_start","session_id":"ag-1","model":"claude-sonnet-4"}"#;
        let result = plugin.parse(payload).unwrap().unwrap();
        assert_eq!(result.agent, AgentKind::Antigravity);
        assert_eq!(result.event, EventKind::SessionStarted);
        assert_eq!(result.session_id, "ag-1");
    }

    #[test]
    fn test_parse_agent_complete() {
        let plugin = AntigravityPlugin;
        let payload = r#"{"type":"agent_complete","session_id":"ag-1","tokens_input":2000,"tokens_output":1000}"#;
        let result = plugin.parse(payload).unwrap().unwrap();
        assert_eq!(result.event, EventKind::SessionCompleted);
        assert_eq!(result.tokens_input, Some(2000));
        assert_eq!(result.tokens_output, Some(1000));
    }

    #[test]
    fn test_parse_agent_error() {
        let plugin = AntigravityPlugin;
        let payload = r#"{"type":"agent_error","session_id":"ag-1","error":"rate limit exceeded"}"#;
        let result = plugin.parse(payload).unwrap().unwrap();
        assert_eq!(result.event, EventKind::SessionFailed);
        assert_eq!(result.error.as_deref(), Some("rate limit exceeded"));
    }

    #[test]
    fn test_parse_activity() {
        let plugin = AntigravityPlugin;
        let payload = r#"{"type":"agent_activity","session_id":"ag-1","cwd":"/project"}"#;
        let result = plugin.parse(payload).unwrap().unwrap();
        assert_eq!(result.event, EventKind::ActivityUpdated);
        assert_eq!(result.cwd.as_deref(), Some("/project"));
    }

    #[test]
    fn test_parse_heartbeat() {
        let plugin = AntigravityPlugin;
        let payload = r#"{"type":"heartbeat","session_id":"ag-1"}"#;
        let result = plugin.parse(payload).unwrap().unwrap();
        assert_eq!(result.event, EventKind::Heartbeat);
    }

    #[test]
    fn test_parse_unsupported_event() {
        let plugin = AntigravityPlugin;
        let payload = r#"{"type":"unknown_event","session_id":"ag-1"}"#;
        let result = plugin.parse(payload);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_json() {
        let plugin = AntigravityPlugin;
        let result = plugin.parse("not json");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_session_id_generated() {
        let plugin = AntigravityPlugin;
        let payload = r#"{"type":"session_start"}"#;
        let result = plugin.parse(payload).unwrap().unwrap();
        assert!(!result.session_id.is_empty());
        assert_eq!(result.agent, AgentKind::Antigravity);
    }
}
