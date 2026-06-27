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
