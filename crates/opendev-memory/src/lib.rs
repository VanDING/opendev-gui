pub mod cascade;
pub mod config;
pub mod decay;
pub mod error;
pub mod facade;
pub mod migration;
pub mod provider;
pub mod repo;
pub mod short_term;
pub mod types;
pub mod write_gate;

#[cfg(test)]
mod tests;

pub use cascade::{CascadeBuffer, PendingMemory};
pub use config::MemoryConfig;
pub use decay::MemoryDecay;
pub use error::MemoryError;
pub use facade::{DecayReport, MemoryFacade, WriteTask};
pub use short_term::ShortTermMemory;
pub use types::{
    MemoryCategory, MemoryEntry, MemoryProvider, MemorySessionContext, MemorySource, RecallOptions,
    VerifiedMemory, WriteGateTier,
};
pub use write_gate::WriteGate;
