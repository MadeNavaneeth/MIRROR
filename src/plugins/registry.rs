use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRegistry {
    pub plugins: Vec<RegistryPlugin>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryPlugin {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub repository: String,
    pub download_url: String,
    pub verified: bool,
}

impl PluginRegistry {
    /// Create empty registry
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Fetch registry from remote (GitHub or custom URL)
    pub async fn fetch(url: Option<&str>) -> anyhow::Result<Self> {
        let registry_url =
            url.unwrap_or("https://raw.githubusercontent.com/mirror/plugins/main/registry.json");

        tracing::info!("📡 Fetching plugin registry from {}", registry_url);

        // TODO: Fetch from URL
        // For now, return empty registry
        Ok(Self::new())
    }

    /// Search for plugins
    pub fn search(&self, query: &str) -> Vec<&RegistryPlugin> {
        let query_lower = query.to_lowercase();
        self.plugins
            .iter()
            .filter(|p| {
                p.name.to_lowercase().contains(&query_lower)
                    || p.description.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    /// Get plugin by exact name
    pub fn get(&self, name: &str) -> Option<&RegistryPlugin> {
        self.plugins.iter().find(|p| p.name == name)
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_search() {
        let mut registry = PluginRegistry::new();
        registry.plugins.push(RegistryPlugin {
            name: "google-auth".to_string(),
            version: "1.0.0".to_string(),
            description: "Google OAuth authentication".to_string(),
            author: "mirror".to_string(),
            repository: "https://github.com/mirror/google-auth".to_string(),
            download_url: "https://github.com/mirror/google-auth/releases/latest".to_string(),
            verified: true,
        });

        let results = registry.search("google");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "google-auth");
    }
}
