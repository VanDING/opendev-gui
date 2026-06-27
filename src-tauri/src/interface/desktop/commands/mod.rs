//! Desktop Commands — DTO mapping only, no business logic.
//!
//! Each command:
//! 1. Receives deserialized DTO (from Tauri invoke)
//! 2. Maps to service input
//! 3. Calls Application Service
//! 4. Returns DTO response

pub mod chat;
pub mod config;
pub mod files;
pub mod mcp;
pub mod session;
pub mod skills;
pub mod workflow;
