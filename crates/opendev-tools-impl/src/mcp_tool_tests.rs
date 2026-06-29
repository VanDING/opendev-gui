use super::*;

#[test]
fn test_bridge_tool_name_prefixed() {
    let schema = McpToolSchema::new(
        "sqlite__query".to_string(),
        "Run a SQL query".to_string(),
        serde_json::json!({"type": "object", "properties": {"sql": {"type": "string"}}}),
        "sqlite".to_string(),
        "query".to_string(),
    );
    let manager = Arc::new(McpManager::new(None));
    let tool = McpBridgeTool::from_schema(&schema, manager);

    assert_eq!(tool.name(), "mcp__sqlite__query");
    assert_eq!(tool.description(), "Run a SQL query");
}

#[test]
fn test_bridge_tool_schema() {
    let input_schema = serde_json::json!({
        "type": "object",
        "properties": {
            "path": {"type": "string", "description": "File path"}
        },
        "required": ["path"]
    });
    let schema = McpToolSchema::new(
        "fs__read".to_string(),
        "Read a file".to_string(),
        input_schema.clone(),
        "fs".to_string(),
        "read".to_string(),
    );
    let manager = Arc::new(McpManager::new(None));
    let tool = McpBridgeTool::from_schema(&schema, manager);

    assert_eq!(tool.parameter_schema(), input_schema);
}

#[test]
fn test_bridge_tool_fallback_schema() {
    let schema = McpToolSchema::new(
        "test__noop".to_string(),
        "No-op".to_string(),
        serde_json::Value::Null,
        "test".to_string(),
        "noop".to_string(),
    );
    let manager = Arc::new(McpManager::new(None));
    let tool = McpBridgeTool::from_schema(&schema, manager);

    let ps = tool.parameter_schema();
    assert_eq!(ps["type"], "object");
}
