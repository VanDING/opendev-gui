use thiserror::Error;

/// Errors from secret store operations.
#[derive(Error, Debug)]
pub enum SecretError {
    #[error("Backend error: {0}")]
    Backend(String),
    
    #[error("Entry not found: {0}")]
    NotFound(String),
    
    #[error("Access denied: {0}")]
    AccessDenied(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Keyring error: {0}")]
    Keyring(#[from] keyring::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
