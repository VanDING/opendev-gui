use std::sync::Arc;

use opendev_memory::{MemoryCategory, MemoryFacade, MemorySessionContext, MemorySource};
use tokio::sync::Mutex;

use super::super::{Attachment, ContextCollector, TurnContext};
use crate::prompts::reminders::MessageClass;

/// Collector that reads relevant memories from the MemoryFacade into context.
pub struct CabinetMemoryReader {
    facade: Arc<Mutex<MemoryFacade>>,
    interval: usize,
}

impl CabinetMemoryReader {
    pub fn new(facade: Arc<Mutex<MemoryFacade>>, interval: usize) -> Self {
        Self { facade, interval }
    }
}

#[async_trait::async_trait]
impl ContextCollector for CabinetMemoryReader {
    fn name(&self) -> &'static str {
        "cabinet_memory_reader"
    }

    fn should_fire(&self, ctx: &TurnContext<'_>) -> bool {
        ctx.turn_number > 0 && ctx.turn_number % self.interval == 0 && ctx.last_user_query.is_some()
    }

    async fn collect(&self, ctx: &TurnContext<'_>) -> Option<Attachment> {
        let query = ctx.last_user_query?;
        let query = query.to_owned();
        let facade = self.facade.lock().await;

        let memories = facade.search(&query, None, 5).await.ok().unwrap_or_default();

        if memories.is_empty() {
            return None;
        }

        let mut content = String::from("<relevant_memories>\n");
        for m in &memories {
            use std::fmt::Write;
            let _ = writeln!(
                content,
                "- [{}] {} (confidence: {:.2}, importance: {:.2})",
                format!("{:?}", m.entry.category).to_lowercase(),
                m.entry.content,
                m.relevance_score,
                m.entry.importance,
            );
        }
        content.push_str("</relevant_memories>");

        Some(Attachment { name: "cabinet_memory_reader", content, class: MessageClass::Nudge })
    }
}

/// Collector that writes session memories via MemoryFacade.
pub struct CabinetMemoryWriter {
    facade: Arc<Mutex<MemoryFacade>>,
    last_saved_at: std::sync::Mutex<u64>,
}

impl CabinetMemoryWriter {
    pub fn new(facade: Arc<Mutex<MemoryFacade>>) -> Self {
        Self { facade, last_saved_at: std::sync::Mutex::new(0) }
    }
}

#[async_trait::async_trait]
impl ContextCollector for CabinetMemoryWriter {
    fn name(&self) -> &'static str {
        "cabinet_memory_writer"
    }

    fn should_fire(&self, ctx: &TurnContext<'_>) -> bool {
        if let Some(tokens) = ctx.cumulative_input_tokens {
            if tokens == 0 {
                return false;
            }
            let mut last = self.last_saved_at.lock().unwrap_or_else(|e| e.into_inner());
            if tokens.saturating_sub(*last) >= 50_000 {
                *last = tokens;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    async fn collect(&self, ctx: &TurnContext<'_>) -> Option<Attachment> {
        let messages = ctx.recent_messages?;
        let content: Vec<String> = messages
            .iter()
            .filter_map(|m| m.get("content").and_then(|c| c.as_str()))
            .map(|s| s.to_string())
            .collect();

        if content.is_empty() {
            return None;
        }

        let summary = content.join("\n");
        let facade = self.facade.lock().await;

        let _ = facade
            .save(
                &summary,
                MemoryCategory::TechnicalNote,
                MemorySource::Agent,
                None,
                0.5,
                0.7,
                &MemorySessionContext::root(),
            )
            .await;

        Some(Attachment {
            name: "cabinet_memory_writer",
            content: format!("Session memory saved ({} messages processed).", content.len()),
            class: MessageClass::Nudge,
        })
    }
}
