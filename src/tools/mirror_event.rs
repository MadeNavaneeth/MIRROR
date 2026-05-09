use super::traits::{Tool, ToolResult};
use crate::mirror::blackboard::BlackboardDb;
use crate::security::SecurityPolicy;
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;

/// Tool to append events into the Mirror blackboard.
///
/// This is the primary "Journal -> Blackboard" bridge, and also supports
/// other agents posting structured events (career risk, readiness update, etc).
pub struct MirrorEventTool {
    security: Arc<SecurityPolicy>,
}

impl MirrorEventTool {
    pub fn new(security: Arc<SecurityPolicy>) -> Self {
        Self { security }
    }
}

#[async_trait]
impl Tool for MirrorEventTool {
    fn name(&self) -> &str {
        "mirror_event"
    }

    fn description(&self) -> &str {
        "Write a structured event to the local Mirror blackboard (SQLite)"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "kind": { "type": "string", "description": "Event kind (e.g. journal_observation, plan_request, career_risk, readiness_update)" },
                "agent": { "type": "string", "description": "Optional agent tag to activate (e.g. planner, career, health, journal)" },
                "payload": { "type": "object", "description": "JSON payload for the event" },
                "blast_radius": { "type": "integer", "description": "Estimated downstream impact count; used for circuit breaker gating", "default": 0 },
                "requires_approval": { "type": "boolean", "description": "If true, event will be gated for supervised approval", "default": false }
            },
            "required": ["kind", "payload"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let kind = args
            .get("kind")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'kind' parameter"))?;

        let agent = args.get("agent").and_then(|v| v.as_str());

        let payload = args
            .get("payload")
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Missing 'payload' parameter"))?;

        let blast_radius = args
            .get("blast_radius")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        let requires_approval = args
            .get("requires_approval")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let bb = BlackboardDb::open(&self.security.workspace_dir)?;
        let id = bb.enqueue_event(kind, agent, &payload, blast_radius, requires_approval)?;
        let _ = bb.append_audit(
            "mirror_event_enqueue",
            "tool:mirror_event",
            &serde_json::json!({
                "event_id": id,
                "kind": kind,
                "agent": agent,
                "blast_radius": blast_radius,
                "requires_approval": requires_approval
            }),
        );

        Ok(ToolResult::success(
            serde_json::json!({ "event_id": id }).to_string(),
        ))
    }
}
