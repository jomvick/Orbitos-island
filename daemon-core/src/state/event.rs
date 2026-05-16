use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AgentKind {
    Opencode,
    Claude,
    Codex,
    Antigravity,
    Aider,
    Gemini,
    Cursor,
    Kimi,
    Qoder,
    Qwen,
    Factory,
    Codebuddy,
    Copilot,
    DeepSeek,
    Custom(String),
}

impl std::fmt::Display for AgentKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentKind::Opencode => write!(f, "opencode"),
            AgentKind::Claude => write!(f, "claude"),
            AgentKind::Codex => write!(f, "codex"),
            AgentKind::Antigravity => write!(f, "antigravity"),
            AgentKind::Aider => write!(f, "aider"),
            AgentKind::Gemini => write!(f, "gemini"),
            AgentKind::Cursor => write!(f, "cursor"),
            AgentKind::Kimi => write!(f, "kimi"),
            AgentKind::Qoder => write!(f, "qoder"),
            AgentKind::Qwen => write!(f, "qwen"),
            AgentKind::Factory => write!(f, "factory"),
            AgentKind::Codebuddy => write!(f, "codebuddy"),
            AgentKind::Copilot => write!(f, "copilot"),
            AgentKind::DeepSeek => write!(f, "deepseek"),
            AgentKind::Custom(name) => write!(f, "{}", name),
        }
    }
}

impl std::str::FromStr for AgentKind {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "opencode" => Self::Opencode,
            "claude" => Self::Claude,
            "codex" => Self::Codex,
            "antigravity" => Self::Antigravity,
            "aider" => Self::Aider,
            "gemini" => Self::Gemini,
            "cursor" => Self::Cursor,
            "kimi" => Self::Kimi,
            "qoder" => Self::Qoder,
            "qwen" => Self::Qwen,
            "factory" => Self::Factory,
            "codebuddy" => Self::Codebuddy,
            "copilot" => Self::Copilot,
            "deepseek" => Self::DeepSeek,
            _ => Self::Custom(s.to_string()),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    SessionStarted,
    ActivityUpdated,
    PermissionRequested,
    QuestionAsked,
    SessionCompleted,
    SessionFailed,
    SessionPaused,
    Heartbeat,
    TokenUsage,
    JumpTargetUpdated,
    ActionableStateResolved,
    PlanProposed,
    PlanApproved,
    PlanRejected,
    DiffAvailable,
    DiffApplied,
    DiffRejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SessionPhase {
    Running,
    WaitingPermission,
    WaitingQuestion,
    Completed,
    Failed,
    Paused,
    Orphaned,
}

impl std::fmt::Display for SessionPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionPhase::Running => write!(f, "running"),
            SessionPhase::WaitingPermission => write!(f, "waiting_permission"),
            SessionPhase::WaitingQuestion => write!(f, "waiting_question"),
            SessionPhase::Completed => write!(f, "completed"),
            SessionPhase::Failed => write!(f, "failed"),
            SessionPhase::Paused => write!(f, "paused"),
            SessionPhase::Orphaned => write!(f, "orphaned"),
        }
    }
}

impl std::str::FromStr for SessionPhase {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "running" => Self::Running,
            "waiting_permission" => Self::WaitingPermission,
            "waiting_question" => Self::WaitingQuestion,
            "completed" => Self::Completed,
            "failed" => Self::Failed,
            "paused" => Self::Paused,
            "orphaned" => Self::Orphaned,
            _ => return Err(format!("invalid session phase: {}", s)),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRequest {
    pub id: Uuid,
    pub command: String,
    pub description: String,
    pub context: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionPrompt {
    pub id: Uuid,
    pub question: String,
    pub options: Vec<String>,
    pub context: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JumpTarget {
    pub session_id: Uuid,
    pub terminal: String,
    pub pane: Option<String>,
    pub cwd: Option<String>,
    pub pid: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub tokens_input: u64,
    pub tokens_output: u64,
    pub model: Option<String>,
    pub cost: Option<f64>,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanItem {
    pub action: String,
    pub file: Option<String>,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanProposal {
    pub id: Uuid,
    pub summary: String,
    pub items: Vec<PlanItem>,
    pub reasoning: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiff {
    pub file_path: String,
    pub diff_content: String,
    pub language: Option<String>,
    pub status: Option<String>, // "modified", "created", "deleted"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffPayload {
    pub id: Uuid,
    pub session_id: String,
    pub files: Vec<FileDiff>,
    pub summary: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalEvent {
    pub id: Uuid,
    pub agent: AgentKind,
    pub event: EventKind,
    pub session_id: String,
    pub cwd: Option<String>,
    pub branch: Option<String>,
    pub model: Option<String>,
    pub tokens_input: Option<u64>,
    pub tokens_output: Option<u64>,
    pub duration_ms: Option<u64>,
    pub terminal: Option<String>,
    pub pane: Option<String>,
    pub permission: Option<PermissionRequest>,
    pub question: Option<QuestionPrompt>,
    pub jump_target: Option<JumpTarget>,
    pub plan: Option<PlanProposal>,
    pub diff: Option<DiffPayload>,
    pub error: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

impl fmt::Display for UniversalEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "UniversalEvent {{ agent: {}, event: {}, session: {} }}",
            self.agent, self.event, self.session_id
        )
    }
}

impl fmt::Display for EventKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            EventKind::SessionStarted => "session_started",
            EventKind::ActivityUpdated => "activity_updated",
            EventKind::PermissionRequested => "permission_requested",
            EventKind::QuestionAsked => "question_asked",
            EventKind::SessionCompleted => "session_completed",
            EventKind::SessionFailed => "session_failed",
            EventKind::SessionPaused => "session_paused",
            EventKind::Heartbeat => "heartbeat",
            EventKind::TokenUsage => "token_usage",
            EventKind::JumpTargetUpdated => "jump_target_updated",
            EventKind::ActionableStateResolved => "actionable_state_resolved",
            EventKind::PlanProposed => "plan_proposed",
            EventKind::PlanApproved => "plan_approved",
            EventKind::PlanRejected => "plan_rejected",
            EventKind::DiffAvailable => "diff_available",
            EventKind::DiffApplied => "diff_applied",
            EventKind::DiffRejected => "diff_rejected",
        };
        write!(f, "{}", s)
    }
}
