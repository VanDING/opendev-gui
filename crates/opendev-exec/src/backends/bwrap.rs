use crate::backend::{BackendError, SandboxBackend};
use crate::policy::ExecRequest;
use std::process::Command;

/// Bubblewrap-backed sandbox for Linux.
/// Creates a new namespace with restricted filesystem access.
pub struct BwrapBackend {
    bwrap_path: String,
}

impl BwrapBackend {
    pub fn new() -> Self {
        Self { bwrap_path: "bwrap".into() }
    }
}

impl SandboxBackend for BwrapBackend {
    fn name(&self) -> &'static str {
        "bwrap"
    }

    fn supported(&self) -> bool {
        // Check if bwrap binary exists
        std::process::Command::new(&self.bwrap_path)
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    fn apply(&self, cmd: &mut Command, request: &ExecRequest) -> Result<(), BackendError> {
        let original_program = cmd.get_program().to_os_string();
        let original_args: Vec<_> = cmd.get_args().map(|a| a.to_os_string()).collect();
        let original_envs: Vec<_> =
            cmd.get_envs().map(|(k, v)| (k.to_os_string(), v.map(|v| v.to_os_string()))).collect();
        let original_dir = cmd.get_current_dir().map(|d| d.to_path_buf());

        // Build bwrap command
        let mut bwrap = Command::new(&self.bwrap_path);

        // Unshare all namespaces
        bwrap.arg("--unshare-all");

        // Mount minimal /proc
        bwrap.arg("--proc").arg("/proc");

        // Mount /dev
        bwrap.arg("--dev").arg("/dev");

        // Bind-mount common system paths as read-only
        let ro_binds = ["/usr", "/bin", "/lib", "/lib64", "/etc/alternatives"];
        for path in &ro_binds {
            if std::path::Path::new(path).exists() {
                bwrap.arg("--ro-bind").arg(path).arg(path);
            }
        }

        // Bind-mount CWD as read-write
        bwrap.arg("--bind").arg(&request.cwd).arg(&request.cwd);

        // Bind-mount tmp
        bwrap.arg("--bind").arg("/tmp").arg("/tmp");

        // Set working directory
        if let Some(dir) = &original_dir {
            bwrap.arg("--chdir").arg(dir);
        }

        // New session
        bwrap.arg("--new-session");

        // Die with parent
        bwrap.arg("--die-with-parent");

        // The command to run
        bwrap.arg("--");
        bwrap.arg(&original_program);
        for arg in &original_args {
            bwrap.arg(arg);
        }

        // Environment
        for (k, v) in &original_envs {
            if let Some(val) = v {
                bwrap.env(k, val);
            }
        }

        *cmd = bwrap;

        tracing::debug!(backend = "bwrap", "wrapped command with bubblewrap");
        Ok(())
    }
}
