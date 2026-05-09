use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub dependencies: Vec<String>,
    pub enabled: bool,
}

/// Plugin trait - all plugins must implement this
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn description(&self) -> &str;

    fn initialize(&mut self) -> anyhow::Result<()>;
    fn shutdown(&mut self) -> anyhow::Result<()>;

    /// Return tools provided by this plugin
    fn tools(&self) -> Vec<Box<dyn crate::tools::Tool>> {
        Vec::new()
    }

    /// Return auth providers (optional)
    fn auth_providers(&self) -> Vec<Arc<dyn AuthProvider>> {
        Vec::new()
    }

    /// Plugin metadata
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: self.name().to_string(),
            version: self.version().to_string(),
            description: self.description().to_string(),
            author: String::new(),
            dependencies: Vec::new(),
            enabled: true,
        }
    }
}

/// Authentication provider trait
pub trait AuthProvider: Send + Sync {
    fn name(&self) -> &str;
    fn auth_type(&self) -> AuthType;
    fn authenticate(&self) -> anyhow::Result<Credentials>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthType {
    OAuth,
    ApiKey,
    Basic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub provider: String,
    pub token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for Credentials {
    fn default() -> Self {
        Self {
            provider: String::new(),
            token: String::new(),
            refresh_token: None,
            expires_at: None,
        }
    }
}
