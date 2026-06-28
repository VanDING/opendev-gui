use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LogLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn as_filter(&self) -> &'static str {
        match self {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        }
    }
}

impl std::str::FromStr for LogLevel {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "trace" => Ok(LogLevel::Trace),
            "debug" => Ok(LogLevel::Debug),
            "info" => Ok(LogLevel::Info),
            "warn" | "warning" => Ok(LogLevel::Warn),
            "error" => Ok(LogLevel::Error),
            _ => Err(format!("unknown log level: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LogFormat {
    #[default]
    Json,
    Pretty,
}

/// Telemetry configuration — ALL fields are now honored.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// Enable telemetry (default: true).
    /// When false, no log file is created, no OTLP export, no Sentry.
    pub enabled: bool,

    /// Minimum log level to capture.
    pub log_level: LogLevel,

    /// Log format — JSON or Pretty.
    #[serde(default)]
    pub format: LogFormat,

    /// Log retention in days (default: 14).
    /// Files older than this are cleaned up by the janitor task.
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,

    /// Output directory for log files.
    /// If empty, uses the default opendev logs directory.
    #[serde(default)]
    pub log_dir: Option<PathBuf>,

    // ── OTLP (default: off, requires OTEL_EXPORTER_OTLP_ENDPOINT) ──
    #[serde(default)]
    pub otlp_endpoint: Option<String>,

    /// OTLP protocol: "grpc" or "http".
    #[serde(default)]
    pub otlp_protocol: OtlpProtocol,

    // ── Privacy controls ──
    /// Record full LLM prompt content (default: false).
    /// When false, prompt content is redacted from logs.
    pub record_prompt_content: bool,

    /// Record full tool arguments (default: false).
    pub record_tool_args: bool,

    /// Record file paths accessed by tools (default: true).
    pub record_file_location: bool,

    // ── Sentry (opt-in) ──
    /// Sentry DSN. Empty/None = Sentry is disabled.
    #[serde(default)]
    pub sentry_dsn: Option<String>,

    /// Sentry sample rate (default: 0.1 = 10%).
    #[serde(default = "default_sentry_sample_rate")]
    pub sentry_sample_rate: f32,

    // ── Perfetto ──
    /// Generate Perfetto trace on session end (default: false).
    #[serde(default)]
    pub export_perfetto_on_session_end: bool,

    /// Where to write .pftrace files.
    #[serde(default)]
    pub perfetto_output_dir: Option<PathBuf>,

    // ── Session Debug ──
    /// Debug logging for sessions (default: false).
    #[serde(default)]
    pub debug_logging: bool,

    /// Include full payload in debug logging (requires debug_logging=true).
    /// User must explicitly opt in — this includes raw prompts.
    #[serde(default)]
    pub include_full_payload: bool,
}

fn default_retention_days() -> u32 { 14 }
fn default_sentry_sample_rate() -> f32 { 0.1 }

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_level: LogLevel::Info,
            format: LogFormat::Json,
            retention_days: 14,
            log_dir: None,
            otlp_endpoint: None,
            otlp_protocol: OtlpProtocol::default(),
            record_prompt_content: false,
            record_tool_args: false,
            record_file_location: true,
            sentry_dsn: None,
            sentry_sample_rate: 0.1,
            export_perfetto_on_session_end: false,
            perfetto_output_dir: None,
            debug_logging: false,
            include_full_payload: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum OtlpProtocol {
    #[default]
    Grpc,
    Http,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn log_level_parsing() {
        assert_eq!(LogLevel::from_str("debug").unwrap(), LogLevel::Debug);
        assert_eq!(LogLevel::from_str("WARN").unwrap(), LogLevel::Warn);
        assert!(LogLevel::from_str("unknown").is_err());
    }

    #[test]
    fn test_defaults() {
        let cfg = TelemetryConfig::default();
        assert!(cfg.enabled);
        assert_eq!(cfg.retention_days, 14);
        assert_eq!(cfg.format, LogFormat::Json);
        assert!(!cfg.debug_logging);
        assert!(!cfg.record_prompt_content);
    }
}
