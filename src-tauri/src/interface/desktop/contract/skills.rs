//! Skills DTOs.

use serde::Serialize;

/// Skill info for frontend display.
#[derive(Debug, Serialize)]
pub struct SkillResponse {
    pub name: String,
    pub description: String,
    pub namespace: String,
    pub source: String,
    pub pinned: bool,
    pub status: String,
    pub usage_count: u32,
    pub tags: Vec<String>,
}

/// Toggle pin response.
#[derive(Debug, Serialize)]
pub struct TogglePinResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}
