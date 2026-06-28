use serde::{Serialize, Deserialize};
use ts_rs::TS;

/// ── config/get ──
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ConfigGetParams;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ConfigGetResponse {
    pub model_provider: String,
    pub model: String,
    pub temperature: f64,
    pub mode: String,
    pub autonomy_level: String,
    pub working_directory: String,
    pub git_branch: Option<String>,
}

/// ── config/update ──
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ConfigUpdateParams {
    pub model_provider: Option<String>,
    pub model: Option<String>,
    pub temperature: Option<f64>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub enable_bash: Option<bool>,
}

/// ── config/mode/set ──
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ConfigModeSetParams {
    pub mode: String,  // "normal" | "plan"
}

/// ── config/autonomy/set ──
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ConfigAutonomySetParams {
    pub level: String,  // "manual" | "semi-auto" | "auto"
}

/// ── config/model/verify ──
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ConfigModelVerifyParams {
    pub provider: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ConfigModelVerifyResponse {
    pub valid: bool,
    pub error: Option<String>,
}
