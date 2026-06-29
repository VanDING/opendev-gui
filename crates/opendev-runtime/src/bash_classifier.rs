//! Bash Permission Classifier.
//!
//! When a `Bash(prompt: …)` rule triggers, this classifier performs a small
//! synchronous heuristic-based side-query to evaluate whether the user's
//! described intent matches the actual command being executed.
//!
//! This is a heuristic-only classifier; no model calls are made.

use std::collections::HashSet;

/// Classification result from the bash classifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BashClassification {
    /// The command matches the user's described intent → allowed.
    Allowed,
    /// The command contains dangerous patterns → denied.
    Denied,
    /// Cannot determine — fail-closed to denied in production.
    Unknown,
}

/// Heuristic-based classifier that checks whether a bash command's intent
/// matches its description.
pub struct BashClassifier;

impl BashClassifier {
    /// Evaluate whether `command` aligns with `prompt_description`.
    ///
    /// Heuristics:
    /// 1. Extract the executable name from the command.
    /// 2. Extract keywords from the prompt description.
    /// 3. If command executable appears in description → `Allowed`.
    /// 4. If command contains dangerous patterns → `Denied`.
    /// 5. Otherwise → `Unknown` (fail-closed = Denied).
    pub fn evaluate(command: &str, prompt_description: &str) -> BashClassification {
        if command.trim().is_empty() {
            return BashClassification::Denied;
        }

        // Step 4: Check for dangerous patterns first (fail-closed).
        if contains_dangerous_pattern(command) {
            return BashClassification::Denied;
        }

        // Step 1 & 3: Extract executable name and check against description.
        let executable = extract_executable(command);
        if let Some(exe) = executable {
            let desc_lower = prompt_description.to_lowercase();
            let exe_lower = exe.to_lowercase();

            // If the executable name appears in the description → likely matching intent.
            if desc_lower.contains(&exe_lower) {
                return BashClassification::Allowed;
            }

            // Also check common command-description keyword mappings.
            if keyword_match(&exe_lower, &desc_lower) {
                return BashClassification::Allowed;
            }
        }

        // Step 5: Fallback.
        BashClassification::Unknown
    }
}

/// Extract the executable name (first token without path prefix) from a command.
fn extract_executable(command: &str) -> Option<String> {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Split on shell operators and take the first segment.
    let first_segment = trimmed
        .split(|c: char| c == ' ' || c == '\t')
        .next()?;

    // Strip path prefix (/usr/bin/git → git).
    let basename = first_segment.rsplit('/').next().unwrap_or(first_segment);

    if basename.is_empty() {
        return None;
    }

    Some(basename.to_string())
}

/// Keywords that map specific commands to common description wording.
fn keyword_match(executable: &str, description: &str) -> bool {
    // Map of executable → set of keywords that indicate aligned intent.
    let keyword_map: Vec<(&str, &[&str])> = vec![
        ("ls", &["list", "directory", "folder", "file", "show"]),
        ("cat", &["read", "view", "display", "show", "content"]),
        ("grep", &["search", "find", "pattern", "match"]),
        ("find", &["search", "find", "locate", "where"]),
        ("head", &["first", "top", "beginning", "start"]),
        ("tail", &["last", "end", "bottom", "recent", "log"]),
        ("echo", &["print", "display", "output", "show", "message"]),
        ("mkdir", &["create", "directory", "folder", "make"]),
        ("cp", &["copy", "duplicate", "backup"]),
        ("mv", &["move", "rename"]),
        ("rm", &["remove", "delete", "clean"]),
        ("git", &["git", "version control", "commit", "push", "pull", "branch"]),
        ("cargo", &["cargo", "rust", "build", "compile"]),
        ("npm", &["npm", "node", "package", "install"]),
        ("python", &["python", "script", "run"]),
        ("curl", &["curl", "http", "request", "fetch", "download"]),
        ("wget", &["download", "fetch", "http"]),
        ("docker", &["docker", "container", "image"]),
        ("sed", &["replace", "substitute", "edit", "stream"]),
        ("awk", &["process", "parse", "extract"]),
        ("sort", &["sort", "order"]),
        ("uniq", &["unique", "duplicate", "distinct"]),
        ("wc", &["count", "line", "word"]),
        ("diff", &["diff", "compare", "difference"]),
        ("ps", &["process", "running"]),
        ("kill", &["kill", "stop", "terminate", "process"]),
        ("sudo", &["sudo", "root", "superuser", "admin"]),
        ("chmod", &["permission", "mode", "chmod"]),
        ("chown", &["owner", "chown"]),
    ];

    for (exe, keywords) in &keyword_map {
        if executable == *exe {
            return keywords.iter().any(|kw| description.contains(kw));
        }
    }

    false
}

/// Check if a command contains dangerous patterns.
fn contains_dangerous_pattern(command: &str) -> bool {
    let trimmed = command.trim().to_lowercase();

    // Highly destructive commands.
    let dangerous_exes: HashSet<&str> = [
        "dd",
        "mkfs",
        "fdisk",
        "parted",
        "mkswap",
        "shred",
        "wipefs",
        "pvcreate",
        "vgcreate",
        "lvcreate",
        "pvremove",
        "vgremove",
        "lvremove",
    ]
    .iter().copied().collect();

    // Parse the first token to get the executable.
    if let Some(exe) = extract_executable(command) {
        let exe_lower = exe.to_lowercase();
        if dangerous_exes.contains(exe_lower.as_str()) {
            return true;
        }
    }

    // Check for dangerous patterns in the full command.
    // "rm -rf" or "rm -fr" or "rm -r -f" patterns.
    if trimmed.contains("rm -rf") || trimmed.contains("rm -fr") || trimmed.contains("rm -r -f") {
        return true;
    }

    // rm with force on root
    if trimmed.contains("rm ") && trimmed.contains(" --no-preserve-root") {
        return true;
    }

    // Direct format/dd commands.
    if trimmed.starts_with("format ")
        || trimmed.contains(" > /dev/sd")
        || trimmed.contains(" > /dev/nvme")
    {
        return true;
    }

    // Dangerous sudo patterns (sudo with destructive command).
    if trimmed.starts_with("sudo ") {
        let sudo_rest = trimmed
            .strip_prefix("sudo ")
            .map(|s| s.trim())
            .unwrap_or("");

        // Check if sudo is used with a dangerous exe.
        if let Some(sudo_exe) = extract_executable(sudo_rest) {
            let se = sudo_exe.to_lowercase();
            if dangerous_exes.contains(se.as_str()) || se == "rm" {
                return true;
            }
        }

        // Sudo with rm -rf /
        if sudo_rest.contains("rm -rf /") || sudo_rest.contains("rm -fr /") {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Basic classification tests ──

    #[test]
    fn empty_command_is_denied() {
        assert_eq!(BashClassifier::evaluate("", ""), BashClassification::Denied);
        assert_eq!(BashClassifier::evaluate("   ", "some description"), BashClassification::Denied);
    }

    #[test]
    fn matching_executable_is_allowed() {
        assert_eq!(
            BashClassifier::evaluate("ls -la", "List files in the current directory"),
            BashClassification::Allowed
        );
    }

    #[test]
    fn git_command_with_git_description_is_allowed() {
        assert_eq!(
            BashClassifier::evaluate("git status", "Check git status of the repository"),
            BashClassification::Allowed
        );
    }

    #[test]
    fn unknown_command_returns_unknown() {
        assert_eq!(
            BashClassifier::evaluate("some_obscure_tool --flag", "Do something random"),
            BashClassification::Unknown
        );
    }

    // ── Dangerous pattern tests ──

    #[test]
    fn rm_rf_is_denied() {
        assert_eq!(
            BashClassifier::evaluate("rm -rf /tmp/foo", "Remove temporary files"),
            BashClassification::Denied
        );
    }

    #[test]
    fn dd_command_is_denied() {
        assert_eq!(
            BashClassifier::evaluate("dd if=/dev/zero of=/dev/sda bs=4M", "Write zeros to disk"),
            BashClassification::Denied
        );
    }

    #[test]
    fn sudo_with_destructive_is_denied() {
        assert_eq!(
            BashClassifier::evaluate("sudo rm -rf /var/log", "Clean up logs as root"),
            BashClassification::Denied
        );
    }

    #[test]
    fn format_command_is_denied() {
        assert_eq!(
            BashClassifier::evaluate("format /dev/sdb1", "Format the USB drive"),
            BashClassification::Denied
        );
    }

    #[test]
    fn shred_is_denied() {
        assert_eq!(
            BashClassifier::evaluate("shred -u secret_file.txt", "Securely delete a file"),
            BashClassification::Denied
        );
    }

    // ── Keyword mapping tests ──

    #[test]
    fn cat_maps_to_read_keyword() {
        assert_eq!(
            BashClassifier::evaluate("cat main.rs", "Read the main.rs file contents"),
            BashClassification::Allowed
        );
    }

    #[test]
    fn grep_maps_to_search_keyword() {
        assert_eq!(
            BashClassifier::evaluate("grep -r 'TODO' src/", "Search for TODO in source files"),
            BashClassification::Allowed
        );
    }

    #[test]
    fn curl_maps_to_request_keyword() {
        assert_eq!(
            BashClassifier::evaluate("curl https://api.example.com", "Make an HTTP request to the API"),
            BashClassification::Allowed
        );
    }

    #[test]
    fn mkdir_maps_to_create_keyword() {
        assert_eq!(
            BashClassifier::evaluate("mkdir -p build/output", "Create a directory for build output"),
            BashClassification::Allowed
        );
    }

    // ── Executable extraction tests ──

    #[test]
    fn extracts_basename_from_full_path() {
        assert_eq!(extract_executable("/usr/bin/python3 script.py"), Some("python3".into()));
    }

    #[test]
    fn extracts_simple_command() {
        assert_eq!(extract_executable("ls -la"), Some("ls".into()));
    }

    #[test]
    fn extracts_none_for_empty() {
        assert_eq!(extract_executable(""), None);
        assert_eq!(extract_executable("   "), None);
    }

    // ── Dangerous pattern helper tests ──

    #[test]
    fn dangerous_sudo_without_destructive_is_not_flagged() {
        // "sudo" alone is not dangerous; it's sudo + destructive command.
        assert!(!contains_dangerous_pattern("sudo apt-get update"));
    }

    #[test]
    fn rm_without_force_is_not_dangerous() {
        assert!(!contains_dangerous_pattern("rm file.txt"));
    }

    #[test]
    fn simple_safe_commands_are_not_dangerous() {
        assert!(!contains_dangerous_pattern("ls -la"));
        assert!(!contains_dangerous_pattern("cat /etc/hosts"));
        assert!(!contains_dangerous_pattern("git status"));
    }

    // ── Keyword match tests ──

    #[test]
    fn keyword_match_found() {
        assert!(keyword_match("ls", "show directory listing"));
        assert!(keyword_match("cat", "read file content"));
        assert!(keyword_match("grep", "search for pattern"));
    }

    #[test]
    fn keyword_match_not_found() {
        assert!(!keyword_match("ls", "build the project"));
        assert!(!keyword_match("grep", "copy files"));
    }

    #[test]
    fn unknown_tool_has_no_keywords() {
        assert!(!keyword_match("unknown_tool", "do something"));
    }
}
