// Plugin loader - dynamic loading of plugins from filesystem
// This is a placeholder for future dynamic plugin loading

use super::traits::Plugin;
use std::path::Path;
use std::sync::Arc;

pub struct PluginLoader {
    // Future: support for dynamic library loading
}

impl PluginLoader {
    pub fn new() -> Self {
        Self {}
    }

    /// Load a plugin from a directory
    pub fn load_from_dir(&self, _path: &Path) -> anyhow::Result<Arc<dyn Plugin>> {
        // TODO: Implement dynamic plugin loading
        // For now, plugins must be compiled into the binary
        anyhow::bail!("Dynamic plugin loading not yet implemented")
    }
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new()
    }
}
