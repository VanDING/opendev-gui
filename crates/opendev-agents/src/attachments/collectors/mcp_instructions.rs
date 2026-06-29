//! Injects MCP server connectivity status on connect/disconnect events.
//!
//! Fires immediately when connection state changes occur.

use std::sync::atomic::{AtomicBool, Ordering};

use crate::attachments::{Attachment, ContextCollector, TurnContext};
use crate::prompts::reminders::MessageClass;

pub struct McpInstructionsCollector {
    was_connected: AtomicBool,
    has_fired: AtomicBool,
}

impl McpInstructionsCollector {
    pub fn new() -> Self {
        Self { was_connected: AtomicBool::new(false), has_fired: AtomicBool::new(false) }
    }

    /// Notify the collector that MCP servers connected.
    pub fn notify_connected(&self) {
        self.was_connected.store(true, Ordering::Relaxed);
        self.has_fired.store(false, Ordering::Relaxed);
    }

    /// Notify the collector that MCP servers disconnected.
    pub fn notify_disconnected(&self) {
        self.was_connected.store(false, Ordering::Relaxed);
        self.has_fired.store(false, Ordering::Relaxed);
    }
}

#[async_trait::async_trait]
impl ContextCollector for McpInstructionsCollector {
    fn name(&self) -> &'static str {
        "mcp_instructions"
    }

    fn should_fire(&self, _ctx: &TurnContext<'_>) -> bool {
        !self.has_fired.load(Ordering::Relaxed)
    }

    async fn collect(&self, _ctx: &TurnContext<'_>) -> Option<Attachment> {
        let connected = self.was_connected.load(Ordering::Relaxed);
        if connected {
            Some(Attachment {
                name: "mcp_instructions",
                content: "MCP servers are connected and available for use. \
                          Use the MCP tool to interact with connected services."
                    .to_string(),
                class: MessageClass::Directive,
            })
        } else {
            Some(Attachment {
                name: "mcp_instructions",
                content: "MCP servers are not currently connected. \
                          The MCP tool will not be available."
                    .to_string(),
                class: MessageClass::Directive,
            })
        }
    }

    fn did_fire(&self, _turn: usize) {
        self.has_fired.store(true, Ordering::Relaxed);
    }

    fn reset(&self) {
        self.has_fired.store(false, Ordering::Relaxed);
    }
}
