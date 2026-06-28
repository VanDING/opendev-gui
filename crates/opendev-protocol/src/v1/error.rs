use serde::{Serialize, Deserialize};
use ts_rs::TS;

/// Standard v1 error codes.
pub mod codes {
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
    pub const VERSION_MISMATCH: i32 = -32000;
    pub const NOT_CONNECTED: i32 = -32001;
    pub const TIMEOUT: i32 = -32002;
    pub const SESSION_NOT_FOUND: i32 = -32010;
    pub const SESSION_CONFLICT: i32 = -32011;
    pub const TOOL_DENIED: i32 = -32020;
    pub const APPROVAL_EXPIRED: i32 = -32030;
}

/// Detailed error data that may accompany an ErrorFrame.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProtocolErrorData {
    pub code: i32,
    pub message: String,
    pub details: Option<serde_json::Value>,
}
