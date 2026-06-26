use std::path::PathBuf;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, fmt};

use crate::OtelError;
use crate::config::{LogLevel, TelemetryConfig};

pub struct OtelGuard {
    _logger_guard: WorkerGuard,
}

impl OtelGuard {
    pub fn init(config: &TelemetryConfig) -> Result<Self, OtelError> {
        let log_dir = log_dir();
        std::fs::create_dir_all(&log_dir)
            .map_err(|e| OtelError::Init { message: format!("failed to create log dir: {e}") })?;

        let env_filter = build_env_filter(config.log_level);
        let (file_layer, logger_guard) = build_file_layer(&log_dir)?;

        fmt().with_target(true).with_env_filter(env_filter).with_writer(file_layer).init();

        Ok(Self { _logger_guard: logger_guard })
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
) -> Result<(tracing_appender::non_blocking::NonBlocking, WorkerGuard), OtelError> {
    let file_appender = tracing_appender::rolling::RollingFileAppender::new(
        tracing_appender::rolling::Rotation::DAILY,
        log_dir,
        "opendev.log",
    );
    Ok(tracing_appender::non_blocking(file_appender))
}

fn log_dir() -> PathBuf {
    let paths = opendev_config::Paths::new(None);
    paths.global_logs_dir()
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
            let _ = OtelGuard::init(&config);
        });
        tracing::info!(target: "opendev", "test log message");
    }
}
