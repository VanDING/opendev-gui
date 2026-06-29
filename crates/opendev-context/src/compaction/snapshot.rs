//! Snapshot collector — shows file changes since last snapshot.
//!
//! Provides the `SnapshotCollector` struct which aggregates file changes
//! from the `ArtifactIndex` (tracked by the compaction system) and formats
//! them as a human-readable summary. This is used by the agent's context
//! attachment system to show what files have been modified since the last
//! compaction point.
//!
//! The collector lives in the compaction module because it shares the
//! `ArtifactIndex` that compaction already maintains. The actual wiring
//! to the agent's `ContextCollector` trait happens in `opendev-agents`.

use chrono::Local;

use super::artifacts::ArtifactIndex;

/// Collects file change information from the artifact index since the last
/// snapshot/compaction point.
///
/// This struct provides a summary of files touched during the current
/// session segment, suitable for injection into the system prompt.
pub struct SnapshotCollector;

impl SnapshotCollector {
    /// Build a summary of files changed since the last snapshot.
    ///
    /// Reads from the provided `ArtifactIndex` and formats a structured
    /// list of files with their operations, changes, and timestamps.
    ///
    /// Returns `None` if there are no changes to report.
    pub fn collect_changes(artifact_index: &ArtifactIndex) -> Option<String> {
        if artifact_index.is_empty() {
            return None;
        }

        let now = Local::now().format("%H:%M:%S").to_string();
        let mut entries: Vec<_> = artifact_index.entries.iter().collect();
        entries.sort_by(|a, b| b.1.updated_at.cmp(&a.1.updated_at));

        let mut lines = vec![format!(
            "# Files Changed (since last snapshot, {} entries at {})",
            entries.len(),
            now,
        )];
        lines.push(String::new());

        for (path, entry) in entries.iter().take(50) {
            // Truncate path if very long (keep last 120 chars).
            let display_path = if path.len() > 120 {
                format!("...{}", &path[path.len() - 117..])
            } else {
                path.to_string()
            };

            let ops_summary = if entry.operations_seen.len() <= 2 {
                entry.operations_seen.join(", ")
            } else {
                format!(
                    "{} operations ({})",
                    entry.operations_seen.len(),
                    entry.operations_seen.join(", "),
                )
            };

            let detail = if entry.last_details.is_empty() {
                String::new()
            } else {
                let truncated: String = entry.last_details.chars().take(80).collect();
                format!(" — {}", truncated)
            };

            lines.push(format!("- `{}` [{}]{detail}", display_path, ops_summary));
        }

        if entries.len() > 50 {
            lines.push(format!("\n... and {} more files (showing first 50)", entries.len() - 50,));
        }

        lines.push(String::new());
        lines.push(
            "(Run `git diff` or use the snapshot manager to review individual changes.)"
                .to_string(),
        );

        Some(lines.join("\n"))
    }

    /// Count of files changed since the last snapshot.
    pub fn changed_file_count(artifact_index: &ArtifactIndex) -> usize {
        artifact_index.len()
    }

    /// Check whether there are any changes to report.
    pub fn has_changes(artifact_index: &ArtifactIndex) -> bool {
        !artifact_index.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_index_returns_none() {
        let index = ArtifactIndex::new();
        assert!(SnapshotCollector::collect_changes(&index).is_none());
        assert!(!SnapshotCollector::has_changes(&index));
        assert_eq!(SnapshotCollector::changed_file_count(&index), 0);
    }

    #[test]
    fn formats_single_change() {
        let mut index = ArtifactIndex::new();
        index.record("src/main.rs", "edit", "Added new feature");

        let result = SnapshotCollector::collect_changes(&index);
        assert!(result.is_some());
        let output = result.unwrap();
        assert!(output.contains("Files Changed"));
        assert!(output.contains("src/main.rs"));
        assert!(output.contains("edit"));
        assert!(output.contains("Added new feature"));
    }

    #[test]
    fn formats_multiple_changes() {
        let mut index = ArtifactIndex::new();
        index.record("src/main.rs", "edit", "Refactored function");
        index.record("src/lib.rs", "create", "New module");
        index.record("tests/test.rs", "read", "");

        let result = SnapshotCollector::collect_changes(&index);
        assert!(result.is_some());
        let output = result.unwrap();
        assert!(output.contains("src/main.rs"));
        assert!(output.contains("src/lib.rs"));
        assert!(output.contains("tests/test.rs"));
    }

    #[test]
    fn truncates_to_50_entries() {
        let mut index = ArtifactIndex::new();
        for i in 0..60 {
            index.record(&format!("file_{}.rs", i), "edit", "change");
        }

        let result = SnapshotCollector::collect_changes(&index).unwrap();
        assert!(result.contains("showing first 50"));
        assert!(result.contains("... and 10 more files"));
    }
}
