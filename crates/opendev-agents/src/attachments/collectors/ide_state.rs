//! IDE State collector — collects IDE diagnostics state (placeholder for LSP integration).
//!
//! This collector fires every 10 turns and provides a stub for collecting
//! diagnostics from the IDE's Language Server Protocol integration. Currently
//! returns a placeholder message; real LSP integration will replace this
//! when the IDE bridge is implemented.

use std::sync::atomic::{AtomicUsize, Ordering};

use tracing::debug;

use crate::attachments::{Attachment, CadenceGate, ContextCollector, TurnContext};
use crate::prompts::reminders::MessageClass;

/// Collects IDE diagnostics state for context injection.
///
/// Cadence: fires every 10 turns.
/// Content: current diagnostics, errors, and warnings from the IDE's LSP.
///
/// This is a stub implementation. When the LSP integration bridge is
/// complete, this collector will query the IDE for live diagnostics.
pub struct IdeStateCollector {
    gate: CadenceGate,
    /// Simulated diagnostic count for the stub implementation.
    diag_count: AtomicUsize,
}

impl Default for IdeStateCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl IdeStateCollector {
    /// Create a new IDE state collector with default 10-turn cadence.
    pub fn new() -> Self {
        Self { gate: CadenceGate::new(10), diag_count: AtomicUsize::new(0) }
    }

    /// Create a new IDE state collector with a custom cadence.
    pub fn with_cadence(interval: usize) -> Self {
        Self { gate: CadenceGate::new(interval), diag_count: AtomicUsize::new(0) }
    }
}

#[async_trait::async_trait]
impl ContextCollector for IdeStateCollector {
    fn name(&self) -> &'static str {
        "ide_state"
    }

    fn should_fire(&self, ctx: &TurnContext<'_>) -> bool {
        self.gate.should_fire(ctx.turn_number)
    }

    async fn collect(&self, _ctx: &TurnContext<'_>) -> Option<Attachment> {
        debug!("IdeStateCollector: collecting IDE diagnostics state");

        // Stub: simulate incrementing diagnostics count.
        let count = self.diag_count.fetch_add(1, Ordering::Relaxed) + 1;

        // In real implementation, this would query the LSP bridge for:
        // - Active diagnostics (errors, warnings, hints) per file
        // - File-level problem counts
        // - Project-wide error summary
        //
        // For now, return a stub notification.
        let content = format!(
            "IDE diagnostic state (stub #{}): LSP integration not yet active.\n\
             When enabled, this section will show current errors and warnings.",
            count
        );

        Some(Attachment { name: "ide_state", content, class: MessageClass::Nudge })
    }

    fn did_fire(&self, turn: usize) {
        self.gate.mark_fired(turn);
    }

    fn reset(&self) {
        self.gate.reset();
        self.diag_count.store(0, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn make_turn(n: usize) -> TurnContext<'static> {
        TurnContext {
            turn_number: n,
            working_dir: Path::new("/tmp"),
            todo_manager: None,
            shared_state: None,
            last_user_query: None,
            cumulative_input_tokens: None,
            session_id: None,
            recent_messages: None,
        }
    }

    #[test]
    fn fires_every_10_turns() {
        let collector = IdeStateCollector::new();
        // Should not fire on turn 1-9
        for i in 1..10 {
            assert!(!collector.should_fire(&make_turn(i)), "should not fire at turn {}", i);
        }
        // Should fire on turn 10
        assert!(collector.should_fire(&make_turn(10)));
    }

    #[test]
    fn custom_cadence_respected() {
        let collector = IdeStateCollector::with_cadence(5);
        for i in 1..5 {
            assert!(!collector.should_fire(&make_turn(i)));
        }
        assert!(collector.should_fire(&make_turn(5)));
    }

    #[test]
    fn reset_clears_gate() {
        let collector = IdeStateCollector::with_cadence(5);
        collector.did_fire(5);
        // After firing, should not fire again until 5 more turns
        assert!(!collector.should_fire(&make_turn(6)));
        collector.reset();
        // After reset, gate resets to 0, so it should fire at turn 5 again
        assert!(!collector.should_fire(&make_turn(1))); // not yet, need 5 turns
        assert!(collector.should_fire(&make_turn(5))); // now it fires
    }
}
