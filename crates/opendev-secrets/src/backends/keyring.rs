use crate::error::SecretError;
use crate::key::{Namespace, SecretKey};
use crate::store::SecretStore;
use crate::value::SecretValue;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

/// Local index of keys stored in the keyring.
/// The keyring crate does not support enumeration, so we track keys ourselves.
#[derive(Debug, Default, Serialize, Deserialize)]
struct KeyringIndex {
    stored_keys: Vec<String>,
}

/// OS keyring-backed secret store.
///
/// Uses the `keyring` crate with service name `"com.opendev.desktop"`.
/// Backends: macOS Keychain, Linux Secret Service, Windows Credential Manager.
///
/// Maintains a local JSON index (`~/.opendev/.keyring-index.json`) to support
/// key enumeration via [`SecretStore::list`], which the raw keyring crate does not.
pub struct KeyringStore {
    service: String,
    index_path: PathBuf,
    /// Mutex-protected in-memory index for thread safety.
    index: Mutex<KeyringIndex>,
}

impl KeyringStore {
    /// Create a new KeyringStore with default index path (`~/.opendev/.keyring-index.json`).
    pub fn new() -> Self {
        let index_path = default_index_path();
        let index = Self::load_index(&index_path).unwrap_or_default();
        Self { service: "com.opendev.desktop".into(), index_path, index: Mutex::new(index) }
    }

    /// Create a new KeyringStore with a custom config directory for the index file.
    pub fn with_config_dir(config_dir: PathBuf) -> Self {
        let index_path = config_dir.join(".keyring-index.json");
        let index = Self::load_index(&index_path).unwrap_or_default();
        Self { service: "com.opendev.desktop".into(), index_path, index: Mutex::new(index) }
    }

    fn default_config_dir() -> PathBuf {
        std::env::var("HOME")
            .map(PathBuf::from)
            .map(|h| h.join(".opendev"))
            .unwrap_or_else(|_| PathBuf::from("."))
    }

    fn load_index(path: &PathBuf) -> Option<KeyringIndex> {
        std::fs::read_to_string(path).ok().and_then(|s| serde_json::from_str(&s).ok())
    }

    fn save_index(&self) -> Result<(), SecretError> {
        if let Some(parent) = self.index_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(&*self.index.lock().unwrap())
            .map_err(|e| SecretError::Serialization(e.to_string()))?;
        std::fs::write(&self.index_path, json)?;
        Ok(())
    }

    fn index_add(&self, key: &str) -> Result<(), SecretError> {
        let mut idx = self.index.lock().unwrap();
        if !idx.stored_keys.contains(&key.to_string()) {
            idx.stored_keys.push(key.to_string());
        }
        drop(idx);
        self.save_index()
    }

    fn index_remove(&self, key: &str) -> Result<(), SecretError> {
        let mut idx = self.index.lock().unwrap();
        idx.stored_keys.retain(|k| k != key);
        drop(idx);
        self.save_index()
    }
}

fn default_index_path() -> PathBuf {
    KeyringStore::default_config_dir().join(".keyring-index.json")
}

impl Default for KeyringStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SecretStore for KeyringStore {
    async fn get(&self, key: &SecretKey) -> Result<Option<SecretValue>, SecretError> {
        let entry = keyring::Entry::new(&self.service, &key.to_string())?;
        match entry.get_password() {
            Ok(password) => Ok(Some(SecretValue::new(password))),
            Err(keyring::Error::NoEntry) => {
                tracing::debug!(key = %key, "KeyringStore: no entry found");
                Ok(None)
            }
            Err(e) => {
                tracing::warn!(key = %key, error = %e, "KeyringStore: get failed");
                Err(SecretError::Keyring(e))
            }
        }
    }

    async fn set(&self, key: &SecretKey, value: &SecretValue) -> Result<(), SecretError> {
        let entry = keyring::Entry::new(&self.service, &key.to_string())?;
        entry.set_password(value.expose())?;
        // Track in local index
        self.index_add(&key.to_string())?;
        tracing::info!(key = %key, "KeyringStore: secret saved");
        Ok(())
    }

    async fn delete(&self, key: &SecretKey) -> Result<bool, SecretError> {
        let entry = keyring::Entry::new(&self.service, &key.to_string())?;
        match entry.delete_credential() {
            Ok(()) => {
                self.index_remove(&key.to_string())?;
                tracing::info!(key = %key, "KeyringStore: secret deleted");
                Ok(true)
            }
            Err(keyring::Error::NoEntry) => {
                // Also clean up from index in case it was orphaned
                self.index_remove(&key.to_string()).ok();
                Ok(false)
            }
            Err(e) => Err(SecretError::Keyring(e)),
        }
    }

    async fn list(&self, namespace: Namespace) -> Result<Vec<SecretKey>, SecretError> {
        let prefix = format!("{}/", namespace.as_str());
        let idx = self.index.lock().unwrap();
        let keys: Vec<SecretKey> = idx
            .stored_keys
            .iter()
            .filter(|k| k.starts_with(&prefix))
            .filter_map(|k| {
                let account = k.strip_prefix(&prefix)?;
                Some(SecretKey::new(namespace.clone(), account.to_string()))
            })
            .collect();
        Ok(keys)
    }

    fn backend_name(&self) -> &'static str {
        "keyring"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_keyring_get_nonexistent() {
        let store = KeyringStore::new();
        let key = SecretKey::llm("__test_nonexistent_provider__");
        let result = store.get(&key).await;
        // On headless systems without keyring backend, this may error
        // On systems with keyring, it should return None
        match result {
            Ok(v) => assert!(v.is_none()),
            Err(e) => {
                eprintln!("Keyring not available on this system (expected in CI): {}", e);
            }
        }
    }
}
