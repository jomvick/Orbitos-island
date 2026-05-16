use chrono::Utc;
use daemon_core::agents::{AgentPlugin, PluginError, PluginResult};
use daemon_core::state::{AgentKind, EventKind, UniversalEvent};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
struct CodexHookPayload {
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

pub struct CodexPlugin;

impl AgentPlugin for CodexPlugin {
    fn name(&self) -> &'static str {
        "codex"
    }

    fn agent_kind(&self) -> AgentKind {
        AgentKind::Codex
    }

    fn parse(&self, payload: &str) -> PluginResult {
        let hook: CodexHookPayload =
            serde_json::from_str(payload).map_err(|e| PluginError::ParseError(e.to_string()))?;

        let event_kind = match hook.event_type.as_str() {
            "session_start" | "task_start" | "prompt_submit" => EventKind::SessionStarted,
            "session_complete" | "task_complete" | "stop" => EventKind::SessionCompleted,
            "session_failed" | "error" => EventKind::SessionFailed,
            "activity" | "progress" | "shell_execution" => EventKind::ActivityUpdated,
            "heartbeat" => EventKind::Heartbeat,
            "token_usage" => EventKind::TokenUsage,
            _ => return Err(PluginError::UnsupportedEvent(hook.event_type)),
        };

        Ok(Some(UniversalEvent {
            id: Uuid::new_v4(),
            agent: AgentKind::Codex,
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
    fn test_parse_prompt_submit() {
        let plugin = CodexPlugin;
        let payload = r#"{"type":"prompt_submit","session_id":"codex-1","model":"gpt-4"}"#;
        let result = plugin.parse(payload).unwrap().unwrap();
        assert_eq!(result.agent, AgentKind::Codex);
        assert_eq!(result.event, EventKind::SessionStarted);
    }
}
