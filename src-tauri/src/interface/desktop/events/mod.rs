//! Desktop Event definitions.
//!
//! All events follow the `domain.object.action` naming convention.

use serde::Serialize;

/// Event category for desktop events.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum DesktopEvent {
    Config(ConfigEvent),
    Session(SessionEvent),
    Chat(ChatEvent),
    Workflow(WorkflowEvent),
    Mcp(McpEvent),
    System(SystemEvent),
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum ConfigEvent {
    #[serde(rename = "config.updated")]
    Updated,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum SessionEvent {
    #[serde(rename = "session.activity")]
    Activity { session_id: String, running: bool },
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum ChatEvent {
    #[serde(rename = "chat.message.start")]
    MessageStart { session_id: String },
    #[serde(rename = "chat.message.chunk")]
    MessageChunk { session_id: String, content: String },
    #[serde(rename = "chat.message.completed")]
    MessageCompleted { session_id: String },
    #[serde(rename = "chat.tool.executing")]
    ToolExecuting { session_id: String, tool_name: String, tool_id: String },
    #[serde(rename = "chat.tool.completed")]
    ToolCompleted { session_id: String, tool_name: String, tool_id: String, success: bool },
    #[serde(rename = "chat.thinking.block")]
    ThinkingBlock { session_id: String, content: String, block_start: bool },
    #[serde(rename = "chat.approval.required")]
    ApprovalRequired { session_id: String, id: String, tool_name: String, description: String },
    #[serde(rename = "chat.message.error")]
    Error { session_id: String, message: String },
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum WorkflowEvent {
    #[serde(rename = "workflow.step.waiting")]
    StepWaiting { session_id: String },
    #[serde(rename = "workflow.step.completed")]
    StepCompleted { session_id: String },
    #[serde(rename = "workflow.plan.ready")]
    PlanReady { session_id: String, content: String },
    #[serde(rename = "workflow.approval.required")]
    ApprovalRequired { session_id: String, request_id: String, questions: serde_json::Value },
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum McpEvent {
    #[serde(rename = "mcp.server.connected")]
    ServerConnected { server_name: String },
    #[serde(rename = "mcp.server.disconnected")]
    ServerDisconnected { server_name: String },
    #[serde(rename = "mcp.servers.updated")]
    ServersUpdated { action: String, server_name: String },
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum SystemEvent {
    #[serde(rename = "system.status")]
    Status { status: String },
}
