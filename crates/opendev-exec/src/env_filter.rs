use std::collections::HashMap;

/// Sensitive env var name suffixes — stripped by filtered_env().
const SENSITIVE_ENV_SUFFIXES: &[&str] = &[
    "_API_KEY",
    "_SECRET_KEY",
    "_SECRET",
    "_TOKEN",
    "_PASSWORD",
    "_CREDENTIALS",
    // ── Added per design §4.3.4 ──
    "_PRIVATE_KEY",
    "_CLIENT_SECRET",
    "_ACCESS_KEY",
    "_CONNECTION_STRING",
    "_KEYFILE",
    "_CERT",
    "_TLS_KEY",
    "_TLS_CERT",
    "_PASS",
];

/// Sensitive env var name exact matches — stripped by filtered_env().
const SENSITIVE_ENV_EXACT: &[&str] = &[
    // Existing
    "OPENAI_API_KEY",
    "ANTHROPIC_API_KEY",
    "AZURE_OPENAI_API_KEY",
    "GROQ_API_KEY",
    "MISTRAL_API_KEY",
    "DEEPINFRA_API_KEY",
    "OPENROUTER_API_KEY",
    "FIREWORKS_API_KEY",
    "GOOGLE_API_KEY",
    "GITHUB_TOKEN",
    "GH_TOKEN",
    "NPM_TOKEN",
    "PYPI_TOKEN",
    // ── Added per design §4.3.4 ──
    "OAUTH_TOKEN",
    "OAUTH_ACCESS_TOKEN",
    "OAUTH_REFRESH_TOKEN",
    "JWT_SECRET",
    "JWT_PRIVATE_KEY",
    "BEARER_TOKEN",
    "DATABASE_URL",
    "REDIS_URL",
    "POSTGRES_PASSWORD",
    "POSTGRES_URL",
    "AWS_ACCESS_KEY_ID",
    "AWS_SECRET_ACCESS_KEY",
    "AWS_SESSION_TOKEN",
    "GCP_SERVICE_ACCOUNT_KEY",
    "AZURE_CLIENT_SECRET",
    "SENTRY_DSN",
    "SENTRY_AUTH_TOKEN",
    "TELEGRAM_BOT_TOKEN",
    "HMAC_SECRET",
    "ENCRYPTION_KEY",
    "SSH_AUTH_SOCK",
    "SSH_AGENT_PID",
    "APP_SECRET",
    "MASTER_KEY",
];

/// Protected env vars — always passed through, never stripped.
const PROTECTED_ENV_PREFIXES: &[&str] = &[
    "PATH", "HOME", "USER", "LOGNAME", "SHELL", "LANG", "LC_", "TERM", "COLORTERM", "TMPDIR",
    "TMP", "TEMP", "XDG_", "DBUS_", "DISPLAY", "WAYLAND_", "SSH_TTY", "SSH_CONNECTION",
    "SSH_CLIENT", "PYTHONPATH", "CARGO_", "RUST_", "GOPATH", "NODE_PATH", "JAVA_HOME",
    "OPENDEV_", // Allow opendev-specific vars through (they're controlled by us)
];

/// Check if an env var name is sensitive and should be stripped.
pub fn is_sensitive_env(name: &str) -> bool {
    let upper = name.to_uppercase();

    // Exact match
    if SENSITIVE_ENV_EXACT.iter().any(|e| upper == *e) {
        return true;
    }

    // Suffix match
    if SENSITIVE_ENV_SUFFIXES
        .iter()
        .any(|suffix| upper.ends_with(suffix))
    {
        return true;
    }

    false
}

/// Check if an env var name is protected and should always pass through.
pub fn is_protected_env(name: &str) -> bool {
    let upper = name.to_uppercase();
    PROTECTED_ENV_PREFIXES
        .iter()
        .any(|prefix| upper.starts_with(prefix))
}

/// Filter environment variables: strip sensitive ones, keep protected and benign ones.
pub fn filtered_env() -> HashMap<String, String> {
    std::env::vars()
        .filter(|(k, _)| !is_sensitive_env(k) || is_protected_env(k))
        .collect()
}

/// Apply env filter to a Command — clears env then adds filtered + protected vars.
pub fn apply(cmd: &mut std::process::Command) {
    let safe_env = filtered_env();
    cmd.env_clear();
    for (k, v) in safe_env {
        cmd.env(k, v);
    }
    // Always set PYTHONUNBUFFERED
    cmd.env("PYTHONUNBUFFERED", "1");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_sensitive_env_suffixes() {
        assert!(is_sensitive_env("OPENAI_API_KEY"));
        assert!(is_sensitive_env("MY_SECRET_KEY"));
        assert!(is_sensitive_env("GITHUB_TOKEN"));
        assert!(is_sensitive_env("DATABASE_PASSWORD"));
        assert!(is_sensitive_env("AWS_PRIVATE_KEY"));
        assert!(is_sensitive_env("OAUTH_CLIENT_SECRET"));
    }

    #[test]
    fn test_is_sensitive_env_exact() {
        assert!(is_sensitive_env("DATABASE_URL"));
        assert!(is_sensitive_env("REDIS_URL"));
        assert!(is_sensitive_env("AWS_ACCESS_KEY_ID"));
        assert!(is_sensitive_env("SENTRY_DSN"));
        assert!(is_sensitive_env("JWT_SECRET"));
    }

    #[test]
    fn test_is_not_sensitive_env() {
        assert!(!is_sensitive_env("PATH"));
        assert!(!is_sensitive_env("HOME"));
        assert!(!is_sensitive_env("USER"));
        assert!(!is_sensitive_env("LANG"));
        assert!(!is_sensitive_env("TERM"));
        assert!(!is_sensitive_env("CARGO_HOME"));
        assert!(!is_sensitive_env("RUST_LOG"));
    }

    #[test]
    fn test_is_protected_env() {
        assert!(is_protected_env("PATH"));
        assert!(is_protected_env("HOME"));
        assert!(is_protected_env("XDG_CONFIG_HOME"));
        assert!(is_protected_env("CARGO_HOME"));
        assert!(is_protected_env("RUST_BACKTRACE"));
        assert!(is_protected_env("OPENDEV_DIR"));
    }
}
