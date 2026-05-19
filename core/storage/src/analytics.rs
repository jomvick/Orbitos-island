use serde::Serialize;

use crate::db::Database;

#[derive(Debug, Serialize)]
pub struct AgentAnalytics {
    pub agent: String,
    pub total_sessions: u64,
    pub active_sessions: u64,
    pub completed_sessions: u64,
    pub failed_sessions: u64,
    pub total_tokens_input: u64,
    pub total_tokens_output: u64,
    pub avg_duration_ms: f64,
    pub total_cost_estimate: f64,
}

#[derive(Debug, Serialize)]
pub struct SessionStats {
    pub total_sessions: u64,
    pub active_count: u64,
    pub total_tokens: u64,
    pub total_duration_hours: f64,
    pub agents: Vec<AgentAnalytics>,
}

#[derive(Debug, Serialize)]
pub struct TimelineEntry {
    pub session_id: String,
    pub agent: String,
    pub event_kind: String,
    pub timestamp: i64,
    pub summary: String,
}

impl Database {
    pub fn get_session_stats(&self) -> Result<SessionStats, crate::db::DbError> {
        let total_sessions: u64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))?;

        let active_count: u64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sessions WHERE phase IN ('running', 'waiting_permission', 'waiting_question', 'paused')",
                [],
                |row| row.get(0),
            )?;

        let total_tokens: u64 = self.conn.query_row(
            "SELECT COALESCE(SUM(tokens_input + tokens_output), 0) FROM sessions",
            [],
            |row| row.get(0),
        )?;

        let total_duration_ms: u64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(duration_ms), 0) FROM sessions WHERE phase IN ('completed', 'failed')",
                [],
                |row| row.get(0),
            )?;

        let agents = self.get_agent_analytics()?;

        Ok(SessionStats {
            total_sessions,
            active_count,
            total_tokens,
            total_duration_hours: total_duration_ms as f64 / 3_600_000.0,
            agents,
        })
    }

    pub fn get_agent_analytics(&self) -> Result<Vec<AgentAnalytics>, crate::db::DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT
                agent,
                COUNT(*) as total,
                SUM(CASE WHEN phase IN ('running', 'waiting_permission', 'waiting_question', 'paused') THEN 1 ELSE 0 END) as active,
                SUM(CASE WHEN phase = 'completed' THEN 1 ELSE 0 END) as completed,
                SUM(CASE WHEN phase = 'failed' THEN 1 ELSE 0 END) as failed,
                COALESCE(SUM(tokens_input), 0) as total_in,
                COALESCE(SUM(tokens_output), 0) as total_out,
                COALESCE(AVG(CASE WHEN phase IN ('completed', 'failed') THEN duration_ms ELSE NULL END), 0.0) as avg_dur,
                COALESCE(SUM(tokens_input + tokens_output), 0) as total_tokens
             FROM sessions
             GROUP BY agent
             ORDER BY total_tokens DESC",
        )?;

        let rows = stmt
            .query_map([], |row| {
                Ok(AgentAnalytics {
                    agent: row.get(0)?,
                    total_sessions: row.get(1)?,
                    active_sessions: row.get(2)?,
                    completed_sessions: row.get(3)?,
                    failed_sessions: row.get(4)?,
                    total_tokens_input: row.get(5)?,
                    total_tokens_output: row.get(6)?,
                    avg_duration_ms: row.get(7)?,
                    total_cost_estimate: {
                        let tokens: f64 = row.get::<_, u64>(8)? as f64;
                        model_cost_per_token(&row.get::<_, String>(0)?) * tokens
                    },
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(rows)
    }

    pub fn get_timeline(&self, limit: u32) -> Result<Vec<TimelineEntry>, crate::db::DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT e.session_id, e.agent, e.event_kind, e.timestamp, s.cwd
             FROM events e
             LEFT JOIN sessions s ON e.session_id = s.id
             ORDER BY e.timestamp DESC
             LIMIT ?1",
        )?;

        let entries = stmt
            .query_map([limit], |row| {
                let cwd: Option<String> = row.get(4)?;
                Ok(TimelineEntry {
                    session_id: row.get(0)?,
                    agent: row.get(1)?,
                    event_kind: row.get(2)?,
                    timestamp: row.get(3)?,
                    summary: cwd.unwrap_or_default(),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    pub fn search_sessions(
        &self,
        query: &str,
    ) -> Result<Vec<crate::models::StoredSession>, crate::db::DbError> {
        let pattern = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT id, agent, phase, cwd, branch, model, tokens_input, tokens_output, duration_ms,
                    terminal, pane, metadata, error, current_action, pid, created_at, updated_at, last_heartbeat, event_count
             FROM sessions
             WHERE agent LIKE ?1
                OR id LIKE ?1
                OR cwd LIKE ?1
                OR branch LIKE ?1
                OR model LIKE ?1
             ORDER BY updated_at DESC
             LIMIT 50",
        )?;

        let sessions = stmt
            .query_map([&pattern], |row| {
                Ok(crate::models::StoredSession {
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
                    current_action: row.get(13)?,
                    pid: row.get(14)?,
                    created_at: row.get(15)?,
                    updated_at: row.get(16)?,
                    last_heartbeat: row.get(17)?,
                    event_count: row.get(18)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(sessions)
    }
}

fn model_cost_per_token(agent: &str) -> f64 {
    match agent {
        "claude" => 0.000_003,
        "opencode" => 0.000_002,
        "codex" => 0.000_003,
        "antigravity" => 0.000_002,
        "gemini" => 0.000_001_5,
        _ => 0.000_003,
    }
}
