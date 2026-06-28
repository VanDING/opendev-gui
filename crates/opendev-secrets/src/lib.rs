//! Secret/key management for OpenDev.
//!
//! Provides a unified `SecretStore` trait with three backends:
//! 1. `EnvStore` — environment variable override (always wins)
//! 2. `KeyringStore` — OS keyring (macOS Keychain / Linux Secret Service / Windows Credential Manager)
//! 3. `FileStore` — age-encrypted file fallback (for headless/CI)
//!
//! Chain resolve order: env → keyring → file

pub mod audit;
pub mod backends;
pub mod doctor;
pub mod error;
pub mod key;
pub mod migration;
pub mod provider;
pub mod resolver;
pub mod rotate;
pub mod store;
pub mod value;

pub use error::SecretError;
pub use key::{Namespace, SecretKey};
pub use migration::MigrationReport;
pub use provider::SecretProvider;
pub use resolver::ChainedSecretStore;
pub use store::SecretStore;
pub use value::SecretValue;
