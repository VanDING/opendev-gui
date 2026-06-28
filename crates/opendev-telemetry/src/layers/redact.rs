//! Redact layer — automatically redacts known sensitive fields from tracing events.
//!
//! Sensitive field name patterns: api_key, token, password, secret, bearer, key, etc.
//! When these fields appear in log events, their values are replaced with [REDACTED].

/// Check if a field name is sensitive and should be redacted.
pub fn is_sensitive_field(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("api_key")
        || lower.contains("token")
        || lower.contains("password")
        || lower.contains("secret")
        || lower == "bearer"
        || lower == "key"
        || lower.contains("client_secret")
        || lower.contains("bot_token")
        || lower.contains("private_key")
        || lower.contains("access_key")
        || lower.contains("oauth")
        || lower.contains("jwt")
        || lower.contains("session_key")
        || lower.contains("hmac")
        || lower.contains("master_key")
        || lower.contains("passphrase")
        || lower.contains("credential")
}

/// Redact sensitive values from a JSON value recursively.
pub fn redact_value(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let redacted: serde_json::Map<String, serde_json::Value> = map
                .into_iter()
                .map(|(k, v)| {
                    if is_sensitive_field(&k) {
                        (k, serde_json::Value::String("[REDACTED]".to_string()))
                    } else {
                        (k, redact_value(v))
                    }
                })
                .collect();
            serde_json::Value::Object(redacted)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(redact_value).collect())
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensitive_field_detection() {
        assert!(is_sensitive_field("api_key"));
        assert!(is_sensitive_field("openai_api_key"));
        assert!(is_sensitive_field("bot_token"));
        assert!(is_sensitive_field("client_secret"));
        assert!(is_sensitive_field("oauth_token"));
        assert!(!is_sensitive_field("model_provider"));
        assert!(!is_sensitive_field("temperature"));
        assert!(!is_sensitive_field("session_id"));
    }

    #[test]
    fn test_redact_value() {
        let input = serde_json::json!({
            "api_key": "sk-ant-test123",
            "model": "claude-3",
            "nested": {
                "token": "secret-value",
                "temperature": 0.7,
            }
        });
        let redacted = redact_value(input);
        assert_eq!(redacted["api_key"], "[REDACTED]");
        assert_eq!(redacted["model"], "claude-3");
        assert_eq!(redacted["nested"]["token"], "[REDACTED]");
        assert_eq!(redacted["nested"]["temperature"], 0.7);
    }
}
