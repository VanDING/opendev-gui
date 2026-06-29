//! MCP tool bridge: wraps an MCP server tool as a `BaseTool`.
//!
//! Each `McpBridgeTool` instance represents a single tool from a connected
//! MCP server. It stores the tool's metadata (name, description, schema)
//! and holds an `Arc<McpManager>` to dispatch `call_tool` requests.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use opendev_mcp::McpManager;
use opendev_mcp::models::{DEFAULT_MAX_MCP_OUTPUT_CHARS, McpContent, McpToolSchema};
use opendev_tools_core::traits::{BaseTool, ToolContext, ToolResult};

/// Maximum image dimensions for inline image data (1920x1080).
/// Reserved for future use when decoding image dimensions from base64.
#[allow(dead_code)]
const IMAGE_MAX_DIMENSIONS: (u32, u32) = (1920, 1080);

/// Maximum image data size (5 MB).
const IMAGE_MAX_BYTES: usize = 5_000_000;

/// A `BaseTool` wrapper around a single MCP server tool.
///
/// The tool name is the namespaced MCP name (e.g., `sqlite__query`),
/// prefixed with `mcp__` for the agent's tool registry.
pub struct McpBridgeTool {
    /// Fully qualified tool name for the registry (e.g., `mcp__sqlite__query`).
    tool_name: String,
    /// Human-readable description.
    tool_description: String,
    /// JSON Schema for the tool's parameters.
    schema: serde_json::Value,
    /// Server name for routing the call.
    server_name: String,
    /// Original tool name on the MCP server.
    original_name: String,
    /// Shared MCP manager for dispatching calls.
    manager: Arc<McpManager>,
    /// Maximum output characters. When exceeded, output is truncated.
    max_output_chars: usize,
}

impl std::fmt::Debug for McpBridgeTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpBridgeTool")
            .field("tool_name", &self.tool_name)
            .field("server_name", &self.server_name)
            .field("original_name", &self.original_name)
            .finish()
    }
}

impl McpBridgeTool {
    /// Create a bridge tool from an MCP tool schema and a shared manager.
    pub fn from_schema(schema: &McpToolSchema, manager: Arc<McpManager>) -> Self {
        Self {
            tool_name: format!("mcp__{}", schema.name),
            tool_description: schema.description.clone(),
            schema: schema.parameters.clone(),
            server_name: schema.server_name.clone(),
            original_name: schema.original_name.clone(),
            manager,
            max_output_chars: schema.max_mcp_output_chars,
        }
    }
}

#[async_trait]
impl BaseTool for McpBridgeTool {
    fn name(&self) -> &str {
        &self.tool_name
    }

    fn description(&self) -> &str {
        &self.tool_description
    }

    fn parameter_schema(&self) -> serde_json::Value {
        // Return the schema as-is; it's already a JSON Schema object from the MCP server
        if self.schema.is_object() {
            self.schema.clone()
        } else {
            // Fallback: wrap in a minimal object schema
            serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            })
        }
    }

    fn category(&self) -> opendev_tools_core::ToolCategory {
        opendev_tools_core::ToolCategory::Mcp
    }

    async fn execute(
        &self,
        args: HashMap<String, serde_json::Value>,
        _ctx: &ToolContext,
    ) -> ToolResult {
        let arguments = serde_json::Value::Object(args.into_iter().collect());

        match self.manager.call_tool(&self.server_name, &self.original_name, arguments).await {
            Ok(result) => {
                // Convert MCP content blocks to a single output string
                let mut output = result
                    .content
                    .iter()
                    .filter_map(|c| match c {
                        McpContent::Text { text } => Some(text.clone()),
                        McpContent::Image { data, mime_type } => {
                            if data.len() > IMAGE_MAX_BYTES {
                                return Some(format!(
                                    "[Image omitted: {:.1} MB exceeds {:.1} MB limit]",
                                    data.len() as f64 / 1_000_000.0,
                                    IMAGE_MAX_BYTES as f64 / 1_000_000.0,
                                ));
                            }

                            // Check approximate dimensions from base64 length
                            // Base64 encodes 3 bytes as 4 chars, so raw size ≈ len * 3/4
                            // For 1920x1080 RGBA: 1920 * 1080 * 4 ≈ 8.3 MB raw
                            let raw_size_estimate = data.len() * 3 / 4;
                            if raw_size_estimate > IMAGE_MAX_BYTES {
                                return Some(format!(
                                    "[Image omitted: estimated {:.1} MB exceeds limit]",
                                    raw_size_estimate as f64 / 1_000_000.0,
                                ));
                            }

                            // Include preview-truncated base64 data in output.
                            // Full data is available via the MCP tool result metadata.
                            let max_display = 1000;
                            let preview = if data.len() > max_display {
                                format!(
                                    "{}...[{} more bytes]",
                                    &data[..max_display],
                                    data.len() - max_display
                                )
                            } else {
                                data.clone()
                            };

                            Some(format!("[Image: {mime_type}, {} bytes]\n{preview}", data.len(),))
                        }
                        McpContent::Resource { uri } => {
                            // For resources, persist to a temp file in ~/.opendev/mcp-output/
                            let output_dir = dirs::data_dir()
                                .map(|d| d.join("opendev").join("mcp-output"))
                                .unwrap_or_else(|| {
                                    std::path::PathBuf::from("/tmp/opendev-mcp-output")
                                });
                            let _ = std::fs::create_dir_all(&output_dir);
                            let safe_name = uri.replace(
                                |c: char| !c.is_alphanumeric() && c != '-' && c != '.',
                                "_",
                            );
                            let filename = format!("{}_{}.bin", safe_name, uuid::Uuid::new_v4());
                            let path = output_dir.join(&filename);

                            // Attempt to fetch resource content or create a reference file
                            match std::fs::write(&path, uri.as_bytes()) {
                                Ok(_) => {
                                    Some(format!("[Resource: {uri} — saved to {}]", path.display()))
                                }
                                Err(_) => Some(format!("[Resource: {uri}]")),
                            }
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                // Truncate if output exceeds the per-tool budget
                if output.len() > self.max_output_chars {
                    let truncated = &output[..self.max_output_chars];
                    let marker = format!(
                        "\n\n[OUTPUT TRUNCATED - exceeded {} character limit. \
                         Use the MCP tool with more specific parameters to get targeted results.]",
                        self.max_output_chars
                    );
                    output = format!("{truncated}{marker}");
                }

                if result.is_error {
                    ToolResult::fail(if output.is_empty() {
                        "MCP tool returned an error".to_string()
                    } else {
                        output
                    })
                } else {
                    ToolResult::ok(if output.is_empty() {
                        "(no output)".to_string()
                    } else {
                        output
                    })
                }
            }
            Err(e) => ToolResult::fail(format!("MCP call failed: {e}")),
        }
    }
}

#[cfg(test)]
#[path = "mcp_tool_tests.rs"]
mod tests;
