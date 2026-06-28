//! WebSocket transport for the Web client.
//! Uses axum WebSocket with JSONL framing.
//! This is the server-side WebSocket handler; the client side
//! is the TypeScript HttpTransport.

use crate::envelope::{Payload, WireEnvelope, new_request_id};
use crate::events::Event;
use crate::methods::Method;
use crate::transport::{EventHandle, EventStream, NegotiatedVersion, ProtocolError, Transport};
use crate::version::ProtocolVersion;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;

/// Server-side WebSocket transport.
/// Wraps an axum WebSocket connection.
pub struct WebSocketTransport {
    /// Outgoing sender — sends frames to the WebSocket client.
    send_tx: mpsc::UnboundedSender<String>,
    /// Active subscriptions keyed by event name.
    subscriptions: Arc<Mutex<Vec<(Event, mpsc::Sender<serde_json::Value>)>>>,
    /// Negotiated version info.
    negotiated_version: NegotiatedVersion,
}

impl WebSocketTransport {
    /// Build a new WebSocketTransport from an axum WebSocket.
    /// The caller provides a `send` channel for outgoing JSONL frames.
    pub fn new(send_tx: mpsc::UnboundedSender<String>) -> Self {
        Self {
            send_tx,
            subscriptions: Arc::new(Mutex::new(Vec::new())),
            negotiated_version: NegotiatedVersion {
                requested: ProtocolVersion::V1_0_0,
                selected: ProtocolVersion::V1_0_0,
                min_supported: ProtocolVersion::V1_0_0,
                max_supported: ProtocolVersion::V1_0_0,
            },
        }
    }

    /// Send a notification event to the WebSocket client.
    pub fn send_notification(
        &self,
        event: &Event,
        data: serde_json::Value,
    ) -> Result<(), ProtocolError> {
        let frame = WireEnvelope::Notification(crate::envelope::NotificationFrame {
            v: ProtocolVersion::V1_0_0,
            seq: 0, // seq managed by server-side counter
            src: "server".into(),
            event: event.clone(),
            data,
        });

        let json =
            serde_json::to_string(&frame).map_err(|e| ProtocolError::Internal(e.to_string()))?;

        self.send_tx
            .send(json)
            .map_err(|e| ProtocolError::Transport(format!("send failed: {}", e)))?;

        Ok(())
    }
}

#[async_trait]
impl Transport for WebSocketTransport {
    async fn call<P: Payload, R: Payload>(
        &self,
        _method: Method,
        _params: P,
    ) -> Result<R, ProtocolError> {
        // WebSocket request/response is handled by the axum handler loop.
        // This method is for server→client calls (rare).
        Err(ProtocolError::Internal("server-initiated calls not supported via WebSocket".into()))
    }

    async fn subscribe(&self, event: Event) -> Result<EventStream, ProtocolError> {
        let (tx, rx) = mpsc::channel(256);
        let handle = EventHandle { event: event.clone(), id: new_request_id() };

        let mut subs = self.subscriptions.lock().await;
        subs.push((event, tx));

        Ok(EventStream { rx, _handle: handle })
    }

    async fn unsubscribe(&self, handle: EventHandle) -> Result<(), ProtocolError> {
        let mut subs = self.subscriptions.lock().await;
        subs.retain(|(e, _)| e != &handle.event);
        Ok(())
    }

    async fn negotiate(&self) -> Result<NegotiatedVersion, ProtocolError> {
        Ok(self.negotiated_version.clone())
    }
}
