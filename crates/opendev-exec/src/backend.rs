use crate::policy::ExecRequest;
use std::process::Command;

/// Errors from sandbox backend operations.
#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error("Backend not supported on this platform: {0}")]
    NotSupported(String),
    #[error("Backend apply failed: {0}")]
    ApplyFailed(String),
    #[error("Backend post-spawn check failed: {0}")]
    PostSpawnCheckFailed(String),
    #[error("Backend init failed: {0}")]
    InitFailed(String),
}

/// The sandbox backend trait — applies OS-level isolation.
pub trait SandboxBackend: Send + Sync {
    /// Human-readable backend name.
    fn name(&self) -> &'static str;

    /// Check if this backend is supported on the current system.
    fn supported(&self) -> bool;

    /// Apply isolation to a Command before spawning.
    /// This is where Landlock rulesets, Seatbelt profiles, etc. are applied.
    ///
    /// # Safety
    /// On Unix, this may involve unsafe pre_exec hooks.
    fn apply(&self, cmd: &mut Command, request: &ExecRequest) -> Result<(), BackendError>;

    /// Post-spawn verification (e.g., check the child is actually confined).
    fn post_spawn_check(&self, child_pid: u32) -> Result<(), BackendError> {
        let _ = child_pid;
        Ok(()) // default: no check
    }
}

/// Auto-detect the best available backend for the current platform.
pub fn detect_backend() -> Option<Box<dyn SandboxBackend>> {
    #[cfg(target_os = "linux")]
    {
        let landlock = crate::backends::landlock::LandlockBackend;
        if landlock.supported() {
            tracing::info!("Sandbox: using Landlock backend");
            return Some(Box::new(landlock));
        }
        let bwrap = crate::backends::bwrap::BwrapBackend::new();
        if bwrap.supported() {
            tracing::info!("Sandbox: using bwrap backend");
            return Some(Box::new(bwrap));
        }
    }
    #[cfg(target_os = "macos")]
    {
        let seatbelt = crate::backends::seatbelt::SeatbeltBackend;
        if seatbelt.supported() {
            tracing::info!("Sandbox: using Seatbelt backend");
            return Some(Box::new(seatbelt));
        }
    }
    #[cfg(target_os = "windows")]
    {
        let windows_backend = crate::backends::windows::WindowsBackend;
        if windows_backend.supported() {
            tracing::info!("Sandbox: using Windows Job Object backend");
            return Some(Box::new(windows_backend));
        }
    }
    tracing::warn!("No sandbox backend available; using NoneBackend (env_filter only)");
    Some(Box::new(crate::backends::none::NoneBackend))
}
