//! Credential store — compat facade over `opendev-secrets`.
//!
//! This replaces the old `auth.json`-backed CredentialStore.
//! All operations are delegated to `opendev_secrets::ChainedSecretStore`.
//!
//! Environment variables always take precedence over stored values.

use opendev_secrets::{ChainedSecretStore, SecretKey, SecretValue};
use std::path::PathBuf;
use std::sync::Arc;

use crate::models::HttpError;

/// Status of a provider's credential.
#[derive(Debug, Clone)]
pub struct ProviderStatus {
    pub provider: String,
    pub has_env_key: bool,
    pub has_stored_key: bool,
    pub env_var: String,
}

/// Well-known LLM provider env vars (used for status reporting).
const PROVIDER_ENV_VARS: &[(&str, &str)] = &[
    ("openai", "OPENAI_API_KEY"),
    ("anthropic", "ANTHROPIC_API_KEY"),
    ("fireworks", "FIREWORKS_API_KEY"),
    ("google", "GOOGLE_API_KEY"),
    ("groq", "GROQ_API_KEY"),
    ("mistral", "MISTRAL_API_KEY"),
    ("deepinfra", "DEEPINFRA_API_KEY"),
    ("openrouter", "OPENROUTER_API_KEY"),
    ("azure", "AZURE_OPENAI_API_KEY"),
];

/// Secure credential store — compat facade over opendev-secrets.
///
/// Environment variables always take precedence over stored values.
/// Backed by the SecretStore chain: env → keyring → file.
pub struct CredentialStore {
    secrets: Arc<ChainedSecretStore>,
}

impl CredentialStore {
    /// Create a new CredentialStore backed by the given SecretStore chain.
    pub fn new(secrets: Arc<ChainedSecretStore>) -> Self {
        Self { secrets }
    }

    /// Get API key for a provider. Environment variable takes precedence.
    pub async fn get_key(&self, provider: &str) -> Option<String> {
        let key = SecretKey::llm(provider);

        // SecretStore chain handles env → keyring → file
        self.secrets.get(&key).await.ok()?.map(|v| v.expose().to_string())
    }

    /// Store an API key for a provider.
    pub async fn set_key(&self, provider: &str, key: &str) -> Result<(), HttpError> {
        let secret_key = SecretKey::llm(provider);
        let secret_value = SecretValue::new(key.to_string());
        self.secrets
            .set(&secret_key, &secret_value)
            .await
            .map_err(|e| HttpError::Other(e.to_string()))?;
        tracing::info!("Stored API key for {}", provider);
        Ok(())
    }

    /// Remove a stored API key.
    pub async fn remove_key(&self, provider: &str) -> Result<bool, HttpError> {
        let key = SecretKey::llm(provider);
        self.secrets.delete(&key).await.map_err(|e| HttpError::Other(e.to_string()))
    }

    /// List all known providers with their credential status.
    pub async fn list_providers(&self) -> Vec<ProviderStatus> {
        let mut result = Vec::new();
        for &(provider, env_var) in PROVIDER_ENV_VARS {
            let has_env = std::env::var(env_var).map(|v| !v.is_empty()).unwrap_or(false);
            let key = SecretKey::llm(provider);
            let has_stored = self.secrets.get(&key).await.ok().flatten().is_some();
            result.push(ProviderStatus {
                provider: provider.to_string(),
                has_env_key: has_env,
                has_stored_key: has_stored,
                env_var: env_var.to_string(),
            });
        }
        result
    }

    /// Get the path to the auth file (deprecated).
    pub fn path(&self) -> Option<PathBuf> {
        None // auth.json is no longer used
    }
}

impl std::fmt::Debug for CredentialStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CredentialStore").finish()
    }
}

#[cfg(test)]
#[path = "auth_tests.rs"]
mod tests;
