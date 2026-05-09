//! Multi-agent conflict resolution for the Mirror system.
//!
//! Implements a deterministic mediator inspired by the Altruistic Gradient
//! Adjustment (AgA) algorithm described in research §4.2.
//!
//! ## The Problem (§4.1 — The Alignment Dilemma)
//!
//! When the Career Agent advocates for an extended deep-work block and the
//! Health Agent simultaneously detects low biological readiness, the Planner
//! receives contradictory signals. A binary override (Health always beats
//! Career) produces suboptimal long-term outcomes. The system needs a weighted
//! mediator that can choose *compromise* strategies, not just binary winners.
//!
//! ## The Solution (§4.2 — AgA-inspired weighted mediation)
//!
//! Each agent submits an `AgentObjective` with a 0.0–1.0 priority score and
//! an urgency coefficient. `mediate()` computes weighted scores and returns one
//! of three `ConflictResolution` strategies:
//!
//! - **`HealthOverride`** — severe biological deficit; Career agent blocked.
//! - **`CompromiseWithBreaks`** — mild deficit; Career approved with enforced
//!   Pomodoro-style break intervals.
//! - **`CareerApproved`** — Career urgency dominates; full deep-work block.
//!
//! The health agent's weight is multiplied by `HEALTH_PRIMACY_FACTOR` (1.3)
//! to reflect the research finding that biological capacity is the ultimate
//! constraint on sustained performance.

// ── Constants ─────────────────────────────────────────────────────────────────

/// Biological primacy multiplier applied to the Health agent's priority score.
/// Reflects the research principle: no career urgency overrides severe physiological
/// deficit. Value of 1.3 means Health dominates unless Career urgency is 30% higher.
const HEALTH_PRIMACY_FACTOR: f32 = 1.3;

/// If health_weighted_score / career_weighted_score exceeds this threshold,
/// the health override is absolute (no compromise offered).
const HARD_OVERRIDE_RATIO: f32 = 1.5;

/// Default break interval (minutes) when a compromise is reached.
const DEFAULT_BREAK_INTERVAL_MINUTES: u32 = 25;

// ── Types ─────────────────────────────────────────────────────────────────────

/// An agent's objective submitted to the conflict mediator.
///
/// Scores are in the range [0.0, 1.0]. Urgency is a time-pressure coefficient
/// that scales the priority — a high-priority but low-urgency career task can
/// be outweighed by a moderate-priority health concern.
#[derive(Debug, Clone)]
pub struct AgentObjective {
    /// Identifying label for the agent (e.g. `"health"`, `"career"`).
    pub agent_tag: &'static str,
    /// Objective priority: 0.0 = background, 1.0 = critical.
    pub priority_score: f32,
    /// Time-pressure coefficient: 0.0 = no deadline, 1.0 = deadline imminent.
    pub urgency_score: f32,
}

impl AgentObjective {
    /// Composite weighted score used in mediation.
    fn weighted_score(&self) -> f32 {
        // Clamp inputs defensively; values outside [0, 1] are programmer errors.
        let p = self.priority_score.clamp(0.0, 1.0);
        let u = self.urgency_score.clamp(0.0, 1.0);
        p * u.max(0.1) // floor urgency at 0.1 so low-urgency tasks still register
    }
}

/// The mediator's resolution strategy returned to the Planner.
#[derive(Debug, Clone, PartialEq)]
pub enum ConflictResolution {
    /// Severe biological deficit — deep-work blocks cancelled; recovery prioritised.
    HealthOverride,
    /// Mild deficit — Career objective approved, but structured breaks enforced.
    CompromiseWithBreaks {
        /// Minutes between mandatory breaks (Pomodoro-style).
        break_interval_minutes: u32,
    },
    /// Career urgency dominates — full deep-work block approved.
    CareerApproved,
}

impl ConflictResolution {
    /// Human-readable label for audit and telemetry.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::HealthOverride => "health_override",
            Self::CompromiseWithBreaks { .. } => "compromise_with_breaks",
            Self::CareerApproved => "career_approved",
        }
    }
}

// ── Mediator ──────────────────────────────────────────────────────────────────

/// Mediate between competing Health and Career objectives.
///
/// Returns the `ConflictResolution` strategy the Planner should apply when
/// scheduling the next work block.
///
/// ```
/// use mirror::mirror::conflict::{AgentObjective, mediate, ConflictResolution};
///
/// let health = AgentObjective { agent_tag: "health", priority_score: 0.9, urgency_score: 1.0 };
/// let career = AgentObjective { agent_tag: "career", priority_score: 0.7, urgency_score: 0.8 };
/// assert_eq!(mediate(&health, &career), ConflictResolution::HealthOverride);
/// ```
pub fn mediate(health: &AgentObjective, career: &AgentObjective) -> ConflictResolution {
    let health_w = health.weighted_score() * HEALTH_PRIMACY_FACTOR;
    let career_w = career.weighted_score();

    if health_w > career_w * HARD_OVERRIDE_RATIO {
        // Severe imbalance — health completely dominates.
        ConflictResolution::HealthOverride
    } else if health_w > career_w {
        // Mild imbalance — approve career work with structured recovery breaks.
        ConflictResolution::CompromiseWithBreaks {
            break_interval_minutes: DEFAULT_BREAK_INTERVAL_MINUTES,
        }
    } else {
        // Career urgency dominates or scores are roughly equal.
        ConflictResolution::CareerApproved
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn health(p: f32, u: f32) -> AgentObjective {
        AgentObjective {
            agent_tag: "health",
            priority_score: p,
            urgency_score: u,
        }
    }

    fn career(p: f32, u: f32) -> AgentObjective {
        AgentObjective {
            agent_tag: "career",
            priority_score: p,
            urgency_score: u,
        }
    }

    #[test]
    fn severe_health_deficit_overrides_career() {
        // Health: score 1.0 * 1.3 = 1.3. Career: 0.5 * 0.5 = 0.25. Ratio = 5.2 > 1.5.
        let result = mediate(&health(1.0, 1.0), &career(0.5, 0.5));
        assert_eq!(result, ConflictResolution::HealthOverride);
    }

    #[test]
    fn mild_health_deficit_yields_compromise() {
        // Health: 0.7 * 1.3 = 0.91. Career: 0.8 * 0.9 = 0.72. Ratio = 1.26 < 1.5.
        let result = mediate(&health(0.7, 1.0), &career(0.8, 0.9));
        assert!(
            matches!(
                result,
                ConflictResolution::CompromiseWithBreaks {
                    break_interval_minutes: 25
                }
            ),
            "Expected compromise with 25-min breaks, got {:?}",
            result
        );
    }

    #[test]
    fn high_career_urgency_approved_when_health_ok() {
        // Health: 0.2 * 1.3 = 0.26. Career: 0.9 * 1.0 = 0.90. Career wins.
        let result = mediate(&health(0.2, 1.0), &career(0.9, 1.0));
        assert_eq!(result, ConflictResolution::CareerApproved);
    }

    #[test]
    fn equal_scores_yield_career_approved() {
        // Equal after multiplier: health 0.5 * 1.3 = 0.65, career 0.65 — career wins.
        let result = mediate(&health(0.5, 1.0), &career(0.65, 1.0));
        assert_eq!(result, ConflictResolution::CareerApproved);
    }

    #[test]
    fn health_primacy_factor_applied() {
        // Without HEALTH_PRIMACY_FACTOR these would be equal (0.6 vs 0.6).
        // With factor 1.3: health_w = 0.78 > career_w = 0.6 → compromise not override.
        let result = mediate(&health(0.6, 1.0), &career(0.6, 1.0));
        assert!(matches!(
            result,
            ConflictResolution::CompromiseWithBreaks { .. }
        ));
    }

    #[test]
    fn as_str_labels_are_stable() {
        assert_eq!(
            ConflictResolution::HealthOverride.as_str(),
            "health_override"
        );
        assert_eq!(
            ConflictResolution::CompromiseWithBreaks {
                break_interval_minutes: 25
            }
            .as_str(),
            "compromise_with_breaks"
        );
        assert_eq!(
            ConflictResolution::CareerApproved.as_str(),
            "career_approved"
        );
    }
}
