//! Chat DTOs.

use serde::{Deserialize, Serialize};

/// Chat query request.
#[derive(Debug, Deserialize)]
pub struct ChatQueryRequest {
    pub message: String,
    #[serde(default)]
    pub session_id: Option<String>,
}

/// Interrupt request.
#[derive(Debug, Deserialize)]
pub struct InterruptRequest {
    #[serde(default)]
    pub session_id: Option<String>,
}

/// Chat action response.
#[derive(Debug, Serialize)]
pub struct ChatActionResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Chat stream event sent via Data Stream.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum ChatStreamEvent {
    #[serde(rename = "chat.message.start")]
    MessageStart { session_id: String },
    #[serde(rename = "chat.message.chunk")]
    MessageChunk { session_id: String, content: String },
    #[serde(rename = "chat.message.completed")]
    MessageCompleted { session_id: String, role: String, content: String },
    #[serde(rename = "chat.tool.executing")]
    ToolExecuting { session_id: String, tool_name: String, tool_id: String },
    #[serde(rename = "chat.tool.completed")]
    ToolCompleted { session_id: String, tool_name: String, tool_id: String, success: bool },
    #[serde(rename = "chat.thinking.block")]
    ThinkingBlock { session_id: String, content: String, block_start: bool },
    #[serde(rename = "chat.approval.required")]
    ApprovalRequired { session_id: String, id: String, tool_name: String, description: String },
}
