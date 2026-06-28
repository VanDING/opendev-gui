#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod application;
mod interface;
mod server;

use opendev_web::state::WsBroadcast;
use tauri::{Emitter, Manager};

use application::AppServices;
use interface::desktop::commands;

/// Bridge event broadcasts from the agent runtime to Tauri IPC events.
///
/// Implements dual-emit strategy (v0.2.0 → v0.3.0):
/// - Emits original legacy event name (backward compat for existing frontend handlers)
/// - Also emits v1 protocol event name (for new frontend handlers migrating to v1 protocol)
///
/// The dual-emit period ends at v0.3.0 when all frontend handlers have migrated.
/// The legacy shim is removed at v0.4.0.
///
/// See also: `opendev-protocol` crate for Transport trait implementations.
fn spawn_event_bridge(
    app: tauri::AppHandle,
    mut broadcast_rx: tokio::sync::broadcast::Receiver<WsBroadcast>,
) {
    tokio::spawn(async move {
        loop {
            match broadcast_rx.recv().await {
                Ok(msg) => {
                    // ── Legacy emit (backward compat) ──
                    // Frontend receives this via Transport.onEvent() using old event names.
                    let _ = app.emit(&msg.msg_type, msg.data.clone());

                    // ── V1 protocol emit (dual-emit for migration period) ──
                    // Frontend handlers can migrate to subscribe to v1 event names.
                    if let Some(v1_name) = crate::server::legacy_event_name_to_v1(&msg.msg_type) {
                        let _ = app.emit(v1_name, msg.data);
                    }
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
    // ── Initialize Telemetry FIRST ──
    // This must happen before any other initialization so all
    // subsequent operations are observable.
    let _telemetry_guard =
        opendev_telemetry::TelemetryGuard::init(&opendev_telemetry::TelemetryConfig {
            enabled: true,
            log_level: opendev_telemetry::LogLevel::Info,
            format: opendev_telemetry::LogFormat::Json,
            retention_days: 14,
            ..Default::default()
        })
        .map_err(|e| {
            eprintln!("Warning: failed to initialize telemetry: {e}");
        })
        .ok();

    // Install panic handler
    opendev_telemetry::layers::panic::install_crash_handler();

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
            std::fs::create_dir_all(&session_dir).expect("Failed to create session directory");
            let session_manager = opendev_history::SessionManager::new(session_dir)
                .expect("Failed to init session manager");

            let model_registry = opendev_config::ModelRegistry::new();

            // ── Secret Store Migration ──────────────────────────────
            // Check if settings.json has unmigrated plaintext secrets.
            let global_settings = paths.global_settings();
            if let Ok(has_secrets) =
                opendev_secrets::migration::has_unmigrated_secrets(&global_settings)
            {
                if has_secrets {
                    eprintln!(
                        "⚠️  settings.json contains API keys in plaintext. \
                         Run `opendev secret migrate` to migrate to OS keyring."
                    );
                }
            }

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
