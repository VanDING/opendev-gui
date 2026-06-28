use serde::{Serialize, Deserialize};
use ts_rs::TS;

/// ── mcp/server/list ──
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct McpServerListParams;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct McpServerListResponse {
    pub servers: Vec<McpServerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct McpServerInfo {
    pub name: String,
    pub command: String,
    pub status: String,  // "connected" | "disconnected" | "error"
    pub tools_count: u32,
}

/// ── mcp/server/get ──
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct McpServerGetParams {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct McpServerGetResponse {
    pub server: McpServerDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct McpServerDetail {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub envs: std::collections::HashMap<String, String>,
    pub status: String,
    pub tools_count: u32,
}

/// ── mcp/server/create ──
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct McpServerCreateParams {
    pub name: String,
    pub command: String,
    pub args: Option<Vec<String>>,
    pub envs: Option<std::collections::HashMap<String, String>>,
}

/// ── mcp/server/update ──
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct McpServerUpdateParams {
    pub name: String,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub envs: Option<std::collections::HashMap<String, String>>,
}

/// ── mcp/server/delete ──
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct McpServerDeleteParams {
    pub name: String,
}

/// ── mcp/server/connect ──
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct McpServerConnectParams {
    pub name: String,
}

/// ── mcp/server/disconnect ──
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct McpServerDisconnectParams {
    pub name: String,
}
