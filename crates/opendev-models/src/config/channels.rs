//! Channel integration configuration types (Telegram, etc.).

use super::permissions::default_true;
use serde::{Deserialize, Serialize};

/// Channel integrations configuration (Telegram, etc.).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChannelsConfig {
    /// Telegram bot configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub telegram: Option<TelegramChannelConfig>,
}

/// DM access policy for a channel.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DmPolicy {
    Open,
    #[default]
    Pairing,
    Allowlist,
}

/// Telegram channel configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramChannelConfig {
    /// Bot token from @BotFather.
    #[serde(default, deserialize_with = "deserialize_secret_string")]
    #[serde(serialize_with = "serialize_secret_string")]
    pub bot_token: secrecy::SecretString,
    /// Whether the Telegram channel is enabled (default: false).
    /// Must be explicitly set to `true` to activate — prevents the remote
    /// session claim from killing other TUI instances sharing the same token.
    #[serde(default)]
    pub enabled: bool,
    /// Only respond in groups when @mentioned or replied to.
    #[serde(default = "default_true")]
    pub group_mention_only: bool,
    /// DM access policy: "open", "pairing", or "allowlist".
    #[serde(default)]
    pub dm_policy: DmPolicy,
    /// Allowed Telegram user IDs (as strings).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_users: Vec<String>,
}

fn serialize_secret_string<S: serde::Serializer>(value: &secrecy::SecretString, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(secrecy::ExposeSecret::expose_secret(value))
}

fn deserialize_secret_string<'de, D: serde::Deserializer<'de>>(d: D) -> Result<secrecy::SecretString, D::Error> {
    let s = String::deserialize(d)?;
    Ok(s.into())
}

pub fn is_channels_default(c: &ChannelsConfig) -> bool {
    c.telegram.is_none()
}
