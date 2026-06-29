//! WebSocket transport for MCP server connections.
//!
//! Provides an [`McpTransport`] implementation that communicates with an
//! MCP server over WebSocket connections with keepalive and reconnection.
//!
//! NOTE: The actual WebSocket connection requires `tokio-tungstenite` (or
//! similar). This implementation provides the structural scaffolding and
//! will need a real WebSocket crate dependency to function. The default
//! fallback is a placeholder that returns a "not yet implemented" error.

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

use crate::error::{McpError, McpResult};
use crate::models::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};
use crate::transport::McpTransport;

/// Default initial reconnect delay (1 second).
const INITIAL_RECONNECT_DELAY: Duration = Duration::from_secs(1);

/// Maximum reconnect delay (30 seconds).
const MAX_RECONNECT_DELAY: Duration = Duration::from_secs(30);

/// WebSocket transport for MCP server communication.
///
/// # TODO
/// Replace the placeholder connection logic with a real WebSocket
/// implementation using `tokio-tungstenite`:
///
/// ```ignore
/// use tokio_tungstenite::connect_async;
/// let (ws_stream, _) = connect_async(&url).await?;
/// let (write, read) = ws_stream.split();
/// ```
#[derive(Debug)]
pub struct McpWebSocketTransport {
    /// WebSocket endpoint URL.
    url: String,
    /// HTTP headers to send during upgrade.
    headers: std::collections::HashMap<String, String>,
    /// Whether the transport is currently connected.
    connected: Arc<Mutex<bool>>,
    /// Channel for sending JSON-RPC requests to the connection task.
    request_tx: Option<tokio::sync::mpsc::UnboundedSender<InternalMessage>>,
    /// Channel for receiving notifications from the server.
    notification_tx: Option<tokio::sync::mpsc::UnboundedSender<JsonRpcNotification>>,
    /// Receiver for notifications (taken by `take_notification_receiver`).
    notification_rx: Option<tokio::sync::mpsc::UnboundedReceiver<JsonRpcNotification>>,
}

/// Internal message for the WebSocket connection task.
#[derive(Debug)]
enum InternalMessage {
    Request {
        id: serde_json::Value,
        method: String,
        params: Option<std::collections::HashMap<String, serde_json::Value>>,
        response_tx: tokio::sync::oneshot::Sender<McpResult<JsonRpcResponse>>,
    },
    Close,
}

impl McpWebSocketTransport {
    /// Create a new WebSocket transport.
    pub fn new(url: String, headers: std::collections::HashMap<String, String>) -> Self {
        let (notification_tx, notification_rx) = tokio::sync::mpsc::unbounded_channel();
        Self {
            url,
            headers,
            connected: Arc::new(Mutex::new(false)),
            request_tx: None,
            notification_tx: Some(notification_tx),
            notification_rx: Some(notification_rx),
        }
    }

    /// Run the WebSocket connection with keepalive pings and reconnection.
    ///
    /// This spawns the background task that manages the WebSocket lifecycle.
    async fn run_connection(
        url: String,
        _headers: std::collections::HashMap<String, String>,
        request_rx: tokio::sync::mpsc::UnboundedReceiver<InternalMessage>,
        notification_tx: tokio::sync::mpsc::UnboundedSender<JsonRpcNotification>,
        connected: Arc<Mutex<bool>>,
    ) {
        let mut request_rx = request_rx;
        let mut reconnect_delay = INITIAL_RECONNECT_DELAY;

        loop {
            // ---------------------------------------------------------------
            // TODO: Replace with real WebSocket connection:
            //
            // use tokio_tungstenite::connect_async;
            // use futures_util::StreamExt;
            //
            // let (ws_stream, _) = connect_async(&url).await?;
            // let (mut write, read) = ws_stream.split();
            //
            // // Start keepalive ping/pong task
            // let ping_handle = tokio::spawn(async move {
            //     loop {
            //         tokio::time::sleep(Duration::from_secs(15)).await;
            //         write.send(Message::Ping(vec![])).await.ok();
            //     }
            // });
            // ---------------------------------------------------------------

            // Placeholder: mark as connected briefly, then break for reconnect
            {
                let mut c = connected.lock().await;
                *c = true;
            }

            // Wait for requests or close signal (simulated with a short sleep
                        // since there's no real WebSocket to poll).
                        tokio::select! {
                            msg = request_rx.recv() => {
                                match msg {
                                    Some(InternalMessage::Request { id: _id, params: _params, response_tx, .. }) => {
                                        let _ = response_tx.send(Err(McpError::Transport(
                                            "WebSocket transport is a placeholder; \
                                             add tokio-tungstenite dependency for real WebSocket support"
                                                .to_string(),
                                        )));
                                    }
                                    Some(InternalMessage::Close) | None => {
                                        // Normal shutdown
                                        let mut c = connected.lock().await;
                                        *c = false;
                                        return;
                                    }
                                }
                            }
                            _ = sleep(Duration::from_millis(100)) => {
                                // Small sleep to yield the task; real impl would poll the WS stream.
                            }
                        }

            // Mark as disconnected
            {
                let mut c = connected.lock().await;
                *c = false;
            }

            // Exponential backoff reconnection
            tokio::select! {
                _ = sleep(reconnect_delay) => {}
                _ = async {
                    // If we receive a Close while waiting to reconnect, exit.
                    while let Ok(msg) = request_rx.try_recv() {
                        if let InternalMessage::Close = msg {
                            return;
                        }
                    }
                } => { return; }
            }

            reconnect_delay = (reconnect_delay * 2).min(MAX_RECONNECT_DELAY);
        }
    }
}

#[async_trait::async_trait]
impl McpTransport for McpWebSocketTransport {
    async fn connect(&mut self) -> McpResult<()> {
        let (request_tx, request_rx) = tokio::sync::mpsc::unbounded_channel();
        self.request_tx = Some(request_tx);

        let url = self.url.clone();
        let headers = self.headers.clone();
        let notification_tx = self
            .notification_tx
            .clone()
            .ok_or_else(|| McpError::Transport("notification channel already taken".to_string()))?;
        let connected = self.connected.clone();

        tokio::spawn(Self::run_connection(url, headers, request_rx, notification_tx, connected));

        Ok(())
    }

    async fn send_request(&self, request: &JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        let tx = self.request_tx.as_ref().ok_or_else(|| {
            McpError::Transport("WebSocket transport not connected".to_string())
        })?;

        let (response_tx, response_rx) = tokio::sync::oneshot::channel();

        tx.send(InternalMessage::Request {
            id: serde_json::json!(request.id),
            method: request.method.clone(),
            params: request.params.clone(),
            response_tx,
        })
        .map_err(|_| McpError::Transport("WebSocket connection task died".to_string()))?;

        response_rx
            .await
            .map_err(|_| McpError::Transport("response channel closed".to_string()))?
    }

    async fn send_notification(&self, _notification: &JsonRpcNotification) -> McpResult<()> {
        Err(McpError::Transport(
            "WebSocket notifications not yet supported".to_string(),
        ))
    }

    async fn close(&self) -> McpResult<()> {
        if let Some(tx) = self.request_tx.as_ref() {
            let _ = tx.send(InternalMessage::Close);
        }
        let mut c = self.connected.lock().await;
        *c = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        // Best-effort check; if we can't acquire the lock, assume disconnected.
        match self.connected.try_lock() {
            Ok(guard) => *guard,
            Err(_) => false,
        }
    }

    fn transport_type(&self) -> &str {
        "websocket"
    }

    async fn take_notification_receiver(
        &mut self,
    ) -> Option<tokio::sync::mpsc::UnboundedReceiver<JsonRpcNotification>> {
        self.notification_rx.take()
    }
}
