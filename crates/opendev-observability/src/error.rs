use std::fmt;

#[derive(Debug)]
pub enum OtelError {
    Init { message: String },
    Export { message: String },
}

impl fmt::Display for OtelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OtelError::Init { message } => write!(f, "telemetry init failed: {message}"),
            OtelError::Export { message } => write!(f, "telemetry export failed: {message}"),
        }
    }
}

impl std::error::Error for OtelError {}
