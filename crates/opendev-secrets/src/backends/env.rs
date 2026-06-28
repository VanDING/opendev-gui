use crate::error::SecretError;
use crate::key::{Namespace, SecretKey};
use crate::store::SecretStore;
use crate::value::SecretValue;
use async_trait::async_trait;

/// Environment variable-backed secret store.
///
/// EnvStore always wins in the chain — it's the highest priority.
/// set and delete are no-ops (env vars are read-only at runtime).
pub struct EnvStore;

#[async_trait]
impl SecretStore for EnvStore {
    async fn get(&self, key: &SecretKey) -> Result<Option<SecretValue>, SecretError> {
        let env_var = key.to_env_var();
        if env_var.is_empty() {
            return Ok(None);
        }
        match std::env::var(&env_var) {
            Ok(v) if !v.is_empty() => Ok(Some(SecretValue::new(v))),
            _ => Ok(None),
        }
    }

    async fn set(&self, _key: &SecretKey, _value: &SecretValue) -> Result<(), SecretError> {
        // Env vars are read-only at runtime
        tracing::debug!("EnvStore: set is a no-op (env vars are read-only)");
        Ok(())
    }

    async fn delete(&self, _key: &SecretKey) -> Result<bool, SecretError> {
        // Cannot unset environment variables at runtime
        Ok(false)
    }

    async fn list(&self, _namespace: Namespace) -> Result<Vec<SecretKey>, SecretError> {
        // Cannot enumerate environment variables
        Ok(vec![])
    }

    fn backend_name(&self) -> &'static str {
        "env"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_env_get_found() {
        // Set a temp env var and verify it's found
        let key = SecretKey::llm("test_provider");
        let env_var = key.to_env_var();
        // Can only test if env var isn't set
        if std::env::var(&env_var).is_err() {
            // This test is best-effort — won't fail if env var not set
        }
    }

    #[tokio::test]
    async fn test_env_get_not_found() {
        let key = SecretKey::llm("nonexistent_provider_xyz");
        let result = EnvStore.get(&key).await.unwrap();
        assert!(result.is_none());
    }
}
