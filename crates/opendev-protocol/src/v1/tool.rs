use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// ── tool/list ──
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ToolListParams {
    pub include_deferred: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ToolListResponse {
    pub tools: Vec<ToolInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub category: String,
    pub is_deferred: bool,
    pub schema: serde_json::Value,
}

/// ── tool/search ──
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ToolSearchParams {
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ToolSearchResponse {
    pub tools: Vec<ToolInfo>,
}

/// ── tool/approve ──
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ToolApproveParams {
    pub request_id: String,
    pub approved: bool,
    pub reason: Option<String>,
}
