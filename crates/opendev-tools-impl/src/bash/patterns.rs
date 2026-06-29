//! Command classification patterns and environment variable filtering.
//!
//! Delegates dangerous-command detection and env filtering to `opendev-exec`.
//! Server-command and interactive-command patterns remain local (they are
//! specific to BashTool behavior, not sandbox policy).

use std::collections::HashMap;
use std::sync::LazyLock;

use regex::Regex;

// ---------------------------------------------------------------------------
// Sensitive environment variable filtering (delegated to opendev-exec)
// ---------------------------------------------------------------------------

/// Check if an environment variable name is sensitive and should be stripped.
///
/// Delegates to `opendev_exec::env_filter` for centralized secret management.
#[allow(dead_code)]
pub fn is_sensitive_env(name: &str) -> bool {
    opendev_exec::env_filter::is_sensitive_env(name)
}

/// Build a filtered environment map: inherits all env vars except sensitive ones.
///
/// Delegates to `opendev_exec::env_filter` for centralized secret management.
#[allow(dead_code)]
pub fn filtered_env() -> HashMap<String, String> {
    opendev_exec::env_filter::filtered_env()
}

// ---------------------------------------------------------------------------
// Dangerous-command detection (delegated to opendev-exec)
// ---------------------------------------------------------------------------

/// Check if a command matches known dangerous patterns (e.g., `rm -rf /`, `sudo`).
///
/// Delegates to `opendev_exec::patterns` for centralized dangerous command detection.
pub fn is_dangerous(command: &str) -> bool {
    opendev_exec::patterns::is_dangerous(command)
}

// ---------------------------------------------------------------------------
// Interactive-command patterns (auto-confirm with `yes |`)
// ---------------------------------------------------------------------------

const INTERACTIVE_PATTERNS: &[&str] = &[
    r"\bnpx\b",
    r"\bnpm\s+(init|create)\b",
    r"\byarn\s+create\b",
    r"\bng\s+new\b",
    r"\bvue\s+create\b",
    r"\bcreate-react-app\b",
    r"\bnext\s+create\b",
    r"\bvite\s+create\b",
    r"\bpnpm\s+create\b",
    r"\bpip\s+install\b",
];

// ---------------------------------------------------------------------------
// Auto-background command patterns.
//
// These commands are promoted to background automatically because they
// are long-running (servers, watchers, monitors) rather than producing
// useful foreground output.
// ---------------------------------------------------------------------------

const AUTO_BACKGROUND_PATTERNS: &[&str] = &[
    // Python web servers
    r"flask\s+run",
    r"python.*manage\.py\s+runserver",
    r"uvicorn",
    r"gunicorn",
    r"python.*-m\s+http\.server",
    r"hypercorn",
    r"daphne",
    r"waitress",
    r"fastapi",
    // Node.js
    r"npm\s+(run\s+)?(start|dev|serve)",
    r"yarn\s+(run\s+)?(start|dev|serve)",
    r"pnpm\s+(run\s+)?(start|dev|serve)",
    r"bun\s+(run\s+)?(start|dev|serve)",
    r"node.*server",
    r"nodemon",
    r"next\s+(dev|start)",
    r"nuxt\s+(dev|start)",
    r"vite(\s+dev)?$",
    r"webpack.*(dev.?server|serve)",
    // Ruby / PHP / Other
    r"rails\s+server",
    r"php.*artisan\s+serve",
    r"php\s+-S\s+",
    r"hugo\s+server",
    r"jekyll\s+serve",
    // Go
    r"go\s+run.*server",
    // Rust
    r"cargo\s+(run|watch)",
    // Java
    r"mvn.*spring-boot:run",
    r"gradle.*bootRun",
    // Background watchers / monitors
    r"\bsleep\s+\d+",   // Sleep commands
    r"\btail\s+-f\b",   // Tail follow
    r"\binotifywait\b", // File watchers
    r"\byes\b",         // yes infinite stream
    r"\btop\b",         // System monitors
    r"\bhtop\b",
    r"\bwatch\b",
    r"\bping\b", // Network monitors
    r"\btcpdump\b",
    r"\bngrok\b", // Tunnel services
    // Generic
    r"live-server",
    r"http-server",
    r"serve\s+-",
    r"browser-sync",
    r"docker\s+compose\s+up",
];

// ---------------------------------------------------------------------------
// Regex cache helpers
// ---------------------------------------------------------------------------

/// Pre-compiled regex set for pattern matching. Avoids recompiling on every call.
struct CompiledPatterns {
    regexes: Vec<Regex>,
}

impl CompiledPatterns {
    fn new(patterns: &[&str]) -> Self {
        Self { regexes: patterns.iter().filter_map(|p| Regex::new(p).ok()).collect() }
    }

    fn matches(&self, text: &str) -> bool {
        self.regexes.iter().any(|re| re.is_match(text))
    }
}

static AUTO_BACKGROUND_COMPILED: LazyLock<CompiledPatterns> =
    LazyLock::new(|| CompiledPatterns::new(AUTO_BACKGROUND_PATTERNS));

static INTERACTIVE_COMPILED: LazyLock<CompiledPatterns> =
    LazyLock::new(|| CompiledPatterns::new(INTERACTIVE_PATTERNS));

/// Maximum time a command runs in the foreground before auto-background promotion.
/// Default: 15 seconds.
pub const MAX_FOREGROUND_MS: u64 = 15000;

/// Check if a command should be auto-promoted to background execution.
///
/// Returns `true` for commands that are known to be long-running or
/// continuously running (servers, watchers, monitors, blocking commands).
///
/// This replaces the previous `is_server_command` with a broader set of
/// background-worthy patterns. When matched, the bash tool automatically
/// backgrounds the command after `MAX_FOREGROUND_MS` of foreground execution.
pub(crate) fn is_auto_background_command(command: &str) -> bool {
    AUTO_BACKGROUND_COMPILED.matches(command)
}

/// Legacy alias for backward compatibility.
pub(crate) fn is_server_command(command: &str) -> bool {
    is_auto_background_command(command)
}

pub(crate) fn needs_auto_confirm(command: &str) -> bool {
    INTERACTIVE_COMPILED.matches(command)
}

#[cfg(test)]
#[path = "patterns_tests.rs"]
mod tests;
