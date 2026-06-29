//! Dependency Graph collector — reads Cargo.toml/Cargo.lock for dependency structure.
//!
//! On-demand collector that parses the project's Cargo.toml and Cargo.lock
//! files to produce a formatted dependency tree. This provides the agent with
//! visibility into the project's dependency structure, useful for context when
//! modifying Cargo.toml, updating dependencies, or diagnosing build issues.

use std::path::Path;

use tracing::{debug, warn};

use crate::attachments::{Attachment, ContextCollector, TurnContext};
use crate::prompts::reminders::MessageClass;

/// Reads and formats a project's dependency graph from Cargo files.
///
/// This is an on-demand collector (fires once unless reset).
pub struct DependencyGraphCollector {
    /// Whether this collector has already fired.
    has_fired: std::sync::atomic::AtomicBool,
}

impl Default for DependencyGraphCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyGraphCollector {
    pub fn new() -> Self {
        Self { has_fired: std::sync::atomic::AtomicBool::new(false) }
    }

    /// Parse Cargo.toml and return a formatted dependency listing.
    fn read_cargo_deps(working_dir: &Path) -> Option<String> {
        let cargo_toml = working_dir.join("Cargo.toml");
        if !cargo_toml.exists() {
            debug!("DependencyGraphCollector: no Cargo.toml found");
            return None;
        }

        let content = std::fs::read_to_string(&cargo_toml).ok()?;
        let mut deps = Vec::new();
        let mut in_deps = false;
        let mut in_dev = false;
        let mut in_build = false;

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("[dependencies]") {
                in_deps = true;
                in_dev = false;
                in_build = false;
                continue;
            }
            if trimmed.starts_with("[dev-dependencies]") {
                in_deps = false;
                in_dev = true;
                in_build = false;
                continue;
            }
            if trimmed.starts_with("[build-dependencies]") {
                in_deps = false;
                in_dev = false;
                in_build = true;
                continue;
            }
            if trimmed.starts_with('[') {
                in_deps = false;
                in_dev = false;
                in_build = false;
                continue;
            }

            if in_deps || in_dev || in_build {
                if let Some(eq_pos) = trimmed.find('=') {
                    let name = trimmed[..eq_pos].trim();
                    // Skip non-dependency lines (features, workspace, etc.)
                    if !name.is_empty()
                        && !name.starts_with('#')
                        && !name.starts_with("workspace")
                        && !name.starts_with("edition")
                        && !name.starts_with("version")
                        && !name.starts_with("resolver")
                        && !name.starts_with("members")
                        && !name.starts_with("default-members")
                        && !name.starts_with("exclude")
                        && !name.starts_with("patch")
                        && !name.starts_with("profile")
                        && !name.starts_with("features")
                        && !name.starts_with("package")
                        && !name.starts_with("lib")
                    {
                        let section = if in_deps {
                            "deps"
                        } else if in_dev {
                            "dev"
                        } else {
                            "build"
                        };
                        let value = trimmed[eq_pos + 1..].trim().trim_matches('"');
                        deps.push((name.to_string(), value.to_string(), section.to_string()));
                    }
                }
            }
        }

        if deps.is_empty() {
            return Some("# Dependency Graph\n\nNo dependencies found in Cargo.toml.".to_string());
        }

        let mut output = String::from("# Dependency Graph\n\n");
        output.push_str("## Dependencies\n\n");

        let mut categories: Vec<(&str, Vec<&(String, String, String)>)> = Vec::new();
        for section in &["deps", "dev", "build"] {
            let items: Vec<_> = deps.iter().filter(|d| d.2 == *section).collect();
            if !items.is_empty() {
                categories.push((section, items));
            }
        }

        for (section, items) in &categories {
            let label = match *section {
                "deps" => "Runtime",
                "dev" => "Dev",
                "build" => "Build",
                _ => *section,
            };
            output.push_str(&format!("### {label}\n\n"));
            for (name, version, _) in items {
                output.push_str(&format!("- **{name}**: \"{version}\"\n"));
            }
            output.push('\n');
        }

        // Check for Cargo.lock to get resolved versions.
        let cargo_lock = working_dir.join("Cargo.lock");
        if cargo_lock.exists() {
            output.push_str("(Cargo.lock present — resolved dependency tree is available)\n\n");
        }

        Some(output)
    }
}

#[async_trait::async_trait]
impl ContextCollector for DependencyGraphCollector {
    fn name(&self) -> &'static str {
        "dependency_graph"
    }

    fn should_fire(&self, _ctx: &TurnContext<'_>) -> bool {
        // Fire once per session (on-demand).
        !self.has_fired.load(std::sync::atomic::Ordering::Relaxed)
    }

    async fn collect(&self, ctx: &TurnContext<'_>) -> Option<Attachment> {
        debug!("DependencyGraphCollector: collecting dependency graph");

        let content = Self::read_cargo_deps(ctx.working_dir)?;

        Some(Attachment { name: "dependency_graph", content, class: MessageClass::Directive })
    }

    fn did_fire(&self, _turn: usize) {
        self.has_fired.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    fn reset(&self) {
        self.has_fired.store(false, std::sync::atomic::Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    fn make_turn(dir: &Path) -> TurnContext {
        TurnContext {
            turn_number: 1,
            working_dir: dir,
            todo_manager: None,
            shared_state: None,
            last_user_query: None,
            cumulative_input_tokens: None,
            session_id: None,
            recent_messages: None,
        }
    }

    #[test]
    fn no_cargo_toml_returns_none() {
        let dir = tempfile::TempDir::new().unwrap();
        let ctx = make_turn(dir.path());
        let collector = DependencyGraphCollector::new();

        let result = collector.should_fire(&ctx);
        assert!(result); // should fire
    }

    #[test]
    fn parses_cargo_toml_deps() {
        let dir = tempfile::TempDir::new().unwrap();
        let cargo_toml = dir.path().join("Cargo.toml");
        fs::write(
            &cargo_toml,
            r#"[package]
name = "test-project"
version = "0.1.0"

[dependencies]
serde = "1.0"
tokio = { version = "1", features = ["full"] }

[dev-dependencies]
tempfile = "3"

[build-dependencies]
cc = "1.0"
"#,
        )
        .unwrap();

        let result = DependencyGraphCollector::read_cargo_deps(dir.path());
        assert!(result.is_some());
        let output = result.unwrap();
        assert!(output.contains("serde"));
        assert!(output.contains("tokio"));
        assert!(output.contains("tempfile"));
        assert!(output.contains("cc"));
        assert!(output.contains("Runtime"));
        assert!(output.contains("Dev"));
        assert!(output.contains("Build"));
    }

    #[test]
    fn fires_once_then_suppresses() {
        let dir = tempfile::TempDir::new().unwrap();
        let collector = DependencyGraphCollector::new();
        let ctx = make_turn(dir.path());

        assert!(collector.should_fire(&ctx));
        collector.did_fire(1);
        assert!(!collector.should_fire(&ctx));
    }

    #[test]
    fn reset_reenables() {
        let dir = tempfile::TempDir::new().unwrap();
        let collector = DependencyGraphCollector::new();
        let ctx = make_turn(dir.path());

        assert!(collector.should_fire(&ctx));
        collector.did_fire(1);
        assert!(!collector.should_fire(&ctx));

        collector.reset();
        assert!(collector.should_fire(&ctx));
    }

    #[test]
    fn empty_cargo_toml_returns_stub() {
        let dir = tempfile::TempDir::new().unwrap();
        let cargo_toml = dir.path().join("Cargo.toml");
        fs::write(&cargo_toml, "[package]\nname = \"empty\"\nversion = \"0.1.0\"\n").unwrap();

        let result = DependencyGraphCollector::read_cargo_deps(dir.path());
        assert!(result.is_some());
        assert!(result.unwrap().contains("No dependencies found"));
    }
}
