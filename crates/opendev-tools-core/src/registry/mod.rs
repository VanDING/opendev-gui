//! Tool registry for discovery and dispatch.
//!
//! Stores `Arc<dyn BaseTool>` instances and dispatches execution by tool name.
//! Supports middleware pipelines, parameter validation, per-tool timeouts,
//! and same-turn call deduplication.

mod execution;
mod helpers;

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};

use crate::middleware::ToolMiddleware;
use crate::sanitizer::ToolResultSanitizer;
use crate::traits::{BaseTool, ToolDisplayMeta, ToolResult, ToolTimeoutConfig};

/// Registry that maps tool names to implementations and dispatches execution.
///
/// Features:
/// - Middleware pipeline (before/after hooks)
/// - JSON Schema parameter validation
/// - Per-tool timeout configuration
/// - Same-turn call deduplication
///
/// Uses interior mutability (`RwLock`) so tools can be registered via `&self`,
/// enabling late registration (e.g. `SpawnSubagentTool` after `Arc<ToolRegistry>` is created).
pub struct ToolRegistry {
    pub(super) tools: RwLock<HashMap<String, Arc<dyn BaseTool>>>,
    pub(super) middleware: RwLock<Vec<Arc<dyn ToolMiddleware>>>,
    /// Per-tool timeout overrides keyed by tool name.
    pub(super) tool_timeouts: RwLock<HashMap<String, ToolTimeoutConfig>>,
    /// Dedup cache for same-turn identical calls.
    pub(super) dedup_cache: Mutex<HashMap<String, ToolResult>>,
    /// Aliases mapping old tool names to canonical new names (for backward compat).
    aliases: RwLock<HashMap<String, String>>,
    /// Sanitizer for truncating oversized tool outputs.
    pub(super) sanitizer: ToolResultSanitizer,
    /// Optional directory for overflow file storage.
    #[allow(dead_code)]
    overflow_dir: Option<std::path::PathBuf>,
    /// Core tool names — always included in LLM API calls.
    /// Non-core (deferred) tools are only included after activation via ToolSearch.
    core_tools: RwLock<HashSet<String>>,
}

impl std::fmt::Debug for ToolRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tool_count = self.tools.read().map(|t| t.len()).unwrap_or(0);
        let mw_count = self.middleware.read().map(|m| m.len()).unwrap_or(0);
        f.debug_struct("ToolRegistry")
            .field("tool_count", &tool_count)
            .field("middleware_count", &mw_count)
            .finish()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: RwLock::new(HashMap::new()),
            middleware: RwLock::new(Vec::new()),
            tool_timeouts: RwLock::new(HashMap::new()),
            dedup_cache: Mutex::new(HashMap::new()),
            aliases: RwLock::new(HashMap::new()),
            sanitizer: ToolResultSanitizer::new(),
            overflow_dir: None,
            core_tools: RwLock::new(HashSet::new()),
        }
    }

    /// Create with an overflow directory for storing full tool outputs to disk
    /// when they exceed inline size limits.
    pub fn with_overflow_dir(overflow_dir: std::path::PathBuf) -> Self {
        Self {
            tools: RwLock::new(HashMap::new()),
            middleware: RwLock::new(Vec::new()),
            tool_timeouts: RwLock::new(HashMap::new()),
            dedup_cache: Mutex::new(HashMap::new()),
            aliases: RwLock::new(HashMap::new()),
            sanitizer: ToolResultSanitizer::new().with_overflow_dir(overflow_dir.clone()),
            overflow_dir: Some(overflow_dir),
            core_tools: RwLock::new(HashSet::new()),
        }
    }

    /// Register an alias mapping an old tool name to a canonical new name.
    pub fn register_alias(&self, old_name: impl Into<String>, new_name: impl Into<String>) {
        let mut aliases = self.aliases.write().unwrap_or_else(|e| e.into_inner());
        aliases.insert(old_name.into(), new_name.into());
    }

    /// Register all legacy aliases from the tool_names module.
    pub fn register_legacy_aliases(&self) {
        for (old, new) in crate::tool_names::legacy_aliases() {
            self.register_alias(old, new);
        }
    }

    /// Resolve a name through the alias table. Returns the canonical name.
    pub fn resolve_alias(&self, name: &str) -> Option<String> {
        let aliases = self.aliases.read().unwrap_or_else(|e| e.into_inner());
        aliases.get(name).cloned()
    }

    /// Register a tool. If a tool with the same name exists, it's replaced.
    pub fn register(&self, tool: Arc<dyn BaseTool>) {
        let name = tool.name().to_string();
        let mut tools = self.tools.write().unwrap_or_else(|e| e.into_inner());
        tools.insert(name, tool);
    }

    /// Remove a tool by name and return it, if found.
    pub fn unregister(&self, name: &str) -> Option<Arc<dyn BaseTool>> {
        let mut tools = self.tools.write().unwrap_or_else(|e| e.into_inner());
        tools.remove(name)
    }

    /// Get a tool by exact name.
    pub fn get(&self, name: &str) -> Option<Arc<dyn BaseTool>> {
        let tools = self.tools.read().unwrap_or_else(|e| e.into_inner());
        if let Some(t) = tools.get(name) {
            return Some(t.clone());
        }
        if let Some(canonical) = self.resolve_alias(name) {
            return tools.get(&canonical).cloned();
        }
        None
    }

    /// Check if a tool is registered.
    pub fn contains(&self, name: &str) -> bool {
        let tools = self.tools.read().unwrap_or_else(|e| e.into_inner());
        let name = name.strip_prefix("functions.").unwrap_or(name);
        if tools.contains_key(name) {
            return true;
        }
        if let Some(canonical) = self.resolve_alias(name) {
            return tools.contains_key(&canonical);
        }
        false
    }

    /// Get sorted list of all registered tool names.
    pub fn tool_names(&self) -> Vec<String> {
        let tools = self.tools.read().unwrap_or_else(|e| e.into_inner());
        let mut names: Vec<String> = tools.keys().cloned().collect();
        names.sort();
        names
    }

    /// Number of registered tools.
    pub fn len(&self) -> usize {
        self.tools.read().unwrap_or_else(|e| e.into_inner()).len()
    }

    /// Whether no tools are registered.
    pub fn is_empty(&self) -> bool {
        self.tools.read().unwrap_or_else(|e| e.into_inner()).is_empty()
    }

    /// Add a middleware to the pipeline.
    pub fn add_middleware(&self, mw: Box<dyn ToolMiddleware>) {
        let mut middleware = self.middleware.write().unwrap_or_else(|e| e.into_inner());
        middleware.push(Arc::from(mw));
    }

    /// Number of registered middleware.
    pub fn middleware_count(&self) -> usize {
        self.middleware.read().unwrap_or_else(|e| e.into_inner()).len()
    }

    /// Set a per-tool timeout override.
    pub fn set_tool_timeout(&self, tool_name: impl Into<String>, config: ToolTimeoutConfig) {
        let mut timeouts = self.tool_timeouts.write().unwrap_or_else(|e| e.into_inner());
        timeouts.insert(tool_name.into(), config);
    }

    /// Set multiple per-tool timeouts at once (bulk).
    pub fn set_tool_timeouts(&self, timeouts: HashMap<String, ToolTimeoutConfig>) {
        let mut current = self.tool_timeouts.write().unwrap_or_else(|e| e.into_inner());
        current.extend(timeouts);
    }

    /// Get the timeout config for a specific tool, if set.
    pub fn get_tool_timeout(&self, tool_name: &str) -> Option<ToolTimeoutConfig> {
        self.tool_timeouts.read().unwrap_or_else(|e| e.into_inner()).get(tool_name).cloned()
    }

    /// Clear the dedup cache (call at each turn boundary).
    pub fn clear_dedup_cache(&self) {
        if let Ok(mut cache) = self.dedup_cache.lock() {
            cache.clear();
        }
    }

    /// Number of entries in the dedup cache.
    pub fn dedup_cache_size(&self) -> usize {
        self.dedup_cache.lock().map(|c| c.len()).unwrap_or(0)
    }

    /// Build a map of tool name → display metadata from all registered tools
    /// that implement `display_meta()`.
    pub fn build_display_map(&self) -> HashMap<String, ToolDisplayMeta> {
        let tools = self.tools.read().unwrap_or_else(|e| e.into_inner());
        let mut map = HashMap::new();
        for (name, tool) in tools.iter() {
            if let Some(meta) = tool.display_meta() {
                map.insert(name.clone(), meta);
            }
        }
        map
    }

    /// Mark a tool as "core" — always included in LLM API calls.
    pub fn mark_as_core(&self, name: &str) {
        let mut core = self.core_tools.write().unwrap_or_else(|e| e.into_inner());
        core.insert(name.to_string());
    }

    /// Mark multiple tools as core.
    pub fn mark_core_tools(&self, names: &[&str]) {
        let mut core = self.core_tools.write().unwrap_or_else(|e| e.into_inner());
        for name in names {
            core.insert((*name).to_string());
        }
    }

    /// Check if a tool is marked as core.
    pub fn is_core(&self, name: &str) -> bool {
        let core = self.core_tools.read().unwrap_or_else(|e| e.into_inner());
        core.contains(name)
    }

    /// Get the set of core tool names.
    pub fn core_tool_names(&self) -> HashSet<String> {
        self.core_tools.read().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// Whether tool deferral is active (any core tools are marked).
    pub fn has_deferred_tools(&self) -> bool {
        let core = self.core_tools.read().unwrap_or_else(|e| e.into_inner());
        !core.is_empty()
    }

    /// Get schemas only for the given tool names (core + activated).
    pub fn get_schemas_for(&self, names: &HashSet<String>) -> Vec<serde_json::Value> {
        let tools = self.tools.read().unwrap_or_else(|e| e.into_inner());
        tools
            .values()
            .filter(|tool| names.contains(tool.name()))
            .map(|tool| {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": tool.name(),
                        "description": tool.description(),
                        "parameters": tool.parameter_schema()
                    }
                })
            })
            .collect()
    }

    /// Get compact summaries of deferred (non-core) tools as categorized markdown.
    ///
    /// Returns a formatted markdown string grouped by `ToolCategory`. Each group
    /// lists available tool names with descriptions in a compact format suitable
    /// for embedding in the system prompt. The LLM can use `ToolSearch` to
    /// discover these tools with their full schemas if needed.
    ///
    /// Example output:
    /// ```markdown
    /// ## Deferred Tools
    ///
    /// ### Read (available)
    /// - `Glob`: Find files by glob pattern
    /// - `Grep`: Search file contents
    ///
    /// ### Web (available)
    /// - `WebFetch`: Fetch a URL
    /// - `WebSearch`: Search the web
    /// ```
    pub fn get_deferred_summaries(&self) -> String {
        let tools = self.tools.read().unwrap_or_else(|e| e.into_inner());
        let core = self.core_tools.read().unwrap_or_else(|e| e.into_inner());

        // Collect deferred tools grouped by category
        use std::collections::BTreeMap;
        let mut by_category: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();

        for tool in tools.values() {
            if core.contains(tool.name()) {
                continue;
            }
            let cat = tool.category().to_string();
            by_category
                .entry(cat)
                .or_default()
                .push((tool.name().to_string(), tool.description().to_string()));
        }

        if by_category.is_empty() {
            return String::new();
        }

        let mut output = String::from("\n\n## Deferred Tools\n\n");
        output.push_str(
            "These tools are available but not loaded. Use ToolSearch to discover and activate them.\n\n",
        );

        for (category, tools) in &by_category {
            output.push_str(&format!("### {category}\n"));
            for (name, desc) in tools {
                output.push_str(&format!("- `{name}`: {desc}\n"));
            }
            output.push('\n');
        }

        output
    }

    /// Build a map of `ToolCategory` → tool names from all registered tools.
    ///
    /// Used by `ToolPolicy::resolve_from_registry()` to dynamically derive
    /// groups from `BaseTool::category()` instead of hardcoded lists.
    pub fn build_category_map(&self) -> HashMap<crate::traits::ToolCategory, Vec<String>> {
        let tools = self.tools.read().unwrap_or_else(|e| e.into_inner());
        let mut map: HashMap<crate::traits::ToolCategory, Vec<String>> = HashMap::new();
        for (name, tool) in tools.iter() {
            map.entry(tool.category()).or_default().push(name.clone());
        }
        map
    }

    /// Get OpenAI-compatible function schemas for all registered tools.
    pub fn get_schemas(&self) -> Vec<serde_json::Value> {
        let tools = self.tools.read().unwrap_or_else(|e| e.into_inner());
        tools
            .values()
            .map(|tool| {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": tool.name(),
                        "description": tool.description(),
                        "parameters": tool.parameter_schema()
                    }
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests;
