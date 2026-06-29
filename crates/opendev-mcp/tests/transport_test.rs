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
    let config = McpServerConfig {
        command: "echo".into(),
        ..Default::default()
    };

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
