use std::sync::Arc;

use tokio::time::{interval, Duration};
use tracing::{debug, info, warn};
use uuid::Uuid;

use daemon_core::state::{AgentKind, EventKind, SessionPhase, UniversalEvent, apply_event};

use crate::server::DaemonState;

/// How often to poll /proc for live PID status.
const POLL_INTERVAL: Duration = Duration::from_secs(5);

/// Check if a process is still alive by probing /proc/<pid>/status (Linux)
/// or sending signal 0 as a portable fallback.
fn is_pid_alive(pid: u32) -> bool {
    #[cfg(target_os = "linux")]
    {
        std::path::Path::new(&format!("/proc/{}/status", pid)).exists()
    }
    #[cfg(not(target_os = "linux"))]
    {
        // Portable: signal 0 checks existence without killing the process.
        // SAFETY: signal 0 does not modify process state — it only checks
        // whether the target PID exists. The pid_t cast from u32 is valid
        // on all POSIX platforms where this cfg gate applies.
        unsafe { libc::kill(pid as libc::pid_t, 0) == 0 }
    }
}

/// Background task that watches tracked PIDs and synthesizes session_completed
/// events for sessions whose processes have vanished without sending an explicit
/// lifecycle event (e.g. agent crashed or was kill -9'd).
pub async fn start_process_watcher(state: Arc<DaemonState>) {
    let mut ticker = interval(POLL_INTERVAL);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    info!(
        "process watcher started (poll interval: {}s)",
        POLL_INTERVAL.as_secs()
    );

    loop {
        ticker.tick().await;

        // Collect all running sessions that have a tracked PID.
        let tracked: Vec<(String, u32, AgentKind)> = {
            let s = state.session_state.read().await;
            s.sessions
                .values()
                .filter(|s| {
                    matches!(
                        s.phase,
                        SessionPhase::Running
                            | SessionPhase::WaitingPermission
                            | SessionPhase::WaitingQuestion
                            | SessionPhase::Paused
                    ) && s.pid.is_some()
                })
                .map(|s| (s.id.clone(), s.pid.expect("tracked session should have a PID"), s.agent.clone()))
                .collect()
        };

        if tracked.is_empty() {
            continue;
        }

        debug!(count = %tracked.len(), "watcher checking PIDs");

        for (session_id, pid, agent) in tracked {
            if !is_pid_alive(pid) {
                warn!(
                    session_id = %session_id,
                    pid = %pid,
                    agent = %agent,
                    "process vanished — synthesizing session_completed"
                );

                let synthetic_event = Arc::new(UniversalEvent {
                    id: Uuid::new_v4(),
                    agent: agent.clone(),
                    event: EventKind::SessionCompleted,
                    session_id: session_id.clone(),
                    cwd: None,
                    branch: None,
                    model: None,
                    tokens_input: None,
                    tokens_output: None,
                    duration_ms: None,
                    terminal: None,
                    pane: None,
                    permission: None,
                    question: None,
                    jump_target: None,
                    plan: None,
                    diff: None,
                    error: Some("process exited unexpectedly".to_string()),
                    current_action: None,
                    metadata: None,
                    pid: None,
                    ppid: None,
                    timestamp: chrono::Utc::now(),
                });

                // Publish to EventBus so the frontend gets notified.
                let _ = state.event_bus.publish(Arc::clone(&synthetic_event));

                // Update in-memory session state.
                let mut ss = state.session_state.write().await;
                *ss = apply_event(ss.clone(), (*synthetic_event).clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_pid_alive_self() {
        let own_pid = std::process::id();
        assert!(is_pid_alive(own_pid), "own process must be alive");
    }

    #[test]
    fn test_is_pid_alive_dead() {
        // PID u32::MAX is astronomically unlikely to exist on any system.
        assert!(!is_pid_alive(u32::MAX), "impossible PID must not be alive");
    }

    #[tokio::test]
    async fn test_watcher_synthesizes_completed_for_dead_pid() {
        use agentos_storage::Database;
        use daemon_core::state::{AgentSession, SessionPhase};
        use chrono::Utc;
        use tokio::sync::Mutex;

        let db = Arc::new(Mutex::new(Database::open_in_memory().expect("in-memory database should open")));
        let registry = crate::plugin_loader::load_default_plugins();
        let state = Arc::new(DaemonState::new(registry, Some(db)));

        // Inject a session with a guaranteed-dead PID.
        let session = AgentSession {
            id: "watcher-test-session".to_string(),
            agent: AgentKind::Opencode,
            phase: SessionPhase::Running,
            cwd: None,
            branch: None,
            model: None,
            tokens_input: 0,
            tokens_output: 0,
            duration_ms: 0,
            terminal: None,
            pane: None,
            permission: None,
            question: None,
            jump_target: None,
            plan: None,
            diff: None,
            error: None,
            current_action: None,
            metadata: None,
            pid: Some(u32::MAX),
            ppid: None,
            terminal_kind: None,
            terminal_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_heartbeat: Utc::now(),
            event_count: 1,
        };

        {
            let mut ss = state.session_state.write().await;
            ss.sessions.insert(session.id.clone(), session);
        }

        let mut rx = state.event_bus.subscribe();

        // Run one tick manually (mirror watcher logic).
        let tracked: Vec<(String, u32, AgentKind)> = {
            let s = state.session_state.read().await;
            s.sessions
                .values()
                .filter(|s| s.pid.is_some())
                .map(|s| (s.id.clone(), s.pid.expect("tracked session should have a PID"), s.agent.clone()))
                .collect()
        };

        for (session_id, pid, agent) in tracked {
            if !is_pid_alive(pid) {
                let synthetic = Arc::new(UniversalEvent {
                    id: Uuid::new_v4(),
                    agent: agent.clone(),
                    event: EventKind::SessionCompleted,
                    session_id: session_id.clone(),
                    cwd: None, branch: None, model: None,
                    tokens_input: None, tokens_output: None, duration_ms: None,
                    terminal: None, pane: None, permission: None, question: None,
                    jump_target: None, plan: None, diff: None,
                    error: Some("process exited unexpectedly".to_string()),
                    current_action: None,
                    metadata: None, pid: None, ppid: None,
                    timestamp: Utc::now(),
                });
                let _ = state.event_bus.publish(Arc::clone(&synthetic));
                let mut ss = state.session_state.write().await;
                *ss = apply_event(ss.clone(), (*synthetic).clone());
            }
        }

        // Verify event was emitted.
        let received = rx.try_recv().expect("should have received synthetic event");
        assert_eq!(received.event, EventKind::SessionCompleted);
        assert_eq!(received.session_id, "watcher-test-session");

        // Verify session transitioned to Completed.
        let ss = state.session_state.read().await;
        let final_session = ss.sessions.get("watcher-test-session").expect("test session should exist");
        assert_eq!(final_session.phase, SessionPhase::Completed);
    }
}
