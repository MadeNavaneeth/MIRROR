//! Input taxonomy classifier for the Mirror Journal Agent.
//!
//! Implements the tri-partite "Reality Taxonomy" from research §5.1:
//!
//! - **Informational (I)** — neutral facts, references, observations.
//!   Routes ONLY to the vector RAG store; does not wake downstream agents.
//!
//! - **Behavioral (B)** — concrete actions taken or explicitly omitted.
//!   Posted to the blackboard → triggers Health Agent readiness recalc and
//!   Career Agent proof-of-work audit.
//!
//! - **Critical (C)** — emotional distress, burnout signals, identity-level
//!   reflections. Highest blackboard priority; triggers immediate Planner
//!   recalibration and bypasses standard queue ordering.
//!
//! Also detects **Zeigarnik open loops** (research §5.2): incomplete tasks
//! that the user intends to return to. These are escalated to the Planner for
//! an immediate temporal anchor rather than being filed as a passive note.
//!
//! ## Design notes
//!
//! The classifier is intentionally keyword/heuristic-based and runs in
//! microseconds with zero allocations beyond the input slice. It can be
//! replaced by an edge SLM call (Gemma 3 4B, ~32 ms) in a future iteration
//! without changing the callsite API.

// ── Input classification ──────────────────────────────────────────────────────

/// The tri-partite classification of a journal input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputClass {
    /// Neutral facts / references — RAG-only, no agent wake-up.
    Informational,
    /// Concrete actions taken or omitted — triggers Health + Career agents.
    Behavioral,
    /// Emotional distress / burnout — highest priority, immediate Planner recalibration.
    Critical,
}

impl InputClass {
    /// Human-readable label for logging and audit trails.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Informational => "informational",
            Self::Behavioral => "behavioral",
            Self::Critical => "critical",
        }
    }

    /// Map to the blackboard ontology event kind that should be posted.
    pub fn to_blackboard_kind(self) -> crate::mirror::blackboard::BlackboardEventKind {
        use crate::mirror::blackboard::BlackboardEventKind;
        match self {
            Self::Informational => BlackboardEventKind::JournalObservationInformational,
            Self::Behavioral => BlackboardEventKind::JournalObservationBehavioral,
            Self::Critical => BlackboardEventKind::JournalObservationCritical,
        }
    }
}

// ── Open-loop detection ───────────────────────────────────────────────────────

/// Returns `true` when the input contains language indicating an incomplete
/// task that the user intends to complete later (Zeigarnik open-loop pattern).
///
/// When detected, the caller should enqueue a `JournalOpenLoop` blackboard
/// event so the Planner can generate a concrete temporal anchor.
pub fn has_open_loop(text: &str) -> bool {
    let lower = text.to_lowercase();
    // Explicit deferral markers
    lower.contains("need to")
        || lower.contains("have to")
        || lower.contains("got to")
        || lower.contains("should finish")
        || lower.contains("need to finish")
        || lower.contains("follow up")
        || lower.contains("follow-up")
        || lower.contains("get back to")
        || lower.contains("will do")
        || lower.contains("remind me")
        || lower.contains("later today")
        || lower.contains("later this week")
        // Incomplete state markers
        || lower.contains("still pending")
        || lower.contains("not done yet")
        || lower.contains("haven't finished")
        || lower.contains("still need to")
}

// ── Main classifier ───────────────────────────────────────────────────────────

/// Classify a raw journal input string into one of the three taxonomy tiers.
///
/// Runs in O(n) time with no allocations beyond `to_lowercase()`. Call before
/// routing to memory or the blackboard.
///
/// Precedence: **Critical > Behavioral > Informational**.
pub fn classify_input(text: &str) -> InputClass {
    let lower = text.to_lowercase();

    // ── Critical signals ───────────────────────────────────────────────────
    // Emotional distress, burnout, overwhelm, identity-level reflection.
    // Checked first because they require immediate Planner intervention.
    let critical_signals: &[&str] = &[
        "overwhelmed",
        "burnout",
        "burning out",
        "burned out",
        "burnt out",
        "anxious",
        "anxiety",
        "panicking",
        "can't focus",
        "cannot focus",
        "can't concentrate",
        "can't cope",
        "losing motivation",
        "lost motivation",
        "want to quit",
        "feeling stuck",
        "depressed",
        "exhausted",
        "mental breakdown",
        "breaking point",
        "too much to handle",
        "completely drained",
        "totally drained",
    ];
    if critical_signals.iter().any(|s| lower.contains(s)) {
        return InputClass::Critical;
    }

    // ── Behavioral signals ─────────────────────────────────────────────────
    // Concrete actions taken or explicitly missed.
    let behavioral_signals: &[&str] = &[
        "skipped",
        "missed",
        "didn't",
        "did not",
        "couldn't",
        "could not",
        "completed",
        "finished",
        "submitted",
        "deployed",
        "pushed",
        "committed",
        "merged",
        "shipped",
        "spent",
        "worked on",
        "studied",
        "practiced",
        "ran",
        "exercised",
        "went to",
        "attended",
        "procrastinated",
        "avoided",
        "ignored",
        "scrolled",
        "wasted",
    ];
    if behavioral_signals.iter().any(|s| lower.contains(s)) {
        return InputClass::Behavioral;
    }

    // ── Default: Informational ─────────────────────────────────────────────
    InputClass::Informational
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mirror::blackboard::BlackboardEventKind;

    // Critical tier
    #[test]
    fn classify_overwhelmed_is_critical() {
        assert_eq!(
            classify_input("I feel completely overwhelmed by this sprint"),
            InputClass::Critical
        );
    }

    #[test]
    fn classify_burnout_is_critical() {
        assert_eq!(
            classify_input("I think I'm burning out"),
            InputClass::Critical
        );
    }

    #[test]
    fn classify_anxiety_is_critical() {
        assert_eq!(
            classify_input("Feeling really anxious about the deadline"),
            InputClass::Critical
        );
    }

    // Behavioral tier
    #[test]
    fn classify_skipped_gym_is_behavioral() {
        assert_eq!(
            classify_input("Skipped the gym this morning again"),
            InputClass::Behavioral
        );
    }

    #[test]
    fn classify_completed_task_is_behavioral() {
        assert_eq!(
            classify_input("Completed the auth module PR today"),
            InputClass::Behavioral
        );
    }

    #[test]
    fn classify_procrastinated_is_behavioral() {
        assert_eq!(
            classify_input("procrastinated on the docs for two hours"),
            InputClass::Behavioral
        );
    }

    // Informational tier
    #[test]
    fn classify_article_reference_is_informational() {
        assert_eq!(
            classify_input("Read an article about SLM quantization"),
            InputClass::Informational
        );
    }

    #[test]
    fn classify_meeting_note_is_informational() {
        assert_eq!(
            classify_input("Team standup was at 10am today"),
            InputClass::Informational
        );
    }

    // Open-loop detection
    #[test]
    fn open_loop_detected_for_defer_language() {
        assert!(has_open_loop(
            "I need to finish the AWS deployment docs later"
        ));
        assert!(has_open_loop(
            "Should follow up with the design team this week"
        ));
        assert!(has_open_loop("Will do the code review later today"));
        assert!(has_open_loop("Remind me to update the README"));
    }

    #[test]
    fn no_open_loop_for_completed_action() {
        assert!(!has_open_loop(
            "I finished the deployment docs this morning"
        ));
        assert!(!has_open_loop("Read an article about Rust async"));
    }

    // Blackboard kind mapping
    #[test]
    fn critical_maps_to_critical_event_kind() {
        assert_eq!(
            InputClass::Critical.to_blackboard_kind(),
            BlackboardEventKind::JournalObservationCritical
        );
    }

    #[test]
    fn informational_maps_to_informational_event_kind() {
        assert_eq!(
            InputClass::Informational.to_blackboard_kind(),
            BlackboardEventKind::JournalObservationInformational
        );
    }

    // Precedence: Critical > Behavioral
    #[test]
    fn critical_beats_behavioral_when_both_match() {
        // "completed" would match Behavioral, but "overwhelmed" takes precedence.
        let input = "I completed the sprint but I feel totally overwhelmed";
        assert_eq!(classify_input(input), InputClass::Critical);
    }
}
