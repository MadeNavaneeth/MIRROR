use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use uuid::Uuid;

// ── Typed blackboard ontology (research §3.2: Shared State Ontology) ─────────

/// Strongly-typed blackboard event kinds.
///
/// Every event posted to the shared blackboard must be one of these variants so
/// the circuit breaker, router, and individual agents can apply deterministic
/// rules without string-matching. The `as_str` representation is stored in the
/// SQLite `kind` column for human-readable queries and backward compat.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlackboardEventKind {
    // ── Journal Agent outputs ──────────────────────────────────────────────
    /// Neutral facts / references (e.g. "read article about X").
    /// Routes ONLY to the vector RAG store — does NOT wake other agents.
    JournalObservationInformational,
    /// Concrete actions taken or omitted (e.g. "skipped the gym").
    /// Triggers Health Agent readiness recalc + Career Agent proof-of-work audit.
    JournalObservationBehavioral,
    /// Emotional distress / burnout / identity-level signals.
    /// Highest priority — triggers immediate Planner recalibration, bypasses queue.
    JournalObservationCritical,
    /// An open cognitive loop detected in input (Zeigarnik open-loop capture).
    /// Triggers Planner to produce a concrete temporal anchor for the task.
    JournalOpenLoop,

    // ── Health Agent outputs ───────────────────────────────────────────────
    /// Updated biometric readiness score (HRV, circadian alignment, RHR delta).
    HealthReadinessScore,
    /// Severe autonomic suppression detected — forces Planner to drop deep-work.
    HealthCapabilityOverride,

    // ── Career Agent outputs ───────────────────────────────────────────────
    /// Passive-vs-active ratio imbalance or Bloom's taxonomy gap detected.
    CareerRiskFlag,
    /// Updated snapshot of mapped competencies from proof-of-work data.
    CareerCompetencyUpdate,

    // ── Planner Agent outputs ──────────────────────────────────────────────
    /// A proposed schedule modification (subject to circuit-breaker blast-radius check).
    PlannerScheduleProposal,
    /// An If-Then implementation intention posted to OS integration layer.
    PlannerImplementationIntention,
    /// EDZL-ranked overnight task re-prioritisation result.
    PlannerEdzlOvernightRerank,

    // ── System / governance events ─────────────────────────────────────────
    /// A proposed tool execution requiring governance checks.
    ToolExecutionProposed,
    /// Circuit breaker tripped — cascading agent action halted.
    SystemCircuitBreakerTripped,
    /// Escalation to user dashboard required (HITL checkpoint).
    SystemHitlRequest,

    // ── Fallback for unknown/legacy string kinds ───────────────────────────
    Unknown(String),
}

impl BlackboardEventKind {
    /// Canonical snake_case string stored in the `kind` column.
    pub fn as_str(&self) -> &str {
        match self {
            Self::JournalObservationInformational => "journal_observation_informational",
            Self::JournalObservationBehavioral => "journal_observation_behavioral",
            Self::JournalObservationCritical => "journal_observation_critical",
            Self::JournalOpenLoop => "journal_open_loop",
            Self::HealthReadinessScore => "health_readiness_score",
            Self::HealthCapabilityOverride => "health_capability_override",
            Self::CareerRiskFlag => "career_risk_flag",
            Self::CareerCompetencyUpdate => "career_competency_update",
            Self::PlannerScheduleProposal => "planner_schedule_proposal",
            Self::PlannerImplementationIntention => "planner_implementation_intention",
            Self::PlannerEdzlOvernightRerank => "planner_edzl_overnight_rerank",
            Self::ToolExecutionProposed => "tool_execution_proposed",
            Self::SystemCircuitBreakerTripped => "system_circuit_breaker_tripped",
            Self::SystemHitlRequest => "system_hitl_request",
            Self::Unknown(s) => s.as_str(),
        }
    }

    /// Parse from the stored string (DB round-trip).
    pub fn from_str(s: &str) -> Self {
        match s {
            "journal_observation_informational" => Self::JournalObservationInformational,
            "journal_observation_behavioral" => Self::JournalObservationBehavioral,
            "journal_observation_critical" => Self::JournalObservationCritical,
            "journal_open_loop" => Self::JournalOpenLoop,
            "health_readiness_score" => Self::HealthReadinessScore,
            "health_capability_override" => Self::HealthCapabilityOverride,
            "career_risk_flag" => Self::CareerRiskFlag,
            "career_competency_update" => Self::CareerCompetencyUpdate,
            "planner_schedule_proposal" => Self::PlannerScheduleProposal,
            "planner_implementation_intention" => Self::PlannerImplementationIntention,
            "planner_edzl_overnight_rerank" => Self::PlannerEdzlOvernightRerank,
            "tool_execution_proposed" => Self::ToolExecutionProposed,
            "system_circuit_breaker_tripped" => Self::SystemCircuitBreakerTripped,
            "system_hitl_request" => Self::SystemHitlRequest,
            other => Self::Unknown(other.to_string()),
        }
    }

    /// Returns true for events that should bypass the normal queue and wake
    /// agents immediately (Critical journal observations, circuit breaker trips).
    pub fn is_high_priority(&self) -> bool {
        matches!(
            self,
            Self::JournalObservationCritical
                | Self::HealthCapabilityOverride
                | Self::SystemCircuitBreakerTripped
                | Self::SystemHitlRequest
        )
    }

    /// Returns true for events that are purely informational and should NOT
    /// trigger agent wake-ups — only RAG indexing.
    pub fn is_rag_only(&self) -> bool {
        matches!(self, Self::JournalObservationInformational)
    }
}

impl std::fmt::Display for BlackboardEventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BlackboardEventStatus {
    Pending,
    Processed,
    Skipped,
    RateLimited,
    NeedsApproval,
    Error,
}

impl BlackboardEventStatus {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Processed => "processed",
            Self::Skipped => "skipped",
            Self::RateLimited => "rate_limited",
            Self::NeedsApproval => "needs_approval",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlackboardEvent {
    /// Raw string stored in the DB (backward compat).
    pub kind: String,
    /// Parsed typed representation of `kind`.
    pub typed_kind: BlackboardEventKind,
    pub id: String,
    pub agent_tag: Option<String>,
    pub payload: serde_json::Value,
    pub blast_radius: i64,
    pub requires_approval: bool,
    pub status: BlackboardEventStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct BlackboardDb {
    conn: Mutex<Connection>,
    #[allow(dead_code)]
    db_path: PathBuf,
}

impl BlackboardDb {
    pub fn open(workspace_dir: &Path) -> Result<Self> {
        let db_path = workspace_dir.join("mirror").join("blackboard.db");
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create mirror directory: {}", parent.display())
            })?;
        }

        let conn = Connection::open(&db_path)
            .with_context(|| format!("Failed to open blackboard DB: {}", db_path.display()))?;

        // Keep the same "production-ish" tuning you already use elsewhere.
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous  = NORMAL;
             PRAGMA mmap_size    = 8388608;
             PRAGMA cache_size   = -2000;
             PRAGMA temp_store   = MEMORY;",
        )
        .context("Failed to set blackboard DB PRAGMAs")?;

        Self::init_schema(&conn)?;

        Ok(Self {
            conn: Mutex::new(conn),
            db_path,
        })
    }

    fn init_schema(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS blackboard_events (
                id                TEXT PRIMARY KEY,
                kind              TEXT NOT NULL,
                agent_tag         TEXT,
                payload_json      TEXT NOT NULL,
                blast_radius      INTEGER NOT NULL DEFAULT 0,
                requires_approval INTEGER NOT NULL DEFAULT 0,
                status            TEXT NOT NULL DEFAULT 'pending',
                created_at        TEXT NOT NULL,
                updated_at        TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_blackboard_events_status_created
                ON blackboard_events(status, created_at);

            CREATE TABLE IF NOT EXISTS blackboard_audit (
                id          TEXT PRIMARY KEY,
                action      TEXT NOT NULL,
                actor       TEXT NOT NULL,
                details_json TEXT NOT NULL,
                created_at  TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_blackboard_audit_action_created
                ON blackboard_audit(action, created_at);",
        )
        .context("Failed to initialize blackboard schema")?;
        Ok(())
    }

    /// Preferred entry point: enqueue using the typed ontology.
    /// The string representation is stored in the DB for human-readable queries.
    pub fn enqueue_typed(
        &self,
        kind: BlackboardEventKind,
        agent_tag: Option<&str>,
        payload: &serde_json::Value,
        blast_radius: i64,
        requires_approval: bool,
    ) -> Result<String> {
        self.enqueue_event(
            kind.as_str(),
            agent_tag,
            payload,
            blast_radius,
            requires_approval,
        )
    }

    /// Legacy string-kind entry point — kept for backward compat.
    pub fn enqueue_event(
        &self,
        kind: &str,
        agent_tag: Option<&str>,
        payload: &serde_json::Value,
        blast_radius: i64,
        requires_approval: bool,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let payload_json = serde_json::to_string(payload).context("Failed to serialize payload")?;

        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        conn.execute(
            "INSERT INTO blackboard_events
             (id, kind, agent_tag, payload_json, blast_radius, requires_approval, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'pending', ?7, ?8)",
            params![
                id,
                kind,
                agent_tag,
                payload_json,
                blast_radius,
                if requires_approval { 1 } else { 0 },
                now,
                now
            ],
        )
        .context("Failed to insert blackboard event")?;

        Ok(id)
    }

    /// Parse a status string from the DB into the enum variant.
    fn parse_status(s: &str) -> BlackboardEventStatus {
        match s {
            "processed" => BlackboardEventStatus::Processed,
            "skipped" => BlackboardEventStatus::Skipped,
            "rate_limited" => BlackboardEventStatus::RateLimited,
            "needs_approval" => BlackboardEventStatus::NeedsApproval,
            "error" => BlackboardEventStatus::Error,
            _ => BlackboardEventStatus::Pending,
        }
    }

    /// Map a SQLite row (SELECT id, kind, agent_tag, payload_json, blast_radius,
    /// requires_approval, status, created_at) into a `BlackboardEvent`.
    fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<BlackboardEvent> {
        let kind_raw: String = row.get(1)?;
        let payload_raw: String = row.get(3)?;
        let payload: serde_json::Value =
            serde_json::from_str(&payload_raw).unwrap_or(serde_json::json!({ "raw": payload_raw }));
        let status_raw: String = row.get(6)?;
        let created_at_raw: String = row.get(7)?;
        let created_at = DateTime::parse_from_rfc3339(&created_at_raw)
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        Ok(BlackboardEvent {
            id: row.get(0)?,
            typed_kind: BlackboardEventKind::from_str(&kind_raw),
            kind: kind_raw,
            agent_tag: row.get(2)?,
            payload,
            blast_radius: row.get(4)?,
            requires_approval: row.get::<_, i64>(5)? != 0,
            status: Self::parse_status(&status_raw),
            created_at,
        })
    }

    pub fn fetch_pending_events(&self, limit: usize) -> Result<Vec<BlackboardEvent>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        #[allow(clippy::cast_possible_wrap)]
        let limit_i64 = limit as i64;

        let mut stmt = conn.prepare(
            "SELECT id, kind, agent_tag, payload_json, blast_radius, requires_approval, status, created_at
             FROM blackboard_events
             WHERE status = 'pending'
             ORDER BY created_at ASC
             LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit_i64], Self::map_row)?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn set_event_status(
        &self,
        id: &str,
        status: BlackboardEventStatus,
        error: Option<&str>,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        let status_str = status.as_str();

        if let Some(err) = error {
            // Preserve error details inside audit log instead of adding a new column.
            let _ = self.append_audit(
                "blackboard_event_error",
                "system",
                &serde_json::json!({ "event_id": id, "status": status_str, "error": err }),
            );
        }

        conn.execute(
            "UPDATE blackboard_events SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status_str, now, id],
        )
        .context("Failed to update event status")?;
        Ok(())
    }

    pub fn append_audit(
        &self,
        action: &str,
        actor: &str,
        details: &serde_json::Value,
    ) -> Result<()> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let details_json =
            serde_json::to_string(details).context("Failed to serialize audit details")?;

        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        conn.execute(
            "INSERT INTO blackboard_audit (id, action, actor, details_json, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, action, actor, details_json, now],
        )
        .context("Failed to insert blackboard audit row")?;
        Ok(())
    }

    pub fn count_audit_actions_in_last_hour(&self, action: &str) -> Result<u32> {
        let cutoff = (Utc::now() - Duration::hours(1)).to_rfc3339();
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM blackboard_audit WHERE action = ?1 AND created_at >= ?2",
            params![action, cutoff],
            |row| row.get(0),
        )?;
        Ok(u32::try_from(count).unwrap_or(u32::MAX))
    }

    pub fn get_event(&self, id: &str) -> Result<Option<BlackboardEvent>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        let mut stmt = conn.prepare(
            "SELECT id, kind, agent_tag, payload_json, blast_radius, requires_approval, status, created_at
             FROM blackboard_events WHERE id = ?1 LIMIT 1",
        )?;
        let row = stmt
            .query_row(params![id], |row| {
                let payload_raw: String = row.get(3)?;
                let payload: serde_json::Value = serde_json::from_str(&payload_raw)
                    .unwrap_or(serde_json::json!({ "raw": payload_raw }));
                let status_raw: String = row.get(6)?;
                let status = match status_raw.as_str() {
                    "pending" => BlackboardEventStatus::Pending,
                    "processed" => BlackboardEventStatus::Processed,
                    "skipped" => BlackboardEventStatus::Skipped,
                    "rate_limited" => BlackboardEventStatus::RateLimited,
                    "needs_approval" => BlackboardEventStatus::NeedsApproval,
                    "error" => BlackboardEventStatus::Error,
                    _ => BlackboardEventStatus::Pending,
                };
                let created_at_raw: String = row.get(7)?;
                let created_at = DateTime::parse_from_rfc3339(&created_at_raw)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                Ok(BlackboardEvent {
                    id: row.get(0)?,
                    typed_kind: BlackboardEventKind::from_str(&row.get::<_, String>(1)?),
                    kind: row.get(1)?,
                    agent_tag: row.get(2)?,
                    payload,
                    blast_radius: row.get(4)?,
                    requires_approval: row.get::<_, i64>(5)? != 0,
                    status,
                    created_at,
                })
            })
            .optional()?;

        Ok(row)
    }

    /// Fetch pending events matching a specific typed kind (used by circuit breaker).
    pub fn fetch_pending_by_kind(
        &self,
        kind: &BlackboardEventKind,
        limit: usize,
    ) -> Result<Vec<BlackboardEvent>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        #[allow(clippy::cast_possible_wrap)]
        let limit_i64 = limit as i64;
        let kind_str = kind.as_str();
        let mut stmt = conn.prepare(
            "SELECT id, kind, agent_tag, payload_json, blast_radius, requires_approval, status, created_at
             FROM blackboard_events
             WHERE status = 'pending' AND kind = ?1
             ORDER BY created_at ASC
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![kind_str, limit_i64], Self::map_row)?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    /// Count pending events of a specific kind (quick check for EDZL / rate limits).
    pub fn count_pending_by_kind(&self, kind: &BlackboardEventKind) -> Result<u32> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        let kind_str = kind.as_str();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM blackboard_events WHERE status = 'pending' AND kind = ?1",
            params![kind_str],
            |row| row.get(0),
        )?;
        Ok(u32::try_from(count).unwrap_or(u32::MAX))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn blackboard_open_and_enqueue_and_fetch() {
        let tmp = TempDir::new().unwrap();
        let bb = BlackboardDb::open(tmp.path()).unwrap();

        let id = bb
            .enqueue_event(
                "journal_observation",
                Some("planner"),
                &serde_json::json!({"classification":"behavioral","text":"missed gym"}),
                2,
                false,
            )
            .unwrap();

        let events = bb.fetch_pending_events(10).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, id);
        assert_eq!(events[0].kind, "journal_observation");
        assert_eq!(events[0].agent_tag.as_deref(), Some("planner"));
        assert_eq!(events[0].blast_radius, 2);
        assert!(!events[0].requires_approval);
    }

    #[test]
    fn blackboard_typed_enqueue_round_trip() {
        let tmp = TempDir::new().unwrap();
        let bb = BlackboardDb::open(tmp.path()).unwrap();

        let id = bb
            .enqueue_typed(
                BlackboardEventKind::JournalObservationBehavioral,
                Some("health"),
                &serde_json::json!({"text":"skipped gym"}),
                1,
                false,
            )
            .unwrap();

        let e = bb.get_event(&id).unwrap().unwrap();
        assert_eq!(e.kind, "journal_observation_behavioral");
        assert_eq!(
            e.typed_kind,
            BlackboardEventKind::JournalObservationBehavioral
        );
        assert!(!e.typed_kind.is_high_priority());
        assert!(!e.typed_kind.is_rag_only());
    }

    #[test]
    fn blackboard_critical_is_high_priority() {
        assert!(BlackboardEventKind::JournalObservationCritical.is_high_priority());
        assert!(BlackboardEventKind::HealthCapabilityOverride.is_high_priority());
        assert!(!BlackboardEventKind::PlannerScheduleProposal.is_high_priority());
    }

    #[test]
    fn blackboard_informational_is_rag_only() {
        assert!(BlackboardEventKind::JournalObservationInformational.is_rag_only());
        assert!(!BlackboardEventKind::JournalObservationBehavioral.is_rag_only());
    }

    #[test]
    fn fetch_pending_by_kind_filters_correctly() {
        let tmp = TempDir::new().unwrap();
        let bb = BlackboardDb::open(tmp.path()).unwrap();

        bb.enqueue_typed(
            BlackboardEventKind::PlannerScheduleProposal,
            None,
            &serde_json::json!({"blocks":3}),
            2,
            false,
        )
        .unwrap();
        bb.enqueue_typed(
            BlackboardEventKind::CareerRiskFlag,
            None,
            &serde_json::json!({"ratio":0.8}),
            1,
            false,
        )
        .unwrap();

        let planner = bb
            .fetch_pending_by_kind(&BlackboardEventKind::PlannerScheduleProposal, 10)
            .unwrap();
        assert_eq!(planner.len(), 1);
        assert_eq!(
            planner[0].typed_kind,
            BlackboardEventKind::PlannerScheduleProposal
        );

        let career = bb
            .fetch_pending_by_kind(&BlackboardEventKind::CareerRiskFlag, 10)
            .unwrap();
        assert_eq!(career.len(), 1);
    }

    #[test]
    fn blackboard_status_update() {
        let tmp = TempDir::new().unwrap();
        let bb = BlackboardDb::open(tmp.path()).unwrap();

        let id = bb
            .enqueue_event("test", None, &serde_json::json!({"x":1}), 0, false)
            .unwrap();
        bb.set_event_status(&id, BlackboardEventStatus::Processed, None)
            .unwrap();

        let e = bb.get_event(&id).unwrap().unwrap();
        assert_eq!(e.status, BlackboardEventStatus::Processed);
    }

    #[test]
    fn audit_count_in_last_hour() {
        let tmp = TempDir::new().unwrap();
        let bb = BlackboardDb::open(tmp.path()).unwrap();
        bb.append_audit(
            "mirror_event_process",
            "system",
            &serde_json::json!({"ok":true}),
        )
        .unwrap();
        let count = bb
            .count_audit_actions_in_last_hour("mirror_event_process")
            .unwrap();
        assert!(count >= 1);
    }
}
