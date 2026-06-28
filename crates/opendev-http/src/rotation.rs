//! API key rotation with SecretStore backing.
//!
//! Uses opendev-secrets SecretStore chain instead of raw Vec<String>.
//! Cooldown: 429=30s, 401=300s, 403=600s, 5xx=30-60s.

use opendev_secrets::{ChainedSecretStore, SecretKey, SecretStore, SecretValue};
use std::sync::Arc;
use std::time::Instant;
use tracing::{info, warn};

/// Cooldown durations in seconds by HTTP status code.
fn cooldown_seconds(status: u16) -> f64 {
    match status {
        429 => 30.0,
        401 => 300.0,
        402 => 300.0,
        403 => 600.0,
        500 => 60.0,
        502 => 30.0,
        503 => 60.0,
        _ => 60.0,
    }
}

struct AuthProfile {
    value: SecretValue,
    provider: String,
    failed_at: Option<Instant>,
    failure_status: u16,
    cooldown_until: Option<Instant>,
    request_count: u64,
    failure_count: u64,
}

impl AuthProfile {
    fn new(value: SecretValue, provider: String) -> Self {
        Self {
            value,
            provider,
            failed_at: None,
            failure_status: 0,
            cooldown_until: None,
            request_count: 0,
            failure_count: 0,
        }
    }

    fn is_available(&self) -> bool {
        match self.cooldown_until {
            None => true,
            Some(until) => Instant::now() >= until,
        }
    }

    fn mark_success(&mut self) {
        self.request_count += 1;
        self.failed_at = None;
        self.failure_status = 0;
        self.cooldown_until = None;
    }

    fn mark_failure(&mut self, status_code: u16) {
        self.failure_count += 1;
        let now = Instant::now();
        self.failed_at = Some(now);
        self.failure_status = status_code;
        let cooldown = cooldown_seconds(status_code);
        self.cooldown_until = Some(now + std::time::Duration::from_secs_f64(cooldown));
        warn!(
            provider = %self.provider,
            status_code,
            cooldown_secs = cooldown,
            "Auth profile failed, cooling down"
        );
    }
}

/// Manages multiple API keys per provider with rotation and failover.
///
/// Keys are loaded from SecretStore chain (env → keyring → file).
/// Multi-key support: {PROVIDER}/account-1, account-2, etc.
pub struct AuthProfileManager {
    provider: String,
    secrets: Arc<dyn SecretStore>,
    profiles: Vec<AuthProfile>,
    current_index: usize,
}

impl AuthProfileManager {
    /// Create a new manager using the SecretStore.
    ///
    /// Tries single key first, then account-1..9 for multi-key support.
    pub async fn new(provider: &str, secrets: Arc<ChainedSecretStore>) -> Result<Self, String> {
        let mut profiles = Vec::new();
        let secrets: Arc<dyn SecretStore> = secrets;

        // Single key
        let key = SecretKey::llm(provider);
        if let Some(value) = secrets.get(&key).await.map_err(|e| e.to_string())? {
            profiles.push(AuthProfile::new(value, provider.to_string()));
        }

        // Multi-key: account-1, account-2, ...
        for i in 1..=9 {
            let mkey = SecretKey::new(
                opendev_secrets::Namespace::Llm,
                format!("{}/account-{}", provider, i),
            );
            if let Some(value) = secrets.get(&mkey).await.map_err(|e| e.to_string())? {
                profiles.push(AuthProfile::new(value, provider.to_string()));
            }
        }

        if profiles.is_empty() {
            return Err(format!("No API keys found for provider '{}'", provider));
        }

        Ok(Self { provider: provider.to_string(), secrets, profiles, current_index: 0 })
    }

    pub fn get_active_key(&mut self) -> Option<&str> {
        if self.profiles.is_empty() {
            return None;
        }

        if self.profiles[self.current_index].is_available() {
            return Some(self.profiles[self.current_index].value.expose());
        }

        let len = self.profiles.len();
        for i in 1..len {
            let idx = (self.current_index + i) % len;
            if self.profiles[idx].is_available() {
                self.current_index = idx;
                info!(provider = %self.provider, "Rotated to next API key");
                return Some(self.profiles[idx].value.expose());
            }
        }

        let soonest = self
            .profiles
            .iter()
            .filter_map(|p| {
                p.cooldown_until.map(|u| u.saturating_duration_since(Instant::now()).as_secs_f64())
            })
            .fold(f64::MAX, f64::min);
        warn!(
            total = self.profiles.len(),
            provider = %self.provider,
            soonest_available_secs = soonest,
            "All API keys are in cooldown"
        );
        None
    }

    pub fn mark_success(&mut self) {
        if !self.profiles.is_empty() {
            self.profiles[self.current_index].mark_success();
        }
    }

    pub fn mark_failure(&mut self, status_code: u16) {
        if !self.profiles.is_empty() {
            self.profiles[self.current_index].mark_failure(status_code);
        }
    }

    pub fn profile_count(&self) -> usize {
        self.profiles.len()
    }
    pub fn available_count(&self) -> usize {
        self.profiles.iter().filter(|p| p.is_available()).count()
    }
    pub fn provider(&self) -> &str {
        &self.provider
    }
}

impl std::fmt::Debug for AuthProfileManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthProfileManager")
            .field("provider", &self.provider)
            .field("profile_count", &self.profiles.len())
            .field("current_index", &self.current_index)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cooldown_seconds() {
        assert_eq!(cooldown_seconds(429), 30.0);
        assert_eq!(cooldown_seconds(401), 300.0);
        assert_eq!(cooldown_seconds(403), 600.0);
        assert_eq!(cooldown_seconds(500), 60.0);
    }
}
