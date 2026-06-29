use crate::backend::{BackendError, SandboxBackend};
use crate::policy::ExecRequest;
use std::process::Command;

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
        let profile_path = tmp_dir.join(format!("opendev-seatbelt-{}.sb", std::process::id()));
        std::fs::write(&profile_path, &profile).map_err(|e| {
            BackendError::ApplyFailed(format!("Failed to write seatbelt profile: {}", e))
        })?;

        // Wrap the command with sandbox-exec
        let original_program = cmd.get_program().to_os_string();
        let original_args: Vec<_> = cmd.get_args().map(|a| a.to_os_string()).collect();
        let original_envs: Vec<_> =
            cmd.get_envs().map(|(k, v)| (k.to_os_string(), v.map(|v| v.to_os_string()))).collect();
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
    profile.push_str(&format!("  (subpath \"{}\")\n", request.cwd.display()));
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
    profile.push_str(&format!("  (subpath \"{}\")\n", request.cwd.display()));
    profile.push_str(")\n"); // close file-write*

    // ── Network (opt-in with domain policies) ──
    if request.capabilities.network {
        if !request.allowed_domains.is_empty() {
            // Allow only specific remote domains.
            for domain in &request.allowed_domains {
                profile.push_str(&format!("(allow network-outbound (remote \"{}\"))\n", domain));
            }
        } else {
            // Allow all outbound network.
            profile.push_str("(allow network-outbound (remote \"*\"))\n");
        }

        // Deny specific domains if listed (overrides allow for those domains).
        for domain in &request.denied_domains {
            profile.push_str(&format!("(deny network-outbound (remote \"{}\"))\n", domain));
        }
    } else {
        // Network denied: block all network access.
        profile.push_str("(deny network*)\n");
    }

    profile
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::{RequiredCapabilities, ToolKind};
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn make_request(
        network: bool,
        read: Vec<&str>,
        write: Vec<&str>,
        allowed_domains: Vec<&str>,
        denied_domains: Vec<&str>,
    ) -> ExecRequest {
        ExecRequest {
            tool: ToolKind::Bash,
            command: "echo hello".into(),
            argv: vec!["echo".into(), "hello".into()],
            cwd: PathBuf::from("/Users/user/project"),
            env: HashMap::new(),
            requested_paths: vec![],
            requested_net: None,
            capabilities: RequiredCapabilities {
                network,
                read: read.iter().map(|p| PathBuf::from(p)).collect(),
                write: write.iter().map(|p| PathBuf::from(p)).collect(),
                ..Default::default()
            },
            allowed_domains: allowed_domains.iter().map(|s| s.to_string()).collect(),
            denied_domains: denied_domains.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn profile_starts_with_version_and_deny_default() {
        let req = make_request(false, vec![], vec![], vec![], vec![]);
        let profile = build_seatbelt_profile(&req);
        assert!(profile.starts_with("(version 1)\n(deny default)\n"));
    }

    #[test]
    fn profile_allows_process_lifecycle() {
        let req = make_request(false, vec![], vec![], vec![], vec![]);
        let profile = build_seatbelt_profile(&req);
        assert!(profile.contains("(allow process-exec)\n"));
        assert!(profile.contains("(allow process-fork)\n"));
        assert!(profile.contains("(allow signal)\n"));
        assert!(profile.contains("(allow sysctl-read)\n"));
    }

    #[test]
    fn profile_includes_workspace_read_path() {
        let req = make_request(false, vec![], vec![], vec![], vec![]);
        let profile = build_seatbelt_profile(&req);
        assert!(profile.contains("(subpath \"/Users/user/project\")"));
    }

    #[test]
    fn profile_includes_file_read_system_paths() {
        let req = make_request(false, vec![], vec![], vec![], vec![]);
        let profile = build_seatbelt_profile(&req);
        assert!(profile.contains("(subpath \"/usr\")"));
        assert!(profile.contains("(subpath \"/bin\")"));
        assert!(profile.contains("(subpath \"/tmp\")"));
    }

    #[test]
    fn profile_includes_read_capabilities() {
        let req = make_request(false, vec!["/Users/user/project/src"], vec![], vec![], vec![]);
        let profile = build_seatbelt_profile(&req);
        assert!(profile.contains("(subpath \"/Users/user/project/src\")"));
    }

    #[test]
    fn profile_includes_write_capabilities() {
        let req = make_request(false, vec![], vec!["/Users/user/project/output"], vec![], vec![]);
        let profile = build_seatbelt_profile(&req);
        assert!(profile.contains("(subpath \"/Users/user/project/output\")"));
    }

    #[test]
    fn profile_allows_network_outbound_when_network_enabled() {
        let req = make_request(true, vec![], vec![], vec![], vec![]);
        let profile = build_seatbelt_profile(&req);
        assert!(profile.contains("(allow network-outbound (remote \"*\"))"));
        assert!(!profile.contains("(deny network*)"));
    }

    #[test]
    fn profile_denies_network_when_network_disabled() {
        let req = make_request(false, vec![], vec![], vec![], vec![]);
        let profile = build_seatbelt_profile(&req);
        assert!(profile.contains("(deny network*)"));
        assert!(!profile.contains("(allow network-outbound)"));
    }

    #[test]
    fn profile_allows_only_specific_domains() {
        let req = make_request(true, vec![], vec![], vec!["api.example.com"], vec![]);
        let profile = build_seatbelt_profile(&req);
        assert!(profile.contains("(allow network-outbound (remote \"api.example.com\"))"));
        assert!(!profile.contains("(allow network-outbound (remote \"*\"))"));
    }

    #[test]
    fn profile_denies_specific_domains() {
        let req = make_request(
            true, vec![], vec![], vec![],
            vec!["malicious.example.com"],
        );
        let profile = build_seatbelt_profile(&req);
        assert!(profile.contains("(allow network-outbound (remote \"*\"))"));
        assert!(profile.contains("(deny network-outbound (remote \"malicious.example.com\"))"));
    }

    #[test]
    fn profile_combined_allow_and_deny_domains() {
        let req = make_request(
            true, vec![], vec![],
            vec!["good.example.com"],
            vec!["bad.example.com"],
        );
        let profile = build_seatbelt_profile(&req);
        assert!(profile.contains("(allow network-outbound (remote \"good.example.com\"))"));
        assert!(!profile.contains("(allow network-outbound (remote \"*\"))"));
        assert!(profile.contains("(deny network-outbound (remote \"bad.example.com\"))"));
    }

    #[test]
    fn profile_writes_are_gated_to_tmp_and_dev_and_cwd() {
        let req = make_request(false, vec![], vec![], vec![], vec![]);
        let profile = build_seatbelt_profile(&req);
        // Write section should include tmp and dev.
        assert!(profile.contains("(allow file-write*"));
        assert!(profile.contains("(subpath \"/tmp\")"));
        assert!(profile.contains("(subpath \"/dev\")"));
    }

    #[test]
    fn profile_writes_include_workspace_dir() {
        let req = make_request(false, vec![], vec!["/Users/user/project"], vec![], vec![]);
        let profile = build_seatbelt_profile(&req);
        assert!(profile.contains("(subpath \"/Users/user/project\")"));
    }
}
