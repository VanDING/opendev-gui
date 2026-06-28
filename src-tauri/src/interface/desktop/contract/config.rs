//! Config DTOs.

use serde::{Deserialize, Serialize};

/// Config update request sent from frontend.
#[derive(Debug, Deserialize)]
pub struct UpdateConfigRequest {
    pub model_provider: Option<String>,
    pub model: Option<String>,
    pub model_vlm_provider: Option<String>,
    pub model_vlm: Option<String>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub enable_bash: Option<bool>,
    pub api_key: Option<String>,
    pub api_base_url: Option<String>,
}

/// Mode update request.
#[derive(Debug, Deserialize)]
pub struct ModeUpdateRequest {
    pub mode: String,
}

/// Autonomy update request.
#[derive(Debug, Deserialize)]
pub struct AutonomyUpdateRequest {
    pub level: String,
}

/// Verify model request.
#[derive(Debug, Deserialize)]
pub struct VerifyModelRequest {
    pub provider: String,
    pub model: String,
}

/// Config response sent to frontend.
#[derive(Debug, Serialize)]
pub struct ConfigResponse {
    pub model_provider: String,
    pub model: String,
    pub model_vlm_provider: Option<String>,
    pub model_vlm: Option<String>,
    pub model_compact_provider: Option<String>,
    pub model_compact: Option<String>,
    pub api_key: Option<String>,
    pub api_base_url: Option<String>,
    pub temperature: f64,
    pub max_tokens: u32,
    pub enable_bash: bool,
    pub mode: String,
    pub autonomy_level: String,
    pub working_dir: String,
    pub git_branch: Option<String>,
    /// List of env vars that shadow keyring-stored secrets.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shadowed_env_vars: Vec<String>,
}

/// Verify model response.
#[derive(Debug, Serialize)]
pub struct VerifyModelResponse {
    pub valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
