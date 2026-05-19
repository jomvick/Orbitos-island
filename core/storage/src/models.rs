use daemon_core::state::{AgentSession, SessionPhase};

pub struct StoredSession {
    pub id: String,
    pub agent: String,
    pub phase: String,
    pub cwd: Option<String>,
    pub branch: Option<String>,
    pub model: Option<String>,
    pub tokens_input: u64,
    pub tokens_output: u64,
    pub duration_ms: u64,
    pub terminal: Option<String>,
    pub pane: Option<String>,
    pub metadata: Option<String>,
    pub error: Option<String>,
    pub current_action: Option<String>,
    /// OS process ID — may be None for sessions started before v2 migration.
    pub pid: Option<u32>,
    pub created_at: i64,
    pub updated_at: i64,
    pub last_heartbeat: i64,
    pub event_count: u64,
}

impl StoredSession {
    pub fn to_domain(&self) -> Result<AgentSession, String> {
        Ok(AgentSession {
            id: self.id.clone(),
            agent: self.agent.parse().map_err(|e: String| e)?,
            phase: self.phase.parse::<SessionPhase>().map_err(|e| e.to_string())?,
            cwd: self.cwd.clone(),
            branch: self.branch.clone(),
            model: self.model.clone(),
            tokens_input: self.tokens_input,
            tokens_output: self.tokens_output,
            duration_ms: self.duration_ms,
            terminal: self.terminal.clone(),
            pane: self.pane.clone(),
            permission: None,
            question: None,
            jump_target: None,
            plan: None,
            diff: None,
            error: self.error.clone(),
            current_action: self.current_action.clone(),
            metadata: self
                .metadata
                .as_ref()
                .and_then(|m| serde_json::from_str(m).ok()),
            pid: self.pid,
            ppid: None,
            terminal_kind: None,
            terminal_id: None,
            created_at: chrono::DateTime::from_timestamp(self.created_at, 0).unwrap_or_default(),
            updated_at: chrono::DateTime::from_timestamp(self.updated_at, 0).unwrap_or_default(),
            last_heartbeat: chrono::DateTime::from_timestamp(self.last_heartbeat, 0)
                .unwrap_or_default(),
            event_count: self.event_count,
        })
    }
}
