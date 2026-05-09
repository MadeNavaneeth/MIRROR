use crate::auth::handler::AuthHandler;
use crate::security::SecurityPolicy;
use crate::tools::traits::{Tool, ToolResult};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct GogParams {
    pub action: String,
    pub query: Option<String>,
    pub limit: Option<usize>,
}

pub struct GogTool {
    security: Arc<SecurityPolicy>,
    auth_handler: Arc<AuthHandler>,
}

impl GogTool {
    pub fn new(security: Arc<SecurityPolicy>, auth_handler: Arc<AuthHandler>) -> Self {
        Self {
            security,
            auth_handler,
        }
    }

    async fn run_gog_command(&self, args: &[&str]) -> Result<String> {
        let mut command = Command::new("gog");

        // Ensure we have a valid token (refreshes if needed)
        if let Ok(token) = self.auth_handler.ensure_google_token().await {
            command.env("GOG_TOKEN", &token);
            tracing::debug!("Injected managed Google token into gog CLI");
        } else {
            tracing::warn!(
                "No managed Google token available for gog tool. Falling back to CLI auth."
            );
        }

        let output = command
            .args(args)
            .output()
            .context("Failed to execute 'gog' CLI. Is it installed?")?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Gog CLI error: {}", err);
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

#[async_trait]
impl Tool for GogTool {
    fn name(&self) -> &str {
        "gog"
    }

    fn description(&self) -> &str {
        "Google Workspace CLI - Access Gmail, Calendar, Drive, and Sheets."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["list-events", "list-emails", "search-drive", "read-sheet"],
                    "description": "The action to perform in Google Workspace."
                },
                "query": {
                    "type": "string",
                    "description": "Optional search query or identifier."
                },
                "limit": {
                    "type": "number",
                    "description": "Maximum number of results to return."
                }
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        let p: GogParams =
            serde_json::from_value(args).with_context(|| "Invalid parameters for gog tool")?;

        // Simple security check for tool execution
        if !self.security.is_command_allowed("gog") {
            return Ok(ToolResult::error(
                "Gog tool is not allowed by security policy".into(),
            ));
        }

        let mut cmd_args = vec![p.action.as_str()];
        if let Some(ref q) = p.query {
            cmd_args.push(q);
        }
        let limit_str;
        if let Some(l) = p.limit {
            limit_str = l.to_string();
            cmd_args.push("--limit");
            cmd_args.push(&limit_str);
        }

        match self.run_gog_command(&cmd_args).await {
            Ok(output) => Ok(ToolResult::success(output)),
            Err(e) => Ok(ToolResult::error(format!("Gog execution failed: {e}"))),
        }
    }
}
