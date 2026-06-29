//! Skill Listing collector — lists available skills with descriptions.
//!
//! Fires once per session (suppress_after = true) and injects a formatted
//! list of all discovered skills, sourced from the SkillLoader.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use tracing::debug;

use crate::attachments::{Attachment, ContextCollector, TurnContext};
use crate::prompts::reminders::MessageClass;
use crate::skills::SkillLoader;

/// Lists available skills with descriptions for context injection.
///
/// Uses `SkillLoader::build_skills_index()` to generate a formatted listing.
/// Marks `suppress_after: true` so it only fires once at the beginning of
/// the session.
pub struct SkillListingCollector {
    /// The skill loader instance (shared with the agent).
    loader: Mutex<Option<SkillLoader>>,
    /// Whether the collector has already fired this session.
    has_fired: AtomicBool,
}

impl SkillListingCollector {
    /// Create a new skill listing collector.
    ///
    /// The `loader` is consumed and wrapped in a Mutex for interior mutability.
    pub fn new(loader: SkillLoader) -> Self {
        Self { loader: Mutex::new(Some(loader)), has_fired: AtomicBool::new(false) }
    }
}

#[async_trait::async_trait]
impl ContextCollector for SkillListingCollector {
    fn name(&self) -> &'static str {
        "skill_listing"
    }

    fn should_fire(&self, _ctx: &TurnContext<'_>) -> bool {
        // Only fire once per session.
        !self.has_fired.load(Ordering::Relaxed)
    }

    async fn collect(&self, _ctx: &TurnContext<'_>) -> Option<Attachment> {
        debug!("SkillListingCollector: collecting available skills");

        let mut loader_guard = self.loader.lock().ok()?;
        let loader = loader_guard.as_mut()?;

        let index = loader.build_skills_index();
        if index.is_empty() {
            debug!("SkillListingCollector: no skills discovered");
            return None;
        }

        let content = format!(
            "# Available Skills\n\
             \n\
             The following skills are available for use. Invoke them with the \
             `Skill` tool to load skill content into the conversation context.\n\
             \n\
             {index}"
        );

        Some(Attachment { name: "skill_listing", content, class: MessageClass::Directive })
    }

    fn did_fire(&self, _turn: usize) {
        self.has_fired.store(true, Ordering::Relaxed);
    }

    fn reset(&self) {
        // Do NOT reset has_fired — this collector is session-scoped and
        // should only fire once per session even after compaction.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn make_turn() -> TurnContext<'static> {
        TurnContext {
            turn_number: 1,
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
    fn fires_only_once() {
        let loader = SkillLoader::new(vec![]);
        let collector = SkillListingCollector::new(loader);

        // First fire: should fire
        assert!(collector.should_fire(&make_turn()));
        collector.did_fire(1);

        // Subsequent turns: should NOT fire (suppress_after)
        assert!(!collector.should_fire(&make_turn()));
        assert!(!collector.should_fire(&make_turn()));
    }

    #[test]
    fn reset_does_not_reenable() {
        let loader = SkillLoader::new(vec![]);
        let collector = SkillListingCollector::new(loader);

        assert!(collector.should_fire(&make_turn()));
        collector.did_fire(1);
        assert!(!collector.should_fire(&make_turn()));

        // Reset should NOT re-enable (session-scoped).
        collector.reset();
        assert!(!collector.should_fire(&make_turn()));
    }
}
