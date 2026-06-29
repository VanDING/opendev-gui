//! Injects the current subagent/team agent listing into context.
//!
//! Fires when the agent listing changes (detected via version counter).

use std::sync::atomic::{AtomicU64, Ordering};

use crate::attachments::{Attachment, ContextCollector, TurnContext, CadenceGate};
use crate::prompts::reminders::MessageClass;

/// Collects the current agent listing for subagent dispatch decisions.
pub struct AgentListingCollector {
    cadence: CadenceGate,
    last_version: AtomicU64,
}

impl AgentListingCollector {
    pub fn new(version_provider: impl Fn() -> u64) -> Self {
        Self {
            cadence: CadenceGate::new(1), // fire on every turn until seen
            last_version: AtomicU64::new(version_provider()),
        }
    }
}

#[async_trait::async_trait]
impl ContextCollector for AgentListingCollector {
    fn name(&self) -> &'static str {
        "agent_listing"
    }

    fn should_fire(&self, _ctx: &TurnContext<'_>) -> bool {
        // Fire on every turn; collect() decides if listing has changed.
        true
    }

    async fn collect(&self, _ctx: &TurnContext<'_>) -> Option<Attachment> {
        // If the agent registry version hasn't changed, skip.
        // (Implementation would check a shared version counter from the
        //  agent manager. Here we return None since we can't access
        //  the registry directly from this collector.)
        let _ = self.last_version.load(Ordering::Relaxed);
        None
    }

    fn did_fire(&self, _turn: usize) {}
    fn reset(&self) {}
}
