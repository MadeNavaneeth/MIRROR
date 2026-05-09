//! Deterministic circuit breaker for the Mirror blackboard system.
//!
//! Implements the governance layer described in research §10.2:
//! "Implementing Deterministic Circuit Breakers". Evaluates every proposed
//! blackboard action before it is committed, enforcing:
//!
//! 1. **Rate limiting** — max N schedule reshuffles per hour.
//! 2. **Blast-radius cap** — proposals touching > K downstream events escalate
//!    to the user via HITL (Human-In-The-Loop).
//! 3. **Mandatory HITL** — any event with `requires_approval = true` is
//!    immediately gated regardless of rate/blast state.
//!
//! This module intentionally has no async dependencies and no LLM calls.
//! All decisions are deterministic so they can be audited under pressure.

use crate::mirror::blackboard::{BlackboardDb, BlackboardEvent, BlackboardEventKind};
use anyhow::Result;

// ── Policy constants ──────────────────────────────────────────────────────────

/// Maximum number of `PlannerScheduleProposal` events processed per hour.
/// Exceeding this rate-limits further proposals to prevent runaway reshuffles.
const DEFAULT_MAX_RESHUFFLES_PER_HOUR: u32 = 2;

/// Any proposal whose `blast_radius` exceeds this threshold requires user
/// approval before being applied.
const DEFAULT_MAX_BLAST_RADIUS: i64 = 3;

// ── Decision ─────────────────────────────────────────────────────────────────

/// The circuit breaker's verdict for a proposed blackboard action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CircuitBreakerDecision {
    /// Action is safe to proceed immediately.
    Allow,
    /// Action exceeds the hourly rate limit — defer until the window resets.
    RateLimit,
    /// Blast radius is too large or `requires_approval` is set — pause and
    /// escalate to the user dashboard (HITL checkpoint).
    EscalateToUser { reason: &'static str },
}

impl CircuitBreakerDecision {
    /// Returns `true` when the action should be blocked (rate-limited or escalated).
    pub fn is_blocked(&self) -> bool {
        !matches!(self, Self::Allow)
    }
}

// ── Policy ────────────────────────────────────────────────────────────────────

/// Configurable circuit breaker policy.
///
/// Instantiate once per mirror runtime cycle and reuse; it holds no mutable
/// state itself — all rate counts are queried live from `BlackboardDb`.
#[derive(Debug, Clone)]
pub struct CircuitBreakerPolicy {
    /// Max `PlannerScheduleProposal` events processed per hour.
    pub max_reshuffles_per_hour: u32,
    /// Blast-radius threshold above which HITL approval is required.
    pub max_blast_radius: i64,
}

impl Default for CircuitBreakerPolicy {
    fn default() -> Self {
        Self {
            max_reshuffles_per_hour: DEFAULT_MAX_RESHUFFLES_PER_HOUR,
            max_blast_radius: DEFAULT_MAX_BLAST_RADIUS,
        }
    }
}

impl CircuitBreakerPolicy {
    /// Evaluate whether a proposed blackboard event should be allowed, rate-
    /// limited, or escalated to the user.
    ///
    /// Checks are performed in priority order:
    /// 1. Explicit `requires_approval` flag (from the producer).
    /// 2. Blast-radius cap (structural risk).
    /// 3. Hourly rate limit for `PlannerScheduleProposal` events.
    pub fn evaluate(
        &self,
        event: &BlackboardEvent,
        db: &BlackboardDb,
    ) -> Result<CircuitBreakerDecision> {
        // 1. Mandatory HITL — producer explicitly requested user approval.
        if event.requires_approval {
            return Ok(CircuitBreakerDecision::EscalateToUser {
                reason: "producer flagged requires_approval",
            });
        }

        // 2. Blast-radius guard — too many downstream changes cascade risk.
        if event.blast_radius > self.max_blast_radius {
            return Ok(CircuitBreakerDecision::EscalateToUser {
                reason: "blast_radius exceeds policy threshold",
            });
        }

        // 3. Rate limit for schedule reshuffles.
        if event.typed_kind == BlackboardEventKind::PlannerScheduleProposal {
            let recent = db.count_audit_actions_in_last_hour(
                BlackboardEventKind::PlannerScheduleProposal.as_str(),
            )?;
            if recent >= self.max_reshuffles_per_hour {
                return Ok(CircuitBreakerDecision::RateLimit);
            }
        }

        Ok(CircuitBreakerDecision::Allow)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mirror::blackboard::{BlackboardDb, BlackboardEventKind};
    use tempfile::TempDir;

    fn open_bb() -> (TempDir, BlackboardDb) {
        let tmp = TempDir::new().unwrap();
        let bb = BlackboardDb::open(tmp.path()).unwrap();
        (tmp, bb)
    }

    fn fetch_first(bb: &BlackboardDb) -> BlackboardEvent {
        bb.fetch_pending_events(1)
            .unwrap()
            .into_iter()
            .next()
            .unwrap()
    }

    #[test]
    fn allow_normal_event() {
        let (_tmp, bb) = open_bb();
        bb.enqueue_typed(
            BlackboardEventKind::CareerRiskFlag,
            None,
            &serde_json::json!({}),
            1,
            false,
        )
        .unwrap();
        let ev = fetch_first(&bb);
        let policy = CircuitBreakerPolicy::default();
        let decision = policy.evaluate(&ev, &bb).unwrap();
        assert_eq!(decision, CircuitBreakerDecision::Allow);
    }

    #[test]
    fn escalate_when_requires_approval() {
        let (_tmp, bb) = open_bb();
        bb.enqueue_typed(
            BlackboardEventKind::PlannerScheduleProposal,
            None,
            &serde_json::json!({}),
            1,
            true, // requires_approval = true
        )
        .unwrap();
        let ev = fetch_first(&bb);
        let policy = CircuitBreakerPolicy::default();
        let decision = policy.evaluate(&ev, &bb).unwrap();
        assert!(matches!(
            decision,
            CircuitBreakerDecision::EscalateToUser { .. }
        ));
    }

    #[test]
    fn escalate_when_blast_radius_exceeded() {
        let (_tmp, bb) = open_bb();
        bb.enqueue_typed(
            BlackboardEventKind::PlannerScheduleProposal,
            None,
            &serde_json::json!({}),
            10, // blast_radius > DEFAULT_MAX_BLAST_RADIUS (3)
            false,
        )
        .unwrap();
        let ev = fetch_first(&bb);
        let policy = CircuitBreakerPolicy::default();
        let decision = policy.evaluate(&ev, &bb).unwrap();
        assert!(matches!(
            decision,
            CircuitBreakerDecision::EscalateToUser { .. }
        ));
    }

    #[test]
    fn rate_limit_when_too_many_reshuffles() {
        let (_tmp, bb) = open_bb();

        // Simulate 2 reshuffles already processed this hour in the audit log.
        let kind_str = BlackboardEventKind::PlannerScheduleProposal.as_str();
        bb.append_audit(kind_str, "planner", &serde_json::json!({}))
            .unwrap();
        bb.append_audit(kind_str, "planner", &serde_json::json!({}))
            .unwrap();

        // Now propose a third — should be rate-limited.
        bb.enqueue_typed(
            BlackboardEventKind::PlannerScheduleProposal,
            None,
            &serde_json::json!({}),
            1,
            false,
        )
        .unwrap();
        let ev = fetch_first(&bb);
        let policy = CircuitBreakerPolicy::default();
        let decision = policy.evaluate(&ev, &bb).unwrap();
        assert_eq!(decision, CircuitBreakerDecision::RateLimit);
    }

    #[test]
    fn non_schedule_event_not_rate_limited() {
        let (_tmp, bb) = open_bb();

        // Flood the audit with planner reshuffles.
        let kind_str = BlackboardEventKind::PlannerScheduleProposal.as_str();
        for _ in 0..5 {
            bb.append_audit(kind_str, "planner", &serde_json::json!({}))
                .unwrap();
        }

        // A career-risk flag should still be allowed.
        bb.enqueue_typed(
            BlackboardEventKind::CareerRiskFlag,
            None,
            &serde_json::json!({}),
            1,
            false,
        )
        .unwrap();
        let ev = fetch_first(&bb);
        let policy = CircuitBreakerPolicy::default();
        let decision = policy.evaluate(&ev, &bb).unwrap();
        assert_eq!(decision, CircuitBreakerDecision::Allow);
    }

    #[test]
    fn circuit_breaker_decision_is_blocked_helper() {
        assert!(!CircuitBreakerDecision::Allow.is_blocked());
        assert!(CircuitBreakerDecision::RateLimit.is_blocked());
        assert!(CircuitBreakerDecision::EscalateToUser { reason: "test" }.is_blocked());
    }
}
