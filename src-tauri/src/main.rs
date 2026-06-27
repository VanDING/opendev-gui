#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod server;
mod application;
mod interface;

use tauri::{Emitter, Manager};
use opendev_web::state::WsBroadcast;

use application::AppServices;
use interface::desktop::commands;

/// Bridge event broadcasts from the agent runtime to Tauri IPC events.
fn spawn_event_bridge(
    app: tauri::AppHandle,
    mut broadcast_rx: tokio::sync::broadcast::Receiver<WsBroadcast>,
) {
    tokio::spawn(async move {
        loop {
            match broadcast_rx.recv().await {
                Ok(msg) => {
                    // Use message type as event name, data as payload.
                    // Frontend receives this via Transport.onEvent().
                    let _ = app.emit(&msg.msg_type, msg.data);
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    eprintln!("Event broadcast lagged by {} messages", n);
                }
            }
        }
    });
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let working_dir =
                std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

            // ── Build Application Services ──────────────────────────────
            let paths = opendev_config::Paths::new(Some(working_dir.to_path_buf()));
            let config = opendev_config::ConfigLoader::load(
                &paths.global_settings(),
                &paths.project_settings(),
            )
            .map_err(|e| format!("Failed to load config: {}", e))
            .expect("Failed to load config");

            let session_dir = paths.global_sessions_dir();
            std::fs::create_dir_all(&session_dir)
                .expect("Failed to create session directory");
            let session_manager = opendev_history::SessionManager::new(session_dir)
                .expect("Failed to init session manager");

            let model_registry = opendev_config::ModelRegistry::new();

            let services = crate::interface::services::build_services(
                config,
                session_manager,
                model_registry,
                working_dir.to_string_lossy().to_string(),
            );

            // Register Application Services as Tauri managed state
            app.manage(services);

            // ── Event Broadcast Infrastructure ──────────────────────────
            // Sets up the event broadcast channel (no HTTP server).
            // Agent events flow through this channel to Tauri IPC events.
            match crate::server::setup_event_broadcast(&working_dir) {
                Ok(handle) => {
                    spawn_event_bridge(app.handle().clone(), handle.broadcast_rx);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to set up event broadcast: {}", e);
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Config commands
            commands::config::get_app_config,
            commands::config::update_app_config,
            commands::config::set_operation_mode,
            commands::config::set_autonomy_level,
            commands::config::list_model_providers,
            commands::config::verify_model,
            // Session commands
            commands::session::list_sessions,
            commands::session::create_session,
            commands::session::get_session,
            commands::session::delete_session,
            commands::session::resume_session,
            commands::session::get_session_messages,
            commands::session::get_session_model,
            commands::session::update_session_model,
            commands::session::clear_session_model,
            // Chat commands
            commands::chat::send_chat_query,
            commands::chat::interrupt_chat,
            commands::chat::clear_chat,
            commands::chat::get_chat_messages,
            // Workflow commands
            commands::workflow::approve_tool,
            commands::workflow::respond_to_ask,
            commands::workflow::respond_to_plan,
            // MCP commands
            commands::mcp::list_mcp_servers,
            commands::mcp::get_mcp_server,
            commands::mcp::create_mcp_server,
            commands::mcp::update_mcp_server,
            commands::mcp::delete_mcp_server,
            commands::mcp::connect_mcp_server,
            commands::mcp::disconnect_mcp_server,
            // Skills commands
            commands::skills::list_skills,
            commands::skills::toggle_skill_pin,
            // Files commands
            commands::files::browse_directory,
            commands::files::verify_path,
            commands::files::list_workspace_files,
        ])
        .run(tauri::generate_context!())
        .expect("error while running OpenDev Desktop");
}
