//! MCPService — MCP (Model Context Protocol) server management.
//!
//! Handles CRUD operations on MCP server configurations stored in JSON files.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// MCP server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub auto_start: bool,
}

fn default_enabled() -> bool {
    true
}

/// MCP server create request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerCreate {
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerUpdate {
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    pub enabled: Option<bool>,
    pub auto_start: Option<bool>,
}

/// MCP server response for list endpoint.
#[derive(Debug, Clone, Serialize)]
pub struct McpServerItem {
    pub name: String,
    pub status: String,
    pub config: McpServerConfigResponse,
    pub tools_count: usize,
    pub config_location: String,
    pub config_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct McpServerConfigResponse {
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub enabled: bool,
    pub auto_start: bool,
}

/// Global MCP config path.
fn global_config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config").join("opendev").join("mcp_servers.json")
}

/// Project MCP config path.
fn project_config_path(working_dir: &str) -> PathBuf {
    PathBuf::from(working_dir).join(".opendev").join("mcp_servers.json")
}

/// Load all MCP server configurations.
fn load_all_servers(working_dir: &str) -> HashMap<String, McpServerConfig> {
    let mut servers = HashMap::new();

    // Load global servers.
    let global_path = global_config_path();
    if let Ok(data) = std::fs::read_to_string(&global_path) {
        if let Ok(parsed) = serde_json::from_str::<HashMap<String, McpServerConfig>>(&data) {
            servers.extend(parsed);
        }
    }

    // Load project-local servers (override globals).
    let project_path = project_config_path(working_dir);
    if let Ok(data) = std::fs::read_to_string(&project_path) {
        if let Ok(parsed) = serde_json::from_str::<HashMap<String, McpServerConfig>>(&data) {
            for (key, val) in parsed {
                servers.insert(key, val);
            }
        }
    }

    servers
}

/// Save a server config to a file.
fn save_server_to_config(
    name: &str,
    config: &McpServerConfig,
    path: &PathBuf,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let mut servers = if path.exists() {
        let data =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read config: {}", e))?;
        serde_json::from_str::<HashMap<String, McpServerConfig>>(&data).unwrap_or_default()
    } else {
        HashMap::new()
    };

    servers.insert(name.to_string(), config.clone());

    let data = serde_json::to_string_pretty(&servers)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    std::fs::write(path, data).map_err(|e| format!("Failed to write config: {}", e))?;
    Ok(())
}

/// Remove a server config from a file.
fn remove_server_from_config(name: &str, path: &PathBuf) -> Result<bool, String> {
    if !path.exists() {
        return Ok(false);
    }

    let data =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read config: {}", e))?;
    let mut servers: HashMap<String, McpServerConfig> =
        serde_json::from_str(&data).unwrap_or_default();

    if servers.remove(name).is_some() {
        let data = serde_json::to_string_pretty(&servers)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        std::fs::write(path, data).map_err(|e| format!("Failed to write config: {}", e))?;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub struct MCPService {
    working_dir: String,
}

impl MCPService {
    pub fn new(working_dir: String) -> Self {
        Self { working_dir }
    }

    /// List all configured MCP servers.
    pub fn list_servers(&self) -> Vec<McpServerItem> {
        let servers = load_all_servers(&self.working_dir);
        let global_path = global_config_path();

        servers
            .iter()
            .map(|(name, config)| McpServerItem {
                name: name.clone(),
                status: "disconnected".to_string(),
                config: McpServerConfigResponse {
                    command: config.command.clone(),
                    args: config.args.clone(),
                    env: config.env.clone(),
                    enabled: config.enabled,
                    auto_start: config.auto_start,
                },
                tools_count: 0,
                config_location: "global".to_string(),
                config_path: global_path.to_string_lossy().to_string(),
            })
            .collect()
    }

    /// Get a specific server config.
    pub fn get_server(&self, name: &str) -> Option<McpServerItem> {
        let servers = load_all_servers(&self.working_dir);
        servers.get(name).map(|config| {
            let global_path = global_config_path();
            McpServerItem {
                name: name.to_string(),
                status: "disconnected".to_string(),
                config: McpServerConfigResponse {
                    command: config.command.clone(),
                    args: config.args.clone(),
                    env: config.env.clone(),
                    enabled: config.enabled,
                    auto_start: config.auto_start,
                },
                tools_count: 0,
                config_location: "global".to_string(),
                config_path: global_path.to_string_lossy().to_string(),
            }
        })
    }

    /// Create a new MCP server.
    pub fn create_server(&self, input: McpServerCreate) -> Result<(), String> {
        let servers = load_all_servers(&self.working_dir);
        if servers.contains_key(&input.name) {
            return Err(format!("Server '{}' already exists", input.name));
        }

        let config = McpServerConfig {
            command: input.command,
            args: input.args,
            env: input.env,
            enabled: input.enabled,
            auto_start: input.auto_start,
        };

        save_server_to_config(&input.name, &config, &global_config_path())?;
        Ok(())
    }

    /// Update an existing MCP server.
    pub fn update_server(&self, name: &str, update: McpServerUpdate) -> Result<(), String> {
        let servers = load_all_servers(&self.working_dir);
        let existing = servers.get(name).ok_or_else(|| format!("Server '{}' not found", name))?;

        let config = McpServerConfig {
            command: update.command.unwrap_or_else(|| existing.command.clone()),
            args: update.args.unwrap_or_else(|| existing.args.clone()),
            env: update.env.unwrap_or_else(|| existing.env.clone()),
            enabled: update.enabled.unwrap_or(existing.enabled),
            auto_start: update.auto_start.unwrap_or(existing.auto_start),
        };

        save_server_to_config(name, &config, &global_config_path())?;
        Ok(())
    }

    /// Delete an MCP server.
    pub fn delete_server(&self, name: &str) -> Result<(), String> {
        let servers = load_all_servers(&self.working_dir);
        if !servers.contains_key(name) {
            return Err(format!("Server '{}' not found", name));
        }

        let global_removed = remove_server_from_config(name, &global_config_path())?;
        let project_removed =
            remove_server_from_config(name, &project_config_path(&self.working_dir))?;

        if !global_removed && !project_removed {
            return Err(format!("Server '{}' found in memory but not in config files", name));
        }

        Ok(())
    }

    /// Connect to an MCP server (tests connectivity and discovers tools).
    pub async fn connect_server(&self, name: &str) -> Result<usize, String> {
        let servers = load_all_servers(&self.working_dir);
        let server_config =
            servers.get(name).ok_or_else(|| format!("Server '{}' not found", name))?;

        let mcp_config = opendev_mcp::McpServerConfig {
            command: server_config.command.clone(),
            args: server_config.args.clone(),
            env: server_config.env.clone(),
            enabled: server_config.enabled,
            auto_start: server_config.auto_start,
            ..Default::default()
        };

        let manager = opendev_mcp::McpManager::new(Some(PathBuf::from(&self.working_dir)));
        manager
            .add_server(name.to_string(), mcp_config)
            .await
            .map_err(|e| format!("Failed to register server: {}", e))?;

        manager
            .connect_server(name)
            .await
            .map_err(|e| format!("Failed to connect to MCP server '{}': {}", name, e))?;

        let schemas = manager.get_all_tool_schemas().await;
        let tools_count = schemas.len();

        let _ = manager.disconnect_server(name).await;

        Ok(tools_count)
    }
}
