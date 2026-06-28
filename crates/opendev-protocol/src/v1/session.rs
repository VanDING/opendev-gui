use serde::{Serialize, Deserialize};
use ts_rs::TS;

/// ── session/list ──
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SessionListParams {
    pub max_count: Option<u32>,
    pub include_archived: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SessionListResponse {
    pub sessions: Vec<SessionSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SessionSummary {
    pub id: String,
    pub title: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub message_count: u32,
    pub mode: String,
}

/// ── session/start ──
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SessionStartParams {
    pub title: Option<String>,
    pub model: Option<String>,
    pub mode: Option<String>,
    pub workspace: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SessionStartResponse {
    pub session_id: String,
    pub created_at: i64,
}

/// ── session/resume ──
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SessionResumeParams {
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SessionResumeResponse {
    pub session_id: String,
    pub state: String,
}

/// ── session/delete ──
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SessionDeleteParams {
    pub session_id: String,
}

/// ── session/turns ──
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SessionTurnsParams {
    pub session_id: String,
    pub max_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SessionTurnsResponse {
    pub turns: Vec<TurnSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TurnSummary {
    pub id: String,
    pub user_message: String,
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub tool_calls: u32,
    pub status: String,
}
