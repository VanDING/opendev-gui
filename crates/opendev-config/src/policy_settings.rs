//! Managed Policy Settings.
//!
//! Loads policy configuration from `/etc/opendev/settings.json` or the path
//! specified by the `OPENDEV_POLICY_SETTINGS_PATH` environment variable.
//!
//! This provides a mechanism for enterprise/managed deployments to enforce
//! specific permission rules that cannot be overridden by end users.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A single managed permission rule.
///
/// Managed rules are enforced by the system administrator and cannot be
/// modified or bypassed by the end user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedRule {
    /// Glob pattern matched against `"tool_name:args"` (e.g. `"bash:rm *"`).
    pub pattern: String,
    /// Action to take when the pattern matches (`"allow"`, `"deny"`, `"prompt"`).
    pub action: String,
}

/// Managed policy settings loaded from the system policy file.
///
/// When `allow_managed_permission_rules_only` is `true`, the runtime MUST
/// reject any user-defined permission rules — only managed rules are
/// considered authoritative.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicySettings {
    /// If true, only managed permission rules are allowed; user rules are ignored.
    #[serde(default)]
    pub allow_managed_permission_rules_only: bool,
    /// List of managed permission rules.
    #[serde(default)]
    pub managed_permission_rules: Vec<ManagedRule>,
}

impl PolicySettings {
    /// Load policy settings from the default system path or env override.
    ///
    /// Resolution order:
    /// 1. `OPENDEV_POLICY_SETTINGS_PATH` env var (if set and non-empty)
    /// 2. `/etc/opendev/settings.json`
    ///
    /// Returns `None` if neither file exists or if parsing fails.
    pub fn load() -> Option<Self> {
        let path = Self::resolve_path()?;
        Self::load_from(&path)
    }

    /// Load policy settings from a specific file path.
    pub fn load_from(path: &PathBuf) -> Option<Self> {
        let content = std::fs::read_to_string(path).ok()?;
        let settings: PolicySettings = serde_json::from_str(&content).ok()?;
        Some(settings)
    }

    /// Resolve the policy settings file path.
    fn resolve_path() -> Option<PathBuf> {
        // 1. Check env override first.
        if let Ok(env_path) = std::env::var("OPENDEV_POLICY_SETTINGS_PATH") {
            if !env_path.is_empty() {
                let p = PathBuf::from(env_path);
                if p.exists() {
                    return Some(p);
                }
            }
        }

        // 2. Fall back to default system path.
        let default_path = PathBuf::from("/etc/opendev/settings.json");
        if default_path.exists() {
            return Some(default_path);
        }

        None
    }

    /// Check if a given tool+args pattern is managed.
    pub fn is_managed_pattern(&self, pattern: &str) -> bool {
        self.managed_permission_rules.iter().any(|r| r.pattern == pattern)
    }

    /// Get the action for a matching managed rule pattern.
    pub fn action_for_pattern(&self, pattern: &str) -> Option<&str> {
        self.managed_permission_rules
            .iter()
            .find(|r| r.pattern == pattern)
            .map(|r| r.action.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_default_settings() {
        let json = r#"{
            "allow_managed_permission_rules_only": false,
            "managed_permission_rules": [
                {"pattern": "bash:rm *", "action": "deny"},
                {"pattern": "bash:git *", "action": "allow"}
            ]
        }"#;

        let settings: PolicySettings = serde_json::from_str(json).unwrap();
        assert!(!settings.allow_managed_permission_rules_only);
        assert_eq!(settings.managed_permission_rules.len(), 2);
        assert_eq!(settings.managed_permission_rules[0].pattern, "bash:rm *");
        assert_eq!(settings.managed_permission_rules[0].action, "deny");
        assert_eq!(settings.managed_permission_rules[1].pattern, "bash:git *");
        assert_eq!(settings.managed_permission_rules[1].action, "allow");
    }

    #[test]
    fn default_for_missing_fields() {
        let json = r#"{}"#;
        let settings: PolicySettings = serde_json::from_str(json).unwrap();
        assert!(!settings.allow_managed_permission_rules_only);
        assert!(settings.managed_permission_rules.is_empty());
    }

    #[test]
    fn managed_pattern_detection() {
        let json = r#"{
            "managed_permission_rules": [
                {"pattern": "bash:rm *", "action": "deny"}
            ]
        }"#;

        let settings: PolicySettings = serde_json::from_str(json).unwrap();
        assert!(settings.is_managed_pattern("bash:rm *"));
        assert!(!settings.is_managed_pattern("bash:ls *"));
    }

    #[test]
    fn action_for_matching_pattern() {
        let json = r#"{
            "managed_permission_rules": [
                {"pattern": "bash:rm *", "action": "deny"},
                {"pattern": "bash:cat *", "action": "allow"}
            ]
        }"#;

        let settings: PolicySettings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.action_for_pattern("bash:rm *"), Some("deny"));
        assert_eq!(settings.action_for_pattern("bash:cat *"), Some("allow"));
        assert_eq!(settings.action_for_pattern("bash:ls *"), None);
    }

    #[test]
    fn load_from_nonexistent_path_returns_none() {
        let path = PathBuf::from("/tmp/opendev-nonexistent-settings.json");
        let result = PolicySettings::load_from(&path);
        assert!(result.is_none());
    }

    #[test]
    fn load_from_invalid_json_returns_none() {
        // Write invalid JSON, then try to load it.
        let path = PathBuf::from("/tmp/opendev-test-invalid.json");
        std::fs::write(&path, "not valid json").ok();
        let result = PolicySettings::load_from(&path);
        assert!(result.is_none());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn allow_managed_only_flag_e2e() {
        let json = r#"{
            "allow_managed_permission_rules_only": true,
            "managed_permission_rules": []
        }"#;

        let settings: PolicySettings = serde_json::from_str(json).unwrap();
        assert!(settings.allow_managed_permission_rules_only);
    }
}
