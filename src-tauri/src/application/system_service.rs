//! SystemService — System-level operations.
//!
//! Provides git branch info, working directory, health checks, and bridge info.

pub struct SystemService {
    working_dir: String,
}

impl SystemService {
    pub fn new(working_dir: String) -> Self {
        Self { working_dir }
    }

    /// Get the working directory.
    pub fn working_dir(&self) -> &str {
        &self.working_dir
    }

    /// Get the current git branch.
    pub fn git_branch(&self) -> Option<String> {
        let output = std::process::Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(&self.working_dir)
            .output()
            .ok()?;

        if output.status.success() {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            None
        }
    }

    /// Health check — always returns ok.
    pub fn health(&self) -> serde_json::Value {
        serde_json::json!({
            "status": "ok",
            "service": "opendev-desktop",
        })
    }

    /// Get bridge mode info (always inactive in desktop mode).
    pub fn bridge_info(&self) -> serde_json::Value {
        serde_json::json!({
            "bridge_mode": false,
            "session_id": null,
        })
    }
}
