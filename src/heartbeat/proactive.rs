use crate::agent::loop_::process_message_with_agent;
use crate::auth::handler::AuthHandler;
use crate::auth::storage::CredentialStorage;
use crate::config::Config;
use crate::mirror::blackboard::{BlackboardDb, BlackboardEventStatus};
use crate::security::SecretStore;
use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;
use tokio::fs;
use tracing::{info, warn};

/// Proactive Intelligence Engine
/// Processes tasks from HEARTBEAT.md and prepares context using managed OAuth.
pub struct ProactiveEngine {
    config: Config,
    auth_handler: Arc<AuthHandler>,
}

impl ProactiveEngine {
    pub fn new(config: Config) -> Self {
        let secret_store = Arc::new(SecretStore::new(
            &config.workspace_dir,
            config.secrets.encrypt,
        ));
        let storage = Arc::new(CredentialStorage::new(
            config.workspace_dir.join("credentials.json"),
            secret_store,
        ));
        let auth_handler = Arc::new(AuthHandler::new(config.clone(), storage));

        Self {
            config,
            auth_handler,
        }
    }

    /// Perform a full proactive intelligence scan
    pub async fn scan(&self) -> Result<()> {
        info!("🔍 Starting Proactive Intelligence Scan...");

        // 1. Collect tasks from HEARTBEAT.md
        let tasks = self.collect_tasks().await?;
        let mut did_any = false;

        // 2. Process each task
        for task in tasks {
            did_any = true;
            info!("Proactive Processing: {}", task);
            if let Err(e) = self.process_task(&task).await {
                warn!("Proactive task failed: {}. Error: {}", task, e);
            }
        }

        // 3. Opportunistic activation via Mirror blackboard events
        // This is the core "deterministic blackboard pipeline" (DLBP) loop:
        // agents do not talk to each other directly — they react to shared state transitions.
        if let Err(e) = self.process_blackboard_events().await {
            warn!("Proactive blackboard processing failed: {e}");
        } else {
            did_any = true;
        }

        if !did_any {
            info!("Proactive Scan: No tasks or blackboard events found");
        }

        info!("✅ Proactive Scan Complete.");
        Ok(())
    }

    async fn collect_tasks(&self) -> Result<Vec<String>> {
        let path = self.config.workspace_dir.join("HEARTBEAT.md");
        if !path.exists() {
            return Ok(Vec::new());
        }
        let content = fs::read_to_string(&path).await?;
        Ok(content
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                trimmed.strip_prefix("- ").map(ToString::to_string)
            })
            .collect())
    }

    async fn process_task(&self, task_description: &str) -> Result<()> {
        let (agent_id, clean_task) = if task_description.starts_with('[') {
            if let Some(end_idx) = task_description.find(']') {
                let tag = task_description[1..end_idx].to_lowercase();
                (
                    Some(tag),
                    task_description[end_idx + 1..].trim().to_string(),
                )
            } else {
                (None, task_description.to_string())
            }
        } else {
            (None, task_description.to_string())
        };

        info!(
            "Proactive Reasoning: {} (Agent: {:?})",
            clean_task, agent_id
        );

        let mut compact_config = self.config.clone();
        compact_config.agent.compact_context = true;

        let response =
            process_message_with_agent(compact_config, &clean_task, agent_id.as_deref()).await?;

        // Log results to PROACTIVE_LOG.md
        self.log_proactive_action(task_description, &response)
            .await?;

        Ok(())
    }

    async fn log_proactive_action(&self, task: &str, result: &str) -> Result<()> {
        let log_path = self.config.workspace_dir.join("PROACTIVE_LOG.md");
        let timestamp = Utc::now().to_rfc3339();

        let entry = format!("\n## [{}] {}\n\n{}\n\n---\n", timestamp, task, result);

        if !log_path.exists() {
            fs::write(&log_path, "# Proactive Intelligence Log\n\n").await?;
        }

        let mut file = tokio::fs::OpenOptions::new()
            .append(true)
            .open(&log_path)
            .await?;

        use tokio::io::AsyncWriteExt;
        file.write_all(entry.as_bytes()).await?;

        Ok(())
    }

    async fn process_blackboard_events(&self) -> Result<()> {
        let bb = BlackboardDb::open(&self.config.workspace_dir)?;

        // Deterministic rate limiting: count audit actions in DB (not in-memory).
        let action_budget = self.config.autonomy.max_actions_per_hour;
        let used = bb.count_audit_actions_in_last_hour("mirror_event_process")?;
        if used >= action_budget {
            return Ok(());
        }

        // Process a small batch per scan to keep ticks bounded.
        let remaining = (action_budget - used).min(10);
        let events = bb.fetch_pending_events(remaining as usize)?;
        if events.is_empty() {
            return Ok(());
        }

        for ev in events {
            // Circuit breaker: blast radius + supervised autonomy gating.
            // If an event might cascade beyond 3 downstream changes, require approval.
            let mut requires_approval = ev.requires_approval;
            if ev.blast_radius > 3 {
                requires_approval = true;
            }

            if self.config.autonomy.level == crate::security::AutonomyLevel::Supervised
                && requires_approval
            {
                bb.set_event_status(&ev.id, BlackboardEventStatus::NeedsApproval, None)?;
                let _ = bb.append_audit(
                    "mirror_event_needs_approval",
                    "heartbeat:proactive",
                    &serde_json::json!({
                        "event_id": ev.id,
                        "kind": ev.kind,
                        "agent_tag": ev.agent_tag,
                        "blast_radius": ev.blast_radius
                    }),
                );
                continue;
            }

            // If we run out of budget mid-batch, mark remaining events rate-limited.
            let used_now = bb.count_audit_actions_in_last_hour("mirror_event_process")?;
            if used_now >= action_budget {
                bb.set_event_status(&ev.id, BlackboardEventStatus::RateLimited, None)?;
                continue;
            }

            let agent_tag = ev.agent_tag.as_deref();
            let message = format!(
                "Mirror blackboard event: kind={}\n\npayload={}",
                ev.kind,
                serde_json::to_string_pretty(&ev.payload)
                    .unwrap_or_else(|_| ev.payload.to_string())
            );

            match process_message_with_agent(self.config.clone(), &message, agent_tag).await {
                Ok(response) => {
                    bb.set_event_status(&ev.id, BlackboardEventStatus::Processed, None)?;
                    let _ = bb.append_audit(
                        "mirror_event_process",
                        "heartbeat:proactive",
                        &serde_json::json!({
                            "event_id": ev.id,
                            "kind": ev.kind,
                            "agent_tag": ev.agent_tag,
                            "blast_radius": ev.blast_radius,
                            "response_chars": response.len()
                        }),
                    );
                }
                Err(e) => {
                    bb.set_event_status(
                        &ev.id,
                        BlackboardEventStatus::Error,
                        Some(&e.to_string()),
                    )?;
                }
            }
        }

        Ok(())
    }
}
