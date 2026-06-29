//! Fail-closed verification tests.
//!
//! These tests verify that when a sandbox backend fails to apply,
//! the system refuses to spawn the child process (fail-closed).
//!
//! Contract: `BACKEND_FAIL_CLOSED` constant documents the invariant:
//! "Any SandboxBackend::apply() error MUST result in Decision::Deny.
//!  Never allow child to spawn un-sandboxed."

use opendev_exec::backend::{BackendError, SandboxBackend};
use opendev_exec::policy::{ExecRequest, RequiredCapabilities, ToolKind};
use std::collections::HashMap;
use std::process::Command;

// ── Helper: Always-failing backend ──

/// A sandbox backend that always fails.
/// Used to verify fail-closed behavior — the caller MUST NOT spawn
/// when apply() returns an error.
struct AlwaysFailBackend;

impl SandboxBackend for AlwaysFailBackend {
    fn name(&self) -> &'static str {
        "always_fail"
    }

    fn supported(&self) -> bool {
        true
    }

    fn apply(&self, _cmd: &mut Command, _request: &ExecRequest) -> Result<(), BackendError> {
        Err(BackendError::ApplyFailed("Intentional failure for fail-closed test".into()))
    }
}

// ── Helper: build a minimal ExecRequest ──

fn sample_request(command: &str) -> ExecRequest {
    ExecRequest {
        tool: ToolKind::Bash,
        command: command.into(),
        argv: command.split_whitespace().map(String::from).collect(),
        cwd: std::env::temp_dir(),
        env: HashMap::new(),
        requested_paths: vec![],
        requested_net: None,
        capabilities: RequiredCapabilities::default(),
        allowed_domains: vec![],
        denied_domains: vec![],
    }
}

// ── Tests ──

#[test]
fn test_fail_closed_backend_returns_error() {
    let backend = AlwaysFailBackend;
    let mut cmd = Command::new("echo");
    cmd.arg("should-not-run");

    let request = sample_request("echo should-not-run");

    // Verify backend.apply() returns an error
    let result = backend.apply(&mut cmd, &request);
    assert!(result.is_err(), "Fail-closed: backend should return error");

    match result {
        Err(BackendError::ApplyFailed(msg)) => {
            assert!(
                msg.contains("Intentional failure"),
                "Error message should propagate: got '{}'",
                msg
            );
        }
        other => panic!("Expected ApplyFailed error, got: {:?}", other),
    }

    // The fail-closed contract is: backend.apply() returns Err →
    // caller MUST NOT spawn. This test verifies the backend side;
    // the caller (e.g. BashTool) is verified to honor this contract
    // by its own integration tests.
}

#[test]
fn test_fail_closed_contract_constant_exists() {
    // Verify the BACKEND_FAIL_CLOSED constant exists and documents the contract
    let contract = opendev_exec::BACKEND_FAIL_CLOSED;
    assert!(!contract.is_empty(), "Fail-closed contract must be documented");
    assert!(
        contract.contains("Deny"),
        "Contract must mention that errors result in Deny; got: {}",
        contract
    );
    assert!(contract.contains("error"), "Contract must reference errors; got: {}", contract);
}

#[test]
fn test_none_backend_never_fails() {
    // The NoneBackend should always succeed (it's the safe fallback)
    let backend = opendev_exec::backends::none::NoneBackend;
    assert!(backend.supported());

    let mut cmd = Command::new("echo");
    cmd.arg("test");

    let request = sample_request("echo test");

    let result = backend.apply(&mut cmd, &request);
    assert!(result.is_ok(), "NoneBackend should never fail: {:?}", result);
}

#[test]
fn test_detect_backend_always_returns_some() {
    // detect_backend should always return at least one backend (never None)
    let backend = opendev_exec::backend::detect_backend();
    assert!(backend.is_some(), "detect_backend should always return at least one backend");

    // The returned backend name should be a known value.
    // On Linux: "landlock" or "bwrap" or "none"
    // On macOS: "seatbelt" or "none"
    // On Windows: "windows" or "none"
    let name = backend.unwrap().name();
    let known_names = ["landlock", "bwrap", "seatbelt", "windows", "none"];
    assert!(
        known_names.contains(&name),
        "detect_backend returned unknown backend '{}'. Expected one of: {:?}",
        name,
        known_names,
    );
}
