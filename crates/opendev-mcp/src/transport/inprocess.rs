//! In-process transport for MCP server communication.
//!
//! Provides an [`McpTransport`] implementation that communicates with an
//! MCP handler running in the same process via `tokio::sync::mpsc` channels.
//! Useful for testing and embedded MCP servers.

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::error::{McpError, McpResult};
use crate::models::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};
use crate::transport::McpTransport;

/// Handler function for processing JSON-RPC requests.
///
/// Takes a method name and optional params (as `HashMap`), returns a JSON-RPC response.
pub type JsonRpcHandler = Arc<
    dyn Fn(&str, Option<std::collections::HashMap<String, serde_json::Value>>) -> McpResult<serde_json::Value>
        + Send
        + Sync,
>;

/// In-process MCP transport using channel-based communication.
///
/// The transport sends JSON-RPC requests through a channel and receives
/// responses from a handler running in the same process. This is useful
/// for testing or when embedding an MCP server directly.
pub struct McpInProcessTransport {
    /// Handler for processing requests.
    handler: JsonRpcHandler,
    /// Whether the transport is connected.
    connected: Arc<Mutex<bool>>,
    /// Channel for sending notifications to consumers.
    notification_tx: Option<tokio::sync::mpsc::UnboundedSender<JsonRpcNotification>>,
    /// Receiver for notifications (taken by `take_notification_receiver`).
    notification_rx: Option<tokio::sync::mpsc::UnboundedReceiver<JsonRpcNotification>>,
}

impl McpInProcessTransport {
    /// Create a new in-process transport with the given handler.
    ///
    /// The handler is called for each JSON-RPC request and should return
    /// the result value (not a full `JsonRpcResponse` — the transport wraps
    /// it with the request's `id`).
    pub fn new<H>(handler: H) -> Self
    where
        H: Fn(&str, Option<std::collections::HashMap<String, serde_json::Value>>) -> McpResult<serde_json::Value> + Send + Sync + 'static,
    {
        let (notification_tx, notification_rx) = tokio::sync::mpsc::unbounded_channel();
        Self {
            handler: Arc::new(handler),
            connected: Arc::new(Mutex::new(false)),
            notification_tx: Some(notification_tx),
            notification_rx: Some(notification_rx),
        }
    }

    /// Create an in-process transport from an existing `Arc<dyn ...>` handler.
    pub fn from_handler(handler: JsonRpcHandler) -> Self {
        let (notification_tx, notification_rx) = tokio::sync::mpsc::unbounded_channel();
        Self {
            handler,
            connected: Arc::new(Mutex::new(false)),
            notification_tx: Some(notification_tx),
            notification_rx: Some(notification_rx),
        }
    }

    /// Send a notification to the consumer (e.g., tool list changed).
    pub fn send_notification(&self, notification: JsonRpcNotification) -> McpResult<()> {
        match &self.notification_tx {
            Some(tx) => tx
                .send(notification)
                .map_err(|_| McpError::Transport("notification receiver dropped".to_string())),
            None => Err(McpError::Transport("notification channel not available".to_string())),
        }
    }
}

#[async_trait::async_trait]
impl McpTransport for McpInProcessTransport {
    async fn connect(&mut self) -> McpResult<()> {
        let mut c = self.connected.lock().await;
        *c = true;
        Ok(())
    }

    async fn send_request(&self, request: &JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        if !*self.connected.lock().await {
            return Err(McpError::Transport("in-process transport not connected".to_string()));
        }

        let result = (self.handler)(&request.method, request.params.clone())?;

        Ok(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: Some(request.id),
            result: Some(result),
            error: None,
        })
    }

    async fn send_notification(&self, _notification: &JsonRpcNotification) -> McpResult<()> {
        // In-process notifications are handled via the dedicated channel.
        // This method sends TO the server; for FROM the server, use
        // `send_notification` on the transport itself.
        Ok(())
    }

    async fn close(&self) -> McpResult<()> {
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
        "in-process"
    }

    async fn take_notification_receiver(
        &mut self,
    ) -> Option<tokio::sync::mpsc::UnboundedReceiver<JsonRpcNotification>> {
        self.notification_rx.take()
    }
}

impl std::fmt::Debug for McpInProcessTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpInProcessTransport")
            .field("handler", &"<closure>")
            .field("connected", &self.connected)
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn test_handler(method: &str, _params: Option<std::collections::HashMap<String, serde_json::Value>>) -> McpResult<serde_json::Value> {
        match method {
            "ping" => Ok(json!({})),
            "echo" => Ok(json!({"echoed": true})),
            "tools/list" => Ok(json!({"tools": []})),
            _ => Err(McpError::Protocol(format!("unknown method: {method}"))),
        }
    }

    #[tokio::test]
    async fn test_connect_and_send_request() {
        let mut transport = McpInProcessTransport::new(test_handler);
        transport.connect().await.unwrap();
        assert!(transport.is_connected());

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "ping".to_string(),
            params: None,
        };

        let response = transport.send_request(&request).await.unwrap();
        assert_eq!(response.id, Some(1));
        assert!(response.error.is_none());
        assert_eq!(response.result, Some(json!({})));
    }

    #[tokio::test]
    async fn test_send_request_not_connected() {
        let transport = McpInProcessTransport::new(test_handler);
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "ping".to_string(),
            params: None,
        };
        let result = transport.send_request(&request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_unknown_method() {
        let mut transport = McpInProcessTransport::new(test_handler);
        transport.connect().await.unwrap();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "unknown".to_string(),
            params: None,
        };

        let result = transport.send_request(&request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_close() {
        let mut transport = McpInProcessTransport::new(test_handler);
        transport.connect().await.unwrap();
        assert!(transport.is_connected());

        transport.close().await.unwrap();
        assert!(!transport.is_connected());
    }

    #[tokio::test]
    async fn test_transport_type() {
        let transport = McpInProcessTransport::new(test_handler);
        assert_eq!(transport.transport_type(), "in-process");
    }

    #[tokio::test]
    async fn test_from_handler() {
        let handler: JsonRpcHandler = Arc::new(
            |method, _params: Option<std::collections::HashMap<String, serde_json::Value>>| match method {
                "ping" => Ok(json!({"pong": true})),
                _ => Err(McpError::Protocol("unknown".to_string())),
            },
        );
        let mut transport = McpInProcessTransport::from_handler(handler);
        transport.connect().await.unwrap();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 42,
            method: "ping".to_string(),
            params: None,
        };

        let response = transport.send_request(&request).await.unwrap();
        assert_eq!(response.result, Some(json!({"pong": true})));
    }
}
