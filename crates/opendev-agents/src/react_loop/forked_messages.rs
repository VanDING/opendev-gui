//! Forked message construction for subagent prompt cache sharing.
//!
//! When a subagent is spawned with isolation=None and use_forked_cache=true,
//! the ForkedMessageBuilder clones the parent's conversation history, replaces
//! the last assistant's tool_use blocks with a placeholder result, and appends
//! the subagent task. This enables byte-identical prompt cache hits on the
//! shared system prompt + conversation prefix between parent and child agents,
//! reducing token cost and latency for forked subagents.
//!
//! Mirrors Claude Code's `buildForkedMessages()` which shares parent's system
//! prompt + tools + message prefix for cache hits.

use serde_json::Value;

use crate::subagents::spec::SubAgentSpec;

/// Placeholder result inserted in place of tool_use blocks when forking
/// parent messages for a subagent.
pub const FORK_PLACEHOLDER_RESULT: &str = "Fork started";

/// Builds forked messages for a subagent by cloning and transforming the
/// parent agent's conversation history.
///
/// When the subagent meets the forked caching conditions (isolation=None
/// and use_forked_cache=true), the builder:
/// 1. Clones the parent's message list
/// 2. Finds the last assistant message and replaces tool_use blocks with
///    a placeholder result ("Fork started")
/// 3. Appends the subagent task as a user message
///
/// The resulting message list is byte-identical up to the fork point with
/// the parent's message list, enabling prompt cache hits.
pub struct ForkedMessageBuilder;

impl ForkedMessageBuilder {
    /// Check whether forking is applicable for this subagent.
    ///
    /// Forking is applicable when:
    /// - `isolation` is `None` (shares parent's working directory)
    /// - `use_forked_cache` is `true` on the spec
    /// - The spec has tool restrictions (non-empty tools list)
    ///
    /// When `use_forked_cache` is `false` or isolation is `Worktree`,
    /// the subagent should build fresh messages as usual.
    pub fn should_fork(spec: &SubAgentSpec) -> bool {
        spec.use_forked_cache
            && spec.isolation == crate::subagents::spec::IsolationMode::None
            && spec.has_tool_restriction()
    }

    /// Build forked messages from parent conversation history.
    ///
    /// # Arguments
    ///
    /// * `parent_messages` - The parent agent's full conversation history.
    /// * `spec` - The subagent specification (used to check forking conditions).
    /// * `system_prompt` - The assembled system prompt for the subagent.
    /// * `task` - The subagent task description.
    ///
    /// # Returns
    ///
    /// Returns `Some(Vec<Value>)` with forked messages if forking is applicable,
    /// or `None` if conditions aren't met (caller should fall back to fresh messages).
    pub fn build(
        parent_messages: &[Value],
        spec: &SubAgentSpec,
        system_prompt: &str,
        task: &str,
    ) -> Option<Vec<Value>> {
        if !Self::should_fork(spec) {
            return None;
        }

        let mut messages: Vec<Value> = Vec::with_capacity(parent_messages.len() + 1);

        // Clone parent messages, replacing the system prompt
        let mut found_system = false;
        for msg in parent_messages {
            let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("");

            if role == "system" && !found_system {
                // Replace the parent's system prompt with the subagent's
                let mut system_msg = msg.clone();
                system_msg["content"] = Value::String(system_prompt.to_string());
                messages.push(system_msg);
                found_system = true;
            } else {
                messages.push(msg.clone());
            }
        }

        // If no system message was found in parent, prepend one
        if !found_system {
            messages.insert(0, serde_json::json!({"role": "system", "content": system_prompt}));
        }

        // Find the last assistant message and replace tool_use blocks
        if let Some(last_assistant) = messages
            .iter_mut()
            .rev()
            .find(|m| m.get("role").and_then(|r| r.as_str()) == Some("assistant"))
        {
            // Check if this assistant message has tool_calls
            if last_assistant
                .get("tool_calls")
                .and_then(|t| t.as_array())
                .is_some_and(|arr| !arr.is_empty())
            {
                // Replace tool_use blocks with placeholder results
                let tool_calls =
                    last_assistant["tool_calls"].as_array().cloned().unwrap_or_default();

                let mut new_content = String::new();

                // Preserve existing content text if any
                if let Some(text) =
                    last_assistant.get("content").and_then(|c| c.as_str()).filter(|c| !c.is_empty())
                {
                    new_content.push_str(text);
                    new_content.push_str("\n\n");
                }

                // Add placeholder for each tool call
                let tool_names: Vec<String> = tool_calls
                    .iter()
                    .filter_map(|tc| {
                        tc.get("function")
                            .and_then(|f| f.get("name"))
                            .and_then(|n| n.as_str())
                            .map(|n| n.to_string())
                    })
                    .collect();

                if tool_names.is_empty() {
                    new_content.push_str(FORK_PLACEHOLDER_RESULT);
                } else {
                    new_content.push_str(&format!(
                        "{} [tools: {}]",
                        FORK_PLACEHOLDER_RESULT,
                        tool_names.join(", ")
                    ));
                }

                // Replace tool_calls with empty array (prevents API confusion)
                last_assistant["content"] = Value::String(new_content);
                last_assistant["tool_calls"] = Value::Array(vec![]);
            }
        }

        // Append the subagent task as a user message
        messages.push(serde_json::json!({"role": "user", "content": task}));

        Some(messages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::subagents::spec::{IsolationMode, SubAgentSpec};

    fn test_spec(
        use_forked_cache: bool,
        isolation: IsolationMode,
        tools: Vec<String>,
    ) -> SubAgentSpec {
        SubAgentSpec {
            name: "test".to_string(),
            description: "Test agent".to_string(),
            system_prompt: "You are a test agent".to_string(),
            tools,
            model: None,
            max_steps: None,
            hidden: false,
            temperature: None,
            top_p: None,
            mode: crate::subagents::spec::AgentMode::default_mode(),
            max_tokens: None,
            color: None,
            permission: std::collections::HashMap::new(),
            disable: false,
            permission_mode: Default::default(),
            isolation,
            background: false,
            omit_instructions: false,
            use_forked_cache,
        }
    }

    #[test]
    fn test_should_fork_true() {
        let spec =
            test_spec(true, IsolationMode::None, vec!["Read".to_string(), "Grep".to_string()]);
        assert!(ForkedMessageBuilder::should_fork(&spec));
    }

    #[test]
    fn test_should_fork_false_no_cache() {
        let spec =
            test_spec(false, IsolationMode::None, vec!["Read".to_string(), "Grep".to_string()]);
        assert!(!ForkedMessageBuilder::should_fork(&spec));
    }

    #[test]
    fn test_should_fork_false_worktree() {
        let spec =
            test_spec(true, IsolationMode::Worktree, vec!["Read".to_string(), "Grep".to_string()]);
        assert!(!ForkedMessageBuilder::should_fork(&spec));
    }

    #[test]
    fn test_should_fork_false_no_tools() {
        let spec = test_spec(true, IsolationMode::None, vec![]);
        assert!(!ForkedMessageBuilder::should_fork(&spec));
    }

    #[test]
    fn test_build_forked_messages() {
        let parent_messages = vec![
            serde_json::json!({"role": "system", "content": "Parent system prompt"}),
            serde_json::json!({"role": "user", "content": "Hello"}),
            serde_json::json!({
                "role": "assistant",
                "content": "Let me check that.",
                "tool_calls": [{
                    "id": "call_123",
                    "function": {"name": "Read", "arguments": "{}"}
                }]
            }),
            serde_json::json!({"role": "tool", "tool_call_id": "call_123", "content": "file content"}),
        ];

        let spec =
            test_spec(true, IsolationMode::None, vec!["Read".to_string(), "Grep".to_string()]);

        let result = ForkedMessageBuilder::build(
            &parent_messages,
            &spec,
            "You are a test agent (forked)",
            "Do something",
        );

        assert!(result.is_some());
        let messages = result.unwrap();

        // Should have replaced system prompt
        assert_eq!(messages[0]["content"], "You are a test agent (forked)");

        // Should have user message preserved
        assert_eq!(messages[1]["content"], "Hello");

        // Last assistant should have tool_calls replaced
        assert_eq!(messages[2]["role"], "assistant");
        assert!(messages[2]["content"].as_str().unwrap().contains("Fork started"));
        assert_eq!(messages[2]["tool_calls"].as_array().unwrap().len(), 0);

        // Tool result should be preserved
        assert_eq!(messages[3]["role"], "tool");
        assert_eq!(messages[3]["content"], "file content");

        // Should end with subagent task
        assert_eq!(messages[4]["role"], "user");
        assert_eq!(messages[4]["content"], "Do something");

        // Should have 5 messages
        assert_eq!(messages.len(), 5);
    }

    #[test]
    fn test_build_no_assistant_tool_calls() {
        let parent_messages = vec![
            serde_json::json!({"role": "system", "content": "Parent system"}),
            serde_json::json!({"role": "user", "content": "How are you?"}),
            serde_json::json!({"role": "assistant", "content": "I'm fine, thanks!"}),
        ];

        let spec = test_spec(true, IsolationMode::None, vec!["Read".to_string()]);

        let result = ForkedMessageBuilder::build(
            &parent_messages,
            &spec,
            "You are a test agent",
            "Do something",
        );

        assert!(result.is_some());
        let messages = result.unwrap();

        // Assistant with no tool_calls should be preserved
        assert_eq!(messages[2]["content"], "I'm fine, thanks!");

        // Task appended
        assert_eq!(messages[3]["content"], "Do something");
        assert_eq!(messages.len(), 4);
    }

    #[test]
    fn test_build_no_system_message() {
        let parent_messages = vec![serde_json::json!({"role": "user", "content": "Hello"})];

        let spec = test_spec(true, IsolationMode::None, vec!["Read".to_string()]);

        let result = ForkedMessageBuilder::build(
            &parent_messages,
            &spec,
            "You are a test agent",
            "Do something",
        );

        assert!(result.is_some());
        let messages = result.unwrap();

        // Should have prepended system message
        assert_eq!(messages[0]["role"], "system");
        assert_eq!(messages[0]["content"], "You are a test agent");

        // Original user message preserved
        assert_eq!(messages[1]["content"], "Hello");

        // Task appended
        assert_eq!(messages[2]["content"], "Do something");
        assert_eq!(messages.len(), 3);
    }

    #[test]
    fn test_build_returns_none_when_no_fork() {
        let parent_messages = vec![serde_json::json!({"role": "user", "content": "Hello"})];

        let spec = test_spec(false, IsolationMode::None, vec!["Read".to_string()]);

        let result = ForkedMessageBuilder::build(
            &parent_messages,
            &spec,
            "You are a test agent",
            "Do something",
        );

        assert!(result.is_none());
    }
}
