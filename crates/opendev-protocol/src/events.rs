use serde::{Serialize, Deserialize};

/// All v1 event types. Names use `<noun>/<past-tense>` convention.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    /// Assistant message started
    #[serde(rename = "message/started")]
    MessageStarted,
    /// Text chunk streamed
    #[serde(rename = "message/chunked")]
    MessageChunked,
    /// Message completed
    #[serde(rename = "message/completed")]
    MessageCompleted,
    /// Thinking/reasoning block received
    #[serde(rename = "thinking/chunked")]
    ThinkingChunked,
    /// Tool execution started
    #[serde(rename = "tool/started")]
    ToolStarted,
    /// Tool execution completed
    #[serde(rename = "tool/completed")]
    ToolCompleted,
    /// Subagent spawned
    #[serde(rename = "subagent/spawned")]
    SubagentSpawned,
    /// Subagent completed
    #[serde(rename = "subagent/completed")]
    SubagentCompleted,
    /// Nested tool call within subagent
    #[serde(rename = "nested/tool/started")]
    NestedToolStarted,
    /// Nested tool result within subagent
    #[serde(rename = "nested/tool/completed")]
    NestedToolCompleted,
    /// Status update (tokens, cost, etc.)
    #[serde(rename = "status/updated")]
    StatusUpdated,
    /// Progress update
    #[serde(rename = "progress/updated")]
    ProgressUpdated,
    /// Approval required for tool
    #[serde(rename = "approval/required")]
    ApprovalRequired,
    /// Ask-user dialog required
    #[serde(rename = "ask/required")]
    AskRequired,
    /// Plan approval required
    #[serde(rename = "plan/required")]
    PlanRequired,
    /// Session activity state change
    #[serde(rename = "session/activity")]
    SessionActivity,
    /// MCP server connection state changed
    #[serde(rename = "mcp/server/connected")]
    McpServerConnected,
    /// Error event
    #[serde(rename = "error/raised")]
    ErrorRaised,
}

impl Event {
    /// Returns the wire string for this event.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MessageStarted => "message/started",
            Self::MessageChunked => "message/chunked",
            Self::MessageCompleted => "message/completed",
            Self::ThinkingChunked => "thinking/chunked",
            Self::ToolStarted => "tool/started",
            Self::ToolCompleted => "tool/completed",
            Self::SubagentSpawned => "subagent/spawned",
            Self::SubagentCompleted => "subagent/completed",
            Self::NestedToolStarted => "nested/tool/started",
            Self::NestedToolCompleted => "nested/tool/completed",
            Self::StatusUpdated => "status/updated",
            Self::ProgressUpdated => "progress/updated",
            Self::ApprovalRequired => "approval/required",
            Self::AskRequired => "ask/required",
            Self::PlanRequired => "plan/required",
            Self::SessionActivity => "session/activity",
            Self::McpServerConnected => "mcp/server/connected",
            Self::ErrorRaised => "error/raised",
        }
    }
}

impl std::fmt::Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
