use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::terminals::detector;
use crate::terminals::{TerminalId, TerminalKind};

use super::event::{
    AgentKind, DiffPayload, EventKind, JumpTarget, PermissionRequest, PlanProposal, QuestionPrompt,
    SessionPhase, UniversalEvent,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSession {
    pub id: String,
    pub agent: AgentKind,
    pub phase: SessionPhase,
    pub cwd: Option<String>,
    pub branch: Option<String>,
    pub model: Option<String>,
    pub tokens_input: u64,
    pub tokens_output: u64,
    pub duration_ms: u64,
    pub terminal: Option<String>,
    pub pane: Option<String>,
    pub permission: Option<PermissionRequest>,
    pub question: Option<QuestionPrompt>,
    pub jump_target: Option<JumpTarget>,
    pub plan: Option<PlanProposal>,
    pub diff: Option<DiffPayload>,
    pub error: Option<String>,
    pub current_action: Option<String>,
    pub metadata: Option<serde_json::Value>,
    /// OS process ID provided by the shell wrapper — used by the process watcher.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    /// Parent PID of the hook — used for terminal detection process tree walk.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ppid: Option<u32>,
    /// Detected terminal kind (Tmux, Zellij, Kitty, etc.) — stored once at SessionStart.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal_kind: Option<TerminalKind>,
    /// Precise terminal pane identifier — stored once at SessionStart.
    /// Enables direct jump without re-resolving the pane.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal_id: Option<TerminalId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
    pub event_count: u64,
}

impl AgentSession {
    pub fn new(event: &UniversalEvent) -> Self {
        // Terminal detection at session start: walk the process tree from ppid
        let (terminal_kind, terminal_id) = event
            .ppid
            .and_then(|ppid| {
                let pid = event.pid.unwrap_or(0);
                if pid == 0 {
                    return None;
                }
                detector::detect_terminal_from_pid(pid, ppid)
            })
            .map(|(k, id)| (Some(k), Some(id)))
            .unwrap_or((None, None));

        Self {
            id: event.session_id.clone(),
            agent: event.agent.clone(),
            phase: SessionPhase::Running,
            cwd: event.cwd.clone(),
            branch: event.branch.clone(),
            model: event.model.clone(),
            tokens_input: event.tokens_input.unwrap_or(0),
            tokens_output: event.tokens_output.unwrap_or(0),
            duration_ms: event.duration_ms.unwrap_or(0),
            terminal: event.terminal.clone(),
            pane: event.pane.clone(),
            permission: None,
            question: None,
            jump_target: event.jump_target.clone(),
            plan: event.plan.clone(),
            diff: event.diff.clone(),
            error: None,
            current_action: event.current_action.clone(),
            metadata: event.metadata.clone(),
            pid: event.pid,
            ppid: event.ppid,
            terminal_kind,
            terminal_id,
            created_at: event.timestamp,
            updated_at: event.timestamp,
            last_heartbeat: event.timestamp,
            event_count: 1,
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(
            self.phase,
            SessionPhase::Running | SessionPhase::WaitingPermission | SessionPhase::WaitingQuestion
        )
    }

    pub fn is_stale(&self, threshold: &chrono::Duration) -> bool {
        Utc::now() - self.last_heartbeat > *threshold
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub sessions: HashMap<String, AgentSession>,
    pub total_events_processed: u64,
    pub daemon_started_at: DateTime<Utc>,
}

impl SessionState {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            total_events_processed: 0,
            daemon_started_at: Utc::now(),
        }
    }

    pub fn active_count(&self) -> usize {
        self.sessions.values().filter(|s| s.is_active()).count()
    }

    pub fn total_count(&self) -> usize {
        self.sessions.len()
    }

    pub fn prune_orphaned(&mut self, max_age: chrono::Duration) {
        let now = Utc::now();
        self.sessions.retain(|_, s| {
            if matches!(s.phase, SessionPhase::Completed | SessionPhase::Failed) {
                now - s.updated_at < max_age
            } else {
                true
            }
        });
    }
}

impl Default for SessionState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::event::*;
    use uuid::Uuid;

    fn make_event(session_id: &str, kind: EventKind, agent: AgentKind) -> UniversalEvent {
        UniversalEvent {
            id: Uuid::new_v4(),
            agent,
            event: kind,
            session_id: session_id.to_string(),
            cwd: Some("/test".to_string()),
            branch: Some("main".to_string()),
            model: Some("test-model".to_string()),
            tokens_input: Some(100),
            tokens_output: Some(50),
            duration_ms: Some(1000),
            terminal: Some("tmux".to_string()),
            pane: Some("0".to_string()),
            permission: None,
            question: None,
            jump_target: None,
            plan: None,
            diff: None,
            error: None,
            current_action: None,
            metadata: None,
            pid: None,
            ppid: None,
            timestamp: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_session_start_creates_session() {
        let state = SessionState::new();
        let event = make_event("sess-1", EventKind::SessionStarted, AgentKind::Opencode);
        let state = apply_event(state, event);

        assert_eq!(state.sessions.len(), 1);
        let session = state.sessions.get("sess-1").unwrap();
        assert_eq!(session.agent, AgentKind::Opencode);
        assert_eq!(session.phase, SessionPhase::Running);
        assert_eq!(session.cwd.as_deref(), Some("/test"));
        assert_eq!(session.branch.as_deref(), Some("main"));
        assert_eq!(session.model.as_deref(), Some("test-model"));
        assert_eq!(session.event_count, 1);
    }

    #[test]
    fn test_session_complete_transitions_phase() {
        let state = SessionState::new();
        let start = make_event("sess-1", EventKind::SessionStarted, AgentKind::Claude);
        let state = apply_event(state, start);

        let complete = UniversalEvent {
            tokens_input: Some(5000),
            tokens_output: Some(2000),
            duration_ms: Some(120000),
            ..make_event("sess-1", EventKind::SessionCompleted, AgentKind::Claude)
        };
        let state = apply_event(state, complete);

        let session = state.sessions.get("sess-1").unwrap();
        assert_eq!(session.phase, SessionPhase::Completed);
        assert_eq!(session.tokens_input, 5000);
        assert_eq!(session.tokens_output, 2000);
        assert_eq!(session.duration_ms, 120000);
    }

    #[test]
    fn test_session_failed_sets_error() {
        let state = SessionState::new();
        let start = make_event("sess-1", EventKind::SessionStarted, AgentKind::Codex);
        let state = apply_event(state, start);

        let fail = UniversalEvent {
            error: Some("connection lost".to_string()),
            ..make_event("sess-1", EventKind::SessionFailed, AgentKind::Codex)
        };
        let state = apply_event(state, fail);

        let session = state.sessions.get("sess-1").unwrap();
        assert_eq!(session.phase, SessionPhase::Failed);
        assert_eq!(session.error.as_deref(), Some("connection lost"));
    }

    #[test]
    fn test_permission_request_sets_waiting_phase() {
        let state = SessionState::new();
        let start = make_event("sess-1", EventKind::SessionStarted, AgentKind::Opencode);
        let state = apply_event(state, start);

        let perm = UniversalEvent {
            permission: Some(PermissionRequest {
                id: Uuid::new_v4(),
                command: "git push".to_string(),
                description: "Push to remote".to_string(),
                context: None,
                diff: None,
                created_at: Utc::now(),
                expires_at: Utc::now() + chrono::Duration::minutes(5),
            }),
            ..make_event(
                "sess-1",
                EventKind::PermissionRequested,
                AgentKind::Opencode,
            )
        };
        let state = apply_event(state, perm);

        let session = state.sessions.get("sess-1").unwrap();
        assert_eq!(session.phase, SessionPhase::WaitingPermission);
        assert!(session.permission.is_some());
        assert_eq!(session.permission.as_ref().unwrap().command, "git push");
    }

    #[test]
    fn test_question_asked_sets_waiting_phase() {
        let state = SessionState::new();
        let start = make_event("sess-1", EventKind::SessionStarted, AgentKind::Claude);
        let state = apply_event(state, start);

        let question = UniversalEvent {
            question: Some(QuestionPrompt {
                id: Uuid::new_v4(),
                question: "Continue?".to_string(),
                options: vec!["yes".to_string(), "no".to_string()],
                context: None,
                created_at: Utc::now(),
            }),
            ..make_event("sess-1", EventKind::QuestionAsked, AgentKind::Claude)
        };
        let state = apply_event(state, question);

        let session = state.sessions.get("sess-1").unwrap();
        assert_eq!(session.phase, SessionPhase::WaitingQuestion);
        assert_eq!(session.question.as_ref().unwrap().question, "Continue?");
        assert_eq!(session.question.as_ref().unwrap().options.len(), 2);
    }

    #[test]
    fn test_actionable_state_resolved_returns_to_running() {
        let state = SessionState::new();
        let start = make_event("sess-1", EventKind::SessionStarted, AgentKind::Opencode);
        let state = apply_event(state, start);

        let perm = UniversalEvent {
            permission: Some(PermissionRequest {
                id: Uuid::new_v4(),
                command: "rm -rf".to_string(),
                description: "Dangerous".to_string(),
                context: None,
                diff: None,
                created_at: Utc::now(),
                expires_at: Utc::now() + chrono::Duration::minutes(5),
            }),
            ..make_event(
                "sess-1",
                EventKind::PermissionRequested,
                AgentKind::Opencode,
            )
        };
        let state = apply_event(state, perm);

        let resolve = make_event(
            "sess-1",
            EventKind::ActionableStateResolved,
            AgentKind::Opencode,
        );
        let state = apply_event(state, resolve);

        let session = state.sessions.get("sess-1").unwrap();
        assert_eq!(session.phase, SessionPhase::Running);
        assert!(session.permission.is_none());
        assert!(session.question.is_none());
    }

    #[test]
    fn test_multiple_sessions_independent() {
        let state = SessionState::new();

        let s1 = make_event("sess-1", EventKind::SessionStarted, AgentKind::Opencode);
        let state = apply_event(state, s1);

        let s2 = make_event("sess-2", EventKind::SessionStarted, AgentKind::Claude);
        let state = apply_event(state, s2);

        let s3 = make_event("sess-3", EventKind::SessionStarted, AgentKind::Codex);
        let state = apply_event(state, s3);

        assert_eq!(state.sessions.len(), 3);
        assert_eq!(state.active_count(), 3);

        let complete = make_event("sess-2", EventKind::SessionCompleted, AgentKind::Claude);
        let state = apply_event(state, complete);

        assert_eq!(state.active_count(), 2);
        assert_eq!(
            state.sessions.get("sess-2").unwrap().phase,
            SessionPhase::Completed
        );
    }

    #[test]
    fn test_session_activity_updates_existing() {
        let state = SessionState::new();

        let start = make_event("sess-1", EventKind::SessionStarted, AgentKind::Opencode);
        let state = apply_event(state, start);

        let activity = UniversalEvent {
            cwd: Some("/new/path".to_string()),
            branch: Some("feature/new".to_string()),
            model: Some("claude-4".to_string()),
            ..make_event("sess-1", EventKind::ActivityUpdated, AgentKind::Opencode)
        };
        let state = apply_event(state, activity);

        let session = state.sessions.get("sess-1").unwrap();
        assert_eq!(session.cwd.as_deref(), Some("/new/path"));
        assert_eq!(session.branch.as_deref(), Some("feature/new"));
        assert_eq!(session.model.as_deref(), Some("claude-4"));
        assert_eq!(session.event_count, 2);
    }

    #[test]
    fn test_heartbeat_updates_timestamp() {
        let state = SessionState::new();

        let start = make_event("sess-1", EventKind::SessionStarted, AgentKind::Opencode);
        let state = apply_event(state, start);

        std::thread::sleep(std::time::Duration::from_millis(10));

        let heartbeat = make_event("sess-1", EventKind::Heartbeat, AgentKind::Opencode);
        let state = apply_event(state, heartbeat);

        let session = state.sessions.get("sess-1").unwrap();
        assert!(session.last_heartbeat > session.created_at);
        assert_eq!(session.phase, SessionPhase::Running);
    }

    #[test]
    fn test_session_paused() {
        let state = SessionState::new();
        let start = make_event("sess-1", EventKind::SessionStarted, AgentKind::Antigravity);
        let state = apply_event(state, start);

        let pause = make_event("sess-1", EventKind::SessionPaused, AgentKind::Antigravity);
        let state = apply_event(state, pause);

        let session = state.sessions.get("sess-1").unwrap();
        assert_eq!(session.phase, SessionPhase::Paused);
        assert!(!session.is_active());
    }

    #[test]
    fn test_token_usage_updates_counts() {
        let state = SessionState::new();
        let start = UniversalEvent {
            tokens_input: None,
            tokens_output: None,
            ..make_event("sess-1", EventKind::SessionStarted, AgentKind::Claude)
        };
        let state = apply_event(state, start);

        let tokens = UniversalEvent {
            tokens_input: Some(1000),
            tokens_output: Some(500),
            model: Some("claude-opus-4".to_string()),
            ..make_event("sess-1", EventKind::TokenUsage, AgentKind::Claude)
        };
        let state = apply_event(state, tokens);

        let session = state.sessions.get("sess-1").unwrap();
        assert_eq!(session.tokens_input, 1000);
        assert_eq!(session.tokens_output, 500);
    }

    #[test]
    fn test_jump_target_updated() {
        let state = SessionState::new();
        let start = make_event("sess-1", EventKind::SessionStarted, AgentKind::Opencode);
        let state = apply_event(state, start);

        let jump = UniversalEvent {
            jump_target: Some(JumpTarget {
                session_id: Uuid::new_v4(),
                terminal: "tmux".to_string(),
                pane: Some("1".to_string()),
                cwd: Some("/project".to_string()),
                pid: Some(12345),
            }),
            ..make_event("sess-1", EventKind::JumpTargetUpdated, AgentKind::Opencode)
        };
        let state = apply_event(state, jump);

        let session = state.sessions.get("sess-1").unwrap();
        assert!(session.jump_target.is_some());
        assert_eq!(session.jump_target.as_ref().unwrap().pid, Some(12345));
    }

    #[test]
    fn test_prune_orphaned_sessions() {
        let mut state = SessionState::new();

        let start = make_event("sess-1", EventKind::SessionStarted, AgentKind::Opencode);
        state = apply_event(state, start);

        let complete = make_event("sess-1", EventKind::SessionCompleted, AgentKind::Opencode);
        state = apply_event(state, complete);

        let start2 = make_event("sess-2", EventKind::SessionStarted, AgentKind::Claude);
        state = apply_event(state, start2);

        assert_eq!(state.sessions.len(), 2);

        // Prune with zero max age - should remove the completed session
        state.prune_orphaned(chrono::Duration::seconds(0));

        assert_eq!(state.sessions.len(), 1);
        assert!(state.sessions.contains_key("sess-2"));
        assert!(!state.sessions.contains_key("sess-1"));
    }

    #[test]
    fn test_session_is_stale() {
        let state = SessionState::new();
        let event = make_event("sess-1", EventKind::SessionStarted, AgentKind::Opencode);
        let mut state = apply_event(state, event);

        let session = state.sessions.get_mut("sess-1").unwrap();
        session.last_heartbeat = Utc::now() - chrono::Duration::minutes(10);

        assert!(session.is_stale(&chrono::Duration::minutes(5)));
        assert!(!session.is_stale(&chrono::Duration::minutes(15)));
    }

    #[test]
    fn test_total_events_processed() {
        let mut state = SessionState::new();
        assert_eq!(state.total_events_processed, 0);

        let s1 = make_event("sess-1", EventKind::SessionStarted, AgentKind::Opencode);
        state = apply_event(state, s1);
        assert_eq!(state.total_events_processed, 1);

        let s2 = make_event("sess-1", EventKind::ActivityUpdated, AgentKind::Opencode);
        state = apply_event(state, s2);
        assert_eq!(state.total_events_processed, 2);

        let s3 = make_event("sess-2", EventKind::SessionStarted, AgentKind::Claude);
        state = apply_event(state, s3);
        assert_eq!(state.total_events_processed, 3);
    }

    #[test]
    fn test_session_created_with_no_prior_session() {
        // SessionStarted should work even if no prior state exists
        let state = SessionState::new();
        let event = make_event("sess-1", EventKind::SessionStarted, AgentKind::Opencode);
        let state = apply_event(state, event);

        assert!(state.sessions.contains_key("sess-1"));
        assert_eq!(state.sessions.len(), 1);
    }

    #[test]
    fn test_permission_then_complete() {
        // Full lifecycle: start → permission → resolve → complete
        let state = SessionState::new();

        let events = vec![
            make_event("sess-1", EventKind::SessionStarted, AgentKind::Opencode),
            UniversalEvent {
                permission: Some(PermissionRequest {
                    id: Uuid::new_v4(),
                    command: "write file".to_string(),
                    description: "Write to src/main.rs".to_string(),
                    context: None,
                    diff: None,
                    created_at: Utc::now(),
                    expires_at: Utc::now() + chrono::Duration::minutes(5),
                }),
                ..make_event(
                    "sess-1",
                    EventKind::PermissionRequested,
                    AgentKind::Opencode,
                )
            },
            make_event(
                "sess-1",
                EventKind::ActionableStateResolved,
                AgentKind::Opencode,
            ),
            UniversalEvent {
                tokens_input: Some(5000),
                tokens_output: Some(2000),
                duration_ms: Some(60000),
                ..make_event("sess-1", EventKind::SessionCompleted, AgentKind::Opencode)
            },
        ];

        let state = events.into_iter().fold(state, apply_event);

        let session = state.sessions.get("sess-1").unwrap();
        assert_eq!(session.phase, SessionPhase::Completed);
        assert_eq!(session.tokens_input, 5000);
        assert_eq!(session.tokens_output, 2000);
        assert_eq!(session.duration_ms, 60000);
        assert_eq!(session.event_count, 1); // event_count only tracks session creation
    }

    #[test]
    fn test_plan_proposed_sets_plan() {
        let state = SessionState::new();
        let start = make_event("sess-1", EventKind::SessionStarted, AgentKind::Claude);
        let state = apply_event(state, start);

        let plan = PlanProposal {
            id: Uuid::new_v4(),
            summary: "Refactor auth module".to_string(),
            items: vec![
                PlanItem {
                    action: "Extract login logic".to_string(),
                    file: Some("src/auth.rs".to_string()),
                    details: Some("Move login validation to separate function".to_string()),
                },
                PlanItem {
                    action: "Add JWT middleware".to_string(),
                    file: Some("src/middleware.rs".to_string()),
                    details: None,
                },
            ],
            reasoning: Some("The auth module is too large".to_string()),
            created_at: Utc::now(),
        };

        let proposed = UniversalEvent {
            plan: Some(plan.clone()),
            ..make_event("sess-1", EventKind::PlanProposed, AgentKind::Claude)
        };
        let state = apply_event(state, proposed);

        let session = state.sessions.get("sess-1").unwrap();
        assert!(session.plan.is_some());
        assert_eq!(session.plan.as_ref().unwrap().summary, "Refactor auth module");
        assert_eq!(session.plan.as_ref().unwrap().items.len(), 2);
    }

    #[test]
    fn test_plan_approved_clears_plan() {
        let state = SessionState::new();
        let start = make_event("sess-1", EventKind::SessionStarted, AgentKind::Claude);
        let state = apply_event(state, start);

        let plan = PlanProposal {
            id: Uuid::new_v4(),
            summary: "Test plan".to_string(),
            items: vec![],
            reasoning: None,
            created_at: Utc::now(),
        };

        let proposed = UniversalEvent {
            plan: Some(plan),
            ..make_event("sess-1", EventKind::PlanProposed, AgentKind::Claude)
        };
        let state = apply_event(state, proposed);
        assert!(state.sessions.get("sess-1").unwrap().plan.is_some());

        let approved = make_event("sess-1", EventKind::PlanApproved, AgentKind::Claude);
        let state = apply_event(state, approved);
        assert!(state.sessions.get("sess-1").unwrap().plan.is_none());
    }

    #[test]
    fn test_diff_available_sets_diff() {
        let state = SessionState::new();
        let start = make_event("sess-1", EventKind::SessionStarted, AgentKind::Codex);
        let state = apply_event(state, start);

        let diff = DiffPayload {
            id: Uuid::new_v4(),
            session_id: "sess-1".to_string(),
            files: vec![FileDiff {
                file_path: "src/main.rs".to_string(),
                diff_content: "--- a/src/main.rs\n+++ b/src/main.rs\n@@ -1 +1 @@\n-hello\n+hello world".to_string(),
                language: Some("rust".to_string()),
                status: Some("modified".to_string()),
            }],
            summary: Some("Added greeting".to_string()),
            created_at: Utc::now(),
        };

        let available = UniversalEvent {
            diff: Some(diff.clone()),
            ..make_event("sess-1", EventKind::DiffAvailable, AgentKind::Codex)
        };
        let state = apply_event(state, available);

        let session = state.sessions.get("sess-1").unwrap();
        assert!(session.diff.is_some());
        assert_eq!(session.diff.as_ref().unwrap().files.len(), 1);
        assert_eq!(session.diff.as_ref().unwrap().files[0].file_path, "src/main.rs");
    }

    #[test]
    fn test_diff_applied_clears_diff() {
        let state = SessionState::new();
        let start = make_event("sess-1", EventKind::SessionStarted, AgentKind::Codex);
        let state = apply_event(state, start);

        let diff = DiffPayload {
            id: Uuid::new_v4(),
            session_id: "sess-1".to_string(),
            files: vec![],
            summary: None,
            created_at: Utc::now(),
        };

        let available = UniversalEvent {
            diff: Some(diff),
            ..make_event("sess-1", EventKind::DiffAvailable, AgentKind::Codex)
        };
        let state = apply_event(state, available);
        assert!(state.sessions.get("sess-1").unwrap().diff.is_some());

        let applied = make_event("sess-1", EventKind::DiffApplied, AgentKind::Codex);
        let state = apply_event(state, applied);
        assert!(state.sessions.get("sess-1").unwrap().diff.is_none());
    }

    #[test]
    fn test_multi_agent_lifecycle() {
        let state = SessionState::new();

        // Start multiple sessions from different agents
        let events = vec![
            (
                1,
                make_event("proj-api", EventKind::SessionStarted, AgentKind::Claude),
            ),
            (
                2,
                make_event("proj-web", EventKind::SessionStarted, AgentKind::Codex),
            ),
        ];

        let state = events
            .into_iter()
            .fold(state, |state, (_, event)| apply_event(state, event));
        assert_eq!(state.total_count(), 2);

        // Claude asks a question
        let question = UniversalEvent {
            question: Some(QuestionPrompt {
                id: Uuid::new_v4(),
                question: "Use PostgreSQL?".to_string(),
                options: vec!["yes".to_string(), "no".to_string()],
                context: None,
                created_at: Utc::now(),
            }),
            ..make_event("proj-api", EventKind::QuestionAsked, AgentKind::Claude)
        };
        let state = apply_event(state, question);

        assert_eq!(
            state.sessions.get("proj-api").unwrap().phase,
            SessionPhase::WaitingQuestion
        );
        assert_eq!(
            state.sessions.get("proj-web").unwrap().phase,
            SessionPhase::Running
        );

        // Codex completes
        let complete = UniversalEvent {
            tokens_input: Some(3000),
            tokens_output: Some(1500),
            duration_ms: Some(45000),
            ..make_event("proj-web", EventKind::SessionCompleted, AgentKind::Codex)
        };
        let state = apply_event(state, complete);

        assert_eq!(
            state.sessions.get("proj-web").unwrap().phase,
            SessionPhase::Completed
        );
        assert_eq!(state.active_count(), 1);
    }

    #[test]
    fn test_completed_ignores_post_completion_events() {
        let state = SessionState::new();
        let start = make_event("s-1", EventKind::SessionStarted, AgentKind::Claude);
        let state = apply_event(state, start);

        let complete = UniversalEvent {
            tokens_input: Some(5000),
            tokens_output: Some(2000),
            duration_ms: Some(120000),
            ..make_event("s-1", EventKind::SessionCompleted, AgentKind::Claude)
        };
        let state = apply_event(state, complete);
        assert_eq!(state.sessions.get("s-1").unwrap().phase, SessionPhase::Completed);

        let late = UniversalEvent {
            tokens_input: None,
            tokens_output: None,
            ..make_event("s-1", EventKind::TokenUsage, AgentKind::Claude)
        };
        let state = apply_event(state, late);

        let session = state.sessions.get("s-1").unwrap();
        assert_eq!(session.phase, SessionPhase::Completed);
    }

    #[test]
    fn test_duplicate_session_started_does_not_duplicate() {
        let state = SessionState::new();
        let s1 = make_event("s-1", EventKind::SessionStarted, AgentKind::Opencode);
        let state = apply_event(state, s1);
        let s2 = make_event("s-1", EventKind::SessionStarted, AgentKind::Opencode);
        let state = apply_event(state, s2);
        assert_eq!(state.sessions.len(), 1);
    }

    #[test]
    fn test_metrics_accumulate_correctly() {
        let state = SessionState::new();
        let start = UniversalEvent {
            tokens_input: None,
            tokens_output: None,
            ..make_event("s-1", EventKind::SessionStarted, AgentKind::Claude)
        };
        let state = apply_event(state, start);

        let t1 = UniversalEvent {
            tokens_input: Some(100),
            tokens_output: Some(50),
            ..make_event("s-1", EventKind::TokenUsage, AgentKind::Claude)
        };
        let state = apply_event(state, t1);

        let t2 = UniversalEvent {
            tokens_input: Some(200),
            tokens_output: Some(100),
            ..make_event("s-1", EventKind::TokenUsage, AgentKind::Claude)
        };
        let state = apply_event(state, t2);

        let session = state.sessions.get("s-1").unwrap();
        assert_eq!(session.tokens_input, 300);
        assert_eq!(session.tokens_output, 150);
    }

    #[test]
    fn test_permission_cleared_after_resolution() {
        let state = SessionState::new();
        let start = make_event("s-1", EventKind::SessionStarted, AgentKind::Opencode);
        let state = apply_event(state, start);

        let perm = UniversalEvent {
            permission: Some(PermissionRequest {
                id: Uuid::new_v4(),
                command: "allow bash".to_string(),
                description: "Execute shell command".to_string(),
                context: None,
                diff: None,
                created_at: Utc::now(),
                expires_at: Utc::now() + chrono::Duration::minutes(5),
            }),
            ..make_event("s-1", EventKind::PermissionRequested, AgentKind::Opencode)
        };
        let state = apply_event(state, perm);

        let resolve = make_event("s-1", EventKind::ActionableStateResolved, AgentKind::Opencode);
        let state = apply_event(state, resolve);

        let session = state.sessions.get("s-1").unwrap();
        assert!(session.permission.is_none());
        assert_eq!(session.phase, SessionPhase::Running);
    }

    #[test]
    fn test_prune_orphaned_does_not_touch_active() {
        let mut state = SessionState::new();

        let start = make_event("active", EventKind::SessionStarted, AgentKind::Opencode);
        state = apply_event(state, start);

        let start2 = UniversalEvent {
            tokens_input: None,
            tokens_output: None,
            ..make_event("orphan", EventKind::SessionStarted, AgentKind::Claude)
        };
        state = apply_event(state, start2);

        let complete = make_event("orphan", EventKind::SessionCompleted, AgentKind::Claude);
        state = apply_event(state, complete);

        {
            let session = state.sessions.get_mut("orphan").unwrap();
            session.updated_at = Utc::now() - chrono::Duration::hours(2);
        }

        state.prune_orphaned(chrono::Duration::hours(1));

        assert!(state.sessions.contains_key("active"));
        assert!(!state.sessions.contains_key("orphan"));
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ReducerError {
    #[error("session not found: {0}")]
    SessionNotFound(String),
    #[error("invalid transition from {from:?} with event {event:?}")]
    InvalidTransition {
        from: SessionPhase,
        event: EventKind,
    },
}

pub fn apply_event(mut state: SessionState, event: UniversalEvent) -> SessionState {
    state.total_events_processed += 1;

    let session_id = event.session_id.clone();

    match event.event {
        EventKind::SessionStarted => {
            let session = AgentSession::new(&event);
            state.sessions.insert(session_id, session);
        }
        EventKind::ActivityUpdated => {
            if let Some(session) = state.sessions.get_mut(&session_id) {
                session.updated_at = event.timestamp;
                session.last_heartbeat = event.timestamp;
                session.event_count += 1;
                if let Some(cwd) = event.cwd {
                    session.cwd = Some(cwd);
                }
                if let Some(action) = event.current_action {
                    session.current_action = Some(action);
                }
                if let Some(branch) = event.branch {
                    session.branch = Some(branch);
                }
                if let Some(model) = event.model {
                    session.model = Some(model);
                }
            }
        }
        EventKind::PermissionRequested => {
            let session = state.sessions.entry(session_id).or_insert_with(|| {
                let mut s = AgentSession::new(&event);
                s.phase = SessionPhase::WaitingPermission;
                s
            });
            session.phase = SessionPhase::WaitingPermission;
            session.permission = event.permission;
            session.updated_at = event.timestamp;
            session.last_heartbeat = event.timestamp;
            if let Some(action) = event.current_action {
                session.current_action = Some(action);
            }
        }
        EventKind::QuestionAsked => {
            let session = state.sessions.entry(session_id).or_insert_with(|| {
                let mut s = AgentSession::new(&event);
                s.phase = SessionPhase::WaitingQuestion;
                s
            });
            session.phase = SessionPhase::WaitingQuestion;
            session.question = event.question;
            session.updated_at = event.timestamp;
            session.last_heartbeat = event.timestamp;
        }
        EventKind::SessionCompleted => {
            if let Some(session) = state.sessions.get_mut(&session_id) {
                session.phase = SessionPhase::Completed;
                session.updated_at = event.timestamp;
                session.duration_ms = event.duration_ms.unwrap_or(0);
                session.tokens_input = event.tokens_input.unwrap_or(session.tokens_input);
                session.tokens_output = event.tokens_output.unwrap_or(session.tokens_output);
            }
        }
        EventKind::SessionFailed => {
            let session = state.sessions.entry(session_id).or_insert_with(|| {
                let mut s = AgentSession::new(&event);
                s.phase = SessionPhase::Failed;
                s
            });
            session.phase = SessionPhase::Failed;
            session.error = event.error;
            session.updated_at = event.timestamp;
        }
        EventKind::SessionPaused => {
            if let Some(session) = state.sessions.get_mut(&session_id) {
                session.phase = SessionPhase::Paused;
                session.updated_at = event.timestamp;
            }
        }
        EventKind::Heartbeat => {
            if let Some(session) = state.sessions.get_mut(&session_id) {
                session.last_heartbeat = event.timestamp;
                session.updated_at = event.timestamp;
            }
        }
        EventKind::TokenUsage => {
            if let Some(session) = state.sessions.get_mut(&session_id) {
                session.tokens_input += event.tokens_input.unwrap_or(0);
                session.tokens_output += event.tokens_output.unwrap_or(0);
                session.model = event.model.or(session.model.clone());
                session.updated_at = event.timestamp;
                session.last_heartbeat = event.timestamp;
            }
        }
        EventKind::JumpTargetUpdated => {
            if let Some(session) = state.sessions.get_mut(&session_id) {
                session.jump_target = event.jump_target;
                session.updated_at = event.timestamp;
            }
        }
        EventKind::ActionableStateResolved => {
            if let Some(session) = state.sessions.get_mut(&session_id) {
                session.phase = SessionPhase::Running;
                session.permission = None;
                session.question = None;
                session.updated_at = event.timestamp;
            }
        }
        EventKind::PlanProposed => {
            if let Some(session) = state.sessions.get_mut(&session_id) {
                session.plan = event.plan;
                session.updated_at = event.timestamp;
                session.last_heartbeat = event.timestamp;
            }
        }
        EventKind::PlanApproved => {
            if let Some(session) = state.sessions.get_mut(&session_id) {
                session.plan = None;
                session.updated_at = event.timestamp;
            }
        }
        EventKind::PlanRejected => {
            if let Some(session) = state.sessions.get_mut(&session_id) {
                session.plan = None;
                session.updated_at = event.timestamp;
            }
        }
        EventKind::DiffAvailable => {
            if let Some(session) = state.sessions.get_mut(&session_id) {
                session.diff = event.diff;
                session.updated_at = event.timestamp;
                session.last_heartbeat = event.timestamp;
            }
        }
        EventKind::DiffApplied => {
            if let Some(session) = state.sessions.get_mut(&session_id) {
                session.diff = None;
                session.updated_at = event.timestamp;
            }
        }
        EventKind::DiffRejected => {
            if let Some(session) = state.sessions.get_mut(&session_id) {
                session.diff = None;
                session.updated_at = event.timestamp;
            }
        }
    }

    state
}
