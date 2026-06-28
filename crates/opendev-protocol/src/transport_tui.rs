//! In-process transport for TUI mode. Uses tokio mpsc channels.
//! No network I/O — all communication stays within one process.

use crate::envelope::{Payload, RequestId, new_request_id};
use crate::events::Event;
use crate::methods::Method;
use crate::transport::{EventHandle, EventStream, NegotiatedVersion, ProtocolError, Transport};
use crate::version::ProtocolVersion;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::{mpsc, oneshot};

type RequestSender = mpsc::UnboundedSender<(
    RequestId,
    Method,
    serde_json::Value,
    oneshot::Sender<Result<serde_json::Value, ProtocolError>>,
)>;
type EventBroadcaster = Arc<Mutex<HashMap<Event, mpsc::Sender<serde_json::Value>>>>;

/// Server-side handle for TuiInProcessTransport.
/// The server calls these to respond to requests and emit events.
pub struct TuiTransportServer {
    request_rx: mpsc::UnboundedReceiver<(
        RequestId,
        Method,
        serde_json::Value,
        oneshot::Sender<Result<serde_json::Value, ProtocolError>>,
    )>,
    event_broadcasters: EventBroadcaster,
}

impl TuiTransportServer {
    /// Create a new TUI transport pair.
    pub fn new(_buffer: usize) -> (Self, TuiInProcessTransport) {
        let (request_tx, request_rx) = mpsc::unbounded_channel();
        let event_broadcasters = Arc::new(Mutex::new(HashMap::new()));

        let server = Self { request_rx, event_broadcasters: event_broadcasters.clone() };

        let client = TuiInProcessTransport { request_tx, event_broadcasters };

        (server, client)
    }

    /// Receive the next request from the TUI client.
    pub async fn recv_request(
        &mut self,
    ) -> Option<(
        RequestId,
        Method,
        serde_json::Value,
        oneshot::Sender<Result<serde_json::Value, ProtocolError>>,
    )> {
        self.request_rx.recv().await
    }

    /// Broadcast an event to all TUI subscribers of this event type.
    pub async fn emit_event(&self, event: &Event, data: serde_json::Value) {
        let broadcasters = self.event_broadcasters.lock().await;
        if let Some(tx) = broadcasters.get(event) {
            let _ = tx.send(data).await;
        }
    }
}

/// Client-side in-process transport for TUI mode.
pub struct TuiInProcessTransport {
    request_tx: RequestSender,
    event_broadcasters: EventBroadcaster,
}

#[async_trait]
impl Transport for TuiInProcessTransport {
    async fn call<P: Payload, R: Payload>(
        &self,
        method: Method,
        params: P,
    ) -> Result<R, ProtocolError> {
        let id = new_request_id();
        let params_value = serde_json::to_value(&params)
            .map_err(|e| ProtocolError::InvalidParams(e.to_string()))?;

        let (tx, rx) = oneshot::channel();
        self.request_tx
            .send((id, method, params_value, tx))
            .map_err(|e| ProtocolError::Transport(format!("send failed: {}", e)))?;

        let result =
            rx.await.map_err(|e| ProtocolError::Transport(format!("recv failed: {}", e)))??;

        serde_json::from_value(result)
            .map_err(|e| ProtocolError::Internal(format!("deserialize response: {}", e)))
    }

    async fn subscribe(&self, event: Event) -> Result<EventStream, ProtocolError> {
        let (tx, rx) = mpsc::channel(256);
        let handle = EventHandle { event: event.clone(), id: new_request_id() };

        let mut broadcasters = self.event_broadcasters.lock().await;
        broadcasters.insert(event, tx);

        Ok(EventStream { rx, _handle: handle })
    }

    async fn unsubscribe(&self, handle: EventHandle) -> Result<(), ProtocolError> {
        let mut broadcasters = self.event_broadcasters.lock().await;
        broadcasters.remove(&handle.event);
        Ok(())
    }

    async fn negotiate(&self) -> Result<NegotiatedVersion, ProtocolError> {
        Ok(NegotiatedVersion {
            requested: ProtocolVersion::V1_0_0,
            selected: ProtocolVersion::V1_0_0,
            min_supported: ProtocolVersion::V1_0_0,
            max_supported: ProtocolVersion::V1_0_0,
        })
    }
}
