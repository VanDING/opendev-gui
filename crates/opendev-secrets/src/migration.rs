use serde::{Serialize, Deserialize};
use std::path::Path;
use crate::error::SecretError;
use crate::key::SecretKey;
use crate::value::SecretValue;
use crate::resolver::ChainedSecretStore;

/// Report of what was migrated.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MigrationReport {
    pub moved: Vec<String>,
    pub skipped: Vec<String>,
    pub errors: Vec<String>,
}

/// Migrate secrets from a settings.json file to the secret store.
///
/// Looks for `api_key` and `channels.telegram.bot_token` fields
/// in the JSON and moves them to the SecretStore, then removes
/// them from the JSON (or replaces with a reference marker).
pub async fn migrate_settings_json(
    path: &Path,
    secrets: &ChainedSecretStore,
) -> Result<MigrationReport, SecretError> {
    let raw = std::fs::read_to_string(path)
        .map_err(|e| SecretError::Io(e))?;
    let mut value: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| SecretError::Serialization(e.to_string()))?;
    let mut report = MigrationReport::default();

    // Migrate api_key
    if let Some(api_key) = value.get("api_key")
        .and_then(|v| v.as_str())
        .filter(|k| !k.is_empty())
        .map(String::from)
    {
        let provider = value.get("model_provider")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let key = SecretKey::llm(provider);

        // Write to secret store
        if let Err(e) = secrets.set(&key, &SecretValue::new(api_key.clone())).await {
            report.errors.push(format!("api_key → keyring: {}", e));
            return Err(SecretError::Backend(e.to_string()));
        }

        // Remove from JSON, add reference
        if let Some(obj) = value.as_object_mut() {
            obj.remove("api_key");
            obj.insert("api_key_ref".into(), serde_json::json!(format!("{}/{}", key.namespace().as_str(), key.account())));
        }
        report.moved.push(format!("api_key → {}", key));
    }

    // Migrate telegram bot_token
    if let Some(token) = value.pointer("/channels/telegram/bot_token")
        .and_then(|v| v.as_str())
        .filter(|t| !t.is_empty())
        .map(String::from)
    {
        let key = SecretKey::telegram();
        if let Err(e) = secrets.set(&key, &SecretValue::new(token)).await {
            report.errors.push(format!("telegram.bot_token → keyring: {}", e));
            return Err(SecretError::Backend(e.to_string()));
        }

        // Remove from JSON
        if let Some(channels) = value.pointer_mut("/channels/telegram") {
            if let Some(obj) = channels.as_object_mut() {
                obj.remove("bot_token");
            }
        }
        report.moved.push(format!("telegram.bot_token → {}", key));
    }

    // Save modified JSON
    let json_str = serde_json::to_string_pretty(&value)
        .map_err(|e| SecretError::Serialization(e.to_string()))?;
    std::fs::write(path, json_str)?;

    Ok(report)
}

/// Check if a settings.json has plaintext secrets that need migration.
pub fn has_unmigrated_secrets(path: &Path) -> Result<bool, SecretError> {
    if !path.exists() {
        return Ok(false);
    }
    let raw = std::fs::read_to_string(path)?;
    let value: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| SecretError::Serialization(e.to_string()))?;

    // Check for api_key
    if let Some(key) = value.get("api_key").and_then(|v| v.as_str()) {
        if !key.is_empty() {
            return Ok(true);
        }
    }

    // Check for telegram bot_token
    if let Some(token) = value.pointer("/channels/telegram/bot_token")
        .and_then(|v| v.as_str())
    {
        if !token.is_empty() {
            return Ok(true);
        }
    }

    Ok(false)
}
