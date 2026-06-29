//! Collects recently read/modified files from recent tool calls (every 5 turns).
//!
//! Uses the context's recent messages to identify files the agent has touched.

use crate::attachments::{Attachment, CadenceGate, ContextCollector, TurnContext};

/// Cadence: fire every 5 turns.
const CADENCE: usize = 5;

/// Collects references to recently accessed files from tool call history.
pub struct RecentFilesCollector {
    gate: CadenceGate,
}

impl RecentFilesCollector {
    pub fn new() -> Self {
        Self { gate: CadenceGate::new(CADENCE) }
    }
}

#[async_trait::async_trait]
impl ContextCollector for RecentFilesCollector {
    fn name(&self) -> &'static str {
        "recent_files"
    }

    fn should_fire(&self, ctx: &TurnContext<'_>) -> bool {
        self.gate.should_fire(ctx.turn_number)
    }

    async fn collect(&self, ctx: &TurnContext<'_>) -> Option<Attachment> {
        let messages = ctx.recent_messages?;
        let mut files: Vec<&str> = Vec::new();

        for msg in messages.iter().rev().take(20) {
            if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
                // Look for file paths in tool results
                for line in content.lines() {
                    let lower = line.to_lowercase();
                    if lower.contains("wrote") || lower.contains("edited") || lower.contains("read") {
                        if let Some(path) = line.split_whitespace().find(|w| w.contains('/')) {
                            let clean = path.trim_matches(|c: char| c == '.' || c == '"');
                            if !files.contains(&clean) {
                                files.push(clean);
                            }
                        }
                    }
                }
            }
        }

        if files.is_empty() {
            return None;
        }

        Some(Attachment {
            name: "recent_files",
            content: format!("Recently accessed files:\n{}", files.join("\n")),
            class: crate::prompts::reminders::MessageClass::Nudge,
        })
    }

    fn did_fire(&self, turn: usize) {
        self.gate.mark_fired(turn);
    }
}
