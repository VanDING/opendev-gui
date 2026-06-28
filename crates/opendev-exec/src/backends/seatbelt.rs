use std::process::Command;
use crate::backend::{SandboxBackend, BackendError};
use crate::policy::ExecRequest;

/// macOS Seatbelt-backed sandbox using sandbox-exec.
/// Applies a sandbox profile before execution.
pub struct SeatbeltBackend;

impl SandboxBackend for SeatbeltBackend {
    fn name(&self) -> &'static str {
        "seatbelt"
    }

    fn supported(&self) -> bool {
        // sandbox-exec is available on macOS 10.7+
        std::path::Path::new("/usr/bin/sandbox-exec").exists()
    }

    fn apply(&self, cmd: &mut Command, request: &ExecRequest) -> Result<(), BackendError> {
        // Build a minimal sandbox profile
        let profile = build_seatbelt_profile(request);

        // Write profile to a temp file
        let tmp_dir = std::env::temp_dir();
        let profile_path =
            tmp_dir.join(format!("opendev-seatbelt-{}.sb", std::process::id()));
        std::fs::write(&profile_path, &profile)
            .map_err(|e| BackendError::ApplyFailed(format!("Failed to write seatbelt profile: {}", e)))?;

        // Wrap the command with sandbox-exec
        let original_program = cmd.get_program().to_os_string();
        let original_args: Vec<_> = cmd.get_args().map(|a| a.to_os_string()).collect();
        let original_envs: Vec<_> = cmd
            .get_envs()
            .map(|(k, v)| (k.to_os_string(), v.map(|v| v.to_os_string())))
            .collect();
        let original_dir = cmd.get_current_dir().map(|d| d.to_path_buf());

        // Replace with sandbox-exec
        *cmd = Command::new("/usr/bin/sandbox-exec");
        cmd.arg("-f").arg(&profile_path);
        cmd.arg("--");
        cmd.arg(&original_program);
        for arg in &original_args {
            cmd.arg(arg);
        }
        for (k, v) in &original_envs {
            if let Some(val) = v {
                cmd.env(k, val);
            }
        }
        if let Some(dir) = &original_dir {
            cmd.current_dir(dir);
        }

        Ok(())
    }
}

fn build_seatbelt_profile(request: &ExecRequest) -> String {
    let mut profile = String::new();
    profile.push_str("(version 1)\n");
    profile.push_str("(deny default)\n");

    // ── Process lifecycle ──
    profile.push_str("(allow process-exec)\n");
    profile.push_str("(allow process-fork)\n");
    profile.push_str("(allow signal)\n");
    profile.push_str("(allow sysctl-read)\n");

    // ── File I/O: basic operations required for any command ──
    profile.push_str("(allow file-read-data)\n");
    profile.push_str("(allow file-read-metadata)\n");

    // ── File read access: system paths ──
    profile.push_str("(allow file-read*\n");
    profile.push_str("  (subpath \"/usr\")\n");
    profile.push_str("  (subpath \"/bin\")\n");
    profile.push_str("  (subpath \"/sbin\")\n");
    profile.push_str("  (subpath \"/dev\")\n");
    profile.push_str("  (subpath \"/etc\")\n");
    profile.push_str("  (subpath \"/private\")\n");
    profile.push_str("  (subpath \"/tmp\")\n");
    profile.push_str("  (subpath \"/var\")\n");
    profile.push_str("  (subpath \"/Library\")\n");
    profile.push_str("  (subpath \"/System\")\n");

    // Allow reading workspace paths from request
    for path in &request.capabilities.read {
        profile.push_str(&format!("  (subpath \"{}\")\n", path.display()));
    }

    // Allow reading the working directory
    profile.push_str(&format!(
        "  (subpath \"{}\")\n",
        request.cwd.display()
    ));
    profile.push_str(")\n"); // close file-read*

    // ── File write access: tmp + dev ──
    profile.push_str("(allow file-write*\n");
    profile.push_str("  (subpath \"/tmp\")\n");
    profile.push_str("  (subpath \"/private/tmp\")\n");
    profile.push_str("  (subpath \"/dev\")\n");

    // Allow writing to workspace paths from request
    for path in &request.capabilities.write {
        profile.push_str(&format!("  (subpath \"{}\")\n", path.display()));
    }

    // Allow writing to the working directory
    profile.push_str(&format!(
        "  (subpath \"{}\")\n",
        request.cwd.display()
    ));
    profile.push_str(")\n"); // close file-write*

    // ── Network (opt-in) ──
    if request.capabilities.network {
        profile.push_str("(allow network-outbound)\n");
    }

    profile
}
