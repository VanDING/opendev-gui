//! Configuration loading, tracing setup, crash handler, and utility functions.

use opendev_mcp::config::{
    get_project_config_path, load_config as load_mcp_config_file, merge_configs,
    save_config as save_mcp_config_file,
};

pub fn init_tracing(verbose: bool, _tui_mode: bool) {
    use opendev_telemetry::{LogLevel, TelemetryConfig};

    let log_level = if verbose { LogLevel::Debug } else { LogLevel::Info };

    let config = TelemetryConfig {
        enabled: true,
        log_level,
        format: opendev_telemetry::LogFormat::Json,
        retention_days: 14,
        ..Default::default()
    };

    if let Err(e) = opendev_telemetry::TelemetryGuard::init(&config) {
        // Fallback: basic stderr logging
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::new(if verbose {
                "debug"
            } else {
                "error"
            }))
            .with_target(false)
            .with_writer(std::io::stderr)
            .init();
        eprintln!("Warning: failed to initialize telemetry: {e}");
    }
}

/// Load the merged AppConfig using standard paths for the given working directory.
pub fn load_app_config(working_dir: &std::path::Path) -> opendev_models::AppConfig {
    let paths = opendev_config::Paths::new(Some(working_dir.to_path_buf()));
    let global_settings = paths.global_settings();
    let project_settings = paths.project_settings();

    // Check for unmigrated plaintext secrets
    if let Ok(true) = opendev_secrets::migration::has_unmigrated_secrets(&global_settings) {
        eprintln!(
            "⚠️  settings.json contains plaintext API keys. Consider running `opendev secret migrate`."
        );
    }

    match opendev_config::ConfigLoader::load(&global_settings, &project_settings) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Warning: failed to load config: {e}");
            opendev_models::AppConfig::default()
        }
    }
}

/// Load the merged MCP configuration (global + project).
pub fn load_mcp_config(working_dir: &std::path::Path) -> opendev_mcp::McpConfig {
    let paths = opendev_config::Paths::new(Some(working_dir.to_path_buf()));
    let global_mcp_path = paths.global_mcp_config();
    let project_mcp_path = get_project_config_path(working_dir);

    let global_config = match load_mcp_config_file(&global_mcp_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: failed to load global MCP config: {e}");
            opendev_mcp::McpConfig::default()
        }
    };

    let project_config = project_mcp_path.as_deref().and_then(|p| load_mcp_config_file(p).ok());

    merge_configs(&global_config, project_config.as_ref())
}

/// Save the global MCP configuration.
pub fn save_global_mcp_config(config: &opendev_mcp::McpConfig) {
    let paths = opendev_config::Paths::default();
    let global_mcp_path = paths.global_mcp_config();
    if let Err(e) = save_mcp_config_file(config, &global_mcp_path) {
        eprintln!("Error: failed to save MCP config: {e}");
        std::process::exit(1);
    }
}

/// Install a custom panic hook that writes crash reports to `~/.opendev/crash/`.
pub fn install_panic_handler() {
    // Delegate to opendev-telemetry's crash handler
    opendev_telemetry::layers::panic::install_crash_handler();
}

/// Format a timestamp as a relative time string (e.g., "just now", "5m ago").
pub fn format_relative_time(dt: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let diff = now.signed_duration_since(dt);
    let secs = diff.num_seconds();

    if secs < 60 {
        "just now".to_string()
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86400 {
        format!("{}h ago", secs / 3600)
    } else {
        format!("{}d ago", secs / 86400)
    }
}

/// Detect the current git branch for the given directory.
pub fn detect_git_branch(working_dir: &std::path::Path) -> Option<String> {
    std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(working_dir)
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                None
            }
        })
}
