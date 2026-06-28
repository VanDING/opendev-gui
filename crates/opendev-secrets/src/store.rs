use crate::error::SecretError;
use crate::key::{Namespace, SecretKey};
use crate::value::SecretValue;
use async_trait::async_trait;

/// The core secret store trait.
///
/// Priority chain: env → keyring → file
/// EnvStore always wins (for CI/Docker/temp overrides).
/// KeyringStore is the primary persistent store.
/// FileStore is the encrypted fallback for headless environments.
#[async_trait]
pub trait SecretStore: Send + Sync {
    /// Get a secret by key.
    async fn get(&self, key: &SecretKey) -> Result<Option<SecretValue>, SecretError>;

    /// Set a secret by key.
    /// For EnvStore, this is a no-op (env vars are read-only).
    async fn set(&self, key: &SecretKey, value: &SecretValue) -> Result<(), SecretError>;

    /// Delete a secret by key. Returns true if the entry existed.
    async fn delete(&self, key: &SecretKey) -> Result<bool, SecretError>;

    /// List all keys in a given namespace.
    async fn list(&self, namespace: Namespace) -> Result<Vec<SecretKey>, SecretError>;

    /// Human-readable backend name (for diagnostics).
    fn backend_name(&self) -> &'static str;
}
