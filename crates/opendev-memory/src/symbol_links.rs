//! Memory Symbol Links — automatic linking between file edits and memory entries.
//!
//! When a file edit tool (Edit, Write) successfully modifies a function/struct
//! definition, this module extracts symbol names from the diff and creates
//! links in the memory database so the edited symbols are associated with
//! relevant memory entries.

use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;

use crate::MemoryFacade;

/// Pattern to match Rust function definitions (`fn`, `pub fn`, `pub(crate) fn`, etc.).
static FN_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\s*(pub(\s*\(\s*\w+\s*\))?\s+)?(unsafe\s+)?fn\s+(\w+)")
        .expect("valid regex: fn pattern")
});

/// Pattern to match Rust struct definitions.
static STRUCT_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\s*(pub(\s*\(\s*\w+\s*\))?\s+)?struct\s+(\w+)")
        .expect("valid regex: struct pattern")
});

/// Pattern to match Rust enum definitions.
static ENUM_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\s*(pub(\s*\(\s*\w+\s*\))?\s+)?enum\s+(\w+)")
        .expect("valid regex: enum pattern")
});

/// Pattern to match Rust trait definitions.
static TRAIT_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\s*(pub(\s*\(\s*\w+\s*\))?\s+)?(unsafe\s+)?trait\s+(\w+)")
        .expect("valid regex: trait pattern")
});

/// Pattern to match Rust impl blocks (captures the type being implemented).
static IMPL_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\s*(pub(\s*\(\s*\w+\s*\))?\s+)?(unsafe\s+)?impl(\s*<[^>]+>)?\s+(\w+)")
        .expect("valid regex: impl pattern")
});

/// Pattern to match TypeScript/JavaScript function definitions.
/// Captures: `function name`, `const name = async (...)`, `const name = function`, etc.
static TS_FN_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^\s*(export\s+)?(async\s+)?(function\s+(\w+)|const\s+(\w+)\s*=\s*(async\s+)?\s*\()",
    )
    .expect("valid regex: TS function pattern")
});

/// Pattern to match TypeScript/JavaScript class definitions.
static TS_CLASS_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\s*(export\s+)?(abstract\s+)?class\s+(\w+)")
        .expect("valid regex: TS class pattern")
});

/// Pattern to match TypeScript interface definitions.
static TS_INTERFACE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\s*(export\s+)?interface\s+(\w+)").expect("valid regex: TS interface pattern")
});

/// Pattern to match TypeScript type definitions.
static TS_TYPE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\s*(export\s+)?type\s+(\w+)").expect("valid regex: TS type pattern")
});

/// Extract symbol names from edited file content.
///
/// Parses the edit content (the new version / diff) for function, struct,
/// enum, trait, impl, class, and interface definitions. Returns a list
/// of discovered symbol names.
pub fn extract_symbols_from_edit(edit_content: &str) -> Vec<String> {
    let mut symbols = Vec::new();

    // Rust patterns
    for cap in FN_PATTERN.captures_iter(edit_content) {
        if let Some(name) = cap.get(4) {
            symbols.push(format!("fn {}", name.as_str()));
        }
    }
    for cap in STRUCT_PATTERN.captures_iter(edit_content) {
        if let Some(name) = cap.get(3) {
            symbols.push(format!("struct {}", name.as_str()));
        }
    }
    for cap in ENUM_PATTERN.captures_iter(edit_content) {
        if let Some(name) = cap.get(3) {
            symbols.push(format!("enum {}", name.as_str()));
        }
    }
    for cap in TRAIT_PATTERN.captures_iter(edit_content) {
        if let Some(name) = cap.get(4) {
            symbols.push(format!("trait {}", name.as_str()));
        }
    }
    for cap in IMPL_PATTERN.captures_iter(edit_content) {
        if let Some(name) = cap.get(5) {
            symbols.push(format!("impl {}", name.as_str()));
        }
    }

    // TypeScript/JavaScript patterns
    for cap in TS_FN_PATTERN.captures_iter(edit_content) {
        if let Some(name) = cap.get(4) {
            symbols.push(format!("function {}", name.as_str()));
        } else if let Some(name) = cap.get(5) {
            symbols.push(format!("const {}", name.as_str()));
        } else if let Some(name) = cap.get(7) {
            symbols.push(format!("const {}", name.as_str()));
        }
    }
    for cap in TS_CLASS_PATTERN.captures_iter(edit_content) {
        if let Some(name) = cap.get(3) {
            symbols.push(format!("class {}", name.as_str()));
        }
    }
    for cap in TS_INTERFACE_PATTERN.captures_iter(edit_content) {
        if let Some(name) = cap.get(2) {
            symbols.push(format!("interface {}", name.as_str()));
        }
    }
    for cap in TS_TYPE_PATTERN.captures_iter(edit_content) {
        if let Some(name) = cap.get(2) {
            symbols.push(format!("type {}", name.as_str()));
        }
    }

    symbols.sort();
    symbols.dedup();
    symbols
}

/// Automatically link symbols found in an edit to memory entries.
///
/// When a file edit tool (Edit, Write) successfully modifies a file, call this
/// function to:
/// 1. Parse the edit content for function/struct/enum/trait/class definitions
/// 2. Search memory for entries mentioning those symbols
/// 3. Call `repo.link_to_symbol()` for each matching entry
///
/// # Arguments
/// * `file_path` - Path to the edited file (for project context)
/// * `edit_content` - The new content or diff of the edit
/// * `memory` - The memory facade to search and link through
///
/// # Returns
/// Number of symbol links created.
pub async fn automatically_link_symbols(
    file_path: &str,
    edit_content: &str,
    memory: &MemoryFacade,
) -> usize {
    let symbols = extract_symbols_from_edit(edit_content);
    if symbols.is_empty() {
        return 0;
    }

    let project_path = std::path::Path::new(file_path);
    let project_dir = project_path.parent().unwrap_or(project_path);

    let mut links_created = 0;

    for symbol in &symbols {
        // Search for memory entries that mention this symbol.
        let bare_name = symbol.split_whitespace().last().unwrap_or(symbol);
        let entries =
            memory.recall_by_symbol(bare_name, Some(project_dir), 10).await.unwrap_or_default();

        for entry in &entries {
            // Link the matched memory entry to the symbol.
            if memory.link_symbol(&entry.id, bare_name, bare_name, project_dir).await.is_ok() {
                links_created += 1;
            }
        }

        // Also search by content containing the symbol name using recall_within_budget.
        if let Ok(content_entries) =
            memory.recall_by_symbol_within_budget(bare_name, Some(project_dir), 10).await
        {
            for entry in &content_entries {
                if memory
                    .link_symbol(&entry.entry.id, bare_name, bare_name, project_dir)
                    .await
                    .is_ok()
                {
                    links_created += 1;
                }
            }
        }
    }

    links_created
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Symbol extraction tests ──

    #[test]
    fn extracts_rust_fn() {
        let content = "pub fn process_data() {\n    // ...\n}";
        let symbols = extract_symbols_from_edit(content);
        assert!(symbols.contains(&"fn process_data".to_string()));
    }

    #[test]
    fn extracts_private_fn() {
        let content = "fn helper() {}";
        let symbols = extract_symbols_from_edit(content);
        assert!(symbols.contains(&"fn helper".to_string()));
    }

    #[test]
    fn extracts_unsafe_fn() {
        let content = "pub unsafe fn dangerous() {}";
        let symbols = extract_symbols_from_edit(content);
        assert!(symbols.contains(&"fn dangerous".to_string()));
    }

    #[test]
    fn extracts_struct() {
        let content = "pub struct Config {\n    name: String,\n}";
        let symbols = extract_symbols_from_edit(content);
        assert!(symbols.contains(&"struct Config".to_string()));
    }

    #[test]
    fn extracts_enum() {
        let content = "pub enum Status { Active, Inactive }";
        let symbols = extract_symbols_from_edit(content);
        assert!(symbols.contains(&"enum Status".to_string()));
    }

    #[test]
    fn extracts_trait() {
        let content = "pub trait Handler {\n    fn handle(&self);\n}";
        let symbols = extract_symbols_from_edit(content);
        assert!(symbols.contains(&"trait Handler".to_string()));
    }

    #[test]
    fn extracts_impl() {
        let content = "impl Handler for MyStruct {\n    fn handle(&self) {}\n}";
        let symbols = extract_symbols_from_edit(content);
        assert!(symbols.contains(&"impl Handler".to_string()));
    }

    #[test]
    fn extracts_ts_function() {
        let content = "export function calculateTotal(items: Item[]): number {\n  return items.reduce((a, b) => a + b.price, 0);\n}";
        let symbols = extract_symbols_from_edit(content);
        assert!(symbols.contains(&"function calculateTotal".to_string()));
    }

    #[test]
    fn extracts_ts_const_function() {
        let content = "const getData = async () => {\n  return fetch('/api/data');\n};";
        let symbols = extract_symbols_from_edit(content);
        assert!(symbols.contains(&"const getData".to_string()));
    }

    #[test]
    fn extracts_ts_class() {
        let content = "export abstract class BaseService {\n  protected baseUrl: string;\n}";
        let symbols = extract_symbols_from_edit(content);
        assert!(symbols.contains(&"class BaseService".to_string()));
    }

    #[test]
    fn extracts_ts_interface() {
        let content = "export interface UserProfile {\n  name: string;\n  email: string;\n}";
        let symbols = extract_symbols_from_edit(content);
        assert!(symbols.contains(&"interface UserProfile".to_string()));
    }

    #[test]
    fn extracts_ts_type() {
        let content = "export type Callback = (result: string) => void;";
        let symbols = extract_symbols_from_edit(content);
        assert!(symbols.contains(&"type Callback".to_string()));
    }

    #[test]
    fn empty_content_extracts_nothing() {
        let symbols = extract_symbols_from_edit("");
        assert!(symbols.is_empty());
    }

    #[test]
    fn no_definitions_extracts_nothing() {
        let content = "let x = 5;\nconsole.log(x);";
        let symbols = extract_symbols_from_edit(content);
        assert!(symbols.is_empty());
    }

    #[test]
    fn deduplicates_symbols() {
        let content = "pub fn process() {}\npub fn process() {}";
        let symbols = extract_symbols_from_edit(content);
        assert_eq!(
            symbols.iter().filter(|s| s.as_str() == "fn process").count(),
            1,
            "Symbols should be deduplicated"
        );
    }

    #[test]
    fn multiple_symbol_types() {
        let content = "pub struct Request {\n    id: String,\n}\n\nimpl Request {\n    pub fn new(id: String) -> Self {\n        Self { id }\n    }\n}";
        let symbols = extract_symbols_from_edit(content);
        assert!(symbols.contains(&"struct Request".to_string()));
        assert!(symbols.contains(&"impl Request".to_string()));
        assert!(symbols.contains(&"fn new".to_string()));
    }
}
