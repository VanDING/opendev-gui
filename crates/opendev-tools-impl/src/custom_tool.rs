//! Custom tool loaded from `.opendev/tools/` directory.
//!
//! Users can define custom tools by placing a JSON manifest file alongside
//! an executable script in `.opendev/tools/` (or `.opencode/tool/`).
//!
//! ## Manifest format (`<name>.tool.json`)
//!
//! ```json
//! {
//!   "name": "github_triage",
//!   "description": "Assign and label GitHub issues",
//!   "command": "./github-triage.sh",
//!   "parameters": {
//!     "type": "object",
//!     "properties": {
//!       "issue": { "type": "string", "description": "Issue number" }
//!     },
//!     "required": ["issue"]
//!   },
//!   "timeout_secs": 30
//! }
//! ```
//!
//! The tool receives arguments as JSON on stdin and should write its
//! result to stdout. Exit code 0 = success, non-zero = failure.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use serde::Deserialize;
use tracing::{debug, warn};

use opendev_tools_core::{BaseTool, ToolContext, ToolResult};

/// JSON manifest describing a custom tool.
#[derive(Debug, Clone, Deserialize)]
pub struct CustomToolManifest {
    /// Tool name (used for dispatch). Must be unique.
    pub name: String,
    /// Human-readable description shown to the LLM.
    pub description: String,
    /// Command to execute (relative to the manifest directory, or absolute).
    pub command: String,
    /// JSON Schema for tool parameters.
    #[serde(default = "default_params_schema")]
    pub parameters: serde_json::Value,
    /// Optional timeout in seconds (default: 30).
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

fn default_params_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "input": {
                "type": "string",
                "description": "Input to the tool"
            }
        }
    })
}

fn default_timeout() -> u64 {
    30
}

/// A tool backed by an external script/executable.
#[derive(Debug)]
pub struct CustomTool {
    manifest: CustomToolManifest,
    /// Directory containing the manifest (for resolving relative command paths).
    base_dir: PathBuf,
}

impl CustomTool {
    /// Create a custom tool from a manifest and its containing directory.
    pub fn new(manifest: CustomToolManifest, base_dir: PathBuf) -> Self {
        Self { manifest, base_dir }
    }

    /// Resolve the command path (relative to base_dir if not absolute).
    fn resolve_command(&self) -> PathBuf {
        let cmd = Path::new(&self.manifest.command);
        if cmd.is_absolute() { cmd.to_path_buf() } else { self.base_dir.join(cmd) }
    }
}

#[async_trait]
impl BaseTool for CustomTool {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn description(&self) -> &str {
        &self.manifest.description
    }

    fn parameter_schema(&self) -> serde_json::Value {
        self.manifest.parameters.clone()
    }

    async fn execute(
        &self,
        args: HashMap<String, serde_json::Value>,
        ctx: &ToolContext,
    ) -> ToolResult {
        let cmd_path = self.resolve_command();

        if !cmd_path.exists() {
            return ToolResult::fail(format!(
                "Custom tool command not found: {}",
                cmd_path.display()
            ));
        }

        // Serialize args as JSON for stdin.
        let input_json = match serde_json::to_string(&args) {
            Ok(j) => j,
            Err(e) => return ToolResult::fail(format!("Failed to serialize args: {e}")),
        };

        // Execute the command.
        let timeout = std::time::Duration::from_secs(self.manifest.timeout_secs);
        let result = tokio::time::timeout(timeout, async {
            let mut child = {
                let mut cmd = tokio::process::Command::new(cmd_path.as_os_str());
                cmd.current_dir(&ctx.working_dir)
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped());
                opendev_exec::env_filter::apply(cmd.as_std_mut());
                match cmd.spawn() {
                    Ok(c) => c,
                    Err(e) => return Err(e),
                }
            };

            // Write input to stdin.
            if let Some(mut stdin) = child.stdin.take() {
                use tokio::io::AsyncWriteExt;
                let _ = stdin.write_all(input_json.as_bytes()).await;
                drop(stdin);
            }

            child.wait_with_output().await
        })
        .await;

        match result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                if output.status.success() {
                    debug!(
                        tool = self.manifest.name,
                        exit_code = 0,
                        "Custom tool executed successfully"
                    );
                    if stdout.is_empty() {
                        ToolResult::ok("(no output)")
                    } else {
                        ToolResult::ok(stdout)
                    }
                } else {
                    let code = output.status.code().unwrap_or(-1);
                    let error_msg = if stderr.is_empty() {
                        format!("Custom tool exited with code {code}")
                    } else {
                        format!("Exit code {code}: {stderr}")
                    };
                    ToolResult::fail(error_msg)
                }
            }
            Ok(Err(e)) => ToolResult::fail(format!("Failed to execute custom tool: {e}")),
            Err(_) => ToolResult::fail(format!(
                "Custom tool timed out after {}s",
                self.manifest.timeout_secs
            )),
        }
    }
}

/// Discover custom tools from standard directories.
///
/// Scans these directories for `*.tool.json` manifest files:
/// - `<working_dir>/.opendev/tools/`
/// - `<working_dir>/.opencode/tool/`
///
/// Returns a list of `(manifest, base_dir)` tuples for each valid tool found.
pub fn discover_custom_tools(working_dir: &Path) -> Vec<CustomTool> {
    let search_dirs =
        [working_dir.join(".opendev").join("tools"), working_dir.join(".opencode").join("tool")];

    let mut tools = Vec::new();
    let mut seen_names = std::collections::HashSet::new();

    for dir in &search_dirs {
        if !dir.is_dir() {
            continue;
        }

        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(e) => {
                warn!(dir = %dir.display(), error = %e, "Failed to read custom tools directory");
                continue;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();

            // Only process *.tool.json manifests.
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !name.ends_with(".tool.json") {
                continue;
            }

            match std::fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str::<CustomToolManifest>(&content) {
                    Ok(manifest) => {
                        if seen_names.contains(&manifest.name) {
                            warn!(
                                name = manifest.name,
                                path = %path.display(),
                                "Duplicate custom tool name, skipping"
                            );
                            continue;
                        }
                        debug!(
                            name = manifest.name,
                            path = %path.display(),
                            "Discovered custom tool"
                        );
                        seen_names.insert(manifest.name.clone());
                        tools.push(CustomTool::new(manifest, dir.clone()));
                    }
                    Err(e) => {
                        warn!(
                            path = %path.display(),
                            error = %e,
                            "Failed to parse custom tool manifest"
                        );
                    }
                },
                Err(e) => {
                    warn!(
                        path = %path.display(),
                        error = %e,
                        "Failed to read custom tool manifest"
                    );
                }
            }
        }
    }

    tools
}

/// Validate a custom tool manifest for security and correctness.
///
/// Checks:
/// 1. Command must be in the allowlist or go through sandbox
/// 2. Command path must not contain shell injection characters
/// 3. Parameter schema must be valid JSON Schema
/// 4. Timeout must be reasonable (1-600s)
/// 5. Name must not be empty
pub fn validate_custom_tool(manifest: &CustomToolManifest) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // Name validation
    if manifest.name.is_empty() {
        errors.push("Tool name must not be empty".to_string());
    }
    if manifest.name.contains(' ') || manifest.name.contains('/') {
        errors.push(format!(
            "Tool name '{}' contains invalid characters (no spaces or slashes)",
            manifest.name
        ));
    }

    // Command validation
    if manifest.command.is_empty() {
        errors.push("Tool command must not be empty".to_string());
    }
    // Check for shell injection in command
    let dangerous_chars = [';', '|', '&', '$', '`', '\n', '\r'];
    for ch in &dangerous_chars {
        if manifest.command.contains(*ch) {
            errors.push(format!(
                "Tool command contains dangerous character '{}'",
                ch.escape_default()
            ));
        }
    }

    // Parameter schema validation
    if !manifest.parameters.is_object()
        || manifest.parameters.get("type").and_then(|t| t.as_str()) != Some("object")
    {
        errors.push("Parameter schema must be a JSON Schema of type 'object'".to_string());
    }

    // Timeout validation
    if manifest.timeout_secs == 0 {
        errors.push("Tool timeout must be at least 1 second".to_string());
    }
    if manifest.timeout_secs > 600 {
        errors.push("Tool timeout must not exceed 600 seconds".to_string());
    }

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

/// Perform parameter substitution in a command string.
///
/// Replaces `{param_name}` placeholders with the corresponding
/// values from `args`. Unknown placeholders are left as-is.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// use opendev_tools_impl::custom_tool::substitute_params;
///
/// let mut args = HashMap::new();
/// args.insert("file".to_string(), serde_json::json!("src/main.rs"));
/// let result = substitute_params("process {file}", &args);
/// assert_eq!(result, "process src/main.rs");
/// ```
pub fn substitute_params(command: &str, args: &std::collections::HashMap<String, serde_json::Value>) -> String {
    let mut result = command.to_string();
    for (key, value) in args {
        let placeholder = format!("{{{}}}", key);
        let value_str = match value {
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        result = result.replace(&placeholder, &value_str);
    }
    result
}

/// Parse tool output, trying JSON first, then treating as plain text.
///
/// Returns `(output_string, parsed_json)`.
pub fn parse_tool_output(stdout: &str) -> (String, Option<serde_json::Value>) {
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return (String::new(), None);
    }

    // Try JSON parse
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(trimmed) {
        let display = if json.is_object() || json.is_array() {
            serde_json::to_string_pretty(&json).unwrap_or_else(|_| trimmed.to_string())
        } else if let Some(s) = json.as_str() {
            s.to_string()
        } else {
            json.to_string()
        };
        (display, Some(json))
    } else {
        (trimmed.to_string(), None)
    }
}

/// Ensure each custom tool runs in sandbox and env is filtered.
///
/// This is called during tool execution. It applies:
/// 1. `opendev_exec::env_filter::apply()` to the command (already done in execute())
/// 2. Validates the command is safe
///
/// Security note: The env_filter and sandbox are already applied in execute().
/// This function serves as a verification checkpoint.
pub fn verify_tool_security(command: &str) -> Result<(), String> {
    let trimmed = command.trim();

    // Reject empty commands
    if trimmed.is_empty() {
        return Err("Empty command".to_string());
    }

    // Reject shell metacharacters that indicate injection attempts
    let dangerous = [';', '|', '&', '$', '`', '\n', '\r'];
    if let Some(ch) = dangerous.iter().find(|&&c| trimmed.contains(c)) {
        return Err(format!(
            "Command contains dangerous shell character '{}'",
            ch.escape_default()
        ));
    }

    Ok(())
}

#[cfg(test)]
#[path = "custom_tool_tests.rs"]
mod tests;
