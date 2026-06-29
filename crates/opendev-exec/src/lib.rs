//! Multi-platform sandbox execution layer for OpenDev.
//!
//! Provides:
//! - `ExecPolicy` trait — evaluates whether a command should be allowed
//! - `SandboxBackend` trait — applies OS-level isolation (Landlock/Seatbelt/bwrap/Windows)
//! - `env_filter` — shared environment variable filtering
//! - `net_filter` — SSRF prevention (private URL detection)
//! - `patterns` — dangerous command pattern detection
//! - `capability` — resource limits (rlimit/ulimit)
//!
//! Fail-closed by design: any backend error → child is NOT spawned.

pub mod backend;
pub mod backends;
pub mod bash_ast;
pub mod bash_security;
pub mod capability;
pub mod env_filter;
pub mod net_filter;
pub mod patterns;
pub mod policy;
pub mod process;

pub use backend::{BackendError, SandboxBackend};
pub use env_filter::filtered_env;
pub use net_filter::is_private_url;
pub use patterns::is_dangerous;
pub use policy::{
    Decision, ExecPolicy, ExecRequest, PolicyError, PolicyVerdict, RequiredCapabilities, ToolKind,
};
pub use process::HardenedProcess;

/// Fail-closed constant — code reviewers must verify this is honored.
pub const BACKEND_FAIL_CLOSED: &str = "Any SandboxBackend::apply() error MUST result in Decision::Deny. Never allow child to spawn un-sandboxed.";
