//! MCP Transport integration tests.
//!
//! Tests:
//! - Stdio transport initiation
//! - SSE transport creation
//! - HTTP transport creation

use opendev_mcp::config::McpServerConfig;
use opendev_mcp::transport::{McpTransport, create_transport};

// ─── Transport Creation Tests ─────────────────────────────────────────────────

#[test]
fn test_create_stdio_transport() {
    let config = McpServerConfig {
        command: "echo".into(),
        args: vec!["hello".into()],
        transport: opendev_mcp::config::TransportType::Stdio,
        ..Default::default()
    };

    let transport = create_transport(&config).unwrap();
    assert_eq!(transport.transport_type(), "stdio");
}

#[test]
fn test_transport_not_connected_initially() {
    let config = McpServerConfig {
        command: "echo".into(),
        args: vec!["hello".into()],
        transport: opendev_mcp::config::TransportType::Stdio,
        ..Default::default()
    };

    let transport = create_transport(&config).unwrap();
    assert!(!transport.is_connected(), "Transport should not be connected initially");
}

#[test]
fn test_transport_default_config_works() {
    // Test with minimal config (command-only)
    let config = McpServerConfig { command: "echo".into(), ..Default::default() };

    let transport = create_transport(&config).unwrap();
    assert_eq!(transport.transport_type(), "stdio");
}

#[test]
fn test_transport_with_args() {
    let config = McpServerConfig {
        command: "python".into(),
        args: vec!["-m".into(), "mcp_server".into()],
        transport: opendev_mcp::config::TransportType::Stdio,
        ..Default::default()
    };

    let transport = create_transport(&config).unwrap();
    assert!(!config.command.is_empty());
    assert!(!config.args.is_empty());
    assert_eq!(transport.transport_type(), "stdio");
}

// ─── HTTP Transport Tests ────────────────────────────────────────────────────

#[test]
fn test_create_http_transport() {
    let config = McpServerConfig {
        url: Some("https://mcp.example.com/api".into()),
        transport: opendev_mcp::config::TransportType::Http,
        ..Default::default()
    };

    let transport = create_transport(&config).unwrap();
    assert_eq!(transport.transport_type(), "http");
}

#[test]
fn test_http_transport_requires_url() {
    let config = McpServerConfig {
        transport: opendev_mcp::config::TransportType::Http,
        ..Default::default()
    };

    let result = create_transport(&config);
    assert!(result.is_err(), "HTTP transport without URL should fail");
    let err = result.unwrap_err().to_string();
    assert!(err.contains("URL"), "Error should mention missing URL: {err}");
}

#[test]
fn test_http_transport_with_headers() {
    let mut headers = std::collections::HashMap::new();
    headers.insert("Authorization".to_string(), "Bearer test-token".to_string());
    headers.insert("X-Custom".to_string(), "value".to_string());

    let config = McpServerConfig {
        url: Some("https://mcp.example.com/api".into()),
        headers: headers.clone(),
        transport: opendev_mcp::config::TransportType::Http,
        ..Default::default()
    };

    assert_eq!(config.headers.len(), 2);
    assert_eq!(config.headers.get("Authorization").unwrap(), "Bearer test-token");
}

// ─── SSE Transport Tests ─────────────────────────────────────────────────────

#[test]
fn test_create_sse_transport() {
    let config = McpServerConfig {
        url: Some("https://mcp.example.com/sse".into()),
        transport: opendev_mcp::config::TransportType::Sse,
        ..Default::default()
    };

    let transport = create_transport(&config).unwrap();
    assert_eq!(transport.transport_type(), "sse");
}

#[test]
fn test_sse_transport_requires_url() {
    let config = McpServerConfig {
        transport: opendev_mcp::config::TransportType::Sse,
        ..Default::default()
    };

    let result = create_transport(&config);
    assert!(result.is_err(), "SSE transport without URL should fail");
}

// ─── Invalid Config Tests ─────────────────────────────────────────────────────

#[test]
fn test_empty_command_fails() {
    let config = McpServerConfig {
        command: "".into(),
        transport: opendev_mcp::config::TransportType::Stdio,
        ..Default::default()
    };

    let result = create_transport(&config);
    assert!(result.is_err(), "Empty command should fail");
}

#[test]
fn test_npx_without_args_fails() {
    let config = McpServerConfig {
        command: "npx".into(),
        transport: opendev_mcp::config::TransportType::Stdio,
        ..Default::default()
    };

    let result = create_transport(&config);
    assert!(result.is_err(), "npx without args should fail");
}

// ─── Transport Type Detection ─────────────────────────────────────────────────

#[test]
fn test_default_transport_is_stdio() {
    let config = McpServerConfig { command: "echo".into(), ..Default::default() };

    assert_eq!(config.transport, opendev_mcp::config::TransportType::Stdio);
}

#[test]
fn test_transport_url_configuration() {
    let config = McpServerConfig {
        url: Some("https://api.mcp.com".into()),
        transport: opendev_mcp::config::TransportType::Sse,
        ..Default::default()
    };

    assert_eq!(config.url.as_deref(), Some("https://api.mcp.com"));
    assert_eq!(config.transport, opendev_mcp::config::TransportType::Sse);
}
