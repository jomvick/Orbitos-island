use chrono::Utc;
use daemon_core::agents::{AgentPlugin, PluginError, PluginResult};
use daemon_core::state::{AgentKind, EventKind, PermissionRequest, UniversalEvent};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
enum OpenCodeEvent {
    Typed {
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
        permission: Option<PermissionPayload>,
        terminal: Option<String>,
        pane: Option<String>,
        metadata: Option<serde_json::Value>,
    },
    Raw(serde_json::Value),
}

#[derive(Deserialize)]
struct PermissionPayload {
    command: Option<String>,
    description: Option<String>,
}

pub struct OpenCodePlugin;

impl AgentPlugin for OpenCodePlugin {
    fn name(&self) -> &'static str {
        "opencode"
    }

    fn agent_kind(&self) -> AgentKind {
        AgentKind::Opencode
    }

    fn parse(&self, payload: &str) -> PluginResult {
        let event: OpenCodeEvent =
            serde_json::from_str(payload).map_err(|e| PluginError::ParseError(e.to_string()))?;

        match event {
            OpenCodeEvent::Typed {
                event_type,
                session_id,
                cwd,
                branch,
                model,
                tokens_input,
                tokens_output,
                duration_ms,
                error,
                permission,
                terminal,
                pane,
                metadata,
            } => {
                let event_kind = match event_type.as_str() {
                    "session_start" => EventKind::SessionStarted,
                    "session_complete" => EventKind::SessionCompleted,
                    "session_failed" => EventKind::SessionFailed,
                    "activity" => EventKind::ActivityUpdated,
                    "permission" => EventKind::PermissionRequested,
                    "question" => EventKind::QuestionAsked,
                    "heartbeat" => EventKind::Heartbeat,
                    "token_usage" => EventKind::TokenUsage,
                    _ => return Err(PluginError::UnsupportedEvent(event_type)),
                };

                let permission_req = permission.map(|p| PermissionRequest {
                    id: Uuid::new_v4(),
                    command: p.command.unwrap_or_default(),
                    description: p.description.unwrap_or_default(),
                    context: None,
                    created_at: Utc::now(),
                    expires_at: Utc::now() + chrono::Duration::minutes(5),
                });

                Ok(Some(UniversalEvent {
                    id: Uuid::new_v4(),
                    agent: AgentKind::Opencode,
                    event: event_kind,
                    session_id: session_id.unwrap_or_else(|| Uuid::new_v4().to_string()),
                    cwd,
                    branch,
                    model,
                    tokens_input,
                    tokens_output,
                    duration_ms,
                    terminal,
                    pane,
                    permission: permission_req,
                    question: None,
                    jump_target: None,
                    error,
                    metadata,
                    timestamp: Utc::now(),
                }))
            }
            OpenCodeEvent::Raw(val) => {
                let event: UniversalEvent = serde_json::from_value(val)
                    .map_err(|e| PluginError::ParseError(e.to_string()))?;
                Ok(Some(event))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_session_start() {
        let plugin = OpenCodePlugin;
        let payload = r#"{"type":"session_start","session_id":"abc123","model":"claude-sonnet-4"}"#;
        let result = plugin.parse(payload).unwrap().unwrap();
        assert_eq!(result.agent, AgentKind::Opencode);
        assert_eq!(result.event, EventKind::SessionStarted);
        assert_eq!(result.session_id, "abc123");
    }

    #[test]
    fn test_parse_session_complete() {
        let plugin = OpenCodePlugin;
        let payload = r#"{"type":"session_complete","session_id":"abc123","tokens_input":100,"tokens_output":50}"#;
        let result = plugin.parse(payload).unwrap().unwrap();
        assert_eq!(result.event, EventKind::SessionCompleted);
        assert_eq!(result.tokens_input, Some(100));
    }

    #[test]
    fn test_parse_invalid_json() {
        let plugin = OpenCodePlugin;
        let result = plugin.parse("not json");
        assert!(result.is_err());
    }
}
