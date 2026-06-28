use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// ── turn/start ──
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TurnStartParams {
    pub session_id: String,
    pub message: String,
    pub attachments: Option<Vec<AttachmentInput>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AttachmentInput {
    pub name: String,
    pub content: String,
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TurnStartResponse {
    pub turn_id: String,
    pub accepted: bool,
}

/// ── turn/interrupt ──
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TurnInterruptParams {
    pub session_id: String,
}

/// ── turn/steer ──
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TurnSteerParams {
    pub session_id: String,
    pub message: String,
}
