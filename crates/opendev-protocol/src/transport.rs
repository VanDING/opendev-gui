use async_trait::async_trait;
use crate::envelope::{Payload, RequestId};
use crate::methods::Method;
use crate::events::Event;
use crate::version::ProtocolVersion;
use tokio::sync::mpsc;

/// Negotiated protocol version after client-server handshake.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct NegotiatedVersion {
    /// The version the client requested
    pub requested: ProtocolVersion,
    /// The version the server selected (highest compatible)
    pub selected: ProtocolVersion,
    /// Minimum version the server supports
    pub min_supported: ProtocolVersion,
    /// Maximum version the server supports
    pub max_supported: ProtocolVersion,
}

/// Protocol-level errors.
#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("Transport error: {0}")]
    Transport(String),
    #[error("Version mismatch: requested {requested}, server supports {min}..{max}")]
    VersionMismatch {
        requested: ProtocolVersion,
        min: ProtocolVersion,
        max: ProtocolVersion,
    },
    #[error("Method not found: {0}")]
    MethodNotFound(String),
    #[error("Invalid params: {0}")]
    InvalidParams(String),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Timeout")]
    Timeout,
    #[error("Not connected")]
    NotConnected,
}

/// Handle to an active event subscription. Drop to unsubscribe.
#[derive(Debug)]
pub struct EventHandle {
    pub event: Event,
    pub id: RequestId,
}

/// A stream of typed events from a subscription.
pub struct EventStream {
    pub rx: mpsc::Receiver<serde_json::Value>,
    pub _handle: EventHandle,
}

impl EventStream {
    /// Receive the next event, returning None if the stream is closed.
    pub async fn recv(&mut self) -> Option<serde_json::Value> {
        self.rx.recv().await
    }
}

/// The unified Transport trait — implemented by all 5 client types.
#[async_trait]
pub trait Transport: Send + Sync {
    /// Send an RPC request and wait for the response.
    async fn call<P: Payload, R: Payload>(
        &self,
        method: Method,
        params: P,
    ) -> Result<R, ProtocolError>;

    /// Subscribe to an event type. Returns a stream + handle.
    async fn subscribe(&self, event: Event) -> Result<EventStream, ProtocolError>;

    /// Unsubscribe from an event subscription.
    async fn unsubscribe(&self, handle: EventHandle) -> Result<(), ProtocolError>;

    /// Negotiate the protocol version with the server.
    /// Returns the negotiated version info.
    async fn negotiate(&self) -> Result<NegotiatedVersion, ProtocolError>;
}
