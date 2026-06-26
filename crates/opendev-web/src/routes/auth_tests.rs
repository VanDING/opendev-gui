use super::*;

#[test]
fn test_secret_key_uses_env_var() {
    // When env var is set, use it; otherwise fall back to debug default.
    let key = secret_key();
    // In debug builds, key is either the env var value or the default.
    // Either way it should be non-empty.
    assert!(!key.is_empty());
}

#[test]
fn test_secret_key_consistent() {
    // Multiple calls return the same key.
    assert_eq!(secret_key(), secret_key());
}

#[test]
fn test_build_session_cookie_properties() {
    let cookie = build_session_cookie("test-token-value");
    assert_eq!(cookie.name(), TOKEN_COOKIE);
    assert_eq!(cookie.value(), "test-token-value");
    assert!(cookie.http_only().unwrap_or(false));
    assert_eq!(cookie.path().unwrap_or(""), "/");
}

#[test]
fn test_build_session_cookie_secure_in_release() {
    let cookie = build_session_cookie("test-token");
    // In debug builds, Secure may be absent; in release, it must be set.
    // We test what actually happens — debug builds produce a cookie usable
    // for local development without HTTPS.
    if cfg!(debug_assertions) {
        // Debug: no Secure flag (HTTP localhost development)
        assert!(!cookie.secure().unwrap_or(false));
    } else {
        // Release: Secure flag present (HTTPS production)
        assert!(cookie.secure().unwrap_or(false));
    }
}

#[test]
fn test_token_roundtrip() {
    let user_id = uuid::Uuid::new_v4().to_string();
    let token = create_token(&user_id);
    let result = verify_token(&token).unwrap();
    assert_eq!(result, user_id);
}

#[test]
fn test_token_invalid_signature() {
    let token = create_token("test-user");
    // Corrupt the signature portion.
    let mut parts: Vec<&str> = token.splitn(2, '.').collect();
    parts[1] = "AAAA_invalid_sig";
    let corrupted = format!("{}.{}", parts[0], parts[1]);
    assert!(verify_token(&corrupted).is_err());
}

#[test]
fn test_token_bad_format() {
    assert!(verify_token("no-dot-here").is_err());
}

#[test]
fn test_password_hash_and_verify() {
    let password = "my-secret-password";
    let hash = hash_password(password).unwrap();
    assert!(verify_password(password, &hash));
    assert!(!verify_password("wrong-password", &hash));
}

#[test]
fn test_password_verify_bad_hash() {
    assert!(!verify_password("password", "not-a-valid-hash"));
}

#[test]
fn test_auth_response_serialize() {
    let resp = AuthResponse {
        username: "alice".to_string(),
        email: Some("alice@example.com".to_string()),
        role: "user".to_string(),
    };
    let json = serde_json::to_value(&resp).unwrap();
    assert_eq!(json["username"], "alice");
    assert_eq!(json["email"], "alice@example.com");
}

#[test]
fn test_auth_response_no_email() {
    let resp = AuthResponse { username: "bob".to_string(), email: None, role: "admin".to_string() };
    let json = serde_json::to_value(&resp).unwrap();
    assert_eq!(json["username"], "bob");
    assert!(json.get("email").is_none());
}
