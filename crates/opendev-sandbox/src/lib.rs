//! MicroVM sandbox backend for OpenDev.
//!
//! This crate is feature-gated behind `microsandbox`.
//! Without the feature, it is an empty library.
//!
//! The microVM approach uses `microsandbox` (libkrunfw-backed microVM)
//! for Python code execution. This is NOT the primary sandbox strategy —
//! `opendev-exec` with Landlock/Seatbelt is the primary approach.
//!
//! This crate is retained for experimental/advanced use cases.

#[cfg(feature = "microsandbox")]
pub mod callback;
#[cfg(feature = "microsandbox")]
pub mod errors;
#[cfg(feature = "microsandbox")]
pub mod models;
#[cfg(feature = "microsandbox")]
pub mod parser;
#[cfg(feature = "microsandbox")]
pub mod prompts;
#[cfg(feature = "microsandbox")]
pub mod runtime;
#[cfg(feature = "microsandbox")]
pub mod sandbox;
#[cfg(feature = "microsandbox")]
pub mod session;

#[cfg(feature = "microsandbox")]
pub use errors::{Result, SandboxError};
#[cfg(feature = "microsandbox")]
pub use models::{SandboxContext, SandboxRequest, SandboxResult};
#[cfg(feature = "microsandbox")]
pub use sandbox::{MicroSandbox, SandboxPool};
#[cfg(feature = "microsandbox")]
pub use session::SandboxSession;

/// When built without the microsandbox feature, this crate is empty.
/// Use `opendev-exec` for the primary sandbox (Landlock/Seatbelt/bwrap).
#[cfg(not(feature = "microsandbox"))]
pub const MICROSANDBOX_NOT_ENABLED: &str = "MicroVM sandbox is not enabled. Use `opendev-exec` for primary sandbox isolation. Enable with `--features microsandbox` if needed.";
