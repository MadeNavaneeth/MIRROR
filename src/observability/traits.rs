use std::time::Duration;

// ── Proof-of-Work types (research §7.2 — Career Agent observability) ──────────

/// Distinguishes active skill execution from passive learning.
///
/// The research notes that career risk emerges when passive consumption
/// (watching tutorials, reading docs) dominates over active execution
/// (committing code, shipping features) relative to target-role requirements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofOfWorkType {
    /// Concrete output: commit, PR merge, deployment, architectural design.
    ActiveExecution,
    /// Consumption: tutorial, reading, lecture, passive review.
    PassiveLearning,
}

/// Events the observer can record
#[derive(Debug, Clone)]
pub enum ObserverEvent {
    AgentStart {
        provider: String,
        model: String,
    },
    /// A request is about to be sent to an LLM provider.
    ///
    /// This is emitted immediately before a provider call so observers can print
    /// user-facing progress without leaking prompt contents.
    LlmRequest {
        provider: String,
        model: String,
        messages_count: usize,
    },
    /// Result of a single LLM provider call.
    LlmResponse {
        provider: String,
        model: String,
        duration: Duration,
        success: bool,
        error_message: Option<String>,
    },
    AgentEnd {
        duration: Duration,
        tokens_used: Option<u64>,
    },
    /// A tool call is about to be executed.
    ToolCallStart {
        tool: String,
    },
    ToolCall {
        tool: String,
        duration: Duration,
        success: bool,
    },
    /// The agent produced a final answer for the current user message.
    TurnComplete,
    ChannelMessage {
        channel: String,
        direction: String,
    },
    HeartbeatTick,
    Error {
        component: String,
        message: String,
    },

    // ── Mirror behavioral intelligence events (research §7.2, §8, §10.2) ────
    /// A quantifiable unit of user productivity (Career Agent proof-of-work).
    ///
    /// Emitted when the Career Agent maps a behavioral observation (e.g. a git
    /// commit) to a competency in the Bloom's Taxonomy matrix.
    ProofOfWork {
        /// Active execution vs passive learning.
        activity_type: ProofOfWorkType,
        /// Bloom's Taxonomy level: 1=Remember, 2=Understand, 3=Apply,
        /// 4=Analyze, 5=Evaluate, 6=Create.
        bloom_level: u8,
        /// Domain competency label (e.g. "DevOps", "SystemDesign", "MLOps").
        competency_tag: String,
        /// Duration of the activity in minutes.
        duration_minutes: u32,
    },

    /// Career risk signal emitted when passive/active ratio or Bloom's taxonomy
    /// gap exceeds a configurable threshold.
    CareerRisk {
        /// Passive / (passive + active) ratio over the configured rolling window.
        passive_ratio: f32,
        /// Gap between user's current Bloom level and target role's required level.
        target_bloom_gap: u8,
    },

    /// Circuit breaker decision recorded for governance telemetry.
    BlackboardCircuitBreaker {
        /// The blackboard event kind that was evaluated.
        event_kind: String,
        /// Decision: "allow", "rate_limit", or "escalate_to_user".
        decision: String,
        /// Reason string provided by the circuit breaker policy.
        reason: String,
    },
}

/// Numeric metrics
#[derive(Debug, Clone)]
pub enum ObserverMetric {
    RequestLatency(Duration),
    TokensUsed(u64),
    ActiveSessions(u64),
    QueueDepth(u64),
}

/// Core observability trait — implement for any backend
pub trait Observer: Send + Sync + 'static {
    /// Record a discrete event
    fn record_event(&self, event: &ObserverEvent);

    /// Record a numeric metric
    fn record_metric(&self, metric: &ObserverMetric);

    /// Flush any buffered data (no-op for most backends)
    fn flush(&self) {}

    /// Human-readable name of this observer
    fn name(&self) -> &str;

    /// Downcast to `Any` for backend-specific operations
    fn as_any(&self) -> &dyn std::any::Any
    where
        Self: Sized,
    {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use std::time::Duration;

    #[derive(Default)]
    struct DummyObserver {
        events: Mutex<u64>,
        metrics: Mutex<u64>,
    }

    impl Observer for DummyObserver {
        fn record_event(&self, _event: &ObserverEvent) {
            let mut guard = self.events.lock().unwrap();
            *guard += 1;
        }

        fn record_metric(&self, _metric: &ObserverMetric) {
            let mut guard = self.metrics.lock().unwrap();
            *guard += 1;
        }

        fn name(&self) -> &str {
            "dummy-observer"
        }
    }

    #[test]
    fn observer_records_events_and_metrics() {
        let observer = DummyObserver::default();

        observer.record_event(&ObserverEvent::HeartbeatTick);
        observer.record_event(&ObserverEvent::Error {
            component: "test".into(),
            message: "boom".into(),
        });
        observer.record_metric(&ObserverMetric::TokensUsed(42));

        assert_eq!(*observer.events.lock().unwrap(), 2);
        assert_eq!(*observer.metrics.lock().unwrap(), 1);
    }

    #[test]
    fn observer_default_flush_and_as_any_work() {
        let observer = DummyObserver::default();

        observer.flush();
        assert_eq!(observer.name(), "dummy-observer");
        assert!(observer.as_any().downcast_ref::<DummyObserver>().is_some());
    }

    #[test]
    fn observer_event_and_metric_are_cloneable() {
        let event = ObserverEvent::ToolCall {
            tool: "shell".into(),
            duration: Duration::from_millis(10),
            success: true,
        };
        let metric = ObserverMetric::RequestLatency(Duration::from_millis(8));

        let cloned_event = event.clone();
        let cloned_metric = metric.clone();

        assert!(matches!(cloned_event, ObserverEvent::ToolCall { .. }));
        assert!(matches!(cloned_metric, ObserverMetric::RequestLatency(_)));
    }
}
