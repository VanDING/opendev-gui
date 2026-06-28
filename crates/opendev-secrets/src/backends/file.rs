use std::path::PathBuf;
use async_trait::async_trait;
use secrecy::SecretString;
use serde::{Serialize, Deserialize};
use crate::error::SecretError;
use crate::key::{SecretKey, Namespace};
use crate::value::SecretValue;
use crate::store::SecretStore;

/// Age-encrypted file-backed secret store.
/// 
/// Fallback for headless/CI environments where keyring is not available.
/// Uses age X25519 + scrypt encryption.
pub struct FileStore {
    path: PathBuf,
    passphrase: String,
}

impl FileStore {
    /// Create a new FileStore.
    /// 
    /// `path` is the path to the encrypted secrets file.
    /// `passphrase` is the passphrase for decrypting/encrypting the file.
    ///   If empty, uses `OPENDEV_MASTER_KEY` env var, or generates one.
    pub fn new(path: PathBuf, passphrase: Option<String>) -> Self {
        let passphrase = passphrase
            .or_else(|| std::env::var("OPENDEV_MASTER_KEY").ok())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        Self { path, passphrase }
    }

    fn load_data(&self) -> Result<FileData, SecretError> {
        if !self.path.exists() {
            return Ok(FileData::default());
        }
        let encrypted = std::fs::read(&self.path)?;

        // Decrypt with age scrypt
        let passphrase = SecretString::new(self.passphrase.clone().into_boxed_str());
        let identity = age::scrypt::Identity::new(passphrase);
        let decrypted = age::decrypt(&identity, &encrypted)
            .map_err(|e| SecretError::Backend(format!("age decrypt failed: {}", e)))?;
        let data: FileData = serde_json::from_slice(&decrypted)
            .map_err(|e| SecretError::Serialization(e.to_string()))?;
        Ok(data)
    }

    fn save_data(&self, data: &FileData) -> Result<(), SecretError> {
        let plaintext = serde_json::to_vec(data)
            .map_err(|e| SecretError::Serialization(e.to_string()))?;
        let passphrase = SecretString::new(self.passphrase.clone().into_boxed_str());
        let recipient = age::scrypt::Recipient::new(passphrase);
        let encrypted = age::encrypt(&recipient, &plaintext)
            .map_err(|e| SecretError::Backend(format!("age encrypt failed: {}", e)))?;
        std::fs::write(&self.path, encrypted)?;
        Ok(())
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct FileData {
    entries: std::collections::HashMap<String, String>,
}

#[async_trait]
impl SecretStore for FileStore {
    async fn get(&self, key: &SecretKey) -> Result<Option<SecretValue>, SecretError> {
        let data = self.load_data()?;
        Ok(data.entries.get(&key.to_string()).cloned().map(SecretValue::new))
    }

    async fn set(&self, key: &SecretKey, value: &SecretValue) -> Result<(), SecretError> {
        let mut data = self.load_data()?;
        data.entries.insert(key.to_string(), value.expose().to_string());
        self.save_data(&data)
    }

    async fn delete(&self, key: &SecretKey) -> Result<bool, SecretError> {
        let mut data = self.load_data()?;
        let existed = data.entries.remove(&key.to_string()).is_some();
        if existed {
            self.save_data(&data)?;
        }
        Ok(existed)
    }

    async fn list(&self, namespace: Namespace) -> Result<Vec<SecretKey>, SecretError> {
        let data = self.load_data()?;
        let prefix = format!("{}/", namespace.as_str());
        let keys = data.entries.keys()
            .filter(|k| k.starts_with(&prefix))
            .filter_map(|k| {
                let account = k.strip_prefix(&prefix)?;
                Some(SecretKey::new(namespace.clone(), account.to_string()))
            })
            .collect();
        Ok(keys)
    }

    fn backend_name(&self) -> &'static str {
        "file"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_file_store_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("secrets.age");
        let store = FileStore::new(path.clone(), Some("test-passphrase".into()));

        let key = SecretKey::llm("test-provider");
        let value = SecretValue::new("sk-test-12345");

        // Set
        store.set(&key, &value).await.unwrap();

        // Get
        let retrieved = store.get(&key).await.unwrap().unwrap();
        assert_eq!(retrieved.expose(), "sk-test-12345");

        // Delete
        let existed = store.delete(&key).await.unwrap();
        assert!(existed);

        // Verify gone
        let gone = store.get(&key).await.unwrap();
        assert!(gone.is_none());
    }

    #[tokio::test]
    async fn test_file_store_list_namespace() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("secrets2.age");
        let store = FileStore::new(path, Some("test-passphrase".into()));

        store.set(&SecretKey::llm("openai"), &SecretValue::new("sk-1")).await.unwrap();
        store.set(&SecretKey::llm("anthropic"), &SecretValue::new("sk-2")).await.unwrap();
        store.set(&SecretKey::telegram(), &SecretValue::new("bot:token")).await.unwrap();

        let llm_keys = store.list(Namespace::Llm).await.unwrap();
        assert_eq!(llm_keys.len(), 2);

        let tg_keys = store.list(Namespace::Telegram).await.unwrap();
        assert_eq!(tg_keys.len(), 1);
    }
}
