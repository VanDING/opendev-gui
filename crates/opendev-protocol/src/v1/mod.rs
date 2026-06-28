//! V1 protocol domain types (frozen at v0.2.0 GA).
//! V1 types are stable — only bugfixes allowed, no new methods/fields.

pub mod approval;
pub mod config;
pub mod error;
pub mod fs;
pub mod mcp;
pub mod session;
pub mod skill;
pub mod tool;
pub mod turn;
pub mod workspace;

// Re-export all v1 types for convenience
pub use approval::*;
pub use config::*;
pub use error::*;
pub use fs::*;
pub use mcp::*;
pub use session::*;
pub use skill::*;
pub use tool::*;
pub use turn::*;
pub use workspace::*;
