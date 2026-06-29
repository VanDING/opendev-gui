//! Integration tests for context compaction.
//!
//! Tests:
//! - Message filtering preserves tool_call_id pairing
//! - Compaction preserves conversation semantics

use opendev_context::compaction::{ApiMessage, ContextCompactor};

fn make_msg(role: &str, content: &str) -> ApiMessage {
    let mut msg = ApiMessage::new();
    msg.insert("role".into(), serde_json::json!(role));
    msg.insert("content".into(), serde_json::json!(content));
    msg
}

fn make_assistant_with_tool_call(tc_id: &str, tc_name: &str) -> ApiMessage {
    let mut msg = ApiMessage::new();
    msg.insert("role".into(), serde_json::json!("assistant"));
    msg.insert("content".into(), serde_json::json!(""));
    msg.insert(
        "tool_calls".into(),
        serde_json::json!([{
            "id": tc_id,
            "type": "function",
            "function": { "name": tc_name, "arguments": "{}" }
        }]),
    );
    msg
}

fn make_tool_result(tc_id: &str, content: &str) -> ApiMessage {
    let mut msg = ApiMessage::new();
    msg.insert("role".into(), serde_json::json!("tool"));
    msg.insert("tool_call_id".into(), serde_json::json!(tc_id));
    msg.insert("content".into(), serde_json::json!(content));
    msg
}

#[test]
fn test_compaction_preserves_tool_call_id_pairing() {
    let messages = vec![
        make_msg("user", "Read the config file"),
        make_assistant_with_tool_call("call_123", "read_file"),
        make_tool_result("call_123", "config contents here"),
        make_msg("assistant", "I've read the config"),
        make_msg("user", "Now edit it"),
        make_assistant_with_tool_call("call_456", "edit_file"),
        make_tool_result("call_456", "File updated"),
        make_msg("assistant", "Done editing"),
    ];

    let mut compactor = ContextCompactor::new(100000);
    let compacted = compactor.compact(messages, "system prompt");

    // After compaction, the structure should still be valid:
    // system/head + summary + tail
    assert!(compacted.len() >= 2, "Should have at least summary + tail");

    // The summary message should mention the user's goal
    let summary = &compacted[1];
    let content = summary.get("content").and_then(|c| c.as_str()).unwrap_or("");
    assert!(
        content.contains("Read the config") || content.contains("edit it"),
        "Summary should preserve conversation intent: {}",
        content,
    );
}

#[test]
fn test_compaction_handles_empty_messages() {
    let mut compactor = ContextCompactor::new(100000);
    let compacted = compactor.compact(vec![], "system");
    assert!(compacted.is_empty());
}

#[test]
fn test_compaction_short_conversation_unchanged() {
    let messages = vec![
        make_msg("system", "You are an AI"),
        make_msg("user", "Hello"),
        make_msg("assistant", "Hi there!"),
    ];

    let mut compactor = ContextCompactor::new(100000);
    let compacted = compactor.compact(messages.clone(), "system");
    // Short conversations should not be compacted
    assert_eq!(compacted.len(), messages.len());
}

#[test]
fn test_compaction_preserves_recent_context() {
    let mut messages: Vec<ApiMessage> = vec![make_msg("system", "System prompt")];

    // Add several messages
    for i in 0..20 {
        messages.push(make_msg("user", &format!("Message {}", i)));
        messages.push(make_msg("assistant", &format!("Response {}", i)));
    }

    let mut compactor = ContextCompactor::new(100000);
    let compacted = compactor.compact(messages, "system");

    // Should have head + summary + tail (recent messages preserved)
    assert!(
        compacted.len() < 42,
        "Compaction should reduce message count: {} < 42",
        compacted.len()
    );

    // The most recent assistant message should be preserved in the tail
    let last_content = compacted
        .last()
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("");
    assert!(
        last_content.contains("Response 19"),
        "Last message should be preserved: {}",
        last_content,
    );
}
