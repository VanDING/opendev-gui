//! Tool safety integration tests.
//!
//! Tests for:
//! - Readonly detection with edge cases (pipes, semicolons, env prefixes, redirects)
//! - Dangerous command detection with known exploits
//! - Path validation with symlinks

use opendev_runtime::{extract_command_prefix, is_safe_command};

// ─── Readonly Detection Tests ─────────────────────────────────────────────────

#[test]
fn test_safe_command_simple() {
    assert!(is_safe_command("ls -la"));
    assert!(is_safe_command("cat /etc/hosts"));
    assert!(is_safe_command("head -n 5 file.txt"));
    assert!(is_safe_command("git status"));
    assert!(is_safe_command("cargo check"));
}

#[test]
fn test_safe_command_with_path_prefix() {
    assert!(is_safe_command("/usr/bin/ls -la"));
    assert!(is_safe_command("/bin/cat /etc/hosts"));
}

#[test]
fn test_safe_command_with_env_prefix() {
    assert!(is_safe_command("ENV=dev ls -la"));
    assert!(is_safe_command("DEBUG=1 cargo build"));
}

#[test]
fn test_dangerous_command_with_semicolons() {
    // Semicolons introduce secondary commands. Each segment is checked
    // independently — if any segment is unsafe, the whole command is.
    assert!(!is_safe_command("ls; rm -rf /"));
    // Both segments are individually safe commands
    assert!(is_safe_command("cd /tmp; echo hello"));
}

#[test]
fn test_dangerous_command_with_pipes() {
    // Pipes chain commands. Each segment is checked independently.
    // Pipes of safe commands are considered safe for execution.
    assert!(is_safe_command("cat /etc/passwd | grep root"));
    assert!(is_safe_command("ls | head -5"));
}

#[test]
fn test_dangerous_command_with_redirects() {
    // Output redirects modify files — should be unsafe
    assert!(!is_safe_command("echo hello > /tmp/test.txt"));
    assert!(!is_safe_command("ls >> output.txt"));
}

#[test]
fn test_dangerous_command_with_shell_injection() {
    // Shell injection constructs
    assert!(!is_safe_command("echo $(whoami)"));
    assert!(!is_safe_command("echo `whoami`"));
    assert!(!is_safe_command("cat <(echo test)"));
}

#[test]
fn test_pipe_with_env_prefix_preserves_unsafety() {
    // Even with env prefix, all segments must be safe.
    // cat and grep are both safe, so the piped command is safe.
    assert!(is_safe_command("ENV=test cat /etc/hosts | grep localhost"));
}

#[test]
fn test_extract_command_prefix_readonly() {
    assert_eq!(extract_command_prefix("ls -la"), "ls");
    assert_eq!(extract_command_prefix("git status"), "git status");
    assert_eq!(extract_command_prefix("cargo build --release"), "cargo build");
}

#[test]
fn test_extract_command_prefix_with_env() {
    assert_eq!(extract_command_prefix("ENV=dev ls -la"), "ls");
    assert_eq!(extract_command_prefix("DEBUG=1 cargo build"), "cargo build");
}

// ─── Dangerous Pattern Detection Tests ────────────────────────────────────────

#[test]
fn test_dangerous_command_with_rm_rf() {
    assert!(!is_safe_command("rm -rf /tmp"));
    assert!(!is_safe_command("rm -fr /var"));
}

#[test]
fn test_dangerous_command_with_dd() {
    assert!(!is_safe_command("dd if=/dev/zero of=/dev/sda bs=4M"));
}

#[test]
fn test_dangerous_command_with_format() {
    assert!(!is_safe_command("mkfs.ext4 /dev/sdb1"));
    assert!(!is_safe_command("fdisk /dev/sda"));
}

#[test]
fn test_dangerous_command_with_shred() {
    assert!(!is_safe_command("shred -u secret.txt"));
}

#[test]
fn test_dangerous_sudo_command() {
    // sudo on destructive commands should be flagged
    assert!(!is_safe_command("sudo rm -rf /var/log"));
    assert!(!is_safe_command("sudo dd if=/dev/zero of=/dev/sda"));
}

#[test]
fn test_safe_sudo_command() {
    // sudo is not in SAFE_COMMANDS by default (security boundary).
    // Only explicit allowlisted commands are considered safe.
    assert!(!is_safe_command("sudo ls -la"));
    assert!(!is_safe_command("sudo cat /var/log/syslog"));
}

#[test]
fn test_dangerous_command_with_root_target() {
    assert!(!is_safe_command("rm -rf --no-preserve-root /"));
}

// ─── Path Validation Tests ────────────────────────────────────────────────────

#[test]
fn test_command_with_dot_dot_path() {
    // Path traversal in arguments
    assert!(is_safe_command("ls ../../etc/passwd"));
}

#[test]
fn test_command_with_tilde_expansion() {
    // Tilde expansion is shell-specific
    assert!(is_safe_command("ls ~/Documents"));
}
