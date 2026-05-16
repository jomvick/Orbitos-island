use chrono::Utc;
use daemon_core::agents::{AgentPlugin, PluginError, PluginResult};
use daemon_core::state::{AgentKind, EventKind, UniversalEvent};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
struct AiderHookPayload {
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
    terminal: Option<String>,
    pane: Option<String>,
    metadata: Option<serde_json::Value>,
}

pub struct AiderPlugin;

impl AgentPlugin for AiderPlugin {
    fn name(&self) -> &'static str {
        "aider"
    }

    fn agent_kind(&self) -> AgentKind {
        AgentKind::Aider
    }

    fn parse(&self, payload: &str) -> PluginResult {
        let hook: AiderHookPayload =
            serde_json::from_str(payload).map_err(|e| PluginError::ParseError(e.to_string()))?;

        let event_kind = match hook.event_type.as_str() {
            "session_start" | "task_start" | "start" => EventKind::SessionStarted,
            "session_complete" | "task_complete" | "complete" | "done" => {
                EventKind::SessionCompleted
            }
            "session_failed" | "error" | "fail" => EventKind::SessionFailed,
            "activity" | "progress" | "think" => EventKind::ActivityUpdated,
            "heartbeat" => EventKind::Heartbeat,
            "token_usage" => EventKind::TokenUsage,
            _ => return Err(PluginError::UnsupportedEvent(hook.event_type)),
        };

        Ok(Some(UniversalEvent {
            id: Uuid::new_v4(),
            agent: AgentKind::Aider,
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
            terminal: hook.terminal,
            pane: hook.pane,
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
