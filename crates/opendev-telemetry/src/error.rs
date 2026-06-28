use std::fmt;

#[derive(Debug)]
pub enum TelemetryError {
    Init { message: String },
    Export { message: String },
}

impl fmt::Display for TelemetryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TelemetryError::Init { message } => write!(f, "telemetry init failed: {message}"),
            TelemetryError::Export { message } => write!(f, "telemetry export failed: {message}"),
        }
    }
}

impl std::error::Error for TelemetryError {}
