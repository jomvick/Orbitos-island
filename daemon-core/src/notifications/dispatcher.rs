use crate::notifications::{Notification, NotificationCategory, NotificationPriority};
use crate::state::{EventKind, SessionPhase, UniversalEvent};

const MAX_BODY_LENGTH: usize = 120;

fn truncate(s: &str) -> String {
    if s.len() > MAX_BODY_LENGTH {
        format!("{}…", &s[..MAX_BODY_LENGTH])
    } else {
        s.to_string()
    }
}

pub fn event_to_notification(event: &UniversalEvent) -> Option<Notification> {
    let agent = event.agent.to_string();

    match event.event {
        EventKind::SessionStarted => Some(Notification::new(
            event.agent.clone(),
            format!("{} started", agent),
            truncate(event.cwd.as_deref().unwrap_or("")),
            NotificationPriority::Low,
            NotificationCategory::Info,
            false,
        )),

        EventKind::SessionCompleted => {
            let duration = event
                .duration_ms
                .map(|ms| format!("{:.1}s", ms as f64 / 1000.0))
                .unwrap_or_default();
            let tokens = event
                .tokens_input
                .zip(event.tokens_output)
                .map(|(i, o)| format!("{} in / {} out", i, o))
                .unwrap_or_default();

            Some(Notification::new(
                event.agent.clone(),
                format!("{} completed", agent),
                [duration.as_str(), tokens.as_str()]
                    .iter()
                    .filter(|s| !s.is_empty())
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(" · "),
                NotificationPriority::Normal,
                NotificationCategory::TaskComplete,
                false,
            ))
        }

        EventKind::SessionFailed => {
            let error_msg = event.error.as_deref().unwrap_or("unknown error");
            Some(Notification::new(
                event.agent.clone(),
                format!("{} failed", agent),
                truncate(error_msg),
                NotificationPriority::High,
                NotificationCategory::Error,
                false,
            ))
        }

        EventKind::PermissionRequested => {
            let desc = event
                .permission
                .as_ref()
                .map(|p| p.description.as_str())
                .unwrap_or("permission requested");
            Some(Notification::new(
                event.agent.clone(),
                format!("{} needs permission", agent),
                truncate(desc),
                NotificationPriority::Urgent,
                NotificationCategory::PermissionRequest,
                true,
            ))
        }

        EventKind::QuestionAsked => {
            let q = event
                .question
                .as_ref()
                .map(|q| q.question.as_str())
                .unwrap_or("has a question");
            Some(Notification::new(
                event.agent.clone(),
                format!("{} asks", agent),
                truncate(q),
                NotificationPriority::High,
                NotificationCategory::Info,
                true,
            ))
        }

        EventKind::Heartbeat | EventKind::ActivityUpdated | EventKind::TokenUsage => None,

        EventKind::JumpTargetUpdated
        | EventKind::ActionableStateResolved
        | EventKind::SessionPaused
        | EventKind::PlanProposed
        | EventKind::PlanApproved
        | EventKind::PlanRejected
        | EventKind::DiffAvailable
        | EventKind::DiffApplied
        | EventKind::DiffRejected => None,
    }
}

pub fn phase_to_notification(
    agent: &str,
    old_phase: &SessionPhase,
    new_phase: &SessionPhase,
) -> Option<Notification> {
    match (old_phase, new_phase) {
        (_, SessionPhase::Failed) => Some(Notification::new(
            crate::state::AgentKind::Custom(agent.to_string()),
            format!("{} failed", agent),
            "session encountered an error".to_string(),
            NotificationPriority::High,
            NotificationCategory::Error,
            false,
        )),
        (_, SessionPhase::Completed) => Some(Notification::new(
            crate::state::AgentKind::Custom(agent.to_string()),
            format!("{} completed", agent),
            "session finished".to_string(),
            NotificationPriority::Normal,
            NotificationCategory::TaskComplete,
            false,
        )),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{AgentKind, PermissionRequest, QuestionPrompt};
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;

    #[allow(clippy::too_many_arguments)]
    fn make_event(
        kind: EventKind,
        agent: AgentKind,
        cwd: Option<&str>,
        error: Option<&str>,
        has_permission: bool,
        has_question: bool,
        tokens: (Option<u64>, Option<u64>),
        duration: Option<u64>,
    ) -> UniversalEvent {
        UniversalEvent {
            id: Uuid::new_v4(),
            agent,
            event: kind,
            session_id: "sess-1".to_string(),
            cwd: cwd.map(String::from),
            branch: None,
            model: None,
            tokens_input: tokens.0,
            tokens_output: tokens.1,
            duration_ms: duration,
            terminal: None,
            pane: None,
            permission: has_permission.then(|| PermissionRequest {
                id: Uuid::new_v4(),
                command: "test command".to_string(),
                description: "test description".to_string(),
                context: None,
                created_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
                expires_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 5, 0).unwrap(),
            }),
            question: has_question.then(|| QuestionPrompt {
                id: Uuid::new_v4(),
                question: "Continue?".to_string(),
                options: vec!["yes".to_string(), "no".to_string()],
                context: None,
                created_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
            }),
            jump_target: None,
            plan: None,
            diff: None,
            error: error.map(String::from),
            current_action: None,
            metadata: None,
            pid: None,
            timestamp: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_session_start_notification() {
        let event = make_event(
            EventKind::SessionStarted,
            AgentKind::Opencode,
            Some("/project"),
            None,
            false,
            false,
            (None, None),
            None,
        );
        let n = event_to_notification(&event).unwrap();
        assert_eq!(n.title, "opencode started");
        assert_eq!(n.priority, NotificationPriority::Low);
        assert_eq!(n.category, NotificationCategory::Info);
    }

    #[test]
    fn test_session_completed_notification() {
        let event = make_event(
            EventKind::SessionCompleted,
            AgentKind::Claude,
            None,
            None,
            false,
            false,
            (Some(5000), Some(2000)),
            Some(120000),
        );
        let n = event_to_notification(&event).unwrap();
        assert_eq!(n.title, "claude completed");
        assert_eq!(n.priority, NotificationPriority::Normal);
        assert_eq!(n.category, NotificationCategory::TaskComplete);
    }

    #[test]
    fn test_session_failed_notification() {
        let event = make_event(
            EventKind::SessionFailed,
            AgentKind::Codex,
            None,
            Some("connection timeout"),
            false,
            false,
            (None, None),
            None,
        );
        let n = event_to_notification(&event).unwrap();
        assert_eq!(n.title, "codex failed");
        assert_eq!(n.priority, NotificationPriority::High);
        assert_eq!(n.category, NotificationCategory::Error);
        assert!(n.body.contains("connection timeout"));
    }

    #[test]
    fn test_permission_request_notification() {
        let event = make_event(
            EventKind::PermissionRequested,
            AgentKind::Opencode,
            None,
            None,
            true,
            false,
            (None, None),
            None,
        );
        let n = event_to_notification(&event).unwrap();
        assert_eq!(n.title, "opencode needs permission");
        assert_eq!(n.priority, NotificationPriority::Urgent);
        assert_eq!(n.category, NotificationCategory::PermissionRequest);
        assert!(n.actionable);
    }

    #[test]
    fn test_question_asked_notification() {
        let event = make_event(
            EventKind::QuestionAsked,
            AgentKind::Claude,
            None,
            None,
            false,
            true,
            (None, None),
            None,
        );
        let n = event_to_notification(&event).unwrap();
        assert!(n.title.contains("asks"));
        assert_eq!(n.priority, NotificationPriority::High);
        assert!(n.actionable);
    }

    #[test]
    fn test_heartbeat_no_notification() {
        let event = make_event(
            EventKind::Heartbeat,
            AgentKind::Opencode,
            None,
            None,
            false,
            false,
            (None, None),
            None,
        );
        assert!(event_to_notification(&event).is_none());
    }

    #[test]
    fn test_activity_no_notification() {
        let event = make_event(
            EventKind::ActivityUpdated,
            AgentKind::Opencode,
            None,
            None,
            false,
            false,
            (None, None),
            None,
        );
        assert!(event_to_notification(&event).is_none());
    }

    #[test]
    fn test_token_usage_no_notification() {
        let event = make_event(
            EventKind::TokenUsage,
            AgentKind::Opencode,
            None,
            None,
            false,
            false,
            (None, None),
            None,
        );
        assert!(event_to_notification(&event).is_none());
    }

    #[test]
    fn test_phase_to_notification_failed() {
        let n = phase_to_notification(
            "opencode",
            &SessionPhase::Running,
            &SessionPhase::Failed,
        )
        .unwrap();
        assert_eq!(n.title, "opencode failed");
        assert_eq!(n.priority, NotificationPriority::High);
    }

    #[test]
    fn test_phase_to_notification_completed() {
        let n = phase_to_notification(
            "opencode",
            &SessionPhase::Running,
            &SessionPhase::Completed,
        )
        .unwrap();
        assert_eq!(n.title, "opencode completed");
        assert_eq!(n.priority, NotificationPriority::Normal);
    }

    #[test]
    fn test_phase_to_notification_noop() {
        let n = phase_to_notification(
            "opencode",
            &SessionPhase::Running,
            &SessionPhase::Running,
        );
        assert!(n.is_none());
    }

    #[test]
    fn test_session_start_without_cwd() {
        let event = make_event(
            EventKind::SessionStarted,
            AgentKind::Antigravity,
            None,
            None,
            false,
            false,
            (None, None),
            None,
        );
        let n = event_to_notification(&event).unwrap();
        assert_eq!(n.title, "antigravity started");
        assert_eq!(n.body, "");
    }

    #[test]
    fn test_session_completed_with_duration_only() {
        let event = make_event(
            EventKind::SessionCompleted,
            AgentKind::Codex,
            None,
            None,
            false,
            false,
            (None, None),
            Some(45000),
        );
        let n = event_to_notification(&event).unwrap();
        assert!(n.body.contains("45.0s"));
    }

    #[test]
    fn test_priority_order() {
        assert!(NotificationPriority::Urgent as u8 > NotificationPriority::Low as u8);
    }
}

