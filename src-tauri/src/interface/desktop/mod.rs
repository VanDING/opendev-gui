//! Desktop Interface Layer.
//!
//! This module implements the Desktop Port Adapter.
//! It depends on Tauri for IPC but all business logic is delegated
//! to Application Services.

pub mod contract;
pub mod commands;
pub mod events;
pub mod platform;

pub use platform::{DesktopPlatform, TauriPlatform, StreamSender, StreamReceiver};
