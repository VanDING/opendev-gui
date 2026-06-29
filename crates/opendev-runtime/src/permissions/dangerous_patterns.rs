//! Detection of dangerous allow rules — patterns that grant overly broad
//! permissions and should trigger warnings or critical alerts.
//!
//! When users create fine-grained permission rules, some patterns are so
//! broad that they effectively bypass the permission system. This module
//! identifies those patterns so the UI can warn the user.

/// Severity level for a dangerous allow rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarningLevel {
    /// Moderate risk — broad but may be intentional.
    Warning,
    /// Severe risk — effectively bypasses the permission system.
    Critical,
}

/// A detected dangerous allow rule with its severity and explanation.
#[derive(Debug, Clone)]
pub struct DangerousAllowRule {
    /// The glob pattern that triggered the warning.
    pub pattern: String,
    /// How dangerous this pattern is.
    pub severity: WarningLevel,
    /// Human-readable explanation of why this pattern is dangerous.
    pub message: String,
}

/// Check if a tool permission pattern is dangerously broad.
///
/// Returns `Some(DangerousAllowRule)` if the pattern is dangerous, or `None`
/// if it appears safe.
///
/// # Examples
///
/// ```ignore
/// // "Bash(*)" → Critical — allows any command
/// // "Bash(python:*)" → Critical — allows arbitrary code execution
/// // "Agent(*)" → Critical — auto-approves sub-agent spawns
/// // "WebFetch(*)" → Warning — allows network exfiltration
/// ```
pub fn check_dangerous_pattern(
    tool_pattern: &str,
    arg_pattern: &Option<&str>,
) -> Option<DangerousAllowRule> {
    let pattern_str = match arg_pattern {
        Some(args) => format!("{tool_pattern}({args})"),
        None => tool_pattern.to_string(),
    };

    // ── Critical patterns ──────────────────────────────────────────────

    // Bash(*) or bare Bash — allows any shell command
    if tool_pattern.eq_ignore_ascii_case("Bash")
        && (arg_pattern.is_none() || arg_pattern.map_or(false, |a| a == "*"))
    {
        return Some(DangerousAllowRule {
            pattern: pattern_str,
            severity: WarningLevel::Critical,
            message: "Bash(*) allows execution of ANY shell command. \
                      Consider restricting to specific commands (e.g., 'Bash(ls *)')."
                .to_string(),
        });
    }

    // Bare "Bash" (no args pattern at all)
    if tool_pattern.eq_ignore_ascii_case("Bash") && arg_pattern.is_none() {
        return Some(DangerousAllowRule {
            pattern: pattern_str,
            severity: WarningLevel::Critical,
            message: "Bash allows execution of ANY shell command. \
                      Consider specifying allowed command patterns."
                .to_string(),
        });
    }

    // Bash(python:*) — arbitrary python execution
    if tool_pattern.eq_ignore_ascii_case("Bash")
        && arg_pattern.map_or(false, |a| a.eq_ignore_ascii_case("python:*"))
    {
        return Some(DangerousAllowRule {
            pattern: pattern_str,
            severity: WarningLevel::Critical,
            message: "Bash(python:*) allows execution of arbitrary Python code. \
                      This is equivalent to unrestricted code execution. \
                      Consider allowing specific Python scripts instead."
                .to_string(),
        });
    }

    // Agent(*) — auto-approves any sub-agent spawn
    if tool_pattern.eq_ignore_ascii_case("Agent")
        && (arg_pattern.is_none() || arg_pattern.map_or(false, |a| a == "*"))
    {
        return Some(DangerousAllowRule {
            pattern: pattern_str,
            severity: WarningLevel::Critical,
            message: "Agent(*) auto-approves ALL sub-agent spawns. \
                      Sub-agents can read, write, and execute code. \
                      Consider restricting to specific agent types."
                .to_string(),
        });
    }

    // ── Warning patterns ───────────────────────────────────────────────

    // WebFetch(*) — network exfiltration risk
    if tool_pattern.eq_ignore_ascii_case("WebFetch")
        && (arg_pattern.is_none() || arg_pattern.map_or(false, |a| a == "*"))
    {
        return Some(DangerousAllowRule {
            pattern: pattern_str,
            severity: WarningLevel::Warning,
            message: "WebFetch(*) allows fetching from ANY URL, creating a \
                      potential network exfiltration vector. \
                      Consider restricting to known endpoints."
                .to_string(),
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_bash_star_critical() {
        let result = check_dangerous_pattern("Bash", &Some("*"));
        assert!(result.is_some());
        let rule = result.unwrap();
        assert_eq!(rule.severity, WarningLevel::Critical);
        assert!(rule.message.contains("ANY shell command"));
    }

    #[test]
    fn detects_bare_bash_critical() {
        let result = check_dangerous_pattern("Bash", &None);
        assert!(result.is_some());
        let rule = result.unwrap();
        assert_eq!(rule.severity, WarningLevel::Critical);
    }

    #[test]
    fn detects_bash_python_star_critical() {
        let result = check_dangerous_pattern("Bash", &Some("python:*"));
        assert!(result.is_some());
        let rule = result.unwrap();
        assert_eq!(rule.severity, WarningLevel::Critical);
        assert!(rule.message.contains("Python code"));
    }

    #[test]
    fn detects_agent_star_critical() {
        let result = check_dangerous_pattern("Agent", &Some("*"));
        assert!(result.is_some());
        let rule = result.unwrap();
        assert_eq!(rule.severity, WarningLevel::Critical);
        assert!(rule.message.contains("sub-agent"));
    }

    #[test]
    fn detects_bare_agent_critical() {
        let result = check_dangerous_pattern("Agent", &None);
        assert!(result.is_some());
        let rule = result.unwrap();
        assert_eq!(rule.severity, WarningLevel::Critical);
    }

    #[test]
    fn detects_web_fetch_star_warning() {
        let result = check_dangerous_pattern("WebFetch", &Some("*"));
        assert!(result.is_some());
        let rule = result.unwrap();
        assert_eq!(rule.severity, WarningLevel::Warning);
        assert!(rule.message.contains("exfiltration"));
    }

    #[test]
    fn safe_patterns_return_none() {
        assert!(check_dangerous_pattern("Bash", &Some("ls *")).is_none());
        assert!(check_dangerous_pattern("Bash", &Some("cat *")).is_none());
        assert!(check_dangerous_pattern("Edit", &Some("src/**")).is_none());
        assert!(check_dangerous_pattern("Read", &Some("*.ts")).is_none());
        assert!(check_dangerous_pattern("Write", &Some("*.md")).is_none());
        assert!(check_dangerous_pattern("WebFetch", &Some("https://api.example.com/*")).is_none());
        assert!(check_dangerous_pattern("Agent", &Some("planner")).is_none());
    }

    #[test]
    fn case_insensitive_detection() {
        assert!(check_dangerous_pattern("bash", &Some("*")).is_some());
        assert!(check_dangerous_pattern("BASH", &Some("*")).is_some());
        assert!(check_dangerous_pattern("agent", &Some("*")).is_some());
        assert!(check_dangerous_pattern("webfetch", &Some("*")).is_some());
    }
}
