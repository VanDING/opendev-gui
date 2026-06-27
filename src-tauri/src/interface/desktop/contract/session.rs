//! Session DTOs.

use serde::{Deserialize, Serialize};

/// Create session request.
#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    #[serde(default)]
    pub working_directory: Option<String>,
}

/// Session info response.
#[derive(Debug, Serialize)]
pub struct SessionInfoResponse {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub message_count: usize,
    pub title: Option<String>,
    pub working_directory: Option<String>,
}

/// Create session response.
#[derive(Debug, Serialize)]
pub struct CreateSessionResponse {
    pub id: String,
    pub status: String,
}

/// Session model update.
#[derive(Debug, Deserialize)]
pub struct SessionModelUpdateRequest {
    pub model: Option<String>,
    pub provider: Option<String>,
}
