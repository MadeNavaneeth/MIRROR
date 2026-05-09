use crate::plugins::traits::Credentials;
use crate::security::SecretStore;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

pub struct CredentialStorage {
    path: PathBuf,
    secret_store: Arc<SecretStore>,
    credentials: Arc<RwLock<HashMap<String, Credentials>>>,
}

impl CredentialStorage {
    pub fn new(path: PathBuf, secret_store: Arc<SecretStore>) -> Self {
        let credentials = if path.exists() {
            match Self::load_from_disk(&path, &secret_store) {
                Ok(creds) => Arc::new(RwLock::new(creds)),
                Err(e) => {
                    tracing::error!("Failed to load credentials from disk: {e}");
                    Arc::new(RwLock::new(HashMap::new()))
                }
            }
        } else {
            Arc::new(RwLock::new(HashMap::new()))
        };

        Self {
            path,
            secret_store,
            credentials,
        }
    }

    fn load_from_disk(
        path: &Path,
        secret_store: &SecretStore,
    ) -> Result<HashMap<String, Credentials>> {
        let data = fs::read_to_string(path).context("Failed to read credentials file")?;
        let encrypted_map: HashMap<String, Credentials> =
            serde_json::from_str(&data).context("Failed to parse credentials JSON")?;

        let mut decrypted_map = HashMap::new();
        for (provider, mut creds) in encrypted_map {
            creds.token = secret_store.decrypt(&creds.token)?;
            if let Some(ref rt) = creds.refresh_token {
                creds.refresh_token = Some(secret_store.decrypt(rt)?);
            }
            decrypted_map.insert(provider, creds);
        }

        Ok(decrypted_map)
    }

    pub fn save(&self, provider: &str, creds: Credentials) -> Result<()> {
        // Encrypt tokens before saving to disk
        let mut map = self.credentials.write().unwrap();
        map.insert(provider.to_string(), creds.clone());

        // Create a copy for encryption
        let mut encrypted_creds = creds;
        encrypted_creds.token = self.secret_store.encrypt(&encrypted_creds.token)?;
        if let Some(ref rt) = encrypted_creds.refresh_token {
            encrypted_creds.refresh_token = Some(self.secret_store.encrypt(rt)?);
        }

        let mut current_map = HashMap::new();
        for (p, c) in map.iter() {
            let mut ec = c.clone();
            ec.token = self.secret_store.encrypt(&ec.token)?;
            if let Some(ref rt) = ec.refresh_token {
                ec.refresh_token = Some(self.secret_store.encrypt(rt)?);
            }
            current_map.insert(p.clone(), ec);
        }

        let data = serde_json::to_string_pretty(&current_map)?;
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&self.path, data).context("Failed to write credentials file")?;

        Ok(())
    }

    pub fn get(&self, provider: &str) -> Option<Credentials> {
        let map = self.credentials.read().unwrap();
        map.get(provider).cloned()
    }

    pub fn list_providers(&self) -> Vec<String> {
        let map = self.credentials.read().unwrap();
        map.keys().cloned().collect()
    }

    pub fn delete(&self, provider: &str) -> Result<()> {
        let mut map = self.credentials.write().unwrap();
        map.remove(provider);

        // Re-save the map
        let mut current_map = HashMap::new();
        for (p, c) in map.iter() {
            let mut ec = c.clone();
            ec.token = self.secret_store.encrypt(&ec.token)?;
            if let Some(ref rt) = ec.refresh_token {
                ec.refresh_token = Some(self.secret_store.encrypt(rt)?);
            }
            current_map.insert(p.clone(), ec);
        }

        let data = serde_json::to_string_pretty(&current_map)?;
        fs::write(&self.path, data).context("Failed to update credentials file after deletion")?;

        Ok(())
    }
}
