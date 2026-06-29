//! Injects list of files changed since last git commit (every 5 turns).

use crate::attachments::{Attachment, CadenceGate, ContextCollector, TurnContext};
use crate::prompts::reminders::MessageClass;

const CADENCE: usize = 5;

pub struct ChangedFilesCollector {
    gate: CadenceGate,
}

impl ChangedFilesCollector {
    pub fn new() -> Self {
        Self { gate: CadenceGate::new(CADENCE) }
    }
}

#[async_trait::async_trait]
impl ContextCollector for ChangedFilesCollector {
    fn name(&self) -> &'static str {
        "changed_files"
    }

    fn should_fire(&self, ctx: &TurnContext<'_>) -> bool {
        self.gate.should_fire(ctx.turn_number)
    }

    async fn collect(&self, _ctx: &TurnContext<'_>) -> Option<Attachment> {
        // Run `git status --porcelain` in the working directory
        let output = tokio::process::Command::new("git")
            .args(["status", "--porcelain"])
            .output()
            .await
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let changed: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();

        if changed.is_empty() {
            return None;
        }

        let changes = changed.join("\n");
        Some(Attachment {
            name: "changed_files",
            content: format!("Files changed since last commit:\n{changes}"),
            class: MessageClass::Nudge,
        })
    }

    fn did_fire(&self, turn: usize) {
        self.gate.mark_fired(turn);
    }

    fn reset(&self) {
        self.gate.reset();
    }
}
