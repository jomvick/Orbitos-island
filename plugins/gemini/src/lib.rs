use chrono::Utc;
use daemon_core::agents::{AgentPlugin, PluginError, PluginResult};
use daemon_core::state::{AgentKind, EventKind, PermissionRequest, UniversalEvent};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
struct GeminiHookPayload {
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
    permission: Option<GeminiPermission>,
    terminal: Option<String>,
    pane: Option<String>,
    metadata: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct GeminiPermission {
    command: Option<String>,
    description: Option<String>,
    context: Option<String>,
}

pub struct GeminiPlugin;

impl AgentPlugin for GeminiPlugin {
    fn name(&self) -> &'static str {
        "gemini"
    }

    fn agent_kind(&self) -> AgentKind {
        AgentKind::Gemini
    }

    fn parse(&self, payload: &str) -> PluginResult {
        let hook: GeminiHookPayload =
            serde_json::from_str(payload).map_err(|e| PluginError::ParseError(e.to_string()))?;

        let event_kind = match hook.event_type.as_str() {
            "session_start" | "task_start" => EventKind::SessionStarted,
            "session_complete" | "task_complete" | "done" => EventKind::SessionCompleted,
            "session_failed" | "error" | "task_failed" => EventKind::SessionFailed,
            "activity" | "progress" | "think" => EventKind::ActivityUpdated,
            "permission" | "permission_request" => EventKind::PermissionRequested,
            "heartbeat" => EventKind::Heartbeat,
            "token_usage" => EventKind::TokenUsage,
            "paused" | "waiting" => EventKind::SessionPaused,
            _ => return Err(PluginError::UnsupportedEvent(hook.event_type)),
        };

        let permission = hook.permission.map(|p| PermissionRequest {
            id: Uuid::new_v4(),
            command: p.command.unwrap_or_default(),
            description: p.description.unwrap_or_default(),
            context: p.context,
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::minutes(5),
        });

        Ok(Some(UniversalEvent {
            id: Uuid::new_v4(),
            agent: AgentKind::Gemini,
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
            permission,
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
        let plugin = GeminiPlugin;
        let payload =
            r#"{"type":"session_start","session_id":"gemini-1","model":"gemini-2.0-pro"}"#;
        let result = plugin.parse(payload).unwrap().unwrap();
        assert_eq!(result.agent, AgentKind::Gemini);
        assert_eq!(result.event, EventKind::SessionStarted);
    }

    #[test]
    fn test_parse_task_complete() {
        let plugin = GeminiPlugin;
        let payload = r#"{"type":"done","session_id":"gemini-1","tokens_input":4000,"tokens_output":2000}"#;
        let result = plugin.parse(payload).unwrap().unwrap();
        assert_eq!(result.event, EventKind::SessionCompleted);
        assert_eq!(result.tokens_input, Some(4000));
    }

    #[test]
    fn test_parse_permission_request() {
        let plugin = GeminiPlugin;
        let payload = r#"{"type":"permission_request","session_id":"gemini-1","permission":{"command":"rm -rf /tmp","description":"Clean temp"}}"#;
        let result = plugin.parse(payload).unwrap().unwrap();
        assert_eq!(result.event, EventKind::PermissionRequested);
        assert!(result.permission.is_some());
    }

    #[test]
    fn test_parse_invalid_json() {
        let plugin = GeminiPlugin;
        let result = plugin.parse("not json");
        assert!(result.is_err());
    }
}
