//! Frontmatter and YAML parsing for skill files.
//!
//! Extracts metadata from YAML frontmatter blocks and provides
//! simple key-value YAML parsing without a full YAML library.

use std::collections::HashMap;
use std::path::Path;

use regex::Regex;
use tracing::debug;

use super::metadata::{SkillMetadata, SkillSource, SkillStatus};

/// Parse frontmatter from a file on disk.
pub(super) fn parse_frontmatter_file(path: &Path) -> Option<SkillMetadata> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            debug!(path = %path.display(), error = %e, "failed to read skill file");
            return None;
        }
    };
    let mut meta = parse_frontmatter_str(&content)?;
    if meta.name.is_empty() {
        // Fall back to filename stem.
        meta.name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown").to_string();
    }
    Some(meta)
}

/// Parse YAML frontmatter from a string.
///
/// Expects the format:
/// ```text
/// ---
/// name: foo
/// description: bar
/// namespace: baz
/// ---
/// ```
pub(super) fn parse_frontmatter_str(content: &str) -> Option<SkillMetadata> {
    let re = Regex::new(r"(?s)^---\r?\n(.*?)\r?\n---").ok()?;
    let caps = re.captures(content)?;
    let frontmatter = caps.get(1)?.as_str();

    // Simple key-value parsing (handles the common case without a full YAML parser).
    let data = parse_simple_yaml(frontmatter);

    let name = data.get("name").cloned().unwrap_or_default();
    let description = data
        .get("description")
        .cloned()
        .unwrap_or_else(|| format!("Skill: {}", if name.is_empty() { "unknown" } else { &name }));
    let namespace = data.get("namespace").cloned().unwrap_or_else(|| "default".to_string());

    let model = data.get("model").cloned().filter(|s| !s.is_empty());
    let agent = data.get("agent").cloned().filter(|s| !s.is_empty());

    let pinned = data.get("pinned").map(|s| s == "true").unwrap_or(false);
    let status = match data.get("status").map(|s| s.as_str()) {
        Some("stale") => SkillStatus::Stale,
        Some("archived") => SkillStatus::Archived,
        Some("superseded") => SkillStatus::Superseded,
        _ => SkillStatus::Active,
    };
    let requires_tools =
        data.get("requires_tools").map(|s| parse_list(s)).filter(|l: &Vec<String>| !l.is_empty());
    let fallback_for_tools = data
        .get("fallback_for_tools")
        .map(|s| parse_list(s))
        .filter(|l: &Vec<String>| !l.is_empty());
    let allowed_tools =
        data.get("allowed_tools").map(|s| parse_list(s)).filter(|l: &Vec<String>| !l.is_empty());
    let tags = parse_list(data.get("tags").unwrap_or(&String::new()));

    Some(SkillMetadata {
        name,
        description,
        namespace,
        path: None,
        source: SkillSource::Builtin,
        model,
        agent,
        pinned,
        status,
        requires_tools,
        fallback_for_tools,
        allowed_tools,
        usage_count: 0,
        last_used: None,
        tags,
    })
}

/// Simple YAML-like key:value parser for frontmatter.
///
/// Only handles flat `key: value` pairs. Strips surrounding quotes from values.
/// Parse a comma-separated list from a string.
fn parse_list(s: &str) -> Vec<String> {
    s.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
}

pub(super) fn parse_simple_yaml(text: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = trimmed.split_once(':') {
            let key = key.trim().to_string();
            let mut value = value.trim().to_string();
            // Strip surrounding quotes.
            if (value.starts_with('"') && value.ends_with('"'))
                || (value.starts_with('\'') && value.ends_with('\''))
            {
                value = value[1..value.len() - 1].to_string();
            }
            result.insert(key, value);
        }
    }
    result
}

/// Strip YAML frontmatter from markdown content, returning the body.
pub(super) fn strip_frontmatter(content: &str) -> String {
    let re = match Regex::new(r"(?s)^---\n.*?\n---\n*") {
        Ok(r) => r,
        Err(_) => return content.to_string(),
    };
    re.replace(content, "").to_string()
}

#[cfg(test)]
#[path = "parsing_tests.rs"]
mod tests;
