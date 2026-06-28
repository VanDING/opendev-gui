//! JSON file logging layer with retention.

use crate::config::TelemetryConfig;
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};

/// Build the JSON file logging layer.
pub fn build(config: &TelemetryConfig) -> Result<(NonBlocking, WorkerGuard), String> {
    let log_dir = config.log_dir.clone().unwrap_or_else(|| {
        let paths = opendev_config::Paths::new(None);
        paths.global_logs_dir()
    });

    std::fs::create_dir_all(&log_dir).map_err(|e| format!("failed to create log dir: {e}"))?;

    let file_appender = tracing_appender::rolling::RollingFileAppender::builder()
        .rotation(tracing_appender::rolling::Rotation::DAILY)
        .max_log_files(config.retention_days as usize)
        .filename_prefix("opendev")
        .filename_suffix("log")
        .build(&log_dir)
        .map_err(|e| format!("failed to create log file appender: {e}"))?;

    Ok(tracing_appender::non_blocking(file_appender))
}
