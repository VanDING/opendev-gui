//! Sed command validation — rejects substitution with the `e` (execute) flag.
//!
//! The `e` flag in `sed s///` causes the replacement pattern to be executed
//! as a shell command, enabling arbitrary code execution through what looks
//! like a normal text transformation.

use regex::Regex;

// ── Helpers and pre-compiled regexes ──────────────────────────────────────

/// Character class of valid sed substitution flags (common implementations).
/// Flags: g (global), p (print), I/i (case-insensitive), l (lower), n (numeric),
/// s (swap), w (write), 0-9 (nth occurrence), plus the dangerous `e` (execute).
/// The `e` flag we're looking for must appear IN the flag group, not as a
/// trailing character in a filename.
const SED_FLAG_CLASS: &str = r"[gpsIlnw0-9]*e";

/// Build a regex pattern that matches `s<delim>...<delim>...<delim><flags>` where
/// the flags contain the `e` (execute) character.
fn sed_e_pattern(delim: char) -> String {
    // Escape the delimiter for use in a character class and regex
    let d = if delim == '/' {
        "/".to_string()
    } else {
        format!("\\{}", delim)
    };
    // Pattern: s<delim>non-delim*<delim>non-delim*<delim>flags-containing-e
    // The [^<delim>]* captures pattern and replacement text
    format!(
        r"(?i)\bsed\b.*s{delim}[^{delim}]*{delim}[^{delim}]*{delim}{flags}",
        delim = d,
        flags = SED_FLAG_CLASS
    )
}


// ── Pre-compiled regexes ──────────────────────────────────────────────────

static RE_SED_E_SLASH: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(&sed_e_pattern('/')).unwrap());

static RE_SED_E_PIPE: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(&sed_e_pattern('|')).unwrap());

static RE_SED_E_HASH: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(&sed_e_pattern('#')).unwrap());

static RE_SED_E_COLON: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(&sed_e_pattern(':')).unwrap());

/// Validate a bash command containing `sed` for dangerous execute-flag usage.
///
/// Returns `Some(error_message)` if the command contains a sed substitution
/// with the `e` (execute) flag, or `None` if the command appears safe.
pub fn validate_sed_command(command: &str) -> Option<String> {
    if RE_SED_E_SLASH.is_match(command)
        || RE_SED_E_PIPE.is_match(command)
        || RE_SED_E_HASH.is_match(command)
        || RE_SED_E_COLON.is_match(command)
    {
        Some(
            "sed substitution with 'e' (execute) flag is blocked: \
             executes replacement text as a shell command"
                .into(),
        )
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_slash_e_flag() {
        let cmd = "sed 's/foo/bar/e' file.txt";
        assert!(validate_sed_command(cmd).is_some(), "should reject /e flag");
    }

    #[test]
    fn rejects_slash_ge_flag() {
        let cmd = "sed 's/foo/bar/ge' file.txt";
        assert!(validate_sed_command(cmd).is_some(), "should reject /ge flag");
    }

    #[test]
    fn rejects_pipe_e_flag() {
        let cmd = "sed 's|foo|bar|e' file.txt";
        assert!(validate_sed_command(cmd).is_some(), "should reject |e flag");
    }

    #[test]
    fn rejects_hash_e_flag() {
        let cmd = "sed 's#foo#bar#e' file.txt";
        assert!(validate_sed_command(cmd).is_some(), "should reject #e flag");
    }

    #[test]
    fn rejects_colon_e_flag() {
        let cmd = "sed 's:foo:bar:e' file.txt";
        assert!(validate_sed_command(cmd).is_some(), "should reject :e flag");
    }

    #[test]
    fn allows_normal_substitution() {
        let cmd = "sed 's/foo/bar/' file.txt";
        assert!(validate_sed_command(cmd).is_none(), "should allow normal s///");
    }

    #[test]
    fn allows_other_flags() {
        let cmd = "sed 's/foo/bar/gp' file.txt";
        assert!(validate_sed_command(cmd).is_none(), "should allow /gp flags");
    }

    #[test]
    fn allows_sed_without_substitution() {
        let cmd = "sed -n '1,10p' file.txt";
        assert!(validate_sed_command(cmd).is_none(), "should allow non-s// commands");
    }

    #[test]
    fn allows_nested_slash() {
        let cmd = "sed 's=/foo/bar=/baz=' file.txt";
        assert!(validate_sed_command(cmd).is_none(), "should allow = delimiter");
    }

    #[test]
    fn allows_echo_with_slash() {
        let cmd = "echo s/foo/bar/e";
        assert!(validate_sed_command(cmd).is_none(), "should not flag echo");
    }

    #[test]
    fn rejects_ged_flag() {
        // s/foo/bar/ged — e flag present in multi-flag
        let cmd = "sed 's/foo/bar/ged' file.txt";
        assert!(validate_sed_command(cmd).is_some(), "should reject /ged");
    }

    #[test]
    fn rejects_pipe_ged() {
        let cmd = "sed 's|foo|bar|ged' file.txt";
        assert!(validate_sed_command(cmd).is_some(), "should reject |ged");
    }

    #[test]
    fn allows_dash_e_flag() {
        // -e is the sed script flag, not substitution's e flag
        let cmd = "sed -e 's/foo/bar/' file.txt";
        assert!(validate_sed_command(cmd).is_none(), "should allow -e script flag");
    }

    #[test]
    fn rejects_with_dash_e_and_exec_flag() {
        let cmd = "sed -e 's/foo/bar/e' file.txt";
        assert!(validate_sed_command(cmd).is_some(), "should reject -e with /e");
    }
}
