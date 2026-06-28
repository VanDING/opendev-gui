use crate::events::Event;
use crate::methods::Method;
use crate::version::ProtocolVersion;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Trait for payloads that can be carried in a WireEnvelope.
pub trait Payload: Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static {}
impl<T: Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static> Payload for T {}

/// Unique request/notification identifier — UUID v7 string.
pub type RequestId = String;
/// Participant identifier (client or server node id) — UUID v7 string.
pub type ParticipantId = String;

/// The top-level wire envelope. Field order: v, id/src/dst, kind, payload.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WireEnvelope<P> {
    /// Client → Server RPC request (expects response)
    Request(RequestFrame<P>),
    /// Server → Client RPC response
    Response(ResponseFrame<P>),
    /// Server → Client unsolicited notification (no ack)
    Notification(NotificationFrame<P>),
    /// Error frame (any direction)
    Error(ErrorFrame),
}

/// Client → Server request frame.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct RequestFrame<P> {
    /// Protocol version
    pub v: ProtocolVersion,
    /// Unique request id (UUID v7, time-ordered)
    pub id: RequestId,
    /// Source participant id
    pub src: ParticipantId,
    /// Destination participant id (empty for server)
    pub dst: ParticipantId,
    /// RPC method name (e.g. "session/start")
    pub method: Method,
    /// Method parameters
    pub params: P,
}

/// Server → Client response frame.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct ResponseFrame<P> {
    /// Protocol version
    pub v: ProtocolVersion,
    /// Correlates to the request id
    pub id: RequestId,
    /// Source participant id (server)
    pub src: ParticipantId,
    /// Destination participant id (original request src)
    pub dst: ParticipantId,
    /// Response payload
    pub result: P,
}

/// Server → Client notification (no request id, no ack expected).
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct NotificationFrame<P> {
    /// Protocol version
    pub v: ProtocolVersion,
    /// Monotonic sequence number (for frontend gap detection)
    pub seq: u64,
    /// Source participant id (server)
    pub src: ParticipantId,
    /// Event name (e.g. "message/chunked")
    pub event: Event,
    /// Event payload
    pub data: P,
}

/// Error frame — any direction.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct ErrorFrame {
    /// Protocol version
    pub v: ProtocolVersion,
    /// Correlates to the request id (if applicable)
    pub id: Option<RequestId>,
    /// Source participant id
    pub src: ParticipantId,
    /// Destination participant id
    pub dst: ParticipantId,
    /// Numeric error code
    pub code: i32,
    /// Human-readable error message
    pub message: String,
    /// Optional machine-readable error data
    pub data: Option<serde_json::Value>,
}

/// Helper to generate a new UUID v7 request id.
pub fn new_request_id() -> RequestId {
    Uuid::now_v7().to_string()
}

/// Helper to generate a new participant id.
pub fn new_participant_id() -> ParticipantId {
    Uuid::now_v7().to_string()
}
