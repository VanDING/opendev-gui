//! Legacy server module — maintains the event broadcast infrastructure
//! without an HTTP listener.
//!
//! The AppState and broadcast channel are still needed for agent event
//! routing. The actual HTTP listener has been removed.
//! This module will be fully removed when agent events flow directly
//! through Application Services.

use std::path::{Path, PathBuf};

use opendev_agents::SkillLoader;
use opendev_config::ConfigLoader;
use opendev_config::Paths;
use opendev_history::SessionManager;
use opendev_http::UserStore;
use opendev_web::state::{AppState, WsBroadcast};

/// Handle containing the broadcast receiver for the Tauri event bridge.
pub struct EventBridgeHandle {
    pub broadcast_rx: tokio::sync::broadcast::Receiver<WsBroadcast>,
}

/// Sets up the broadcast infrastructure without starting an HTTP server.
pub fn setup_event_broadcast(working_dir: &Path) -> Result<EventBridgeHandle, String> {
    let paths = Paths::new(Some(working_dir.to_path_buf()));

    let config = ConfigLoader::load(&paths.global_settings(), &paths.project_settings())
        .map_err(|e| format!("Failed to load config: {}", e))?;

    let skill_paths = config.skill_paths.clone();
    let skill_urls = config.skill_urls.clone();

    let session_dir = paths.global_sessions_dir();
    std::fs::create_dir_all(&session_dir)
        .map_err(|e| format!("Failed to create session dir: {}", e))?;
    let session_manager = SessionManager::new(session_dir)
        .map_err(|e| format!("Failed to init session manager: {}", e))?;

    let user_store_dir = paths.data_dir().join("users");
    std::fs::create_dir_all(&user_store_dir)
        .map_err(|e| format!("Failed to create user store dir: {}", e))?;
    let user_store =
        UserStore::new(user_store_dir).map_err(|e| format!("Failed to init user store: {}", e))?;

    let model_registry = opendev_config::ModelRegistry::new();

    let state = AppState::new(
        session_manager,
        config,
        working_dir.to_string_lossy().to_string(),
        user_store,
        model_registry,
    );

    // Skill Loader
    let skill_dirs = resolve_skill_dirs(working_dir, &skill_paths);
    let mut skill_loader = SkillLoader::new(skill_dirs);
    if !skill_urls.is_empty() {
        skill_loader.add_urls(skill_urls);
    }
    let _ = tokio::runtime::Handle::current().block_on(state.set_skill_loader(skill_loader));

    let broadcast_rx = state.ws_subscribe();

    Ok(EventBridgeHandle { broadcast_rx })
}

fn resolve_skill_dirs(working_dir: &Path, skill_paths: &[String]) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    dirs.push(working_dir.join(".opendev").join("skills"));
    if let Some(home) = home_dir() {
        dirs.push(home.join(".opendev").join("skills"));
    }
    for path in skill_paths {
        let resolved = if let Some(rest) = path.strip_prefix("~/") {
            home_dir().map(|h| h.join(rest)).unwrap_or_else(|| PathBuf::from(path))
        } else if Path::new(path).is_absolute() {
            PathBuf::from(path)
        } else {
            working_dir.join(path)
        };
        dirs.push(resolved);
    }
    dirs
}

fn home_dir() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}

/// Maps legacy event names (4 naming conventions) to v1 protocol event names.
/// This shim supports the dual-emit period (v0.2.0 → v0.3.0).
///
/// Naming conventions covered:
/// - snake_case: `message_start`, `mcp_servers_updated`
/// - colon: `mcp:status_changed`, `mcp:servers_updated`
/// - dot: `mcp.server.connected`
/// - underscore alias: `mcp_status_update`
///
/// This shim will be removed in v0.4.0 when all frontend handlers have migrated.
pub fn legacy_event_name_to_v1(legacy_name: &str) -> Option<&'static str> {
    match legacy_name {
        // ── Message lifecycle ──
        "message_start" => Some("message/started"),
        "message_chunk" => Some("message/chunked"),
        "message_complete" => Some("message/completed"),
        "thinking_block" => Some("thinking/chunked"),

        // ── Tool events ──
        "tool_call" => Some("tool/started"),
        "tool_result" => Some("tool/completed"),

        // ── Subagent events ──
        "subagent_start" => Some("subagent/spawned"),
        "subagent_complete" => Some("subagent/completed"),
        "nested_tool_call" => Some("nested/tool/started"),
        "nested_tool_result" => Some("nested/tool/completed"),

        // ── Status & progress ──
        "status_update" => Some("status/updated"),
        "progress" => Some("progress/updated"),

        // ── Approval / ask / plan ──
        "approval_required" => Some("approval/required"),
        "approval_resolved" => Some("approval/required"), // v1 combines approve/resolve
        "ask_user_required" => Some("ask/required"),
        "ask_user_resolved" => Some("ask/required"),
        "plan_approval_required" => Some("plan/required"),
        "plan_approval_resolved" => Some("plan/required"),

        // ── Session ──
        "session_activity" => Some("session/activity"),

        // ── Error ──
        "error" => Some("error/raised"),

        // ── MCP (4 naming conventions → 1 v1 name) ──
        "mcp_servers_updated" | "mcp_servers_update" => Some("mcp/server/connected"),
        "mcp:servers_updated" | "mcp:status_changed" => Some("mcp/server/connected"),
        "mcp.server.connected" | "mcp.server.disconnected" => Some("mcp/server/connected"),
        "mcp_status_update" => Some("mcp/server/connected"),

        // ── Misc ──
        "user_message" => None, // client→server only, no v1 event
        "connected" | "disconnected" => None, // connection lifecycle, Tauri internal
        "pong" | "plan_content" | "full_sync" | "task_completed" => None,
        "parallel_agents_start" | "parallel_agents_done" => None,

        // Unknown → no mapping
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legacy_to_v1_snake_case() {
        assert_eq!(legacy_event_name_to_v1("message_start"), Some("message/started"));
        assert_eq!(legacy_event_name_to_v1("message_chunk"), Some("message/chunked"));
        assert_eq!(legacy_event_name_to_v1("message_complete"), Some("message/completed"));
    }

    #[test]
    fn test_legacy_to_v1_mcp_all_conventions() {
        // All 4 conventions map to the same v1 event
        assert_eq!(legacy_event_name_to_v1("mcp_servers_updated"), Some("mcp/server/connected"));
        assert_eq!(legacy_event_name_to_v1("mcp:servers_updated"), Some("mcp/server/connected"));
        assert_eq!(legacy_event_name_to_v1("mcp.server.connected"), Some("mcp/server/connected"));
        assert_eq!(legacy_event_name_to_v1("mcp_status_update"), Some("mcp/server/connected"));
    }

    #[test]
    fn test_legacy_to_v1_unknown_returns_none() {
        assert_eq!(legacy_event_name_to_v1("nonexistent_event"), None);
        assert_eq!(legacy_event_name_to_v1("user_message"), None);
        assert_eq!(legacy_event_name_to_v1("pong"), None);
    }

    #[test]
    fn test_legacy_to_v1_tool_events() {
        assert_eq!(legacy_event_name_to_v1("tool_call"), Some("tool/started"));
        assert_eq!(legacy_event_name_to_v1("tool_result"), Some("tool/completed"));
    }

    #[test]
    fn test_legacy_to_v1_approval_events() {
        assert_eq!(legacy_event_name_to_v1("approval_required"), Some("approval/required"));
        assert_eq!(legacy_event_name_to_v1("ask_user_required"), Some("ask/required"));
        assert_eq!(legacy_event_name_to_v1("plan_approval_required"), Some("plan/required"));
    }
}
