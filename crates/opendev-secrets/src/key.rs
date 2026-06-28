use serde::{Serialize, Deserialize};

/// Secret namespace — categorizes what the secret is used for.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Namespace {
    /// LLM provider API keys
    Llm,
    /// Telegram bot tokens
    Telegram,
    /// MCP OAuth secrets
    Mcp,
    /// Web session signing HMAC keys
    Hmac,
    /// Web UI user passwords (already Argon2 hashed)
    WebUser,
}

impl Namespace {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Llm => "llm",
            Self::Telegram => "telegram",
            Self::Mcp => "mcp",
            Self::Hmac => "hmac",
            Self::WebUser => "web_user",
        }
    }
}

/// A typed key for looking up secrets.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SecretKey {
    namespace: Namespace,
    account: String,
}

impl SecretKey {
    pub fn new(namespace: Namespace, account: impl Into<String>) -> Self {
        Self {
            namespace,
            account: account.into(),
        }
    }

    /// Create a secret key for an LLM provider.
    pub fn llm(provider: &str) -> Self {
        Self::new(Namespace::Llm, provider.to_lowercase())
    }

    /// Create a secret key for the Telegram bot token.
    pub fn telegram() -> Self {
        Self::new(Namespace::Telegram, "bot".to_string())
    }

    /// Create a secret key for the HMAC session key.
    pub fn hmac_session() -> Self {
        Self::new(Namespace::Hmac, "session".to_string())
    }

    /// Create a secret key for an MCP server.
    pub fn mcp(server_name: &str) -> Self {
        Self::new(Namespace::Mcp, server_name.to_string())
    }

    pub fn namespace(&self) -> &Namespace {
        &self.namespace
    }

    pub fn account(&self) -> &str {
        &self.account
    }

    /// Convert to an environment variable name for this key.
    /// Used by EnvStore.
    pub fn to_env_var(&self) -> String {
        match &self.namespace {
            Namespace::Llm => format!("{}_API_KEY", self.account.to_uppercase()),
            Namespace::Telegram => "TELEGRAM_BOT_TOKEN".into(),
            Namespace::Hmac => "OPENDEV_SECRET_KEY".into(),
            Namespace::Mcp => format!("{}_CLIENT_SECRET", self.account.to_uppercase()),
            Namespace::WebUser => String::new(), // web users use Argon2, not env
        }
    }

    /// Get the keyring entry identifier.
    pub fn keyring_entry(&self) -> (&'static str, String) {
        ("com.opendev.desktop", format!("{}/{}", self.namespace.as_str(), self.account))
    }
}

impl std::fmt::Display for SecretKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.namespace.as_str(), self.account)
    }
}
