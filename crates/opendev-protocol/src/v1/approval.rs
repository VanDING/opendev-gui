use serde::{Serialize, Deserialize};
use ts_rs::TS;

/// ── approval/list ──
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ApprovalListParams;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ApprovalListResponse {
    pub approvals: Vec<ApprovalRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ApprovalRequest {
    pub id: String,
    pub session_id: String,
    pub tool_name: String,
    pub description: String,
    pub created_at: i64,
}

/// ── approval/respond ──
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ApprovalRespondParams {
    pub request_id: String,
    pub approved: bool,
    pub reason: Option<String>,
}
