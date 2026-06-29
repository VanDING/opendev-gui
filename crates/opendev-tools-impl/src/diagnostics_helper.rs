//! Post-edit diagnostic collection helper.
//!
//! After file modifications (edit, write, patch), this module queries
//! the optional `DiagnosticProvider` on the `ToolContext` and formats
//! any errors/warnings into a string that gets appended to the tool output.
//! This gives the LLM immediate feedback about introduced errors.
//!
//! # Edge Case Handling
//!
//! - Timeout: LSP queries are wrapped in a 5-second timeout.
//! - Fallback: If no DiagnosticProvider is configured, returns None gracefully.
//! - Dedup: A simple same-file cache prevents redundant queries within 500ms.
//! - Output cap: Diagnostic output is capped at 2000 characters.

use std::path::Path;
use std::time::Instant;

use opendev_tools_core::ToolContext;

/// Maximum number of diagnostics to include per file.
const MAX_DIAGNOSTICS_PER_FILE: usize = 20;

/// Maximum number of extra project files to report diagnostics for.
const MAX_PROJECT_DIAGNOSTIC_FILES: usize = 5;

/// Timeout for LSP diagnostic queries (5 seconds).
const LSP_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

/// Maximum output characters for diagnostic messages.
const MAX_OUTPUT_CHARS: usize = 2000;

/// Minimum time between diagnostic queries for the same file (dedup window).
const DEDUP_WINDOW_MS: u128 = 500;

use std::collections::HashMap;
/// Simple dedup cache: tracks last query time per file path.
use std::sync::Mutex;

static DIAG_DEDUP_CACHE: std::sync::LazyLock<Mutex<HashMap<std::path::PathBuf, Instant>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

/// Check if we should skip diagnostics for a file (queried too recently).
fn should_skip_dedup(file_path: &Path) -> bool {
    let mut cache = DIAG_DEDUP_CACHE.lock().unwrap();
    if let Some(last) = cache.get(file_path) {
        if last.elapsed().as_millis() < DEDUP_WINDOW_MS {
            return true; // Too recent — skip
        }
    }
    cache.insert(file_path.to_path_buf(), Instant::now());
    false
}

/// Collect LSP diagnostics for a file after modification.
///
/// Returns a formatted string suitable for appending to tool output,
/// or `None` if no diagnostics are available or no provider is configured.
pub async fn collect_post_edit_diagnostics(ctx: &ToolContext, file_path: &Path) -> Option<String> {
    // Skip if queried this file too recently (dedup for multi-edit)
    if should_skip_dedup(file_path) {
        return None;
    }

    let provider = ctx.diagnostic_provider.as_ref()?;

    // Query diagnostics with a 5-second timeout.
    // If LSP is uninitialized or slow, we fall back gracefully rather than blocking.
    let diagnostics = {
        let query = provider.diagnostics_for_file(file_path, 2, MAX_DIAGNOSTICS_PER_FILE);
        match tokio::time::timeout(LSP_TIMEOUT, query).await {
            Ok(diags) => diags,
            Err(_) => {
                tracing::debug!(
                    file = %file_path.display(),
                    "LSP diagnostic query timed out after 5s"
                );
                return None;
            }
        }
    };

    if diagnostics.is_empty() {
        return None;
    }

    let mut output = String::new();

    // Count errors vs warnings
    let error_count = diagnostics.iter().filter(|d| d.severity == 1).count();
    let warning_count = diagnostics.iter().filter(|d| d.severity == 2).count();

    output.push_str("\nLSP diagnostics detected after edit:");
    output.push_str(&format!("\n<diagnostics file=\"{}\">", file_path.display()));

    for diag in &diagnostics {
        output.push('\n');
        output.push_str(&diag.pretty());
    }

    output.push_str("\n</diagnostics>");

    if error_count > 0 {
        output.push_str(&format!(
            "\n\n{error_count} error(s) and {warning_count} warning(s) found. Please fix the errors."
        ));
    }

    // Cap output at MAX_OUTPUT_CHARS
    if output.len() > MAX_OUTPUT_CHARS {
        let mut truncated = String::with_capacity(MAX_OUTPUT_CHARS + 50);
        truncated.push_str(&output[..MAX_OUTPUT_CHARS]);
        truncated
            .push_str(&format!("\n[...diagnostic output truncated at {} chars]", MAX_OUTPUT_CHARS));
        output = truncated;
    }

    Some(output)
}

/// Collect diagnostics for multiple files (used by patch tool).
///
/// Returns formatted diagnostic output for all modified files,
/// limited to `MAX_PROJECT_DIAGNOSTIC_FILES` files.
pub async fn collect_multi_file_diagnostics(
    ctx: &ToolContext,
    file_paths: &[&Path],
) -> Option<String> {
    let provider = ctx.diagnostic_provider.as_ref()?;

    let mut output = String::new();
    let mut files_with_diags = 0;

    for &file_path in file_paths.iter().take(MAX_PROJECT_DIAGNOSTIC_FILES + 1) {
        let diagnostics =
            provider.diagnostics_for_file(file_path, 2, MAX_DIAGNOSTICS_PER_FILE).await;

        if diagnostics.is_empty() {
            continue;
        }

        files_with_diags += 1;
        if files_with_diags > MAX_PROJECT_DIAGNOSTIC_FILES {
            output.push_str(&format!(
                "\n... and more files with diagnostics (showing first {MAX_PROJECT_DIAGNOSTIC_FILES})"
            ));
            break;
        }

        output.push_str(&format!("\n<diagnostics file=\"{}\">", file_path.display()));

        for diag in &diagnostics {
            output.push('\n');
            output.push_str(&diag.pretty());
        }

        output.push_str("\n</diagnostics>");
    }

    if output.is_empty() {
        return None;
    }

    let mut result = String::from("\nLSP diagnostics detected after edit:");
    result.push_str(&output);
    Some(result)
}

#[cfg(test)]
#[path = "diagnostics_helper_tests.rs"]
mod tests;
