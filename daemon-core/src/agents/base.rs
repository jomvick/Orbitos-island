use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

use crate::agents::traits::{PluginError, PluginResult};
use crate::state::{AgentKind, EventKind, JumpTarget, PermissionRequest, QuestionPrompt, UniversalEvent};

#[derive(Deserialize)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum BaseEvent {
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
        permission: Option<BasePermission>,
        question: Option<BaseQuestion>,
        terminal: Option<String>,
        pane: Option<String>,
        current_action: Option<String>,
        metadata: Option<serde_json::Value>,
        /// OS PID forwarded by shell wrapper
        pid: Option<u32>,
    },
    Raw(serde_json::Value),
}

#[derive(Deserialize)]
pub struct BasePermission {
    pub command: Option<String>,
    pub description: Option<String>,
    pub context: Option<String>,
}

#[derive(Deserialize)]
pub struct BaseQuestion {
    pub text: Option<String>,
    pub options: Option<Vec<String>>,
    pub context: Option<String>,
}

pub trait SimplePlugin {
    fn name(&self) -> &'static str;
    fn agent_kind(&self) -> AgentKind;
    fn map_event_type(&self, raw_type: &str) -> Option<EventKind>;

    fn parse_base(&self, payload: &str) -> PluginResult {
        let event: BaseEvent =
            serde_json::from_str(payload).map_err(|e| PluginError::ParseError(e.to_string()))?;

        match event {
            BaseEvent::Typed {
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
                question,
                terminal,
                pane,
                current_action,
                metadata,
                pid,
            } => {
                let event_kind = match self.map_event_type(&event_type) {
                    Some(k) => k,
                    None => return Ok(None),
                };

                let permission_req = permission.map(|p| PermissionRequest {
                    id: Uuid::new_v4(),
                    command: p.command.unwrap_or_default(),
                    description: p.description.unwrap_or_default(),
                    context: p.context,
                    created_at: Utc::now(),
                    expires_at: Utc::now() + chrono::Duration::minutes(5),
                });

                let question_prompt = question.map(|q| QuestionPrompt {
                    id: Uuid::new_v4(),
                    question: q.text.unwrap_or_default(),
                    options: q.options.unwrap_or_default(),
                    context: q.context,
                    created_at: Utc::now(),
                });

                let jump_target = if event_kind == EventKind::SessionStarted {
                    terminal.as_ref().map(|term| JumpTarget {
                        session_id: session_id
                            .as_deref()
                            .and_then(|s| Uuid::parse_str(s).ok())
                            .unwrap_or_else(Uuid::new_v4),
                        terminal: term.clone(),
                        pane: pane.clone(),
                        cwd: cwd.clone(),
                        pid,
                    })
                } else {
                    None
                };

                Ok(Some(UniversalEvent {
                    id: Uuid::new_v4(),
                    agent: self.agent_kind(),
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
                    current_action,
                    permission: permission_req,
                    question: question_prompt,
                    jump_target,
                    plan: None,
                    diff: None,
                    error,
                    metadata,
                    pid,
                    timestamp: Utc::now(),
                }))
            }
            BaseEvent::Raw(val) => {
                let event: UniversalEvent = serde_json::from_value(val)
                    .map_err(|e| PluginError::ParseError(e.to_string()))?;
                Ok(Some(event))
            }
        }
    }
}
