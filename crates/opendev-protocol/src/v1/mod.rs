//! V1 protocol domain types (frozen at v0.2.0 GA).
//! V1 types are stable — only bugfixes allowed, no new methods/fields.

pub mod session;
pub mod turn;
pub mod tool;
pub mod approval;
pub mod mcp;
pub mod skill;
pub mod config;
pub mod fs;
pub mod workspace;
pub mod error;

// Re-export all v1 types for convenience
pub use session::*;
pub use turn::*;
pub use tool::*;
pub use approval::*;
pub use mcp::*;
pub use skill::*;
pub use config::*;
pub use fs::*;
pub use workspace::*;
pub use error::*;
