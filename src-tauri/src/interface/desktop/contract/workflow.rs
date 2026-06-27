//! Workflow DTOs.

use serde::{Deserialize, Serialize};

/// Approval response from frontend.
#[derive(Debug, Deserialize)]
pub struct ApprovalResponse {
    pub approval_id: String,
    pub approved: bool,
    #[serde(default)]
    pub auto_approve: bool,
}

/// Ask-user response from frontend.
#[derive(Debug, Deserialize)]
pub struct AskUserResponse {
    pub request_id: String,
    pub answers: Option<serde_json::Value>,
    #[serde(default)]
    pub cancelled: bool,
}

/// Plan approval response from frontend.
#[derive(Debug, Deserialize)]
pub struct PlanApprovalResponse {
    pub request_id: String,
    pub action: String,
    #[serde(default)]
    pub feedback: String,
}

/// Standard result response.
#[derive(Debug, Serialize)]
pub struct WorkflowActionResult {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}
