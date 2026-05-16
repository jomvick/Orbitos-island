use chrono::Utc;
use daemon_core::agents::{AgentPlugin, PluginError, PluginResult};
use daemon_core::state::{AgentKind, EventKind, PermissionRequest, QuestionPrompt, UniversalEvent};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
struct ClaudeHookPayload {
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
    permission: Option<ClaudePermission>,
    question: Option<ClaudeQuestion>,
    terminal: Option<String>,
    pane: Option<String>,
    metadata: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct ClaudePermission {
    command: Option<String>,
    description: Option<String>,
    context: Option<String>,
}

#[derive(Deserialize)]
struct ClaudeQuestion {
    text: Option<String>,
    options: Option<Vec<String>>,
    context: Option<String>,
}

pub struct ClaudePlugin;

impl AgentPlugin for ClaudePlugin {
    fn name(&self) -> &'static str {
        "claude"
    }

    fn agent_kind(&self) -> AgentKind {
        AgentKind::Claude
    }

    fn parse(&self, payload: &str) -> PluginResult {
        let hook: ClaudeHookPayload =
            serde_json::from_str(payload).map_err(|e| PluginError::ParseError(e.to_string()))?;

        let event_kind = match hook.event_type.as_str() {
            "session_start" | "task_start" => EventKind::SessionStarted,
            "session_complete" | "task_complete" => EventKind::SessionCompleted,
            "session_failed" | "task_failed" | "error" => EventKind::SessionFailed,
            "activity" | "progress" => EventKind::ActivityUpdated,
            "permission" | "permission_request" => EventKind::PermissionRequested,
            "question" | "ask" => EventKind::QuestionAsked,
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

        let question = hook.question.map(|q| QuestionPrompt {
            id: Uuid::new_v4(),
            question: q.text.unwrap_or_default(),
            options: q.options.unwrap_or_default(),
            context: q.context,
            created_at: Utc::now(),
        });

        Ok(Some(UniversalEvent {
            id: Uuid::new_v4(),
            agent: AgentKind::Claude,
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
            question,
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
    fn test_parse_task_complete() {
        let plugin = ClaudePlugin;
        let payload = r#"{"type":"task_complete","session_id":"sess1","tokens_input":5000,"tokens_output":2000}"#;
        let result = plugin.parse(payload).unwrap().unwrap();
        assert_eq!(result.agent, AgentKind::Claude);
        assert_eq!(result.event, EventKind::SessionCompleted);
        assert_eq!(result.tokens_input, Some(5000));
    }

    #[test]
    fn test_parse_permission_request() {
        let plugin = ClaudePlugin;
        let payload = r#"{"type":"permission_request","session_id":"sess1","permission":{"command":"sudo apt install","description":"Install package"}}"#;
        let result = plugin.parse(payload).unwrap().unwrap();
        assert_eq!(result.event, EventKind::PermissionRequested);
        assert!(result.permission.is_some());
    }
}
