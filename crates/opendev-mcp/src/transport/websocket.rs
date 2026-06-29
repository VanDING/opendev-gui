//! WebSocket transport for MCP server connections.
//!
//! Provides an [`McpTransport`] implementation that communicates with an
//! MCP server over WebSocket connections.
//!
//! # WebSocket dependency
//!
//! Full WebSocket support requires `tokio-tungstenite` as a dependency.
//! This implementation provides the structural scaffolding and returns
//! clear errors guiding the user to add the dependency.
//!
//! When `tokio-tungstenite` is available, replace the placeholder
//! `send_request` with:
//!
//! ```ignore
//! use tokio_tungstenite::connect_async;
//! use futures_util::StreamExt;
//!
//! let (ws_stream, _) = connect_async(&url).await?;
//! let (write, read) = ws_stream.split();
//! ```

use crate::error::{McpError, McpResult};
use crate::models::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};
use crate::transport::McpTransport;

/// WebSocket transport for MCP server communication.
///
/// Placeholder implementation that returns clear error messages
/// guiding the developer to add `tokio-tungstenite`.
///
/// When implementing:
/// 1. Connect via WebSocket in `connect()`
/// 2. Keep connection alive with periodic ping/pong
/// 3. On disconnect, attempt reconnection with exponential backoff:
///    1s, 2s, 4s, 8s, max 30s
/// 4. On graceful close, shut down cleanly
#[derive(Debug)]
pub struct McpWebSocketTransport {
    /// WebSocket endpoint URL.
    url: String,
    /// HTTP headers to send during upgrade.
    #[allow(dead_code)]
    headers: std::collections::HashMap<String, String>,
    /// Whether the transport is currently connected.
    connected: std::sync::atomic::AtomicBool,
}

impl McpWebSocketTransport {
    /// Create a new WebSocket transport.
    pub fn new(url: String, headers: std::collections::HashMap<String, String>) -> Self {
        Self {
            url,
            headers,
            connected: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

#[async_trait::async_trait]
impl McpTransport for McpWebSocketTransport {
    async fn connect(&mut self) -> McpResult<()> {
        let _ = &self.url;
        // TODO: Implement real WebSocket connection using tokio-tungstenite:
        //
        //   let (ws_stream, _) = tokio_tungstenite::connect_async(&self.url).await
        //       .map_err(|e| McpError::Transport(format!("WebSocket connect failed: {e}")))?;
        //   self.connected.store(true, Ordering::Relaxed);
        //
        // Then spawn a background task to:
        //   - Read incoming messages and dispatch to request/response matching
        //   - Send periodic ping frames every 15 seconds for keepalive
        //   - On connection loss, attempt reconnection with exponential backoff:
        //     1s, 2s, 4s, 8s, max 30s
        Err(McpError::Transport(
            "WebSocket transport requires `tokio-tungstenite` dependency. \
             Add it to Cargo.toml and implement the connect_async call."
                .to_string(),
        ))
    }

    async fn send_request(&self, request: &JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        let _ = request;
        Err(McpError::Transport(
            "WebSocket transport requires `tokio-tungstenite` dependency. \
             Cannot send requests until the WebSocket connection is implemented."
                .to_string(),
        ))
    }

    async fn send_notification(&self, _notification: &JsonRpcNotification) -> McpResult<()> {
        Err(McpError::Transport(
            "WebSocket transport requires `tokio-tungstenite` dependency.".to_string(),
        ))
    }

    async fn close(&self) -> McpResult<()> {
        self.connected.store(false, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn transport_type(&self) -> &str {
        "websocket"
    }
}
