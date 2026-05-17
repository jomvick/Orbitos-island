use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use daemon_core::state::{AgentKind, AgentSession, UniversalEvent};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IpcMessage {
    Event {
        source: String,
        payload: serde_json::Value,
        timestamp: DateTime<Utc>,
    },
    Command {
        id: Uuid,
        command: IpcCommand,
        timestamp: DateTime<Utc>,
    },
    Response {
        id: Uuid,
        status: IpcStatus,
        data: Option<serde_json::Value>,
        error: Option<String>,
        timestamp: DateTime<Utc>,
    },
    Subscribe {
        channel: String,
        timestamp: DateTime<Utc>,
    },
    SubscriptionEvent {
        channel: String,
        event: Box<UniversalEvent>,
        #[serde(skip_serializing_if = "Option::is_none")]
        session: Option<AgentSession>,
        timestamp: DateTime<Utc>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum IpcCommand {
    ResolvePermission {
        permission_id: Uuid,
        approved: bool,
        response: Option<String>,
    },
    AnswerQuestion {
        question_id: Uuid,
        answer: String,
    },
    JumpToSession {
        session_id: String,
    },
    StopAgent {
        session_id: String,
    },
    GetSessions {
        filter: Option<SessionFilter>,
    },
    GetSession {
        session_id: String,
    },
    GetSessionStats,
    GetAgentAnalytics,
    GetTimeline {
        limit: u32,
    },
    SearchSessions {
        query: String,
    },
    DiscoverAgents,
    Shutdown,
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionFilter {
    All,
    Active,
    ByAgent(AgentKind),
    ByPhase(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IpcStatus {
    Ok,
    Error,
    Timeout,
}

impl IpcMessage {
    pub fn new_event(source: &str, payload: serde_json::Value) -> Self {
        Self::Event {
            source: source.to_string(),
            payload,
            timestamp: Utc::now(),
        }
    }

    pub fn new_command(command: IpcCommand) -> Self {
        Self::Command {
            id: Uuid::new_v4(),
            command,
            timestamp: Utc::now(),
        }
    }

    pub fn new_response(id: Uuid, data: Option<serde_json::Value>) -> Self {
        Self::Response {
            id,
            status: IpcStatus::Ok,
            data,
            error: None,
            timestamp: Utc::now(),
        }
    }

    pub fn new_error(id: Uuid, error: String) -> Self {
        Self::Response {
            id,
            status: IpcStatus::Error,
            data: None,
            error: Some(error),
            timestamp: Utc::now(),
        }
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            IpcMessage::Event { timestamp, .. } => *timestamp,
            IpcMessage::Command { timestamp, .. } => *timestamp,
            IpcMessage::Response { timestamp, .. } => *timestamp,
            IpcMessage::Subscribe { timestamp, .. } => *timestamp,
            IpcMessage::SubscriptionEvent { timestamp, .. } => *timestamp,
        }
    }
}
