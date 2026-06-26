#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    #[error("storage error: {0}")]
    Storage(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("background write channel closed")]
    ChannelClosed,

    #[error("provider error: {0}")]
    Provider(String),
}
