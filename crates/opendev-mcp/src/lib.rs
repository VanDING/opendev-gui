//! MCP (Model Context Protocol) client for OpenDev.
//!
//! This crate provides the MCP client implementation for connecting to
//! and communicating with MCP servers. It supports stdio, SSE, HTTP,
//! WebSocket, and in-process transport mechanisms.
//!
//! # Architecture
//!
//! - **config**: Server configuration loading, merging, and env var expansion
//! - **models**: MCP protocol types (tools, resources, prompts, JSON-RPC)
//! - **transport**: Transport trait and implementations (stdio, SSE, HTTP, WS, in-process)
//! - **manager**: McpManager coordinating multiple server connections
//! - **auth**: OAuth 2.0 + PKCE Authorization Code flow
//! - **elicitation**: User input collection for missing parameters

pub mod auth;
pub mod config;
pub mod elicitation;
pub mod error;
pub mod manager;
pub mod models;
pub mod transport;

pub use auth::{detect_auth_flow, generate_pkce_pair, McpAuthFlow, McpTokenCache};
pub use config::{McpConfig, McpOAuthConfig, McpServerConfig, TransportType};
pub use elicitation::{ElicitMode, ElicitationHandler, ElicitRequest, ElicitResult};
pub use error::{McpError, McpResult};
pub use manager::McpManager;
pub use models::{
    JsonRpcNotification, McpContent, McpPromptSummary, McpServerInfo, McpTool, McpToolResult,
    McpToolSchema,
};
pub use transport::McpTransport;
