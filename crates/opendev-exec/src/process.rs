use std::process::Command;
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use crate::env_filter;
use crate::policy::ExecRequest;

/// A hardened command that applies env_filter and pre_exec hooks.
pub struct HardenedProcess {
    cmd: Command,
}

impl HardenedProcess {
    /// Create a new hardened process from a Command + ExecRequest.
    pub fn new(mut base_cmd: Command, _request: &ExecRequest) -> Self {
        // Apply env filter
        let safe_env = env_filter::filtered_env();
        base_cmd.env_clear();
        for (k, v) in &safe_env {
            base_cmd.env(k, v);
        }

        // Set PYTHONUNBUFFERED for Python tools
        base_cmd.env("PYTHONUNBUFFERED", "1");

        #[cfg(unix)]
        unsafe {
            base_cmd.pre_exec(|| {
                // New process group (so we can kill descendants)
                libc::setpgid(0, 0);
                Ok(())
            });
        }

        Self { cmd: base_cmd }
    }

    /// Spawn the hardened process.
    pub fn spawn(&mut self) -> std::io::Result<std::process::Child> {
        self.cmd.spawn()
    }
}
