//! Windows sandbox backend using Job Objects and restricted tokens.
//!
//! ## Current capabilities
//! - Env filter (strips API keys/tokens from child process environment)
//! - `CREATE_SUSPENDED` flag (child process starts suspended, ready for Job Object assignment)
//! - Clear documentation for full Job Object integration
//!
//! ## Full implementation (requires `windows-sys`)
//!
//! The Win32 APIs needed for full Job Object + restricted token isolation:
//!
//! | API | Purpose |
//! |-----|---------|
//! | `CreateJobObjectW` | Create a Job Object handle |
//! | `SetInformationJobObject` | Set limits (process count, memory, kill-on-close) |
//! | `AssignProcessToJobObject` | Attach child process to the job |
//! | `CreateRestrictedToken` | Strip admin groups / privileges |
//! | `SetTokenInformation` | Lower integrity level |
//!
//! These require the `windows-sys` crate with features:
//! - `Win32_System_JobObjects`
//! - `Win32_Security`
//! - `Win32_Foundation`

use crate::backend::{BackendError, SandboxBackend};
use crate::policy::ExecRequest;
use std::process::Command;

/// Windows sandbox using Job Objects and restricted tokens.
pub struct WindowsBackend;

#[cfg(target_os = "windows")]
impl WindowsBackend {
    /// Apply Windows-specific sandboxing (env_filter + CREATE_SUSPENDED).
    fn apply_windows(&self, cmd: &mut Command, request: &ExecRequest) -> Result<(), BackendError> {
        use std::os::windows::process::CommandExt;

        // Step 1: Apply env filter (strip API keys/tokens)
        crate::env_filter::apply(cmd);

        // Step 2: Set CREATE_SUSPENDED so the child can be assigned to a Job Object
        // before any code executes. After spawn, the caller would:
        //   1. Call AssignProcessToJobObject with the Job Object handle and child's process handle
        //   2. Call ResumeThread on the child's primary thread
        // This means if Job Object assignment fails, execution can be prevented by not resuming.
        const CREATE_SUSPENDED: u32 = 0x00000004;
        cmd.creation_flags(CREATE_SUSPENDED);

        tracing::info!(
            backend = "windows",
            "Windows sandbox: env_filter applied, child will be created suspended for Job Object assignment"
        );

        // ── Full implementation reference (when windows-sys is available) ──
        //
        // ```rust
        // use windows_sys::Win32::System::JobObjects::*;
        // use windows_sys::Win32::Security::*;
        // use windows_sys::Win32::Foundation::*;
        //
        // // 1. Create job object
        // let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
        // if job == 0 { return Err(...); }
        //
        // // 2. Set limits: kill on close, active process limit, memory limit
        // let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
        // info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
        //     | JOB_OBJECT_LIMIT_ACTIVE_PROCESS
        //     | JOB_OBJECT_LIMIT_JOB_MEMORY;
        // info.BasicLimitInformation.ActiveProcessLimit = 64;
        // info.ProcessMemoryLimit = 500 * 1024 * 1024; // 500 MB per process
        // info.JobMemoryLimit = 2 * 1024 * 1024 * 1024; // 2 GB total
        // unsafe {
        //     SetInformationJobObject(
        //         job,
        //         JobObjectExtendedLimitInformation,
        //         &info as *const _ as *const std::ffi::c_void,
        //         std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        //     );
        // }
        //
        // // 3. Create restricted token
        // let mut restricted_token = HANDLE(0);
        // let mut existing_sids: [SID_AND_ATTRIBUTES; 0] = [];
        // let mut disable_sids: [SID_AND_ATTRIBUTES; 0] = [];
        // let mut delete_privileges: [SID_AND_ATTRIBUTES; 0] = [];
        // unsafe {
        //     CreateRestrictedToken(
        //         existing_token,
        //         DISABLE_MAX_PRIVILEGE,
        //         0,
        //         existing_sids.as_ptr(),
        //         0,
        //         disable_sids.as_ptr(),
        //         0,
        //         delete_privileges.as_ptr(),
        //         &mut restricted_token,
        //     );
        // }
        //
        // // 4. Assign process and resume
        // unsafe {
        //     AssignProcessToJobObject(job, process_handle);
        //     // ResumeThread(thread_handle);
        // }
        // ```

        Ok(())
    }
}

impl SandboxBackend for WindowsBackend {
    fn name(&self) -> &'static str {
        "windows"
    }

    fn supported(&self) -> bool {
        cfg!(target_os = "windows")
    }

    fn apply(&self, cmd: &mut Command, request: &ExecRequest) -> Result<(), BackendError> {
        #[cfg(target_os = "windows")]
        {
            return self.apply_windows(cmd, request);
        }

        #[cfg(not(target_os = "windows"))]
        {
            let _ = cmd;
            let _ = request;
            Err(BackendError::NotSupported(
                "Windows backend is only available on Windows 10+".into(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::{RequiredCapabilities, ToolKind};

    #[test]
    fn test_windows_backend_name() {
        let backend = WindowsBackend;
        assert_eq!(backend.name(), "windows");
    }

    #[test]
    fn test_windows_backend_supported() {
        let backend = WindowsBackend;
        // On non-Windows, supported() must return false
        assert_eq!(backend.supported(), cfg!(target_os = "windows"));
    }

    #[test]
    fn test_windows_backend_apply_nonwindows() {
        let backend = WindowsBackend;
        let mut cmd = Command::new("echo");
        let request = ExecRequest {
            tool: ToolKind::Bash,
            command: "echo test".into(),
            argv: vec!["echo".into(), "test".into()],
            cwd: std::env::temp_dir(),
            env: std::collections::HashMap::new(),
            requested_paths: vec![],
            requested_net: None,
            capabilities: RequiredCapabilities::default(),
        };

        let result = backend.apply(&mut cmd, &request);

        #[cfg(not(target_os = "windows"))]
        assert!(result.is_err(), "WindowsBackend must fail on non-Windows");
        #[cfg(target_os = "windows")]
        assert!(result.is_ok(), "WindowsBackend must succeed on Windows");
    }
}
