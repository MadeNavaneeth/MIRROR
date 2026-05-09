use super::traits::{Plugin, PluginMetadata};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

pub struct PluginManager {
    plugins: HashMap<String, Arc<dyn Plugin>>,
    plugin_dir: PathBuf,
}

impl PluginManager {
    pub fn new(plugin_dir: PathBuf) -> Self {
        Self {
            plugins: HashMap::new(),
            plugin_dir,
        }
    }

    /// Load all plugins from plugin directory
    pub fn load_all(&mut self) -> anyhow::Result<()> {
        tracing::info!("🔌 Loading plugins from {:?}", self.plugin_dir);

        // Create plugin directory if it doesn't exist
        if !self.plugin_dir.exists() {
            std::fs::create_dir_all(&self.plugin_dir)?;
        }

        // TODO: Scan directory for plugin manifests
        // TODO: Load and initialize plugins

        Ok(())
    }

    /// Enable a plugin
    pub fn enable(&mut self, name: &str) -> anyhow::Result<()> {
        tracing::info!("🔌 Enabling plugin: {}", name);

        if self.plugins.contains_key(name) {
            tracing::warn!("Plugin {} is already enabled", name);
            return Ok(());
        }

        // TODO: Load plugin from directory
        // TODO: Initialize plugin

        Ok(())
    }

    /// Disable a plugin
    pub fn disable(&mut self, name: &str) -> anyhow::Result<()> {
        tracing::info!("🔌 Disabling plugin: {}", name);

        if let Some(mut plugin) = self.plugins.remove(name) {
            // Shutdown plugin
            Arc::get_mut(&mut plugin)
                .ok_or_else(|| anyhow::anyhow!("Cannot shutdown plugin with active references"))?
                .shutdown()?;
        }

        Ok(())
    }

    /// List all available plugins
    pub fn list(&self) -> Vec<PluginMetadata> {
        self.plugins.values().map(|p| p.metadata()).collect()
    }

    /// Get a plugin by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn Plugin>> {
        self.plugins.get(name).cloned()
    }

    /// Install a plugin from registry or URL
    pub fn install(&mut self, source: &str) -> anyhow::Result<()> {
        tracing::info!("📦 Installing plugin from: {}", source);

        // TODO: Parse source (registry name, GitHub URL, local path)
        // TODO: Download/copy plugin
        // TODO: Extract to plugin directory
        // TODO: Load metadata

        Ok(())
    }

    /// Uninstall a plugin
    pub fn uninstall(&mut self, name: &str) -> anyhow::Result<()> {
        tracing::info!("🗑️  Uninstalling plugin: {}", name);

        // Disable first
        self.disable(name)?;

        // Remove from filesystem
        let plugin_path = self.plugin_dir.join(name);
        if plugin_path.exists() {
            std::fs::remove_dir_all(plugin_path)?;
        }

        Ok(())
    }

    /// Get all tools from all enabled plugins
    pub fn all_tools(&self) -> Vec<Box<dyn crate::tools::Tool>> {
        self.plugins.values().flat_map(|p| p.tools()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_plugin_manager_creation() {
        let tmp = TempDir::new().unwrap();
        let manager = PluginManager::new(tmp.path().to_path_buf());
        assert_eq!(manager.list().len(), 0);
    }

    #[test]
    fn test_load_all_creates_directory() {
        let tmp = TempDir::new().unwrap();
        let plugin_dir = tmp.path().join("plugins");
        let mut manager = PluginManager::new(plugin_dir.clone());

        manager.load_all().unwrap();
        assert!(plugin_dir.exists());
    }
}
