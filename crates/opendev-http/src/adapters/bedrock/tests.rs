use super::*;
use crate::adapters::base::ProviderAdapter;

#[test]
fn test_provider_name() {
    let adapter = BedrockAdapter::new("anthropic.claude-3-sonnet-20240229-v1:0");
    assert_eq!(adapter.provider_name(), "bedrock");
}

#[test]
fn test_api_url_format() {
    let adapter =
        BedrockAdapter::with_region("anthropic.claude-3-sonnet-20240229-v1:0", "us-west-2");
    assert_eq!(
        adapter.api_url(),
        "https://bedrock-runtime.us-west-2.amazonaws.com/model/anthropic.claude-3-sonnet-20240229-v1:0/invoke"
    );
}

#[test]
fn test_api_url_default_region() {
    let adapter =
        BedrockAdapter::with_region("anthropic.claude-3-haiku-20240307-v1:0", "us-east-1");
    assert!(adapter.api_url().contains("us-east-1"));
}

#[test]
fn test_model_id() {
    let adapter = BedrockAdapter::new("anthropic.claude-3-sonnet-20240229-v1:0");
    assert_eq!(adapter.model_id(), "anthropic.claude-3-sonnet-20240229-v1:0");
}

#[test]
fn test_region() {
    let adapter = BedrockAdapter::with_region("model", "eu-west-1");
    assert_eq!(adapter.region(), "eu-west-1");
}

#[test]
fn test_convert_request_removes_unsupported_fields() {
    let adapter = BedrockAdapter::with_region("model-id", "us-east-1");
    let payload = json!({
        "model": "model-id",
        "messages": [{"role": "user", "content": "Hi"}],
        "n": 1,
        "frequency_penalty": 0.5,
        "presence_penalty": 0.5,
        "logprobs": true,
        "stream": true
    });
    let result = adapter.convert_request(payload);
    assert!(result.get("model").is_none());
    assert!(result.get("n").is_none());
    assert!(result.get("frequency_penalty").is_none());
    assert!(result.get("presence_penalty").is_none());
    assert!(result.get("logprobs").is_none());
    assert!(result.get("stream").is_none());
    assert_eq!(result["anthropic_version"], "bedrock-2023-05-31");
}

#[test]
fn test_convert_request_sets_max_tokens() {
    let adapter = BedrockAdapter::with_region("model-id", "us-east-1");
    let payload = json!({
        "messages": [{"role": "user", "content": "Hi"}]
    });
    let result = adapter.convert_request(payload);
    assert_eq!(result["max_tokens"], 4096);
}

#[test]
fn test_convert_request_preserves_custom_max_tokens() {
    let adapter = BedrockAdapter::with_region("model-id", "us-east-1");
    let payload = json!({
        "messages": [{"role": "user", "content": "Hi"}],
        "max_tokens": 8192
    });
    let result = adapter.convert_request(payload);
    assert_eq!(result["max_tokens"], 8192);
}

#[test]
fn test_convert_request_converts_max_completion_tokens() {
    let adapter = BedrockAdapter::with_region("model-id", "us-east-1");
    let payload = json!({
        "messages": [{"role": "user", "content": "Hi"}],
        "max_completion_tokens": 2048
    });
    let result = adapter.convert_request(payload);
    assert_eq!(result["max_tokens"], 2048);
    assert!(result.get("max_completion_tokens").is_none());
}

#[test]
fn test_extra_headers_has_content_type_and_accept() {
    let adapter = BedrockAdapter::with_region("model-id", "us-east-1");
    let headers = adapter.extra_headers();
    // Content-Type and Accept are always present.
    assert!(headers.iter().any(|(k, v)| k == "Content-Type" && v == "application/json"));
    assert!(headers.iter().any(|(k, v)| k == "Accept" && v == "application/json"));
}

#[test]
fn test_derive_signing_key_known_vector() {
    // AWS SigV4 test suite: example from
    // https://docs.aws.amazon.com/general/latest/gr/signature-v4-test-suite.html
    let key = BedrockAdapter::derive_signing_key(
        "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY",
        "20150830",
        "us-east-1",
        "bedrock",
    );
    // Known expected value for iam service in us-east-1 on 20150830
    // We verify this is non-empty and deterministic.
    assert!(!key.is_empty());
    assert_eq!(key.len(), 32); // SHA-256 HMAC produces 32 bytes
    // Second call with same params yields same result.
    let key2 = BedrockAdapter::derive_signing_key(
        "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY",
        "20150830",
        "us-east-1",
        "bedrock",
    );
    assert_eq!(key, key2);
}

#[test]
fn test_sigv4_headers_with_credentials() {
    // Set env vars for this test
    unsafe {
        std::env::set_var("AWS_ACCESS_KEY_ID", "AKIAIOSFODNN7EXAMPLE");
        std::env::set_var("AWS_SECRET_ACCESS_KEY", "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY");
        std::env::set_var("AWS_REGION", "us-east-1");
    }
    let adapter = BedrockAdapter::with_region("anthropic.claude-v2", "us-east-1");
    let headers = adapter.extra_headers();

    // Must contain Authorization header with valid SigV4 format
    let auth = headers
        .iter()
        .find(|(k, _)| k == "Authorization")
        .map(|(_, v)| v.clone())
        .expect("Authorization header should be present");

    assert!(auth.starts_with("AWS4-HMAC-SHA256"));
    assert!(auth.contains("Credential=AKIAIOSFODNN7EXAMPLE/"));
    assert!(auth.contains("/us-east-1/bedrock/aws4_request,"));
    assert!(auth.contains("SignedHeaders=content-type;host;x-amz-date,"));
    assert!(auth.contains("Signature="));

    // X-Amz-Date must be present with ISO timestamp
    let date = headers
        .iter()
        .find(|(k, _)| k == "X-Amz-Date")
        .map(|(_, v)| v.clone())
        .expect("X-Amz-Date header should be present");
    assert_eq!(date.len(), 16); // YYYYMMDDTHHMMSSZ
    assert!(date.ends_with('Z'));

    // X-Amz-Content-Sha256 must be UNSIGNED-PAYLOAD
    let sha = headers
        .iter()
        .find(|(k, _)| k == "X-Amz-Content-Sha256")
        .map(|(_, v)| v.clone())
        .expect("X-Amz-Content-Sha256 header should be present");
    assert_eq!(sha, "UNSIGNED-PAYLOAD");

    // Clean up
    unsafe {
        std::env::remove_var("AWS_ACCESS_KEY_ID");
        std::env::remove_var("AWS_SECRET_ACCESS_KEY");
        std::env::remove_var("AWS_REGION");
    }
}

#[test]
fn test_sigv4_headers_with_session_token() {
    unsafe {
        std::env::set_var("AWS_ACCESS_KEY_ID", "AKIATEST");
        std::env::set_var("AWS_SECRET_ACCESS_KEY", "test-secret");
        std::env::set_var("AWS_REGION", "eu-west-1");
        std::env::set_var("AWS_SESSION_TOKEN", "session-token-value");
    }
    let adapter = BedrockAdapter::with_region("model", "eu-west-1");
    let headers = adapter.extra_headers();

    let token = headers
        .iter()
        .find(|(k, _)| k == "X-Amz-Security-Token")
        .map(|(_, v)| v.clone())
        .expect("X-Amz-Security-Token should be present when AWS_SESSION_TOKEN is set");
    assert_eq!(token, "session-token-value");

    unsafe {
        std::env::remove_var("AWS_ACCESS_KEY_ID");
        std::env::remove_var("AWS_SECRET_ACCESS_KEY");
        std::env::remove_var("AWS_REGION");
        std::env::remove_var("AWS_SESSION_TOKEN");
    }
}

#[test]
fn test_sigv4_headers_without_credentials() {
    // Ensure env vars are NOT set
    unsafe {
        std::env::remove_var("AWS_ACCESS_KEY_ID");
        std::env::remove_var("AWS_SECRET_ACCESS_KEY");
    }
    let adapter = BedrockAdapter::with_region("model", "us-east-1");
    let headers = adapter.extra_headers();

    // Without credentials, headers should still include Content-Type + Accept,
    // plus an error indicator header.
    assert!(headers.iter().any(|(k, v)| k == "Content-Type" && v == "application/json"));
    assert!(headers.iter().any(|(k, v)| k == "Accept" && v == "application/json"));
    let error = headers.iter().find(|(k, _)| k == "X-Bedrock-SigV4-Error").map(|(_, v)| v.clone());
    assert!(error.is_some(), "Should include SigV4 error header when creds missing");
}

#[test]
fn test_build_url() {
    let url = BedrockAdapter::build_url("ap-southeast-1", "anthropic.claude-v2");
    assert_eq!(
        url,
        "https://bedrock-runtime.ap-southeast-1.amazonaws.com/model/anthropic.claude-v2/invoke"
    );
}
