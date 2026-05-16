pub mod dispatcher;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::AgentKind;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Urgent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: Uuid,
    pub agent: AgentKind,
    pub title: String,
    pub body: String,
    pub priority: NotificationPriority,
    pub category: NotificationCategory,
    pub actionable: bool,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationCategory {
    TaskComplete,
    PermissionRequest,
    Error,
    Warning,
    Info,
    Progress,
}

impl Notification {
    pub fn new(
        agent: AgentKind,
        title: String,
        body: String,
        priority: NotificationPriority,
        category: NotificationCategory,
        actionable: bool,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            agent,
            title,
            body,
            priority,
            category,
            actionable,
            timestamp: Utc::now(),
        }
    }
}
