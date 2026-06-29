//! Auto-Approval Classifier.
//!
//! Heuristic-based classifier that decides whether a tool invocation should
//! be automatically approved without user intervention. Uses simple rules
//! and tracks denial counters to fall back to manual approval when the
//! heuristic is consistently rejecting requests.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Decision returned by the approval classifier.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApprovalDecision {
    /// Whether the tool should be auto-approved.
    pub approved: bool,
    /// Human-readable reason for the decision (if any).
    pub reason: Option<String>,
}

impl ApprovalDecision {
    /// Convenience constructor for approval.
    pub fn approved(reason: Option<String>) -> Self {
        Self { approved: true, reason }
    }

    /// Convenience constructor for denial.
    pub fn denied(reason: impl Into<String>) -> Self {
        Self { approved: false, reason: Some(reason.into()) }
    }
}

/// Tools that are always safe to auto-approve.
pub const SAFE_ALLOWLISTED_TOOLS: &[&str] = &[
    "Read",
    "Grep",
    "Glob",
    "WebFetch",
    "WebSearch",
    "TaskList",
];

/// Heuristic-based approval classifier with denial tracking.
///
/// Tracks consecutive and total denials. When thresholds are exceeded,
/// falls back to requiring manual approval for everything.
pub struct ApprovalClassifier {
    /// Number of consecutive denials (resets on approval).
    pub denial_count: u32,
    /// Total cumulative denials.
    pub total_denial_count: u32,
    /// Maximum consecutive denials before fallback.
    max_consecutive_denials: u32,
    /// Maximum total denials before fallback.
    max_total_denials: u32,
    /// Whether the classifier is in fallback mode (manual approval for all).
    pub fallback_mode: bool,
    /// Workspace root path for write-path inspection.
    workspace_root: Option<String>,
}

impl Default for ApprovalClassifier {
    fn default() -> Self {
        Self {
            denial_count: 0,
            total_denial_count: 0,
            max_consecutive_denials: 3,
            max_total_denials: 20,
            fallback_mode: false,
            workspace_root: None,
        }
    }
}

impl ApprovalClassifier {
    /// Create a new classifier with default thresholds.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a classifier with a known workspace root.
    pub fn with_workspace(root: impl Into<String>) -> Self {
        Self {
            workspace_root: Some(root.into()),
            ..Default::default()
        }
    }

    /// Configure the denial thresholds.
    pub fn with_thresholds(mut self, consecutive: u32, total: u32) -> Self {
        self.max_consecutive_denials = consecutive;
        self.max_total_denials = total;
        self
    }

    /// Reset denial counters.
    pub fn reset_counters(&mut self) {
        self.denial_count = 0;
        self.total_denial_count = 0;
        self.fallback_mode = false;
    }

    /// Record a denial and check if fallback should be triggered.
    fn record_denial(&mut self) {
        self.denial_count += 1;
        self.total_denial_count += 1;

        if self.denial_count >= self.max_consecutive_denials
            || self.total_denial_count >= self.max_total_denials
        {
            self.fallback_mode = true;
        }
    }

    /// Record an approval (resets consecutive denial counter).
    fn record_approval(&mut self) {
        self.denial_count = 0;
    }

    /// Evaluate whether a tool invocation should be auto-approved.
    ///
    /// Heuristics (in order):
    /// 1. Read-only tools (Read, Grep, Glob, WebFetch, WebSearch) → auto-approve
    /// 2. Edit with small diff (< 50 lines) → auto-approve
    /// 3. Bash on read-only allowlist → auto-approve
    /// 4. Write within workspace + not sensitive file → auto-approve
    /// 5. Everything else → requires manual approval
    /// 6. If fallback_mode is active → requires manual approval
    pub fn should_auto_approve(
        &mut self,
        tool_name: &str,
        args: &HashMap<String, serde_json::Value>,
    ) -> ApprovalDecision {
        // If in fallback mode, everything requires manual approval.
        if self.fallback_mode {
            let reason = format!(
                "Fallback mode: {} consecutive or {} total denials triggered manual-only policy",
                self.denial_count, self.total_denial_count,
            );
            return ApprovalDecision::denied(reason);
        }

        // Rule 1: Read-only tools are always safe.
        if SAFE_ALLOWLISTED_TOOLS.contains(&tool_name) {
            self.record_approval();
            return ApprovalDecision::approved(Some(format!(
                "{} is in the safe allowlist",
                tool_name
            )));
        }

        // Dispatch based on tool name.
        match tool_name {
            "Edit" => self.evaluate_edit(args),
            "Write" => self.evaluate_write(args),
            "Bash" => self.evaluate_bash(args),
            _ => self.evaluate_other(tool_name, args),
        }
    }

    /// Evaluate an Edit tool invocation.
    fn evaluate_edit(&mut self, args: &HashMap<String, serde_json::Value>) -> ApprovalDecision {
        // If there's a diff/old_string field, estimate changes.
        if let Some(old_str) = args.get("old_string").and_then(|v| v.as_str()) {
            let line_count = old_str.lines().count() as u32;
            // Small edits: auto-approve.
            if line_count < 50 {
                self.record_approval();
                return ApprovalDecision::approved(Some(format!(
                    "Edit changes {} lines (< 50): small change, auto-approved",
                    line_count
                )));
            }
        }

        // No old_string or large diff — require approval.
        self.record_denial();
        ApprovalDecision::denied("Edit changes more than 50 lines; requires manual review")
    }

    /// Evaluate a Write tool invocation.
    fn evaluate_write(&mut self, args: &HashMap<String, serde_json::Value>) -> ApprovalDecision {
        // Check if the write path is within workspace.
        let file_path = args.get("file_path").or_else(|| args.get("path")).or_else(|| {
            // Some write tools put the path in "file" key.
            args.get("file")
        });

        match file_path.and_then(|v| v.as_str()) {
            Some(path) => {
                // Check if writing to a sensitive file.
                if crate::permissions::is_sensitive_file(path) {
                    self.record_denial();
                    return ApprovalDecision::denied(format!(
                        "Write to '{}' targets a sensitive file; requires manual approval",
                        path
                    ));
                }

                // Check if path is within workspace.
                if let Some(root) = self.workspace_root.clone() {
                    let abs_path = if path.starts_with('/') {
                        path.to_string()
                    } else {
                        format!("{}/{}", root.trim_end_matches('/'), path)
                    };

                    if abs_path.starts_with(root.trim_end_matches('/')) {
                        self.record_approval();
                        return ApprovalDecision::approved(Some(format!(
                            "Write to '{}' is within workspace '{}'",
                            path, root
                        )));
                    }
                }

                // No workspace context, but path is not sensitive — allow.
                self.record_approval();
                ApprovalDecision::approved(Some(format!(
                    "Write to '{}' is not a sensitive file",
                    path
                )))
            }
            None => {
                // No file path specified — treat as unknown write.
                self.record_denial();
                ApprovalDecision::denied("Write tool has no target path; requires manual review")
            }
        }
    }

    /// Evaluate a Bash tool invocation.
    fn evaluate_bash(&mut self, args: &HashMap<String, serde_json::Value>) -> ApprovalDecision {
        // Extract the command string.
        let command = args
            .get("command")
            .or_else(|| args.get("cmd"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if command.trim().is_empty() {
            self.record_denial();
            return ApprovalDecision::denied("Bash command is empty; requires manual review");
        }

        // Check if this is a safe/read-only command using the existing safe list.
        if crate::constants::is_safe_command(command) {
            self.record_approval();
            return ApprovalDecision::approved(Some(format!(
                "Bash '{}' is in the safe command allowlist",
                command
            )));
        }

        // Not in safe allowlist — require approval.
        self.record_denial();
        ApprovalDecision::denied(format!(
            "Bash '{}' is not in the safe command allowlist; requires manual review",
            command
        ))
    }

    /// Evaluate any other tool invocation (fallback).
    fn evaluate_other(
        &mut self,
        tool_name: &str,
        _args: &HashMap<String, serde_json::Value>,
    ) -> ApprovalDecision {
        // Unknown tool — require manual approval.
        self.record_denial();
        ApprovalDecision::denied(format!(
            "Tool '{}' is not auto-approvable; requires manual approval",
            tool_name
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_args(pairs: &[(&str, &str)]) -> HashMap<String, serde_json::Value> {
        let mut map = HashMap::new();
        for (k, v) in pairs {
            map.insert(k.to_string(), json!(v));
        }
        map
    }

    // ── Read-only tools ──

    #[test]
    fn read_tool_is_auto_approved() {
        let mut clf = ApprovalClassifier::new();
        let args = make_args(&[("file_path", "src/main.rs")]);
        let decision = clf.should_auto_approve("Read", &args);
        assert!(decision.approved);
        assert!(decision.reason.unwrap().contains("safe allowlist"));
    }

    #[test]
    fn grep_tool_is_auto_approved() {
        let mut clf = ApprovalClassifier::new();
        let args = make_args(&[("pattern", "TODO")]);
        let decision = clf.should_auto_approve("Grep", &args);
        assert!(decision.approved);
    }

    #[test]
    fn glob_tool_is_auto_approved() {
        let mut clf = ApprovalClassifier::new();
        let decision = clf.should_auto_approve("Glob", &HashMap::new());
        assert!(decision.approved);
    }

    // ── Edit tool ──

    #[test]
    fn small_edit_is_auto_approved() {
        let mut clf = ApprovalClassifier::new();
        let small_diff = "line1\nline2\n"; // 2 lines
        let args = make_args(&[("file_path", "src/main.rs"), ("old_string", small_diff)]);
        let decision = clf.should_auto_approve("Edit", &args);
        assert!(decision.approved);
        assert!(decision.reason.unwrap().contains("< 50"));
    }

    #[test]
    fn large_edit_requires_approval() {
        let mut clf = ApprovalClassifier::new();
        let large_diff = (0..100).map(|i| format!("line_{}", i)).collect::<Vec<_>>().join("\n");
        let args = make_args(&[("file_path", "src/main.rs"), ("old_string", &large_diff)]);
        let decision = clf.should_auto_approve("Edit", &args);
        assert!(!decision.approved);
        assert!(decision.reason.unwrap().contains("50 lines"));
    }

    // ── Bash tool ──

    #[test]
    fn safe_bash_command_is_auto_approved() {
        let mut clf = ApprovalClassifier::new();
        let args = make_args(&[("command", "ls -la")]);
        let decision = clf.should_auto_approve("Bash", &args);
        assert!(decision.approved);
    }

    #[test]
    fn dangerous_bash_requires_approval() {
        let mut clf = ApprovalClassifier::new();
        let args = make_args(&[("command", "rm -rf /")]);
        let decision = clf.should_auto_approve("Bash", &args);
        assert!(!decision.approved);
    }

    #[test]
    fn empty_bash_is_denied() {
        let mut clf = ApprovalClassifier::new();
        let args = make_args(&[("command", "")]);
        let decision = clf.should_auto_approve("Bash", &args);
        assert!(!decision.approved);
    }

    // ── Write tool ──

    #[test]
    fn write_within_workspace_is_approved() {
        let mut clf = ApprovalClassifier::with_workspace("/home/user/project");
        let args = make_args(&[("file_path", "/home/user/project/src/main.rs")]);
        let decision = clf.should_auto_approve("Write", &args);
        assert!(decision.approved);
        assert!(decision.reason.unwrap().contains("within workspace"));
    }

    #[test]
    fn write_to_sensitive_file_is_denied() {
        let mut clf = ApprovalClassifier::new();
        let args = make_args(&[("file_path", ".env")]);
        let decision = clf.should_auto_approve("Write", &args);
        assert!(!decision.approved);
        assert!(decision.reason.unwrap().contains("sensitive file"));
    }

    // ── Fallback mode ──

    #[test]
    fn fallback_mode_after_consecutive_denials() {
        let mut clf = ApprovalClassifier::new();
        clf.max_consecutive_denials = 3;

        // 3 consecutive denials should trigger fallback.
        for _ in 0..3 {
            let args = make_args(&[("command", "rm -rf /")]);
            let decision = clf.should_auto_approve("Bash", &args);
            assert!(!decision.approved);
        }

        assert!(clf.fallback_mode);

        // Even safe tools should be denied in fallback mode.
        let decision = clf.should_auto_approve("Read", &make_args(&[]));
        assert!(!decision.approved);
        assert!(decision.reason.unwrap().contains("Fallback mode"));
    }

    #[test]
    fn fallback_mode_after_total_denials() {
        let mut clf = ApprovalClassifier::new();
        clf.max_total_denials = 5;

        for _ in 0..5 {
            let args = make_args(&[("command", "some_unknown_tool")]);
            let decision = clf.should_auto_approve("Bash", &args);
            assert!(!decision.approved);
        }

        assert!(clf.fallback_mode);
    }

    #[test]
    fn approval_resets_consecutive_counter() {
        let mut clf = ApprovalClassifier::new();
        clf.max_consecutive_denials = 3;

        // Two denials.
        let args = make_args(&[("command", "rm -rf /")]);
        assert!(!clf.should_auto_approve("Bash", &args).approved);
        assert!(!clf.should_auto_approve("Bash", &args).approved);
        assert_eq!(clf.denial_count, 2);

        // One approval resets consecutive counter.
        let args = make_args(&[("command", "ls -la")]);
        assert!(clf.should_auto_approve("Bash", &args).approved);
        assert_eq!(clf.denial_count, 0);
    }

    #[test]
    fn unknown_tool_requires_approval() {
        let mut clf = ApprovalClassifier::new();
        let decision = clf.should_auto_approve("SomeUnknownTool", &HashMap::new());
        assert!(!decision.approved);
    }

    #[test]
    fn reset_counters_clears_fallback() {
        let mut clf = ApprovalClassifier::new();
        clf.max_consecutive_denials = 1;

        let args = make_args(&[("command", "rm -rf /")]);
        clf.should_auto_approve("Bash", &args);
        assert!(clf.fallback_mode);

        clf.reset_counters();
        assert!(!clf.fallback_mode);
        assert_eq!(clf.denial_count, 0);
        assert_eq!(clf.total_denial_count, 0);
    }
}
