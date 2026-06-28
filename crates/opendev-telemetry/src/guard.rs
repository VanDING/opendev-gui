use crate::TelemetryError;
use crate::config::{LogLevel, TelemetryConfig};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

pub struct TelemetryGuard {
    _worker_guard: Option<tracing_appender::non_blocking::WorkerGuard>,
}

impl TelemetryGuard {
    /// Initialize telemetry with all layers.
    ///
    /// Call this FIRST in the startup sequence (before secrets, sandbox, protocol).
    /// If init fails, logs go to stderr as fallback.
    pub fn init(config: &TelemetryConfig) -> Result<Self, TelemetryError> {
        if !config.enabled {
            tracing::warn!("Telemetry is disabled. No logs will be written.");
            return Ok(Self { _worker_guard: None });
        }

        let log_dir = config.log_dir.clone().unwrap_or_else(|| {
            let paths = opendev_config::Paths::new(None);
            paths.global_logs_dir()
        });
        std::fs::create_dir_all(&log_dir).map_err(|e| TelemetryError::Init {
            message: format!("failed to create log dir: {e}"),
        })?;

        let env_filter = build_env_filter(config.log_level);
        let (file_writer, worker_guard) = build_file_layer(&log_dir, config.retention_days)
            .map_err(|e| TelemetryError::Init { message: e })?;

        // Build layer stack
        let file_layer = tracing_subscriber::fmt::layer()
            .json()
            .with_current_span(true)
            .with_span_list(false)
            .with_writer(file_writer);

        let registry = tracing_subscriber::registry().with(env_filter).with(file_layer);

        // Init globally
        registry.init();

        Ok(Self { _worker_guard: Some(worker_guard) })
    }

    pub fn flush(&self) {
        tracing::info!("telemetry flushed");
    }
}

fn build_env_filter(level: LogLevel) -> EnvFilter {
    EnvFilter::from_default_env().add_directive(
        format!("opendev={}", level.as_filter())
            .parse()
            .unwrap_or_else(|_| "opendev=info".parse().unwrap()),
    )
}

fn build_file_layer(
    log_dir: &std::path::Path,
    retention_days: u32,
) -> Result<
    (tracing_appender::non_blocking::NonBlocking, tracing_appender::non_blocking::WorkerGuard),
    String,
> {
    // Use rolling file appender with max_log_files for retention
    let file_appender = tracing_appender::rolling::RollingFileAppender::builder()
        .rotation(tracing_appender::rolling::Rotation::DAILY)
        .max_log_files(retention_days as usize)
        .filename_prefix("opendev")
        .filename_suffix("log")
        .build(log_dir)
        .map_err(|e| format!("failed to create log file appender: {e}"))?;

    Ok(tracing_appender::non_blocking(file_appender))
}

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use super::*;

    #[test]
    fn init_does_not_panic() {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            let config = TelemetryConfig { log_level: LogLevel::Debug, ..Default::default() };
            let _ = TelemetryGuard::init(&config);
        });
        tracing::info!(target: "opendev", "test log message");
    }
}
