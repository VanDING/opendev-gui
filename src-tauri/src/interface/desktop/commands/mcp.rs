//! MCP commands — DTO mapping only.

use crate::application::AppServices;
use crate::application::mcp_service::{McpServerCreate, McpServerUpdate};
use crate::interface::desktop::contract::mcp::*;
use tauri::State;

/// List all MCP servers.
#[tauri::command]
pub async fn list_mcp_servers(
    services: State<'_, AppServices>,
) -> Result<MCPServerListResponse, String> {
    let servers = services.mcp.list_servers();
    let items: Vec<MCPServerItem> = servers
        .into_iter()
        .map(|s| MCPServerItem {
            name: s.name,
            status: s.status,
            config: MCPServerConfigData {
                command: s.config.command,
                args: s.config.args,
                env: s.config.env,
                enabled: s.config.enabled,
                auto_start: s.config.auto_start,
            },
            tools_count: s.tools_count,
            config_location: s.config_location,
            config_path: s.config_path,
        })
        .collect();

    Ok(MCPServerListResponse { servers: items })
}

/// Get a specific MCP server.
#[tauri::command]
pub async fn get_mcp_server(
    services: State<'_, AppServices>,
    name: String,
) -> Result<serde_json::Value, String> {
    let server =
        services.mcp.get_server(&name).ok_or_else(|| format!("Server '{}' not found", name))?;

    Ok(serde_json::json!({
        "name": server.name,
        "status": server.status,
        "config": {
            "command": server.config.command,
            "args": server.config.args,
            "env": server.config.env,
            "enabled": server.config.enabled,
            "auto_start": server.config.auto_start,
        },
        "tools": [],
        "capabilities": [],
        "config_path": server.config_path,
    }))
}

/// Create a new MCP server.
#[tauri::command]
pub async fn create_mcp_server(
    services: State<'_, AppServices>,
    req: CreateMCPServerRequest,
) -> Result<MCPActionResponse, String> {
    let input = McpServerCreate {
        name: req.name,
        command: req.command,
        args: req.args,
        env: req.env,
        enabled: req.enabled,
        auto_start: req.auto_start,
    };
    services.mcp.create_server(input)?;
    Ok(MCPActionResponse {
        success: true,
        message: "Server added successfully".to_string(),
        tools_count: None,
    })
}

/// Update an MCP server.
#[tauri::command]
pub async fn update_mcp_server(
    services: State<'_, AppServices>,
    name: String,
    req: UpdateMCPServerRequest,
) -> Result<MCPActionResponse, String> {
    let update = McpServerUpdate {
        command: req.command,
        args: req.args,
        env: req.env,
        enabled: req.enabled,
        auto_start: req.auto_start,
    };
    services.mcp.update_server(&name, update)?;
    Ok(MCPActionResponse {
        success: true,
        message: format!("Server '{}' updated successfully", name),
        tools_count: None,
    })
}

/// Delete an MCP server.
#[tauri::command]
pub async fn delete_mcp_server(
    services: State<'_, AppServices>,
    name: String,
) -> Result<MCPActionResponse, String> {
    services.mcp.delete_server(&name)?;
    Ok(MCPActionResponse {
        success: true,
        message: format!("Server '{}' removed successfully", name),
        tools_count: None,
    })
}

/// Connect to an MCP server.
#[tauri::command]
pub async fn connect_mcp_server(
    services: State<'_, AppServices>,
    name: String,
) -> Result<MCPActionResponse, String> {
    let tools_count = services.mcp.connect_server(&name).await?;
    Ok(MCPActionResponse {
        success: true,
        message: format!("Connected to '{}' — {} tool(s) discovered", name, tools_count),
        tools_count: Some(tools_count),
    })
}

/// Disconnect from an MCP server.
#[tauri::command]
pub async fn disconnect_mcp_server(
    _services: State<'_, AppServices>,
    name: String,
) -> Result<MCPActionResponse, String> {
    Ok(MCPActionResponse {
        success: true,
        message: format!("Not connected to '{}'", name),
        tools_count: None,
    })
}
