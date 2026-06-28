use std::process::Command;
use crate::backend::{SandboxBackend, BackendError};
use crate::policy::ExecRequest;

/// No-op backend — used when no sandbox is available.
/// Only env_filter is applied at the process level.
pub struct NoneBackend;

impl SandboxBackend for NoneBackend {
    fn name(&self) -> &'static str {
        "none"
    }

    fn supported(&self) -> bool {
        true
    }

    fn apply(&self, _cmd: &mut Command, _request: &ExecRequest) -> Result<(), BackendError> {
        // No OS-level isolation — env_filter handles env var stripping.
        // The UI MUST warn the user when this backend is active.
        tracing::warn!("Using NoneBackend — no OS-level sandbox. Only env_filter is active.");
        Ok(())
    }
}
