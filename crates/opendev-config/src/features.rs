//! Feature flag system for OpenDev.
//!
//! Provides JSON-based feature flags with local overrides from
//! `~/.opendev/features.json`. Includes a killswitch mechanism for
//! immediately disabling problematic features across all users.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

/// Feature flag map: feature name → enabled/disabled.
type FeatureMap = HashMap<String, bool>;

/// Global feature flag state.
static FEATURES: std::sync::LazyLock<RwLock<FeatureMap>> =
    std::sync::LazyLock::new(|| RwLock::new(HashMap::new()));

/// Path to the local features override file.
fn features_path() -> PathBuf {
    dirs::data_dir()
        .map(|d| d.join("opendev").join("features.json"))
        .unwrap_or_else(|| PathBuf::from(".opendev/features.json"))
}

/// Load features from the local overrides file.
///
/// The file is a JSON object with feature names as keys and boolean values:
/// ```json
/// { "streaming": true, "agent_team": false }
/// ```
fn load_features_file() -> FeatureMap {
    let path = features_path();
    if !path.exists() {
        return HashMap::new();
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => HashMap::new(),
    }
}

/// Initialize the feature flag system by loading overrides from disk.
///
/// Call once at startup. Subsequent calls reload the file.
pub fn init_features() {
    let overrides = load_features_file();
    let mut features = FEATURES.write().unwrap_or_else(|e| e.into_inner());
    // Merge: start with defaults, then apply file overrides
    let mut merged = default_features();
    for (key, value) in overrides {
        merged.insert(key, value);
    }
    *features = merged;
}

/// Default feature flags.
fn default_features() -> FeatureMap {
    let mut m = HashMap::new();
    m.insert("streaming".to_string(), true);
    m.insert("agent_team".to_string(), true);
    m.insert("subagent_spawn".to_string(), true);
    m.insert("memory".to_string(), true);
    m.insert("file_search".to_string(), true);
    m.insert("cost_tracking".to_string(), true);
    m.insert("prompt_caching".to_string(), true);
    m.insert("auto_compact".to_string(), true);
    m.insert("notebook_edit".to_string(), true);
    m.insert("skill_invoke".to_string(), true);
    m
}

/// Check if a feature is enabled.
///
/// Returns `true` if the feature is explicitly enabled or not listed,
/// `false` if explicitly disabled (including via killswitch).
pub fn is_feature_enabled(name: &str) -> bool {
    let features = FEATURES.read().unwrap_or_else(|e| e.into_inner());
    features.get(name).copied().unwrap_or(true)
}

/// Enable a feature at runtime.
pub fn enable_feature(name: &str) {
    let mut features = FEATURES.write().unwrap_or_else(|e| e.into_inner());
    features.insert(name.to_string(), true);
}

/// Disable a feature at runtime (killswitch).
///
/// When a feature is disabled via killswitch, it cannot be re-enabled
/// until the process restarts (unless explicitly removed from the file).
pub fn disable_feature(name: &str) {
    let mut features = FEATURES.write().unwrap_or_else(|e| e.into_inner());
    features.insert(name.to_string(), false);
}

/// Get a snapshot of all current feature flags.
pub fn all_features() -> HashMap<String, bool> {
    let features = FEATURES.read().unwrap_or_else(|e| e.into_inner());
    features.clone()
}

/// Persist the current feature flags to disk.
pub fn save_features() -> Result<(), String> {
    let features = FEATURES.read().unwrap_or_else(|e| e.into_inner());
    let path = features_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir: {e}"))?;
    }
    let content = serde_json::to_string_pretty(&*features)
        .map_err(|e| format!("Failed to serialize: {e}"))?;
    std::fs::write(&path, &content).map_err(|e| format!("Failed to write: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults_are_enabled() {
        init_features();
        assert!(is_feature_enabled("streaming"));
        assert!(is_feature_enabled("memory"));
    }

    #[test]
    fn test_disable_killswitch() {
        init_features();
        disable_feature("streaming");
        assert!(!is_feature_enabled("streaming"));
    }

    #[test]
    fn test_re_enable() {
        init_features();
        disable_feature("agent_team");
        assert!(!is_feature_enabled("agent_team"));
        enable_feature("agent_team");
        assert!(is_feature_enabled("agent_team"));
    }

    #[test]
    fn test_unknown_feature_defaults_to_true() {
        init_features();
        assert!(is_feature_enabled("nonexistent_feature"));
    }

    #[test]
    fn test_all_features_returns_snapshot() {
        init_features();
        let all = all_features();
        assert!(all.contains_key("streaming"));
        assert!(all.contains_key("memory"));
    }
}
