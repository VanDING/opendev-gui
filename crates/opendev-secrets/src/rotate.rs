use std::sync::Arc;
use std::time::{Duration, Instant};
use crate::error::SecretError;
use crate::key::SecretKey;
use crate::value::SecretValue;
use crate::store::SecretStore;

/// Cooldown seconds based on HTTP status code.
pub fn cooldown_seconds(status: u16) -> f64 {
    match status {
        429 => 30.0,
        401 | 402 => 300.0,
        403 => 600.0,
        500 | 503 => 60.0,
        502 => 30.0,
        _ => 60.0,
    }
}

/// Profile state for a single API key.
struct AuthProfile {
    key: SecretValue,
    provider: String,
    failed_at: Option<Instant>,
    failure_status: u16,
    cooldown_until: Option<Instant>,
    request_count: u64,
    failure_count: u64,
}

/// AuthProfileManager — revived from dead code, now backed by SecretStore.
/// 
/// Manages multiple API keys per provider with automatic rotation on failure.
/// Cooldown: 429=30s, 401=300s, 403=600s, 5xx=30-60s.
pub struct AuthProfileManager {
    provider: String,
    secrets: Arc<dyn SecretStore>,
    profiles: Vec<AuthProfile>,
    current_index: usize,
}

impl AuthProfileManager {
    /// Create a new manager from a SecretStore.
    /// 
    /// Loads all keys for the given provider from the store.
    /// Supports multi-key indexing: `{PROVIDER}/account-1`, `{PROVIDER}/account-2`, etc.
    pub async fn new(provider: &str, secrets: Arc<dyn SecretStore>) -> Result<Self, SecretError> {
        let mut profiles = Vec::new();
        
        // Try single key first
        let single_key = SecretKey::llm(provider);
        if let Some(value) = secrets.get(&single_key).await? {
            profiles.push(AuthProfile {
                key: value,
                provider: provider.to_string(),
                failed_at: None,
                failure_status: 0,
                cooldown_until: None,
                request_count: 0,
                failure_count: 0,
            });
        }

        // Try multi-key indexing
        for i in 1..=9 {
            let multi_key = SecretKey::new(
                crate::key::Namespace::Llm,
                format!("{}/account-{}", provider, i),
            );
            if let Some(value) = secrets.get(&multi_key).await? {
                profiles.push(AuthProfile {
                    key: value,
                    provider: provider.to_string(),
                    failed_at: None,
                    failure_status: 0,
                    cooldown_until: None,
                    request_count: 0,
                    failure_count: 0,
                });
            }
        }

        if profiles.is_empty() {
            return Err(SecretError::NotFound(format!(
                "No API keys found for provider '{}'", provider
            )));
        }

        Ok(Self {
            provider: provider.to_string(),
            secrets,
            profiles,
            current_index: 0,
        })
    }

    /// Get an active key from the current profile.
    pub fn get_active_key(&mut self) -> Option<&str> {
        // Try current index first
        if self.is_profile_available(self.current_index) {
            return Some(self.profiles[self.current_index].key.expose());
        }

        // Scan forward for an available profile
        let n = self.profiles.len();
        for offset in 1..n {
            let idx = (self.current_index + offset) % n;
            if self.is_profile_available(idx) {
                self.current_index = idx;
                return Some(self.profiles[idx].key.expose());
            }
        }

        // None available
        let soonest = self.profiles.iter()
            .filter_map(|p| p.cooldown_until)
            .min()
            .map(|t| {
                let remaining = t.saturating_duration_since(Instant::now());
                remaining.as_secs()
            })
            .unwrap_or(0);
        tracing::warn!(
            provider = %self.provider,
            profiles = %self.profiles.len(),
            cooldown_remaining_secs = %soonest,
            "All API keys are in cooldown"
        );
        None
    }

    fn is_profile_available(&self, idx: usize) -> bool {
        if let Some(cooldown_until) = self.profiles[idx].cooldown_until {
            if Instant::now() < cooldown_until {
                return false;
            }
        }
        true
    }

    /// Mark the current key as successfully used.
    pub fn mark_success(&mut self) {
        if self.current_index < self.profiles.len() {
            let profile = &mut self.profiles[self.current_index];
            profile.failed_at = None;
            profile.failure_status = 0;
            profile.cooldown_until = None;
            profile.request_count += 1;
        }
    }

    /// Mark the current key as failed.
    pub fn mark_failure(&mut self, status_code: u16) {
        if self.current_index < self.profiles.len() {
            let profile = &mut self.profiles[self.current_index];
            let cooldown = cooldown_seconds(status_code);
            profile.failed_at = Some(Instant::now());
            profile.failure_status = status_code;
            profile.cooldown_until = Some(Instant::now() + Duration::from_secs_f64(cooldown));
            profile.failure_count += 1;
            
            tracing::warn!(
                provider = %self.provider,
                status = status_code,
                cooldown_secs = cooldown,
                "API key failed, entering cooldown"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::env::EnvStore;

    #[tokio::test]
    async fn test_auth_profile_manager_from_env() {
        // Just verify construction can work
        let secrets = Arc::new(EnvStore) as Arc<dyn SecretStore>;
        let result = AuthProfileManager::new("__test_nonexistent_provider__", secrets).await;
        // Should fail with NotFound since no env var is set
        assert!(result.is_err());
    }

    #[test]
    fn test_cooldown_seconds() {
        assert_eq!(cooldown_seconds(429), 30.0);
        assert_eq!(cooldown_seconds(401), 300.0);
        assert_eq!(cooldown_seconds(403), 600.0);
        assert_eq!(cooldown_seconds(500), 60.0);
        assert_eq!(cooldown_seconds(200), 60.0); // unknown status
    }
}
