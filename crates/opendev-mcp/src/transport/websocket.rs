//! WebSocket transport for MCP server connections.
//!
//! Provides an [`McpTransport`] implementation that communicates with an
//! MCP server over WebSocket connections using `tokio-tungstenite`.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use futures::StreamExt;
use futures::SinkExt;

use crate::error::{McpError, McpResult};
use crate::models::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};
use crate::transport::McpTransport;

/// Default initial reconnect delay (1 second).
const INITIAL_RECONNECT_DELAY: Duration = Duration::from_secs(1);

/// Maximum reconnect delay (30 seconds).
const MAX_RECONNECT_DELAY: Duration = Duration::from_secs(30);

/// Keepalive ping interval (15 seconds).
const PING_INTERVAL: Duration = Duration::from_secs(15);

/// Internal message to the WebSocket task.
enum WsCommand {
    Send {
        payload: String,
        response_tx: tokio::sync::oneshot::Sender<McpResult<String>>,
    },
    Close,
}

/// WebSocket transport for MCP server communication.
pub struct McpWebSocketTransport {
    url: String,
    headers: Arc<HashMap<String, String>>,
    connected: Arc<AtomicBool>,
    cmd_tx: Arc<Mutex<Option<tokio::sync::mpsc::UnboundedSender<WsCommand>>>>,
}

impl std::fmt::Debug for McpWebSocketTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpWebSocketTransport")
            .field("url", &self.url)
            .field("connected", &self.connected)
            .finish()
    }
}

impl McpWebSocketTransport {
    pub fn new(url: String, headers: HashMap<String, String>) -> Self {
        Self {
            url,
            headers: Arc::new(headers),
            connected: Arc::new(AtomicBool::new(false)),
            cmd_tx: Arc::new(Mutex::new(None)),
        }
    }

    async fn ws_loop(
        url: String,
        headers: Arc<HashMap<String, String>>,
        connected: Arc<AtomicBool>,
        mut cmd_rx: tokio::sync::mpsc::UnboundedReceiver<WsCommand>,
    ) {
        let mut reconnect_delay = INITIAL_RECONNECT_DELAY;

        loop {
            // Build request with headers using http crate
            let mut req_builder = http::Request::builder()
                .uri(&url)
                .method("GET");
            for (key, value) in headers.iter() {
                if let (Ok(name), Ok(val)) = (
                    http::HeaderName::from_bytes(key.as_bytes()),
                    http::HeaderValue::from_str(value),
                ) {
                    req_builder = req_builder.header(name, val);
                }
            }
            let request = match req_builder.body(()) {
                Ok(r) => r,
                Err(_) => {
                    sleep(reconnect_delay).await;
                    reconnect_delay = (reconnect_delay * 2).min(MAX_RECONNECT_DELAY);
                    continue;
                }
            };

            let ws = match connect_async(request).await {
                Ok((ws, _)) => ws,
                Err(e) => {
                    tracing::warn!(error = %e, "WS connect failed, retrying");
                    sleep(reconnect_delay).await;
                    reconnect_delay = (reconnect_delay * 2).min(MAX_RECONNECT_DELAY);
                    continue;
                }
            };

            reconnect_delay = INITIAL_RECONNECT_DELAY;
            connected.store(true, Ordering::Relaxed);
            tracing::info!("WebSocket connected to {url}");

            let (mut write, mut read) = ws.split();
            let mut ping_interval = tokio::time::interval(PING_INTERVAL);
            ping_interval.tick().await;

            // Pending response senders (id -> sender)
            let mut pending: HashMap<u64, tokio::sync::oneshot::Sender<McpResult<String>>> =
                HashMap::new();

            let result = loop {
                tokio::select! {
                    // Read from WebSocket
                    msg = read.next() => {
                        match msg {
                            Some(Ok(Message::Text(text))) => {
                                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                                    if let Some(id) = val.get("id").and_then(|v| v.as_u64()) {
                                        if let Some(tx) = pending.remove(&id) {
                                            let _ = tx.send(Ok(text));
                                        }
                                    }
                                }
                            }
                            Some(Ok(Message::Close(_))) | None => break Err("Connection closed".to_string()),
                            Some(Err(e)) => break Err(format!("Read error: {e}")),
                            _ => {}
                        }
                    }

                    // Send ping
                    _ = ping_interval.tick() => {
                        if let Err(e) = write.send(Message::Ping(vec![])).await {
                            break Err(format!("Ping failed: {e}"));
                        }
                    }

                    // Receive commands
                    cmd = cmd_rx.recv() => {
                        match cmd {
                            Some(WsCommand::Send { payload, response_tx }) => {
                                // Parse id from payload
                                let id = serde_json::from_str::<serde_json::Value>(&payload)
                                    .ok()
                                    .and_then(|v| v.get("id").and_then(|i| i.as_u64()))
                                    .unwrap_or(0);

                                pending.insert(id, response_tx);

                                if let Err(e) = write.send(Message::Text(payload.into())).await {
                                    pending.remove(&id);
                                    break Err(format!("Send failed: {e}"));
                                }
                            }
                            Some(WsCommand::Close) | None => {
                                break Ok(());
                            }
                        }
                    }
                }
            };

            connected.store(false, Ordering::Relaxed);

            // Fail all pending requests
            for (_, tx) in pending.drain() {
                let _ = tx.send(Err(McpError::Transport(
                    "WebSocket disconnected".to_string(),
                )));
            }

            match result {
                Ok(()) => {
                    tracing::info!("WebSocket connection closed gracefully");
                    return;
                }
                Err(e) => {
                    tracing::warn!(error = %e, "WebSocket error, reconnecting");
                    sleep(reconnect_delay).await;
                    reconnect_delay = (reconnect_delay * 2).min(MAX_RECONNECT_DELAY);
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl McpTransport for McpWebSocketTransport {
    async fn connect(&mut self) -> McpResult<()> {
        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
        *self.cmd_tx.lock().await = Some(cmd_tx);

        tokio::spawn(Self::ws_loop(
            self.url.clone(),
            self.headers.clone(),
            self.connected.clone(),
            cmd_rx,
        ));

        Ok(())
    }

    async fn send_request(&self, request: &JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": request.id,
            "method": &request.method,
            "params": &request.params,
        });
        let payload_str = serde_json::to_string(&payload)
            .map_err(|e| McpError::Transport(format!("Serialize error: {e}")))?;

        let (tx, rx) = tokio::sync::oneshot::channel();

        let cmd_tx = self.cmd_tx.lock().await;
        let tx_sender = cmd_tx.as_ref().ok_or_else(|| {
            McpError::Transport("WebSocket not connected".to_string())
        })?;

        tx_sender
            .send(WsCommand::Send { payload: payload_str, response_tx: tx })
            .map_err(|_| McpError::Transport("WebSocket task died".to_string()))?;
        drop(cmd_tx);

        let result = rx.await
            .map_err(|_| McpError::Transport("WebSocket response channel closed".to_string()))?;
        // result is McpResult<String> from the Oneshot
        let response_str: String = result?;

        serde_json::from_str(&response_str)
            .map_err(|e| McpError::Transport(format!("Failed to parse response: {e}")))
    }

    async fn send_notification(&self, _notification: &JsonRpcNotification) -> McpResult<()> {
        Err(McpError::Transport(
            "WebSocket notifications not yet supported".to_string(),
        ))
    }

    async fn close(&self) -> McpResult<()> {
        self.connected.store(false, Ordering::Relaxed);
        let mut guard = self.cmd_tx.lock().await;
        if let Some(tx) = guard.take() {
            let _ = tx.send(WsCommand::Close);
        }
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }

    fn transport_type(&self) -> &str {
        "websocket"
    }
}
