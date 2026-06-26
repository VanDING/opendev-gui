//! AWS Bedrock provider adapter.
//!
//! Transforms OpenAI Chat Completions payloads to Amazon Bedrock's
//! `InvokeModel` format and converts responses back.
//!
//! Bedrock uses SigV4 request signing. This module implements a minimal
//! HMAC-SHA256 signing with `UNSIGNED-PAYLOAD` (no payload access needed
//! at header-generation time).
//!
//! Environment variables:
//! - `AWS_ACCESS_KEY_ID` — IAM access key
//! - `AWS_SECRET_ACCESS_KEY` — IAM secret key
//! - `AWS_REGION` — AWS region (defaults to `us-east-1`)
//! - `AWS_SESSION_TOKEN` — optional session token for temporary credentials

mod request;
mod response;

use hmac::{Hmac, Mac};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

/// Default AWS region when `AWS_REGION` is not set.
const DEFAULT_REGION: &str = "us-east-1";

/// AWS service name for Bedrock SigV4 signing.
const SERVICE: &str = "bedrock";

/// SigV4 content hash sentinel — tells AWS to skip body verification.
const UNSIGNED_PAYLOAD: &str = "UNSIGNED-PAYLOAD";

/// HMAC-SHA256 type alias used throughout SigV4 signing.
type HmacSha256 = Hmac<Sha256>;

/// Adapter for Amazon Bedrock's InvokeModel API.
///
/// Bedrock wraps foundation models behind a REST API at:
/// `https://bedrock-runtime.{region}.amazonaws.com/model/{model_id}/invoke`
///
/// This adapter handles:
/// - Converting Chat Completions messages to Bedrock's Anthropic-style format
/// - Building the correct endpoint URL from region + model
/// - SigV4 request signing using `AWS_ACCESS_KEY_ID` / `AWS_SECRET_ACCESS_KEY`
#[derive(Debug, Clone)]
pub struct BedrockAdapter {
    region: String,
    model_id: String,
    api_url: String,
}

impl BedrockAdapter {
    /// Create a new Bedrock adapter for the given model.
    ///
    /// Reads `AWS_REGION` from the environment (defaults to `us-east-1`).
    pub fn new(model_id: impl Into<String>) -> Self {
        let model_id = model_id.into();
        let region = std::env::var("AWS_REGION").unwrap_or_else(|_| DEFAULT_REGION.to_string());
        let api_url = Self::build_url(&region, &model_id);
        Self { region, model_id, api_url }
    }

    /// Create a new Bedrock adapter with a custom region.
    pub fn with_region(model_id: impl Into<String>, region: impl Into<String>) -> Self {
        let model_id = model_id.into();
        let region = region.into();
        let api_url = Self::build_url(&region, &model_id);
        Self { region, model_id, api_url }
    }

    /// Build the Bedrock InvokeModel URL.
    fn build_url(region: &str, model_id: &str) -> String {
        format!("https://bedrock-runtime.{region}.amazonaws.com/model/{model_id}/invoke")
    }

    /// Get the configured AWS region.
    pub fn region(&self) -> &str {
        &self.region
    }

    /// Get the model ID.
    pub fn model_id(&self) -> &str {
        &self.model_id
    }

    /// Read AWS credentials from environment variables.
    fn credentials(&self) -> Option<(String, String, Option<String>)> {
        let access_key = std::env::var("AWS_ACCESS_KEY_ID").ok()?;
        let secret_key = std::env::var("AWS_SECRET_ACCESS_KEY").ok()?;
        let session_token = std::env::var("AWS_SESSION_TOKEN").ok();
        Some((access_key, secret_key, session_token))
    }

    /// Generate SigV4-signed authorization headers for a Bedrock request.
    ///
    /// Uses `UNSIGNED-PAYLOAD` so we do not require the request body at
    /// header-generation time. This is a standard SigV4 feature also used
    /// by the official AWS SDK for streaming requests.
    fn sigv4_headers(&self) -> Result<Vec<(String, String)>, String> {
        let (access_key, secret_key, session_token) = self
            .credentials()
            .ok_or_else(|| "AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY must be set".to_string())?;

        let now = chrono::Utc::now();
        let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
        let date_stamp = now.format("%Y%m%d").to_string();

        let host = format!("bedrock-runtime.{}.amazonaws.com", self.region);

        // Step 1: Create canonical request
        let canonical_uri = format!("/model/{}/invoke", self.model_id);
        let canonical_querystring = "";
        let canonical_headers =
            format!("content-type:application/json\nhost:{host}\nx-amz-date:{amz_date}\n");
        let signed_headers = "content-type;host;x-amz-date";
        let payload_hash = hex::encode(Sha256::digest(UNSIGNED_PAYLOAD.as_bytes()));

        let canonical_request = format!(
            "POST\n{canonical_uri}\n{canonical_querystring}\n{canonical_headers}\n{signed_headers}\n{payload_hash}"
        );
        let canonical_request_hash = hex::encode(Sha256::digest(canonical_request.as_bytes()));

        // Step 2: Create string-to-sign
        let algorithm = "AWS4-HMAC-SHA256";
        let credential_scope = format!("{date_stamp}/{}/{SERVICE}/aws4_request", self.region);
        let string_to_sign =
            format!("{algorithm}\n{amz_date}\n{credential_scope}\n{canonical_request_hash}");

        // Step 3: Derive signing key
        let signing_key = Self::derive_signing_key(&secret_key, &date_stamp, &self.region, SERVICE);

        // Step 4: Calculate signature
        let mut mac = HmacSha256::new_from_slice(&signing_key)
            .map_err(|e| format!("HMAC init failed: {e}"))?;
        mac.update(string_to_sign.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());

        // Step 5: Build Authorization header
        let authorization = format!(
            "AWS4-HMAC-SHA256 Credential={access_key}/{credential_scope}, \
             SignedHeaders={signed_headers}, Signature={signature}"
        );

        let mut headers = vec![
            ("Content-Type".into(), "application/json".into()),
            ("Accept".into(), "application/json".into()),
            ("X-Amz-Date".into(), amz_date),
            ("X-Amz-Content-Sha256".into(), UNSIGNED_PAYLOAD.to_string()),
            ("Authorization".into(), authorization),
        ];

        if let Some(token) = session_token {
            headers.push(("X-Amz-Security-Token".into(), token));
        }

        Ok(headers)
    }

    /// Derive the SigV4 signing key from the secret access key.
    ///
    /// The derivation chain is:
    /// `HMAC("AWS4" + secret, date) → region → service → aws4_request`
    fn derive_signing_key(
        secret_key: &str,
        date_stamp: &str,
        region: &str,
        service: &str,
    ) -> Vec<u8> {
        fn hmac_sign(key: &[u8], data: &[u8]) -> Vec<u8> {
            let mut mac = HmacSha256::new_from_slice(key).expect("HMAC can take key of any size");
            mac.update(data);
            mac.finalize().into_bytes().to_vec()
        }

        let k_date = hmac_sign(format!("AWS4{secret_key}").as_bytes(), date_stamp.as_bytes());
        let k_region = hmac_sign(&k_date, region.as_bytes());
        let k_service = hmac_sign(&k_region, service.as_bytes());
        hmac_sign(&k_service, b"aws4_request")
    }
}

#[async_trait::async_trait]
impl super::base::ProviderAdapter for BedrockAdapter {
    fn provider_name(&self) -> &str {
        "bedrock"
    }

    fn convert_request(&self, mut payload: Value) -> Value {
        // Strip internal reasoning effort field (Bedrock doesn't support it)
        payload.as_object_mut().map(|obj| obj.remove("_reasoning_effort"));

        request::extract_system(&mut payload);
        request::convert_tools(&mut payload);
        request::convert_tool_messages(&mut payload);
        request::ensure_max_tokens(&mut payload);

        // Bedrock wraps the model in the URL, not the payload.
        // Remove fields Bedrock does not accept.
        if let Some(obj) = payload.as_object_mut() {
            obj.remove("model");
            obj.remove("n");
            obj.remove("frequency_penalty");
            obj.remove("presence_penalty");
            obj.remove("logprobs");
            obj.remove("stream");
        }

        // Set anthropic_version required by Bedrock's Anthropic models.
        payload["anthropic_version"] = json!("bedrock-2023-05-31");

        payload
    }

    fn convert_response(&self, response: Value) -> Value {
        response::response_to_chat_completions(response, &self.model_id)
    }

    fn api_url(&self) -> &str {
        &self.api_url
    }

    fn extra_headers(&self) -> Vec<(String, String)> {
        self.sigv4_headers().unwrap_or_else(|e| {
            let mut headers = vec![
                ("Content-Type".into(), "application/json".into()),
                ("Accept".into(), "application/json".into()),
            ];
            headers.push(("X-Bedrock-SigV4-Error".into(), e));
            headers
        })
    }
}

#[cfg(test)]
mod tests;
