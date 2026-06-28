use std::process::Command;
use crate::backend::{SandboxBackend, BackendError};
use crate::policy::ExecRequest;

/// Landlock-backed sandbox for Linux 5.13+.
/// Restricts filesystem access to explicit path allowlists.
pub struct LandlockBackend;

impl SandboxBackend for LandlockBackend {
    fn name(&self) -> &'static str {
        "landlock"
    }

    fn supported(&self) -> bool {
        // Check if Landlock ABI is available (kernel 5.13+)
        landlock::ABI::V1.is_supported()
    }

    fn apply(&self, cmd: &mut Command, request: &ExecRequest) -> Result<(), BackendError> {
        let abi = landlock::ABI::V1;
        if !abi.is_supported() {
            return Err(BackendError::NotSupported(
                "Landlock ABI v1 not supported".into(),
            ));
        }

        let mut ruleset = landlock::Ruleset::new()
            .handle_access(landlock::AccessFs::from_all(abi))
            .map_err(|e| BackendError::ApplyFailed(format!("Landlock ruleset init failed: {}", e)))?
            .create()
            .map_err(|e| BackendError::ApplyFailed(format!("Landlock ruleset create failed: {}", e)))?;

        // Allow access to common system paths
        let common_paths = [
            "/usr",
            "/bin",
            "/lib",
            "/lib64",
            "/etc",
            "/dev",
            "/proc",
            "/sys",
            "/tmp",
            "/var/tmp",
        ];

        for path in &common_paths {
            if let Ok(metadata) = std::fs::metadata(path) {
                if metadata.is_dir() {
                    let _ = ruleset.add(
                        landlock::path_beneath(path, landlock::AccessFs::from_all(abi)),
                    );
                }
            }
        }

        // Allow requested read paths
        for path in &request.capabilities.read {
            let _ = ruleset.add(landlock::path_beneath(
                path,
                landlock::AccessFs::from_read(abi),
            ));
        }

        // Allow requested write paths
        for path in &request.capabilities.write {
            let _ = ruleset.add(landlock::path_beneath(
                path,
                landlock::AccessFs::from_all(abi),
            ));
        }

        // Allow CWD
        let _ = ruleset.add(landlock::path_beneath(
            &request.cwd,
            landlock::AccessFs::from_all(abi),
        ));

        // Restrict the ruleset
        let status = ruleset
            .restrict_self()
            .map_err(|e| BackendError::ApplyFailed(format!("Landlock restrict_self failed: {}", e)))?;

        tracing::debug!(
            backend = "landlock",
            ruleset_version = status.ruleset_version,
            "Landlock ruleset applied"
        );

        Ok(())
    }
}
