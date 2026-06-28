use crate::backends::{EnvStore, FileStore, KeyringStore};
use crate::error::SecretError;
use crate::key::{Namespace, SecretKey};
use crate::store::SecretStore;
use crate::value::SecretValue;
use async_trait::async_trait;
use std::sync::Arc;

/// Chained secret store resolver.
///
/// Resolves secrets by trying each store in priority order:
/// 1. EnvStore (always wins)
/// 2. KeyringStore
/// 3. FileStore (fallback)
///
/// For `set`: writes to ALL writable stores (keyring + file)
/// For `delete`: deletes from ALL stores
pub struct ChainedSecretStore {
    stores: Vec<Arc<dyn SecretStore>>,
}

impl ChainedSecretStore {
    /// Create a new chain with default stores (env → keyring → file).
    pub fn new(file_path: Option<std::path::PathBuf>, passphrase: Option<String>) -> Self {
        let file_store = if let Some(path) = file_path {
            Some(Arc::new(FileStore::new(path, passphrase)) as Arc<dyn SecretStore>)
        } else {
            None
        };

        let mut stores: Vec<Arc<dyn SecretStore>> =
            vec![Arc::new(EnvStore), Arc::new(KeyringStore::new())];

        if let Some(fs) = file_store {
            stores.push(fs);
        }

        Self { stores }
    }

    /// Create a chain with custom stores (for testing).
    pub fn new_with_stores(stores: Vec<Arc<dyn SecretStore>>) -> Self {
        Self { stores }
    }

    /// Get a secret from the first store that has it.
    pub async fn get(&self, key: &SecretKey) -> Result<Option<SecretValue>, SecretError> {
        for store in &self.stores {
            if let Some(v) = store.get(key).await? {
                tracing::debug!(key = %key, backend = store.backend_name(), "Secret resolved");
                return Ok(Some(v));
            }
        }
        Ok(None)
    }

    /// Set a secret in all writable stores.
    pub async fn set(&self, key: &SecretKey, value: &SecretValue) -> Result<(), SecretError> {
        for store in &self.stores {
            store.set(key, value).await?;
        }
        Ok(())
    }

    /// Delete a secret from all stores.
    pub async fn delete(&self, key: &SecretKey) -> Result<bool, SecretError> {
        let mut any = false;
        for store in &self.stores {
            if store.delete(key).await? {
                any = true;
            }
        }
        Ok(any)
    }

    /// List keys in a namespace from the first store that supports listing.
    pub async fn list(&self, namespace: Namespace) -> Result<Vec<SecretKey>, SecretError> {
        // Try all stores — each may have different keys
        let mut all = Vec::new();
        for store in &self.stores {
            if let Ok(keys) = store.list(namespace.clone()).await {
                all.extend(keys);
            }
        }
        all.sort();
        all.dedup();
        Ok(all)
    }
}

#[async_trait]
impl SecretStore for ChainedSecretStore {
    async fn get(&self, key: &SecretKey) -> Result<Option<SecretValue>, SecretError> {
        ChainedSecretStore::get(self, key).await
    }

    async fn set(&self, key: &SecretKey, value: &SecretValue) -> Result<(), SecretError> {
        ChainedSecretStore::set(self, key, value).await
    }

    async fn delete(&self, key: &SecretKey) -> Result<bool, SecretError> {
        ChainedSecretStore::delete(self, key).await
    }

    async fn list(&self, namespace: Namespace) -> Result<Vec<SecretKey>, SecretError> {
        ChainedSecretStore::list(self, namespace).await
    }

    fn backend_name(&self) -> &'static str {
        "chain"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_chain_env_not_found_then_none() {
        let chain = ChainedSecretStore::new(None, None);
        let key = SecretKey::llm("__test_nonexistent__");
        let result = chain.get(&key).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_chain_stores_listed_in_order() {
        let chain = ChainedSecretStore::new(None, None);
        assert_eq!(chain.stores.len(), 2); // env + keyring (no file)
        assert_eq!(chain.stores[0].backend_name(), "env");
        assert_eq!(chain.stores[1].backend_name(), "keyring");
    }
}
