use super::*;
use serde_json::json;

fn make_msg(role: &str, content: &str) -> ApiMessage {
    let mut msg = ApiMessage::new();
    msg.insert("role".into(), json!(role));
    msg.insert("content".into(), json!(content));
    msg
}

fn make_tool_msg(name: &str, content: &str) -> ApiMessage {
    let mut msg = ApiMessage::new();
    msg.insert("role".into(), json!("tool"));
    msg.insert("name".into(), json!(name));
    msg.insert("content".into(), json!(content));
    msg
}

fn make_array_content_msg(role: &str, text: &str) -> ApiMessage {
    let mut msg = ApiMessage::new();
    msg.insert("role".into(), json!(role));
    msg.insert("content".into(), json!([{"type": "text", "text": text}]));
    msg
}

#[test]
fn test_fallback_summary_basic_structure() {
    let messages = vec![
        make_msg("user", "Fix the login bug in auth.rs"),
        make_tool_msg("read_file", "fn login() { /* broken */ }"),
        make_msg("assistant", "I found the issue in the login function"),
    ];
    let summary = ContextCompactor::fallback_summary(&messages);
    assert!(summary.contains("## Goal"));
    assert!(summary.contains("Fix the login bug"));
    assert!(summary.contains("## Key Actions"));
    assert!(summary.contains("read_file:"));
    assert!(summary.contains("## Current State"));
    assert!(summary.contains("I found the issue"));
}

#[test]
fn test_fallback_summary_with_array_content() {
    let messages = vec![
        make_array_content_msg("user", "Refactor the parser"),
        make_msg("assistant", "Working on it"),
    ];
    let summary = ContextCompactor::fallback_summary(&messages);
    assert!(summary.contains("Refactor the parser"));
}

#[test]
fn test_fallback_summary_tool_results_included() {
    let messages = vec![
        make_msg("user", "Read the config"),
        make_tool_msg("read_file", "key = value"),
        make_tool_msg("search", "found 3 matches"),
        make_msg("assistant", "Done analyzing"),
    ];
    let summary = ContextCompactor::fallback_summary(&messages);
    assert!(summary.contains("read_file: key = value"));
    assert!(summary.contains("search: found 3 matches"));
}

#[test]
fn test_fallback_summary_truncation_at_4000_chars() {
    let long_content = "x".repeat(200);
    let mut messages = Vec::new();
    messages.push(make_msg("user", "Do something"));
    for i in 0..50 {
        messages.push(make_tool_msg(&format!("tool_{i}"), &long_content));
    }
    let summary = ContextCompactor::fallback_summary(&messages);
    // Should stop before including all 50 tool results
    let action_count = summary.matches("- tool_").count();
    assert!(action_count < 50);
    assert!(action_count > 0);
}

#[test]
fn test_fallback_summary_empty_messages() {
    let summary = ContextCompactor::fallback_summary(&[]);
    assert!(summary.contains("Unknown"));
    assert!(summary.contains("None recorded"));
    assert!(summary.contains("No assistant response recorded"));
}

#[test]
fn test_fallback_summary_skips_system_messages_for_goal() {
    let messages = vec![
        make_msg("user", "[SYSTEM] You are an AI assistant"),
        make_msg("user", "Help me with X"),
        make_msg("assistant", "Sure"),
    ];
    let summary = ContextCompactor::fallback_summary(&messages);
    assert!(summary.contains("Help me with X"));
    assert!(!summary.contains("[SYSTEM]"));
}

#[test]
fn test_extract_content_string() {
    let msg = make_msg("user", "hello");
    assert_eq!(ContextCompactor::extract_content(&msg), "hello");
}

#[test]
fn test_extract_content_array() {
    let msg = make_array_content_msg("user", "multi-part content");
    assert_eq!(ContextCompactor::extract_content(&msg), "multi-part content");
}

#[test]
fn test_extract_content_missing() {
    let msg = ApiMessage::new();
    assert_eq!(ContextCompactor::extract_content(&msg), "");
}

#[test]
fn test_sanitize_for_summarization_handles_array_content() {
    let messages = vec![
        make_array_content_msg("user", "array content message"),
        make_msg("assistant", "string content message"),
    ];
    let result = ContextCompactor::sanitize_for_summarization(&messages);
    assert!(result.contains("array content message"));
    assert!(result.contains("string content message"));
}

// ── Integration: check_usage returns Compact at 99% ──

#[test]
fn test_check_usage_returns_compact_at_99_percent() {
    use crate::compaction::levels::OptimizationLevel;

    let mut compactor = ContextCompactor::new(10_000);
    let msgs = vec![make_msg("user", "hello"); 100];
    let system = "system prompt";

    // Simulate high token usage.
    compactor.update_from_api_usage(10_000, msgs.len());
    // Add more messages to push over 99%.
    let extra = vec![make_msg("user", "a"); 10];
    let all: Vec<ApiMessage> = msgs.into_iter().chain(extra).collect();

    let level = compactor.check_usage(&all, system);
    assert_eq!(level, OptimizationLevel::Compact);
}

#[test]
fn test_check_usage_returns_warning_at_70_percent() {
    use crate::compaction::levels::OptimizationLevel;

    let mut compactor = ContextCompactor::new(10_000);
    let msgs = vec![make_msg("user", "hello"); 5];
    let system = "system prompt";

    compactor.update_from_api_usage(7_500, msgs.len());
    let level = compactor.check_usage(&msgs, system);
    assert_eq!(level, OptimizationLevel::Warning);
}

#[test]
fn test_check_usage_returns_none_below_70() {
    use crate::compaction::levels::OptimizationLevel;

    let mut compactor = ContextCompactor::new(10_000);
    let msgs = vec![make_msg("user", "hello")];
    let system = "system prompt";

    compactor.update_from_api_usage(5_000, msgs.len());
    let level = compactor.check_usage(&msgs, system);
    assert_eq!(level, OptimizationLevel::None);
}

// ── Integration: build_compaction_payload constructs valid API payload ──

#[test]
fn test_build_compaction_payload_returns_valid_structure() {
    let compactor = ContextCompactor::new(10_000);
    let msgs = vec![
        make_msg("system", "You are an AI assistant"),
        make_msg("user", "Fix the login bug"),
        make_tool_msg("read_file", "fn login() { /* code */ }"),
        make_msg("assistant", "I found the issue"),
        make_msg("user", "Great, can you fix it?"),
        make_msg("assistant", "Here's the fix"),
    ];

    let result = compactor.build_compaction_payload(&msgs, "system prompt", "claude-3.5-sonnet");
    assert!(result.is_some());

    let (payload, middle_count, keep_recent) = result.unwrap();
    assert!(middle_count > 0);
    assert!(keep_recent > 0);
    assert!(keep_recent <= 5);

    // Verify payload structure.
    assert_eq!(payload["model"], "claude-3.5-sonnet");
    assert!(payload["messages"].is_array());
    assert!(payload["messages"].as_array().unwrap().len() >= 2);
    assert_eq!(payload["max_tokens"], 1024);
    assert_eq!(payload["temperature"], 0.2);
}

#[test]
fn test_build_compaction_payload_returns_none_for_few_messages() {
    let compactor = ContextCompactor::new(10_000);
    let msgs = vec![
        make_msg("user", "hi"),
        make_msg("assistant", "hello"),
    ];

    let result = compactor.build_compaction_payload(&msgs, "system", "model");
    assert!(result.is_none());
}

// ── Integration: apply_llm_compaction correctly replaces middle messages ──

#[test]
fn test_apply_llm_compaction_maintains_structure_and_adds_artifact_summary() {
    let mut compactor = ContextCompactor::new(10_000);

    // Add some artifact entries to verify they survive.
    compactor.artifact_index.record("src/main.rs", "edit", "Added main function");
    compactor.artifact_index.record("src/lib.rs", "create", "Created library module");

    let msgs = vec![
        make_msg("system", "You are an AI assistant"),
        make_msg("user", "Refactor the codebase"),
        make_msg("assistant", "I'll start by examining the files"),
        make_tool_msg("read_file", "Current implementation..."),
        make_msg("assistant", "Here's my plan"),
        make_msg("user", "Looks good, proceed"),
        make_tool_msg("edit_file", "Refactored code"),
        make_msg("assistant", "Done with refactoring"),
    ];

    let summary = "User wanted to refactor the codebase. Read files, made edits.";
    let compacted = compactor.apply_llm_compaction(msgs.clone(), summary, 3);

    // Structure: head (system) + summary msg + tail (3 recent)
    assert_eq!(compacted.len(), 1 + 1 + 3);

    // First message preserved (system).
    assert_eq!(
        compacted[0].get("role").and_then(|r| r.as_str()),
        Some("system")
    );

    // Summary message contains both LLM summary and artifact index.
    let summary_content = compacted[1]
        .get("content")
        .and_then(|c| c.as_str())
        .unwrap_or("");
    assert!(summary_content.contains("[CONVERSATION SUMMARY]"));
    assert!(summary_content.contains(summary));
    assert!(summary_content.contains("Artifact Index"));
    assert!(summary_content.contains("src/main.rs"));
    assert!(summary_content.contains("src/lib.rs"));

    // Tail preserved (3 most recent messages).
    assert_eq!(
        compacted[2].get("role").and_then(|r| r.as_str()),
        Some("user")
    );
    assert_eq!(
        compacted[2].get("content").and_then(|c| c.as_str()),
        Some("Looks good, proceed")
    );

    // Calibration state was invalidated.
    assert_eq!(compactor.api_prompt_tokens, 0);
    assert_eq!(compactor.msg_count_at_calibration, 0);
}

#[test]
fn test_apply_llm_compaction_preserves_artifact_index_on_empty() {
    let mut compactor = ContextCompactor::new(10_000);
    // No artifacts recorded, but artifact_index should still be empty.

    let msgs = vec![
        make_msg("system", "system prompt"),
        make_msg("user", "Hello"),
        make_msg("assistant", "Hi there"),
    ];

    let compacted = compactor.apply_llm_compaction(msgs, "Summary text", 2);
    assert_eq!(compacted.len(), 4); // system + summary + 2 tail (keep_recent=2)

    let summary_content = compacted[1]
        .get("content")
        .and_then(|c| c.as_str())
        .unwrap_or("");
    assert!(summary_content.contains("[CONVERSATION SUMMARY]"));
    // No artifact section since none were recorded.
    assert!(!summary_content.contains("Artifact Index"));
}

#[test]
fn test_apply_llm_compaction_invalidates_calibration() {
    let mut compactor = ContextCompactor::new(10_000);
    compactor.api_prompt_tokens = 5_000;
    compactor.msg_count_at_calibration = 10;
    compactor.warned_70 = true;
    compactor.warned_80 = true;
    compactor.warned_90 = true;

    let msgs = vec![
        make_msg("system", "s"),
        make_msg("user", "u1"),
        make_msg("assistant", "a1"),
        make_msg("user", "u2"),
        make_msg("assistant", "a2"),
    ];

    compactor.apply_llm_compaction(msgs, "summary", 3);

    assert_eq!(compactor.api_prompt_tokens, 0);
    assert_eq!(compactor.msg_count_at_calibration, 0);
    assert!(!compactor.warned_70);
    assert!(!compactor.warned_80);
    assert!(!compactor.warned_90);
}
