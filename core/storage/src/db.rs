use std::path::Path;

use rusqlite::{params, Connection};
use tracing::info;

use daemon_core::state::{AgentSession, UniversalEvent};

use super::models::StoredSession;

pub struct Database {
    pub(crate) conn: Connection,
}

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("not found")]
    NotFound,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self, DbError> {
        let conn = Connection::open(path)?;

        conn.execute_batch("PRAGMA journal_mode = WAL;")?;
        conn.execute_batch("PRAGMA busy_timeout = 5000;")?;
        conn.execute_batch("PRAGMA synchronous = NORMAL;")?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;

        let db = Self { conn };
        db.run_migrations()?;
        Ok(db)
    }

    pub fn open_in_memory() -> Result<Self, DbError> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&self) -> Result<(), DbError> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY,
                applied_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                agent TEXT NOT NULL,
                phase TEXT NOT NULL DEFAULT 'running',
                cwd TEXT,
                branch TEXT,
                model TEXT,
                tokens_input INTEGER NOT NULL DEFAULT 0,
                tokens_output INTEGER NOT NULL DEFAULT 0,
                duration_ms INTEGER NOT NULL DEFAULT 0,
                terminal TEXT,
                pane TEXT,
                metadata TEXT,
                error TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                last_heartbeat INTEGER NOT NULL,
                event_count INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS events (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                agent TEXT NOT NULL,
                event_kind TEXT NOT NULL,
                payload TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                FOREIGN KEY (session_id) REFERENCES sessions(id)
            );

            CREATE INDEX IF NOT EXISTS idx_sessions_agent ON sessions(agent);
            CREATE INDEX IF NOT EXISTS idx_sessions_phase ON sessions(phase);
            CREATE INDEX IF NOT EXISTS idx_sessions_updated ON sessions(updated_at);
            CREATE INDEX IF NOT EXISTS idx_events_session ON events(session_id);
            CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp);
            ",
        )?;

        self.conn.execute(
            "INSERT OR IGNORE INTO schema_version (version, applied_at) VALUES (?1, ?2)",
            params![1, chrono::Utc::now().timestamp()],
        )?;

        info!("database migrations applied successfully");
        Ok(())
    }

    pub fn upsert_session(&self, session: &AgentSession) -> Result<(), DbError> {
        let metadata = session.metadata.as_ref().map(|m| m.to_string());

        self.conn.execute(
            "INSERT INTO sessions (id, agent, phase, cwd, branch, model, tokens_input, tokens_output, duration_ms, terminal, pane, metadata, error, created_at, updated_at, last_heartbeat, event_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
             ON CONFLICT(id) DO UPDATE SET
                phase = excluded.phase,
                cwd = excluded.cwd,
                branch = excluded.branch,
                model = excluded.model,
                tokens_input = excluded.tokens_input,
                tokens_output = excluded.tokens_output,
                duration_ms = excluded.duration_ms,
                terminal = excluded.terminal,
                pane = excluded.pane,
                metadata = excluded.metadata,
                error = excluded.error,
                updated_at = excluded.updated_at,
                last_heartbeat = excluded.last_heartbeat,
                event_count = excluded.event_count",
            params![
                session.id,
                session.agent.to_string(),
                session.phase.to_string(),
                session.cwd,
                session.branch,
                session.model,
                session.tokens_input,
                session.tokens_output,
                session.duration_ms,
                session.terminal,
                session.pane,
                metadata,
                session.error,
                session.created_at.timestamp(),
                session.updated_at.timestamp(),
                session.last_heartbeat.timestamp(),
                session.event_count,
            ],
        )?;

        Ok(())
    }

    pub fn insert_event(&self, event: &UniversalEvent) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT INTO events (id, session_id, agent, event_kind, payload, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                event.id.to_string(),
                event.session_id,
                event.agent.to_string(),
                event.event.to_string(),
                serde_json::to_string(event)?,
                event.timestamp.timestamp(),
            ],
        )?;
        Ok(())
    }

    pub fn get_active_sessions(&self) -> Result<Vec<StoredSession>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, agent, phase, cwd, branch, model, tokens_input, tokens_output, duration_ms,
                    terminal, pane, metadata, error, created_at, updated_at, last_heartbeat, event_count
             FROM sessions
             WHERE phase IN ('running', 'waiting_permission', 'waiting_question', 'paused')
             ORDER BY updated_at DESC",
        )?;

        let sessions = stmt
            .query_map([], |row| {
                Ok(StoredSession {
                    id: row.get(0)?,
                    agent: row.get(1)?,
                    phase: row.get(2)?,
                    cwd: row.get(3)?,
                    branch: row.get(4)?,
                    model: row.get(5)?,
                    tokens_input: row.get(6)?,
                    tokens_output: row.get(7)?,
                    duration_ms: row.get(8)?,
                    terminal: row.get(9)?,
                    pane: row.get(10)?,
                    metadata: row.get(11)?,
                    error: row.get(12)?,
                    created_at: row.get(13)?,
                    updated_at: row.get(14)?,
                    last_heartbeat: row.get(15)?,
                    event_count: row.get(16)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(sessions)
    }

    pub fn get_all_sessions(&self, limit: u32) -> Result<Vec<StoredSession>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, agent, phase, cwd, branch, model, tokens_input, tokens_output, duration_ms,
                    terminal, pane, metadata, error, created_at, updated_at, last_heartbeat, event_count
             FROM sessions
             ORDER BY updated_at DESC
             LIMIT ?1",
        )?;

        let sessions = stmt
            .query_map(params![limit], |row| {
                Ok(StoredSession {
                    id: row.get(0)?,
                    agent: row.get(1)?,
                    phase: row.get(2)?,
                    cwd: row.get(3)?,
                    branch: row.get(4)?,
                    model: row.get(5)?,
                    tokens_input: row.get(6)?,
                    tokens_output: row.get(7)?,
                    duration_ms: row.get(8)?,
                    terminal: row.get(9)?,
                    pane: row.get(10)?,
                    metadata: row.get(11)?,
                    error: row.get(12)?,
                    created_at: row.get(13)?,
                    updated_at: row.get(14)?,
                    last_heartbeat: row.get(15)?,
                    event_count: row.get(16)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(sessions)
    }

    pub fn integrity_check(&self) -> Result<String, DbError> {
        let result: String = self
            .conn
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
        Ok(result)
    }

    pub fn vacuum(&self) -> Result<(), DbError> {
        self.conn.execute_batch("VACUUM;")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use daemon_core::state::{AgentKind, EventKind, UniversalEvent};
    use uuid::Uuid;

    fn test_db() -> Database {
        Database::open_in_memory().unwrap()
    }

    fn sample_session(agent: &str) -> AgentSession {
        AgentSession {
            id: Uuid::new_v4().to_string(),
            agent: agent.parse().unwrap_or(AgentKind::Custom(agent.to_string())),
            phase: daemon_core::state::SessionPhase::Running,
            cwd: Some("/project".to_string()),
            branch: Some("main".to_string()),
            model: Some("test-model".to_string()),
            tokens_input: 1000,
            tokens_output: 500,
            duration_ms: 60000,
            terminal: Some("tmux".to_string()),
            pane: Some("0".to_string()),
            permission: None,
            question: None,
            jump_target: None,
            plan: None,
            diff: None,
            error: None,
            metadata: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_heartbeat: chrono::Utc::now(),
            event_count: 1,
        }
    }

    #[test]
    fn test_open_in_memory() {
        let db = test_db();
        assert_eq!(db.integrity_check().unwrap(), "ok");
    }

    #[test]
    fn test_upsert_and_get_session() {
        let db = test_db();
        let session = sample_session("opencode");

        db.upsert_session(&session).unwrap();

        let sessions = db.get_all_sessions(10).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].agent, "opencode");
        assert_eq!(sessions[0].cwd.as_deref(), Some("/project"));
    }

    #[test]
    fn test_upsert_updates_existing() {
        let db = test_db();
        let mut session = sample_session("claude");
        session.id = "fixed-id".to_string();
        db.upsert_session(&session).unwrap();

        session.tokens_input = 9999;
        db.upsert_session(&session).unwrap();

        let sessions = db.get_all_sessions(10).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].tokens_input, 9999);
    }

    #[test]
    fn test_get_active_sessions() {
        let db = test_db();
        let mut running = sample_session("opencode");
        let mut completed = sample_session("claude");
        completed.phase = daemon_core::state::SessionPhase::Completed;
        let mut failed = sample_session("codex");
        failed.phase = daemon_core::state::SessionPhase::Failed;

        db.upsert_session(&running).unwrap();
        db.upsert_session(&completed).unwrap();
        db.upsert_session(&failed).unwrap();

        let active = db.get_active_sessions().unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].agent, "opencode");
    }

    #[test]
    fn test_insert_event() {
        let db = test_db();
        let session = sample_session("opencode");
        db.upsert_session(&session).unwrap();

        let event = UniversalEvent {
            id: Uuid::new_v4(),
            agent: AgentKind::Opencode,
            event: EventKind::SessionStarted,
            session_id: session.id.clone(),
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
            error: None,
            metadata: None,
            timestamp: chrono::Utc::now(),
        };

        db.insert_event(&event).unwrap();
        let timeline = db.get_timeline(10).unwrap();
        assert_eq!(timeline.len(), 1);
        assert_eq!(timeline[0].agent, "opencode");
    }

    #[test]
    fn test_get_session_stats_empty() {
        let db = test_db();
        let stats = db.get_session_stats().unwrap();
        assert_eq!(stats.total_sessions, 0);
        assert_eq!(stats.active_count, 0);
        assert_eq!(stats.total_tokens, 0);
    }

    #[test]
    fn test_search_sessions() {
        let db = test_db();
        let mut s1 = sample_session("opencode");
        let mut s2 = sample_session("claude");
        s2.cwd = Some("/other".to_string());
        db.upsert_session(&s1).unwrap();
        db.upsert_session(&s2).unwrap();

        let results = db.search_sessions("opencode").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].agent, "opencode");
    }

    #[test]
    fn test_search_sessions_by_cwd() {
        let db = test_db();
        let mut s1 = sample_session("opencode");
        s1.cwd = Some("/home/user/my-project".to_string());
        let mut s2 = sample_session("claude");
        s2.cwd = Some("/other".to_string());
        db.upsert_session(&s1).unwrap();
        db.upsert_session(&s2).unwrap();

        let results = db.search_sessions("my-project").unwrap();
        assert_eq!(results.len(), 1);
    }
}
