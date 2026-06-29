//! Canonical path resolution for LLM-produced tool parameters.
//!
//! LLMs frequently return incorrect paths — relative paths, redundant basename
//! prefixes (e.g., `myproject/src/main.rs` when cwd is already `myproject`),
//! `./` prefixes, `$HOME` paths, etc. This module provides the single source
//! of truth for resolving such paths.

use std::path::{Component, Path, PathBuf};

/// Error returned for dangerous or unsupported path patterns.
#[derive(Debug, Clone)]
pub enum DangerousPath {
    /// UNC path (\\server\share) — potential NTLM credential leak.
    UncPath(String),
    /// Tilde expansion variant that could be used for path traversal.
    TildeExpansion(String),
    /// Shell expansion syntax ($VAR, `${VAR}`, %TEMP%, =cmd).
    ShellExpansion(String),
    /// Glob pattern in write operation.
    GlobPattern(String),
}

impl std::fmt::Display for DangerousPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UncPath(p) => write!(f, "UNC path blocked (credential leak risk): {p}"),
            Self::TildeExpansion(p) => {
                write!(f, "Tilde expansion variant blocked: {p}")
            }
            Self::ShellExpansion(p) => {
                write!(f, "Shell expansion syntax blocked: {p}")
            }
            Self::GlobPattern(p) => write!(f, "Glob pattern blocked in write operation: {p}"),
        }
    }
}

/// Expand tilde (`~`) and `$HOME` prefixes in a path string.
///
/// - `~/foo` -> `/home/user/foo`
/// - `$HOME/foo` -> `/home/user/foo`
/// - `~` -> `/home/user`
/// - Other paths are returned as-is.
pub fn expand_home(path: &str) -> String {
    if path == "~" {
        return dirs::home_dir()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string());
    }
    if let Some(rest) = path.strip_prefix("~/")
        && let Some(home) = dirs::home_dir()
    {
        return format!("{}/{}", home.display(), rest);
    }
    if let Some(rest) = path.strip_prefix("$HOME/")
        && let Some(home) = dirs::home_dir()
    {
        return format!("{}/{}", home.display(), rest);
    }
    if path == "$HOME" {
        return dirs::home_dir()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string());
    }
    path.to_string()
}

/// Check if a path is a UNC path (`\\server\share\...`) which could leak
/// NTLM credentials via SMB authentication.
pub fn is_vulnerable_unc_path(path: &str) -> Option<DangerousPath> {
    if path.starts_with("\\\\") || path.starts_with("//") {
        // True UNC paths start with double separator followed by non-separator.
        let rest = &path[2..];
        if !rest.is_empty() && !rest.starts_with('\\') && !rest.starts_with('/') {
            return Some(DangerousPath::UncPath(path.to_string()));
        }
    }
    None
}

/// Check if a path contains tilde expansion variants that could be exploited
/// for path traversal or user enumeration.
///
/// Blocks: `~root`, `~+`, `~-` while allowing simple `~/` and `~`.
pub fn contains_tilde_expansion(path: &str) -> Option<DangerousPath> {
    // Match ~ followed by a non-/ character (expands to other user's home)
    if path.starts_with('~') && path.len() > 1 && !path[1..].starts_with('/') {
        return Some(DangerousPath::TildeExpansion(path.to_string()));
    }
    // Also check mid-path tilde expansions
    if path.contains("/~") {
        // Find the ~ and check what follows
        if let Some(pos) = path.find("/~") {
            let after_tilde = &path[pos + 2..];
            if !after_tilde.is_empty() && !after_tilde.starts_with('/') {
                return Some(DangerousPath::TildeExpansion(path.to_string()));
            }
        }
    }
    None
}

/// Check if a path contains shell expansion syntax that could execute
/// arbitrary commands or leak environment variables.
///
/// Blocks: `$VAR`, `${VAR}`, `%TEMP%`, `=cmd` patterns.
pub fn contains_shell_expansion(path: &str) -> Option<DangerousPath> {
    // $VAR or ${VAR} expansion
    if path.contains("${") || path.contains('$') && has_env_var_ref(path) {
        return Some(DangerousPath::ShellExpansion(path.to_string()));
    }
    // Windows-style %VAR% expansion
    if path.contains('%') && path.chars().filter(|&c| c == '%').count() >= 2 {
        return Some(DangerousPath::ShellExpansion(path.to_string()));
    }
    // =cmd expansion (zsh equals expansion bypass)
    if path.starts_with('=') || path.contains("/=") {
        return Some(DangerousPath::ShellExpansion(path.to_string()));
    }
    None
}

/// Helper: check if path contains a $VAR reference (not just literal $).
fn has_env_var_ref(path: &str) -> bool {
    let bytes = path.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'$' && i + 1 < bytes.len() {
            let next = bytes[i + 1];
            // $ followed by letter, underscore, or digit is an env var
            if next.is_ascii_alphabetic() || next == b'_' {
                return true;
            }
        }
    }
    false
}

/// Check if a path contains glob patterns, which should be blocked for write operations.
pub fn contains_glob_pattern(path: &str) -> bool {
    path.contains('*') || path.contains('?') || path.contains('[') || path.contains('{')
}

/// Run all dangerous path checks on a path.
/// Returns the first dangerous match, or None if the path is safe.
pub fn check_dangerous_path(path: &str) -> Option<DangerousPath> {
    is_vulnerable_unc_path(path)
        .or_else(|| contains_tilde_expansion(path))
        .or_else(|| contains_shell_expansion(path))
}

/// Strip leading `.` and `./` components from a path, returning the
/// meaningful portion. E.g., `./myproject/src` -> `myproject/src`.
pub fn strip_curdir(path: &Path) -> PathBuf {
    path.components().filter(|c| !matches!(c, Component::CurDir)).collect()
}

/// Normalize a path by collapsing `.` and `..` components without touching the filesystem.
///
/// Unlike `canonicalize()`, this works on paths that don't exist yet.
pub fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();

    for component in path.components() {
        match component {
            Component::CurDir => {} // skip `.`
            Component::ParentDir => {
                // Pop the last component if it's a normal component.
                if let Some(last) = components.last()
                    && !matches!(last, Component::RootDir | Component::Prefix(_))
                {
                    components.pop();
                    continue;
                }
                components.push(component);
            }
            _ => components.push(component),
        }
    }

    components.iter().collect()
}

/// Strip hallucinated Docker-style prefixes like `/workspace/` or `/testbed/`.
///
/// If the path starts with a known fake prefix AND the resulting absolute path
/// doesn't exist, rewrites it to be relative to the working directory.
/// If the original absolute path does exist (e.g., there really is a `/workspace/` dir),
/// it is left unchanged.
fn strip_hallucinated_prefix(path_str: &str, working_dir: &Path) -> String {
    for prefix in HALLUCINATED_PREFIXES {
        if let Some(rest) = path_str.strip_prefix(prefix) {
            let original = Path::new(path_str);
            // Only rewrite if the original doesn't exist but the working_dir version does
            // (or the working_dir version's parent exists for new file creation).
            if !original.exists() {
                let candidate = working_dir.join(rest);
                if candidate.exists() || candidate.parent().map(|p| p.is_dir()).unwrap_or(false) {
                    return candidate.to_string_lossy().to_string();
                }
                // Even if candidate doesn't exist, still rewrite — `/workspace/` is almost
                // certainly wrong on a real system.
                return candidate.to_string_lossy().to_string();
            }
        }
    }
    // Also handle bare `/workspace` or `/testbed` (without trailing slash or subpath)
    let bare_prefixes = ["/workspace", "/testbed"];
    for prefix in &bare_prefixes {
        if path_str == *prefix && !Path::new(prefix).exists() {
            return working_dir.to_string_lossy().to_string();
        }
    }
    path_str.to_string()
}

/// Well-known fake prefixes that LLMs hallucinate from Docker training data.
/// When we see these as absolute path prefixes and the real path doesn't exist,
/// we strip them and resolve relative to the actual working directory.
const HALLUCINATED_PREFIXES: &[&str] = &["/workspace/", "/testbed/"];

/// Resolve a user-provided file path against the working directory.
///
/// Handles common LLM mistakes:
/// - `./src/main.rs` -> strips `./` prefix
/// - `~/file.rs` / `$HOME/file.rs` -> expands home directory
/// - `myproject/main.rs` when cwd is `/home/user/myproject` -> `/home/user/myproject/main.rs`
///   (detects and strips redundant basename prefix)
/// - Absolute paths with doubled project name
/// - `/workspace/foo` or `/testbed/foo` -> `{working_dir}/foo` (LLM hallucination from Docker)
pub fn resolve_file_path(user_path: &str, working_dir: &Path) -> PathBuf {
    let expanded = expand_home(user_path);
    // Rewrite hallucinated Docker prefixes to working_dir-relative paths
    let expanded = strip_hallucinated_prefix(&expanded, working_dir);
    let path = strip_curdir(Path::new(&expanded));
    let path = normalize_path(&path);
    let path = path.as_path();
    if path.is_absolute() {
        if path.exists() {
            return path.to_path_buf();
        }
        // Check if the path has a redundant component matching the working dir basename.
        // e.g., /home/user/myproject/myproject/src/main.rs -> /home/user/myproject/src/main.rs
        if let Ok(rel) = path.strip_prefix(working_dir)
            && let Some(first) = rel.components().next()
        {
            let first_name = first.as_os_str();
            if working_dir.file_name().map(|n| n == first_name).unwrap_or(false) {
                let fixed = working_dir.join(rel.strip_prefix(first_name).unwrap_or(rel));
                // Accept if the file exists OR its parent directory exists
                // (supports new file creation with redundant prefix)
                if fixed.exists() || fixed.parent().map(|p| p.is_dir()).unwrap_or(false) {
                    return fixed;
                }
            }
        }
        path.to_path_buf()
    } else {
        let joined = normalize_path(&working_dir.join(path));
        if joined.exists() {
            return joined;
        }
        // Check if first component matches working dir basename (redundant prefix)
        let mut components = path.components();
        if let Some(first) = components.next() {
            let first_name = first.as_os_str();
            if working_dir.file_name().map(|n| n == first_name).unwrap_or(false) {
                let rest: PathBuf = components.collect();
                if !rest.as_os_str().is_empty() {
                    let fixed = normalize_path(&working_dir.join(&rest));
                    if fixed.exists() || fixed.parent().map(|p| p.is_dir()).unwrap_or(false) {
                        return fixed;
                    }
                }
            }
        }
        joined
    }
}

/// Resolve a user-provided directory path against the working directory.
///
/// Same as [`resolve_file_path`] but optimized for directory paths. If a relative
/// path doesn't exist when joined with working_dir, checks if stripping a redundant
/// leading directory component (matching the working dir's basename) helps.
pub fn resolve_dir_path(user_path: &str, working_dir: &Path) -> PathBuf {
    let expanded = expand_home(user_path);
    // Rewrite hallucinated Docker prefixes to working_dir-relative paths
    let expanded = strip_hallucinated_prefix(&expanded, working_dir);
    let path = strip_curdir(Path::new(&expanded));
    let path = normalize_path(&path);
    let path = path.as_path();
    if path.is_absolute() {
        if path.is_dir() {
            return path.to_path_buf();
        }
        // Check if the path has a redundant component matching the working dir basename.
        if let Ok(rel) = path.strip_prefix(working_dir)
            && let Some(first) = rel.components().next()
        {
            let first_name = first.as_os_str();
            if working_dir.file_name().map(|n| n == first_name).unwrap_or(false) {
                let fixed = working_dir.join(rel.strip_prefix(first_name).unwrap_or(rel));
                if fixed.is_dir() || fixed.parent().map(|p| p.is_dir()).unwrap_or(false) {
                    return fixed;
                }
            }
        }
        // Absolute path doesn't exist as a directory — check if it matches
        // the working directory or is a parent prefix of it.
        if working_dir.starts_with(path) || working_dir == path {
            working_dir.to_path_buf()
        } else {
            path.to_path_buf()
        }
    } else {
        let joined = normalize_path(&working_dir.join(path));
        if joined.is_dir() {
            return joined;
        }
        // Check if first component matches working dir basename (redundant prefix)
        let mut components = path.components();
        if let Some(first) = components.next() {
            let first_name = first.as_os_str();
            if working_dir.file_name().map(|n| n == first_name).unwrap_or(false) {
                let rest: PathBuf = components.collect();
                if rest.as_os_str().is_empty() {
                    // Single component matching basename — fall back to cwd
                    return working_dir.to_path_buf();
                }
                let fixed = working_dir.join(&rest);
                if fixed.is_dir() || fixed.parent().map(|p| p.is_dir()).unwrap_or(false) {
                    return fixed;
                }
            }
        }
        joined
    }
}

/// Check if a path is outside the project working directory.
///
/// Returns `true` for paths outside the working directory that are also
/// not in well-known config locations (`~/.opendev/`, `~/.config/opendev/`, `/tmp/`).
/// Used by the react loop to prompt for user approval on external directory access.
pub fn is_external_path(resolved: &Path, working_dir: &Path) -> bool {
    let normalized = normalize_path(resolved);

    // Within working directory
    if normalized.starts_with(working_dir) {
        return false;
    }

    // Canonical forms for symlinks
    if let (Ok(canon_path), Ok(canon_wd)) = (normalized.canonicalize(), working_dir.canonicalize())
        && canon_path.starts_with(&canon_wd)
    {
        return false;
    }

    // Well-known config dirs (XDG and legacy)
    let paths = opendev_config::Paths::default();
    for prefix in paths.all_base_dirs() {
        if normalized.starts_with(&prefix) {
            return false;
        }
    }
    if let Some(home) = dirs::home_dir() {
        // Always allow legacy ~/.opendev and XDG ~/.config/opendev
        let allowed = [home.join(".opendev"), home.join(".config").join("opendev")];
        for prefix in &allowed {
            if normalized.starts_with(prefix) {
                return false;
            }
        }
    }

    // /tmp is always allowed
    if normalized.starts_with("/tmp") || normalized.starts_with("/var/tmp") {
        return false;
    }

    true
}

#[cfg(test)]
#[path = "path_tests.rs"]
mod tests;
