//! Per-command flag validation for read-only command detection.
//!
//! Checks specific dangerous flag combinations for commands that are
//! normally considered read-only but can have write-like behaviour
//! depending on their arguments (e.g. `xargs -i`, `fd --exec`,
//! `git diff -O <file>`).

use regex::Regex;
use std::sync::LazyLock;

// ---------------------------------------------------------------------------
// xargs flag patterns
// ---------------------------------------------------------------------------

/// `-i` (deprecated, BSD-style) — replaced by `-I`.
static RE_XARGS_I_STANDALONE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bxargs\b.*\s-i\b").unwrap());

/// `-i` in combined short flags like `-iI` or `-i{}`.
static RE_XARGS_I_COMBINED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bxargs\b.*-[a-z]*i[a-z]*").unwrap());

/// `-e` (deprecated, BSD-style) — replaced by `-E`.
static RE_XARGS_E_STANDALONE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bxargs\b.*\s-e\b").unwrap());

/// `-e` in combined short flags like `-eEOF`.
static RE_XARGS_E_COMBINED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bxargs\b.*-[a-z]*e[a-z]+").unwrap());

// ---------------------------------------------------------------------------
// fd / fdfind flag patterns
// ---------------------------------------------------------------------------

/// `-x` / `--exec` — execute command for each file.
static RE_FD_EXEC_SHORT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bfd\b.*(?:\s-x\b|\s--exec\b)").unwrap());

/// `-X` / `--exec-batch` — batch execute.
static RE_FD_EXEC_BATCH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bfd\b.*(?:\s-X\b|\s--exec-batch\b)").unwrap());

/// `-l` / `--list-details` — lists details (write-like side effects).
static RE_FD_LIST_DETAILS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bfd\b.*(?:\s-l\b|\s--list-details\b)").unwrap());

/// Same patterns for `fdfind` alias.
static RE_FDFIND_EXEC_SHORT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bfdfind\b.*(?:\s-x\b|\s--exec\b)").unwrap());

static RE_FDFIND_EXEC_BATCH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bfdfind\b.*(?:\s-X\b|\s--exec-batch\b)").unwrap());

static RE_FDFIND_LIST_DETAILS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bfdfind\b.*(?:\s-l\b|\s--list-details\b)").unwrap());

// ---------------------------------------------------------------------------
// git diff flag patterns
// ---------------------------------------------------------------------------

/// `-O<file>` or `-O <file>` — output to a file (write-like).
static RE_GIT_DIFF_O_FLAG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bgit\b.*\bdiff\b.*(?:\s-O\b|\s-O[^\s])").unwrap());

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Result of validating a read-only command's flags.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the command is safe to treat as read-only.
    pub safe: bool,
    /// Human-readable reason if not safe.
    pub reason: Option<String>,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Validate command flags for commands that have write-like variants.
///
/// Returns `{ safe: true, reason: None }` for commands that do not match
/// any known dangerous flag patterns.
pub fn validate_readonly_command(command: &str) -> ValidationResult {
    // -- xargs ----------------------------------------------------------------
    if RE_XARGS_I_STANDALONE.is_match(command)
        || RE_XARGS_I_COMBINED.is_match(command)
    {
        return ValidationResult {
            safe: false,
            reason: Some("xargs with -i flag is unsafe (use POSIX -I '{}' instead)".into()),
        };
    }
    if RE_XARGS_E_STANDALONE.is_match(command)
        || RE_XARGS_E_COMBINED.is_match(command)
    {
        return ValidationResult {
            safe: false,
            reason: Some("xargs with -e flag is unsafe (use POSIX -E 'EOF' instead)".into()),
        };
    }

    // -- fd -------------------------------------------------------------------
    if RE_FD_EXEC_SHORT.is_match(command) || RE_FDFIND_EXEC_SHORT.is_match(command) {
        return ValidationResult {
            safe: false,
            reason: Some("fd with --exec/-x flag can execute commands".into()),
        };
    }
    if RE_FD_EXEC_BATCH.is_match(command) || RE_FDFIND_EXEC_BATCH.is_match(command) {
        return ValidationResult {
            safe: false,
            reason: Some("fd with --exec-batch/-X flag can execute commands".into()),
        };
    }
    if RE_FD_LIST_DETAILS.is_match(command) || RE_FDFIND_LIST_DETAILS.is_match(command) {
        return ValidationResult {
            safe: false,
            reason: Some("fd with --list-details/-l flag lists file details (write-like)".into()),
        };
    }

    // -- git diff -------------------------------------------------------------
    if RE_GIT_DIFF_O_FLAG.is_match(command) {
        return ValidationResult {
            safe: false,
            reason: Some("git diff with -O flag writes to an output file".into()),
        };
    }

    ValidationResult {
        safe: true,
        reason: None,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- xargs -----------------------------------------------------------

    #[test]
    fn xargs_plain_is_safe() {
        let r = validate_readonly_command("xargs -I '{}' echo < list.txt");
        assert!(r.safe, "xargs -I should be safe");
    }

    #[test]
    fn xargs_e_flag_is_safe() {
        let r = validate_readonly_command("xargs -E 'EOF' echo < list.txt");
        assert!(r.safe, "xargs -E should be safe");
    }

    #[test]
    fn xargs_combined_is_safe() {
        let r = validate_readonly_command("xargs -I '{}' -E 'EOF' echo < list.txt");
        assert!(r.safe, "xargs with -I and -E should be safe");
    }

    #[test]
    fn xargs_i_flag_is_unsafe() {
        let r = validate_readonly_command("xargs -i echo {} < list.txt");
        assert!(!r.safe, "xargs -i should be unsafe");
        assert!(r.reason.as_ref().unwrap().contains("-i"), "reason should mention -i");
    }

    #[test]
    fn xargs_lowercase_e_flag_is_unsafe() {
        let r = validate_readonly_command("xargs -e echo < list.txt");
        assert!(!r.safe, "xargs -e should be unsafe");
        assert!(r.reason.as_ref().unwrap().contains("-e"), "reason should mention -e");
    }

    #[test]
    fn xargs_no_flags_is_safe() {
        let r = validate_readonly_command("xargs echo < list.txt");
        assert!(r.safe, "xargs with no flags should be safe");
    }

    // ---- fd ---------------------------------------------------------------

    #[test]
    fn fd_plain_is_safe() {
        let r = validate_readonly_command("fd . /tmp");
        assert!(r.safe, "plain fd should be safe");
    }

    #[test]
    fn fd_exec_flag_is_unsafe() {
        let r = validate_readonly_command("fd -x cat {}");
        assert!(!r.safe, "fd -x should be unsafe");
    }

    #[test]
    fn fd_long_exec_flag_is_unsafe() {
        let r = validate_readonly_command("fd --exec cat {}");
        assert!(!r.safe, "fd --exec should be unsafe");
    }

    #[test]
    fn fd_exec_batch_flag_is_unsafe() {
        let r = validate_readonly_command("fd -X cat {}");
        assert!(!r.safe, "fd -X should be unsafe");
    }

    #[test]
    fn fd_long_exec_batch_flag_is_unsafe() {
        let r = validate_readonly_command("fd --exec-batch cat {}");
        assert!(!r.safe, "fd --exec-batch should be unsafe");
    }

    #[test]
    fn fd_list_details_flag_is_unsafe() {
        let r = validate_readonly_command("fd -l .");
        assert!(!r.safe, "fd -l should be unsafe");
    }

    #[test]
    fn fd_long_list_details_flag_is_unsafe() {
        let r = validate_readonly_command("fd --list-details .");
        assert!(!r.safe, "fd --list-details should be unsafe");
    }

    #[test]
    fn fd_x_combined_flag_is_unsafe() {
        let r = validate_readonly_command("fd -x some_command");
        assert!(!r.safe, "fd -x should be flagged");
    }

    #[test]
    fn fd_exclude_not_confused_with_exec() {
        // -x is the exclude flag in old versions; we err on the side of caution
        // but this tests -x used in exclude-like context
        let r = validate_readonly_command("fd --exclude target");
        // --exclude doesn't match -x or --exec
        assert!(r.safe, "fd --exclude should be safe");
    }

    // ---- fdfind alias ----------------------------------------------------

    #[test]
    fn fdfind_plain_is_safe() {
        let r = validate_readonly_command("fdfind . /tmp");
        assert!(r.safe, "plain fdfind should be safe");
    }

    #[test]
    fn fdfind_exec_flag_is_unsafe() {
        let r = validate_readonly_command("fdfind -x cat {}");
        assert!(!r.safe, "fdfind -x should be unsafe");
    }

    #[test]
    fn fdfind_list_details_flag_is_unsafe() {
        let r = validate_readonly_command("fdfind -l .");
        assert!(!r.safe, "fdfind -l should be unsafe");
    }

    // ---- git diff --------------------------------------------------------

    #[test]
    fn git_diff_plain_is_safe() {
        let r = validate_readonly_command("git diff HEAD~1");
        assert!(r.safe, "plain git diff should be safe");
    }

    #[test]
    fn git_diff_s_flag_is_safe() {
        let r = validate_readonly_command("git diff -S 'pattern'");
        assert!(r.safe, "git diff -S should be safe");
    }

    #[test]
    fn git_diff_g_flag_is_safe() {
        let r = validate_readonly_command("git diff -G 'regex'");
        assert!(r.safe, "git diff -G should be safe");
    }

    #[test]
    fn git_diff_o_flag_is_unsafe() {
        let r = validate_readonly_command("git diff -O output.patch");
        assert!(!r.safe, "git diff -O should be unsafe");
    }

    #[test]
    fn git_diff_o_separate_arg_is_unsafe() {
        let r = validate_readonly_command("git diff -O output.patch HEAD~1");
        assert!(!r.safe, "git diff -O <file> should be unsafe");
    }

    #[test]
    fn git_other_subcommands_not_affected() {
        let r = validate_readonly_command("git status");
        assert!(r.safe, "git status should not be affected by git diff check");
    }

    // ---- unknown commands return safe ------------------------------------

    #[test]
    fn unknown_command_returns_safe() {
        let r = validate_readonly_command("unknown-cmd --foo");
        assert!(r.safe, "unknown commands should return safe");
    }

    #[test]
    fn empty_command_returns_safe() {
        let r = validate_readonly_command("");
        assert!(r.safe, "empty command should return safe");
    }

    // ---- edge cases ------------------------------------------------------

    #[test]
    fn xargs_uppercase_i_not_confused_with_i() {
        let r = validate_readonly_command("xargs -I {} echo");
        assert!(r.safe, "xargs -I (uppercase) should NOT be confused with -i");
    }

    #[test]
    fn xargs_uppercase_e_not_confused_with_e() {
        let r = validate_readonly_command("xargs -E EOF echo");
        assert!(r.safe, "xargs -E (uppercase) should NOT be confused with -e");
    }

    #[test]
    fn fd_no_flag_is_safe() {
        let r = validate_readonly_command("fd");
        assert!(r.safe, "fd with no args should be safe");
    }
}
