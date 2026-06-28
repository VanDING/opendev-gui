use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use zeroize::ZeroizeOnDrop;

/// A secret value that is automatically zeroized on drop.
/// Display and Debug both print `[REDACTED]` — never logs the actual secret.
#[derive(Clone, ZeroizeOnDrop)]
pub struct SecretValue(SecretString);

impl SecretValue {
    pub fn new(s: impl Into<String>) -> Self {
        Self(SecretString::new(s.into().into_boxed_str()))
    }

    /// Expose the secret value for use.
    /// WARNING: Callers must not log or persist the returned value.
    pub fn expose(&self) -> &str {
        self.0.expose_secret()
    }
}

// Constitution 7 implementation: never log secrets
impl std::fmt::Display for SecretValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl std::fmt::Debug for SecretValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SecretValue([REDACTED])")
    }
}

impl PartialEq for SecretValue {
    fn eq(&self, other: &Self) -> bool {
        self.expose() == other.expose()
    }
}

impl From<String> for SecretValue {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for SecretValue {
    fn from(s: &str) -> Self {
        Self::new(s.to_string())
    }
}

/// Serialize as the exposed secret (for controlled transmission).
/// WARNING: Be careful where you use this.
impl Serialize for SecretValue {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.expose())
    }
}

/// Deserialize from a string.
impl<'de> Deserialize<'de> for SecretValue {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(Self::new(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redacted_display() {
        let sv = SecretValue::new("sk-ant-test123");
        assert_eq!(format!("{}", sv), "[REDACTED]");
    }

    #[test]
    fn test_redacted_debug() {
        let sv = SecretValue::new("sk-ant-test123");
        assert_eq!(format!("{:?}", sv), "SecretValue([REDACTED])");
    }

    #[test]
    fn test_expose_works() {
        let sv = SecretValue::new("sk-ant-test123");
        assert_eq!(sv.expose(), "sk-ant-test123");
    }

    #[test]
    fn test_serialize_round_trip() {
        let sv = SecretValue::new("test-key");
        let json = serde_json::to_string(&sv).unwrap();
        assert_eq!(json, "\"test-key\"");
        let deserialized: SecretValue = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.expose(), "test-key");
    }
}
