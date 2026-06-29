//! OAuth 2.0 + PKCE Authorization Code Flow for MCP server authentication.
//!
//! Provides the machinery for the Authorization Code flow with PKCE,
//! including a local redirect server and token caching.

use secrecy::SecretString;

use crate::config::McpOAuthConfig;
use crate::error::McpError;

// ---------------------------------------------------------------------------
// Auth flow detection
// ---------------------------------------------------------------------------

/// The OAuth 2.0 flow type for an MCP server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpAuthFlow {
    /// Client Credentials flow (no user interaction).
    ClientCredentials,
    /// Authorization Code flow with PKCE (requires browser).
    AuthorizationCode,
    /// No authentication configured.
    None,
}

/// Detect which OAuth flow to use based on the configuration.
///
/// Returns `AuthorizationCode` if `authorization_url` is set,
/// `ClientCredentials` if only `token_url` is set,
/// and `None` if no OAuth config is provided.
pub fn detect_auth_flow(oauth: &McpOAuthConfig) -> McpAuthFlow {
    if oauth.authorization_url.is_some() {
        McpAuthFlow::AuthorizationCode
    } else if !oauth.token_url.is_empty() {
        McpAuthFlow::ClientCredentials
    } else {
        McpAuthFlow::None
    }
}

// ---------------------------------------------------------------------------
// Authorization Code config
// ---------------------------------------------------------------------------

/// Configuration for the Authorization Code flow.
#[derive(Debug, Clone)]
pub struct AuthorizationCodeConfig {
    /// Authorization endpoint URL.
    pub authorization_url: String,
    /// Token endpoint URL.
    pub token_url: String,
    /// OAuth client ID.
    pub client_id: String,
    /// OAuth client secret.
    pub client_secret: SecretString,
    /// Optional redirect URI (defaults to the local server URL).
    pub redirect_uri: Option<String>,
    /// Requested OAuth scopes.
    pub scopes: Vec<String>,
}

impl From<&McpOAuthConfig> for AuthorizationCodeConfig {
    fn from(oauth: &McpOAuthConfig) -> Self {
        Self {
            authorization_url: oauth.authorization_url.clone().unwrap_or_default(),
            token_url: oauth.token_url.clone(),
            client_id: oauth.client_id.clone(),
            client_secret: oauth.client_secret.clone(),
            redirect_uri: None,
            scopes: oauth
                .scope
                .as_deref()
                .unwrap_or("")
                .split(' ')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect(),
        }
    }
}

// ---------------------------------------------------------------------------
// PKCE helpers
// ---------------------------------------------------------------------------

const PKCE_VERIFIER_CHARS: &[u8] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";

/// Generate a PKCE code verifier and its corresponding code challenge.
///
/// Returns `(code_verifier, code_challenge)` where:
/// - `code_verifier`: 64 random characters from the unreserved set
/// - `code_challenge`: base64url-encoded SHA-256 hash of the verifier
pub fn generate_pkce_pair() -> (String, String) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let verifier: String = (0..64)
        .map(|_| {
            let idx = rng.gen_range(0..PKCE_VERIFIER_CHARS.len());
            PKCE_VERIFIER_CHARS[idx] as char
        })
        .collect();

    let challenge = pkce_challenge(&verifier);
    (verifier, challenge)
}

/// Compute the PKCE code challenge from a code verifier.
fn pkce_challenge(verifier: &str) -> String {
    use sha2::Digest;
    let hash = sha2::Sha256::digest(verifier.as_bytes());
    base64_url_encode(&hash)
}

/// Base64url-encode a byte slice (no padding).
fn base64_url_encode(data: &[u8]) -> String {
    use base64::Engine;
    let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    engine.encode(data)
}

// ---------------------------------------------------------------------------
// Local redirect server
// ---------------------------------------------------------------------------

/// Start a local HTTP redirect server for the OAuth callback.
///
/// Binds a `TcpListener` on a random available port, returns the port
/// number and a `oneshot::Receiver` that will deliver the authorization
/// `code` extracted from the query parameters.
pub async fn start_redirect_server() -> Result<(u16, tokio::sync::oneshot::Receiver<String>), String>
{
    use tokio::sync::oneshot;

    // Bind on random port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| format!("Failed to bind redirect server: {e}"))?;
    let port = listener.local_addr().map_err(|e| format!("Failed to get local addr: {e}"))?.port();

    let (tx, rx) = oneshot::channel();

    tokio::spawn(async move {
        match listener.accept().await {
            Ok((mut stream, _)) => {
                // Read the HTTP request
                let mut buf = vec![0u8; 4096];
                let n = match tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await {
                    Ok(n) if n > 0 => n,
                    _ => {
                        let _ = tx.send(String::new());
                        return;
                    }
                };

                let request = String::from_utf8_lossy(&buf[..n]).to_string();

                // Extract the authorization code from the query string
                let code = extract_code_from_request(&request);

                // Send a response (user-facing HTML or plain text)
                let response = if code.is_empty() {
                    "HTTP/1.1 400 Bad Request\r\nContent-Length: 36\r\n\r\n\
                     Missing authorization code parameter."
                } else {
                    "HTTP/1.1 200 OK\r\nContent-Length: 36\r\n\r\n\
                     Authorization complete. You may close this tab."
                };

                let _ = tokio::io::AsyncWriteExt::write_all(&mut stream, response.as_bytes()).await;

                let _ = tx.send(code);
            }
            Err(_) => {
                let _ = tx.send(String::new());
            }
        }
    });

    Ok((port, rx))
}

/// Extract the `code` query parameter from an HTTP GET request.
fn extract_code_from_request(request: &str) -> String {
    // Parse the first line: GET /?code=... HTTP/1.1
    let line = request.lines().next().unwrap_or("");
    let path = line.split_whitespace().nth(1).unwrap_or("");
    let query_start = path.find('?');
    match query_start {
        Some(pos) => {
            let query = &path[pos + 1..];
            for pair in query.split('&') {
                let mut parts = pair.splitn(2, '=');
                if parts.next().unwrap_or("") == "code" {
                    let code = parts.next().unwrap_or("");
                    return urlencoding_decode(code).unwrap_or_else(|| code.to_string());
                }
            }
            String::new()
        }
        None => String::new(),
    }
}

/// Simple percent-decoding for the authorization code.
fn urlencoding_decode(s: &str) -> Option<String> {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if hex.len() == 2 {
                let byte = u8::from_str_radix(&hex, 16).ok()?;
                result.push(byte as char);
            } else {
                return None;
            }
        } else {
            result.push(c);
        }
    }
    Some(result)
}

// ---------------------------------------------------------------------------
// Token cache
// ---------------------------------------------------------------------------

/// In-memory cache for OAuth tokens.
#[derive(Debug, Clone)]
pub struct McpTokenCache {
    /// The cached access token.
    pub access_token: SecretString,
    /// Optional refresh token.
    pub refresh_token: Option<SecretString>,
    /// When the access token expires (if known).
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl McpTokenCache {
    /// Create a new token cache.
    pub fn new(
        access_token: impl Into<String>,
        refresh_token: Option<impl Into<String>>,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Self {
        Self {
            access_token: SecretString::new(access_token.into().into_boxed_str()),
            refresh_token: refresh_token.map(|s| SecretString::new(s.into().into_boxed_str())),
            expires_at,
        }
    }

    /// Check if the access token is expired.
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(exp) => chrono::Utc::now() >= exp,
            None => false,
        }
    }

    /// Get the access token string (exposed for use in HTTP headers).
    pub fn access_token_str(&self) -> &str {
        secrecy::ExposeSecret::expose_secret(&self.access_token)
    }

    /// Get the refresh token string, if available.
    pub fn refresh_token_str(&self) -> Option<&str> {
        self.refresh_token.as_ref().map(|s| secrecy::ExposeSecret::expose_secret(s))
    }
}

// ---------------------------------------------------------------------------
// Token refresh and exchange
// ---------------------------------------------------------------------------

/// Build a URL-encoded body string for a POST request.
fn urlencode_body(pairs: &[(&str, &str)]) -> String {
    let mut body = String::new();
    for (i, (key, val)) in pairs.iter().enumerate() {
        if i > 0 {
            body.push('&');
        }
        body.push_str(&urlencode_param(key));
        body.push('=');
        body.push_str(&urlencode_param(val));
    }
    body
}

/// Percent-encode a single form parameter value.
fn urlencode_param(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                result.push(byte as char);
            }
            b' ' => result.push('+'),
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

/// Post a URL-encoded form body to a token endpoint and parse the JSON response.
async fn post_token_request(
    token_url: &str,
    body_pairs: &[(&str, &str)],
) -> std::result::Result<serde_json::Value, McpError> {
    let client = reqwest::Client::new();
    let body = urlencode_body(body_pairs);

    let response = client
        .post(token_url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .map_err(|e| McpError::Transport(format!("Token request failed: {e}")))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(McpError::Transport(format!("Token request returned HTTP {status}: {text}")));
    }

    response
        .json()
        .await
        .map_err(|e| McpError::Transport(format!("Failed to parse token response: {e}")))
}

/// Parse the common fields from a token endpoint JSON response.
fn parse_token_response(json: &serde_json::Value) -> std::result::Result<McpTokenCache, McpError> {
    let access_token = json
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::Transport("No access_token in response".to_string()))?;

    let refresh_token = json.get("refresh_token").and_then(|v| v.as_str()).map(|s| s.to_string());

    let expires_at = json
        .get("expires_in")
        .and_then(|v| v.as_i64())
        .map(|secs| chrono::Utc::now() + chrono::Duration::seconds(secs));

    Ok(McpTokenCache::new(access_token.to_string(), refresh_token, expires_at))
}

/// Refresh an access token using a refresh token.
///
/// POSTs to the token endpoint with `grant_type=refresh_token` and returns
/// a new [`McpTokenCache`] with the fresh tokens.
pub async fn refresh_access_token(
    refresh_token: &str,
    token_url: &str,
    client_id: &str,
) -> std::result::Result<McpTokenCache, McpError> {
    tracing::info!(token_url = %token_url, "Attempting token refresh");

    let json = post_token_request(
        token_url,
        &[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", client_id),
        ],
    )
    .await?;

    let result = parse_token_response(&json)?;
    tracing::info!("Token refresh succeeded");
    Ok(result)
}

/// Exchange an authorization code for tokens (Authorization Code flow).
///
/// POSTs to the token endpoint with `grant_type=authorization_code`,
/// the authorization `code`, `code_verifier` (PKCE), and `redirect_uri`.
pub async fn exchange_auth_code(
    code: &str,
    code_verifier: &str,
    redirect_uri: &str,
    config: &AuthorizationCodeConfig,
) -> std::result::Result<McpTokenCache, McpError> {
    use secrecy::ExposeSecret;

    tracing::info!(
        token_url = %config.token_url,
        "Exchanging authorization code for tokens"
    );

    let mut pairs = vec![
        ("grant_type", "authorization_code"),
        ("code", code),
        ("code_verifier", code_verifier),
        ("redirect_uri", redirect_uri),
        ("client_id", &config.client_id),
    ];

    let secret = ExposeSecret::expose_secret(&config.client_secret);
    if !secret.is_empty() {
        pairs.push(("client_secret", secret));
    }

    let json = post_token_request(&config.token_url, &pairs).await?;
    let result = parse_token_response(&json)?;
    tracing::info!("Authorization code exchange succeeded");
    Ok(result)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use super::*;

    #[test]
    fn test_detect_auth_flow_authorization_code() {
        let oauth = McpOAuthConfig {
            client_id: "test".to_string(),
            client_secret: SecretString::new(Box::from("secret")),
            authorization_url: Some("https://auth.example.com/authorize".to_string()),
            token_url: "https://auth.example.com/token".to_string(),
            scope: Some("read write".to_string()),
        };
        assert_eq!(detect_auth_flow(&oauth), McpAuthFlow::AuthorizationCode);
    }

    #[test]
    fn test_detect_auth_flow_client_credentials() {
        let oauth = McpOAuthConfig {
            client_id: "test".to_string(),
            client_secret: SecretString::new(Box::from("secret")),
            authorization_url: None,
            token_url: "https://auth.example.com/token".to_string(),
            scope: None,
        };
        assert_eq!(detect_auth_flow(&oauth), McpAuthFlow::ClientCredentials);
    }

    #[test]
    fn test_detect_auth_flow_none() {
        let oauth = McpOAuthConfig {
            client_id: "test".to_string(),
            client_secret: SecretString::new(Box::from("secret")),
            authorization_url: None,
            token_url: String::new(),
            scope: None,
        };
        assert_eq!(detect_auth_flow(&oauth), McpAuthFlow::None);
    }

    #[test]
    fn test_generate_pkce_pair_lengths() {
        let (verifier, challenge) = generate_pkce_pair();
        assert!(
            (43..=128).contains(&verifier.len()),
            "verifier should be 43-128 chars, got {}",
            verifier.len()
        );
        assert!(!challenge.is_empty(), "challenge should not be empty");
        assert_ne!(verifier, challenge, "verifier and challenge should differ");
    }

    #[test]
    fn test_pkce_challenge_deterministic() {
        let verifier = "test-verifier-1234567890123456789012345678901234567890";
        let challenge1 = pkce_challenge(verifier);
        let challenge2 = pkce_challenge(verifier);
        assert_eq!(challenge1, challenge2, "challenge should be deterministic");
    }

    #[test]
    fn test_token_cache_not_expired_when_no_expiry() {
        let cache = McpTokenCache::new("token", None::<String>, None);
        assert!(!cache.is_expired(), "token with no expiry should not be expired");
        assert_eq!(cache.access_token_str(), "token");
    }

    #[test]
    fn test_token_cache_expired() {
        let past = chrono::Utc::now() - chrono::Duration::hours(1);
        let cache = McpTokenCache::new("token", None::<String>, Some(past));
        assert!(cache.is_expired(), "token with past expiry should be expired");
    }

    #[test]
    fn test_token_cache_not_expired_future() {
        let future = chrono::Utc::now() + chrono::Duration::hours(1);
        let cache = McpTokenCache::new("token", Some("refresh"), Some(future));
        assert!(!cache.is_expired(), "token with future expiry should not be expired");
        assert_eq!(cache.refresh_token_str(), Some("refresh"));
    }

    #[test]
    fn test_extract_code_from_request() {
        let req = "GET /callback?code=abc123&state=xyz HTTP/1.1\r\nHost: localhost\r\n";
        assert_eq!(extract_code_from_request(req), "abc123");
    }

    #[test]
    fn test_extract_code_from_request_no_code() {
        let req = "GET /callback?state=xyz HTTP/1.1\r\n";
        assert_eq!(extract_code_from_request(req), "");
    }

    #[test]
    fn test_extract_code_from_request_no_query() {
        let req = "GET /callback HTTP/1.1\r\n";
        assert_eq!(extract_code_from_request(req), "");
    }

    #[test]
    fn test_urlencoding_decode() {
        assert_eq!(urlencoding_decode("abc%20123").as_deref(), Some("abc 123"));
        assert_eq!(urlencoding_decode("simple").as_deref(), Some("simple"));
        assert_eq!(urlencoding_decode("%25").as_deref(), Some("%"));
        assert!(urlencoding_decode("%XY").is_none());
    }

    #[test]
    fn test_authorization_code_config_from_oauth() {
        let oauth = McpOAuthConfig {
            client_id: "my-client".to_string(),
            client_secret: SecretString::new(Box::from("my-secret")),
            authorization_url: Some("https://auth.example.com/authorize".to_string()),
            token_url: "https://auth.example.com/token".to_string(),
            scope: Some("openid profile".to_string()),
        };
        let config = AuthorizationCodeConfig::from(&oauth);
        assert_eq!(config.authorization_url, "https://auth.example.com/authorize");
        assert_eq!(config.client_id, "my-client");
        assert_eq!(config.scopes, vec!["openid", "profile"]);
    }
}
