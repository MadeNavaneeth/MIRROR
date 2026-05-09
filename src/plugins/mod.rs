pub mod loader;
pub mod manager;
pub mod registry;
pub mod traits;

pub use manager::PluginManager;
pub use traits::{AuthProvider, AuthType, Credentials, Plugin, PluginMetadata};
