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

pub mod policy;
pub mod backend;
pub mod backends;
pub mod process;
pub mod env_filter;
pub mod patterns;
pub mod capability;
pub mod net_filter;

pub use policy::{ExecPolicy, ExecRequest, Decision, PolicyVerdict, RequiredCapabilities, ToolKind, PolicyError};
pub use backend::{SandboxBackend, BackendError};
pub use process::HardenedProcess;
pub use env_filter::filtered_env;
pub use patterns::is_dangerous;
pub use net_filter::is_private_url;

/// Fail-closed constant — code reviewers must verify this is honored.
pub const BACKEND_FAIL_CLOSED: &str = "Any SandboxBackend::apply() error MUST result in Decision::Deny. Never allow child to spawn un-sandboxed.";
