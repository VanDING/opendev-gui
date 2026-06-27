//! MCP DTOs.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP server create request.
#[derive(Debug, Deserialize)]
pub struct CreateMCPServerRequest {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    pub enabled: bool,
    pub auto_start: bool,
}

/// MCP server update request.
#[derive(Debug, Deserialize)]
pub struct UpdateMCPServerRequest {
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    pub enabled: Option<bool>,
    pub auto_start: Option<bool>,
}

/// MCP server list response.
#[derive(Debug, Serialize)]
pub struct MCPServerListResponse {
    pub servers: Vec<MCPServerItem>,
}

#[derive(Debug, Serialize)]
pub struct MCPServerItem {
    pub name: String,
    pub status: String,
    pub config: MCPServerConfigData,
    pub tools_count: usize,
    pub config_location: String,
    pub config_path: String,
}

#[derive(Debug, Serialize)]
pub struct MCPServerConfigData {
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub enabled: bool,
    pub auto_start: bool,
}

/// MCP action response.
#[derive(Debug, Serialize)]
pub struct MCPActionResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools_count: Option<usize>,
}
