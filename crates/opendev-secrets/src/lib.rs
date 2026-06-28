//! Secret/key management for OpenDev.
//! 
//! Provides a unified `SecretStore` trait with three backends:
//! 1. `EnvStore` — environment variable override (always wins)
//! 2. `KeyringStore` — OS keyring (macOS Keychain / Linux Secret Service / Windows Credential Manager)
//! 3. `FileStore` — age-encrypted file fallback (for headless/CI)
//! 
//! Chain resolve order: env → keyring → file

pub mod error;
pub mod key;
pub mod value;
pub mod provider;
pub mod store;
pub mod backends;
pub mod resolver;
pub mod audit;
pub mod doctor;
pub mod migration;
pub mod rotate;

pub use error::SecretError;
pub use key::{SecretKey, Namespace};
pub use value::SecretValue;
pub use provider::SecretProvider;
pub use store::SecretStore;
pub use resolver::ChainedSecretStore;
pub use migration::MigrationReport;
