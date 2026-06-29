//! API retry with exponential backoff, jitter, and overload-aware fallback.
//!
//! Provides `with_retry` wrapper that handles:
//! - Max 10 retries with exponential backoff (base 500ms, 25% jitter, cap 32s)
//! - `Retry-After` header overrides
//! - 529 (overloaded) → retry up to 3 times, then trigger model switch signal
//! - 401/403 → signal auth token refresh, retry once
//! - `ECONNRESET`/`EPIPE` → signal stale connection, retry
//! - Context overflow → parse available tokens from error, adjust max_tokens
//! - Logging every retry decision via tracing

use std::time::Duration;

use tracing::{info, warn};

use crate::models::{HttpError, HttpResult};

/// Maximum number of retry attempts.
const MAX_RETRIES: u32 = 10;

/// Base delay for exponential backoff.
const BASE_DELAY_MS: u64 = 500;

/// Maximum delay cap.
const MAX_DELAY_MS: u64 = 32_000;

/// Jitter fraction (25%).
const JITTER_FRACTION: f64 = 0.25;

/// Maximum retries for 529 (overloaded) before triggering model switch signal.
const MAX_OVERLOAD_RETRIES: u32 = 3;

/// Signal emitted by the retry loop to request action from higher layers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetrySignal {
    /// No special action needed.
    None,
    /// Authentication token needs refresh.
    AuthRefresh,
    /// Connection is stale, should be re-established.
    StaleConnection,
    /// Model is overloaded (529), switch to fallback model.
    ModelOverloaded,
    /// Context window overflow — adjust max_tokens.
    ContextOverflow { suggested_max_tokens: u64 },
}

/// Result of a retry attempt.
#[derive(Debug)]
pub struct RetryResult {
    /// The final HTTP result (after retries).
    pub result: HttpResult,
    /// Number of retries actually performed.
    pub retries: u32,
    /// Signal requesting action from higher layers.
    pub signal: RetrySignal,
}

/// Calculate exponential backoff delay for the given attempt.
///
/// delay = min(base * 2^attempt + jitter, max_delay)
fn backoff_delay(attempt: u32) -> Duration {
    let base = BASE_DELAY_MS as f64;
    let exponential = base * (2u64.pow(attempt)) as f64;
    let jitter_range = exponential * JITTER_FRACTION;
    // fastrand gives f64 in [0, 1), shift to [-jitter_range, +jitter_range)
    let jitter = (fastrand::f64() * 2.0 - 1.0) * jitter_range;
    let delay = (exponential + jitter).max(0.0).min(MAX_DELAY_MS as f64);
    Duration::from_millis(delay as u64)
}

/// Parse `Retry-After` header value (seconds).
fn parse_retry_after_header(value: &str) -> Option<Duration> {
    // Try as integer seconds first.
    if let Ok(secs) = value.parse::<u64>() {
        return Some(Duration::from_secs(secs.min(MAX_DELAY_MS / 1000)));
    }
    // Try as HTTP-date (RFC 1123).
    if let Ok(dt) = chrono::DateTime::parse_from_rfc2822(value) {
        let now = chrono::Utc::now();
        let duration = dt.signed_duration_since(now);
        if duration.num_seconds() > 0 {
            return Some(
                Duration::from_secs(duration.num_seconds() as u64)
                    .min(Duration::from_millis(MAX_DELAY_MS)),
            );
        }
    }
    None
}

/// Check if an error message indicates a context overflow condition.
fn is_context_overflow(error: &str) -> Option<u64> {
    let lower = error.to_lowercase();
    // Anthropic: "prompt is too long" / "max_tokens"
    if lower.contains("prompt is too long") || lower.contains("too many tokens") {
        // Try to extract max allowed from error message.
        if let Some(start) = lower.rfind("maximum")
            && let Some(end) = lower[start..].find("tokens")
        {
            let num_part = &lower[start + 8..start + end];
            if let Some(num) = num_part.trim().split_whitespace().next()
                && let Ok(n) = num.parse::<u64>()
            {
                return Some(n);
            }
        }
        return Some(100_000); // fallback default
    }
    // OpenAI: "maximum context length"
    if lower.contains("maximum context length") {
        if let Some(start) = lower.rfind("is ")
            && let Some(end) = lower[start..].find(" tokens")
        {
            let num_part = &lower[start + 3..start + end];
            if let Ok(n) = num_part.trim().parse::<u64>() {
                return Some(n / 2); // half for output
            }
        }
        return Some(64_000);
    }
    None
}

/// Detect connection-level errors (ECONNRESET, EPIPE, etc.).
fn is_connection_error(error: &str) -> bool {
    let lower = error.to_lowercase();
    lower.contains("econnreset")
        || lower.contains("broken pipe")
        || lower.contains("connection reset")
        || lower.contains("epipe")
        || lower.contains("connection refused")
        || lower.contains("econnrefused")
}

/// Wrap a fallible async operation with exponential backoff retry.
///
/// `operation` is an async closure that returns `Result<HttpResult, HttpError>`.
/// The retry logic inspects the result and decides whether to retry.
///
/// Returns a `RetryResult` containing the final outcome and any signals.
pub async fn with_retry<F, Fut>(mut operation: F) -> RetryResult
where
    F: FnMut(u32) -> Fut,
    Fut: std::future::Future<Output = Result<HttpResult, HttpError>>,
{
    let mut last_result = None;
    let mut overload_count = 0;
    let mut signal = RetrySignal::None;

    for attempt in 0..=MAX_RETRIES {
        // Perform the operation.
        match operation(attempt).await {
            Ok(result) => {
                let status = result.status;

                // Success (2xx) — return immediately.
                if let Some(s) = status
                    && (200..300).contains(&s)
                {
                    return RetryResult { result, retries: attempt, signal: RetrySignal::None };
                }

                // 529 — overloaded, model capacity issue.
                if status == Some(529) {
                    overload_count += 1;
                    warn!(
                        attempt,
                        overload_count, "Model overloaded (529), response: {:?}", result.error,
                    );
                    if overload_count >= MAX_OVERLOAD_RETRIES {
                        signal = RetrySignal::ModelOverloaded;
                        warn!("Model overloaded {} times, switching model", overload_count);
                        return RetryResult { result, retries: attempt, signal };
                    }
                    let delay = retry_after_or_backoff(&result, attempt);
                    info!("529 overloaded, retrying in {:?} (attempt {})", delay, attempt);
                    tokio::time::sleep(delay).await;
                    last_result = Some(result);
                    continue;
                }

                // 401/403 — auth failure, signal refresh.
                if status == Some(401) || status == Some(403) {
                    warn!(status, "Auth failure, requesting token refresh");
                    if attempt < 1 {
                        // Retry once after signalling refresh.
                        signal = RetrySignal::AuthRefresh;
                        let delay = retry_after_or_backoff(&result, attempt);
                        tokio::time::sleep(delay).await;
                        last_result = Some(result);
                        continue;
                    }
                    return RetryResult { result, retries: attempt, signal };
                }

                // 429/503 — rate limited, retry with backoff.
                if status == Some(429) || status == Some(503) {
                    if attempt < MAX_RETRIES {
                        let delay = retry_after_or_backoff(&result, attempt);
                        warn!(status, attempt, "Rate limited, retrying in {:?}", delay);
                        tokio::time::sleep(delay).await;
                        last_result = Some(result);
                        continue;
                    }
                    return RetryResult { result, retries: attempt, signal };
                }

                // Check for context overflow in error body.
                if let Some(ref error) = result.error {
                    if let Some(suggested_max) = is_context_overflow(error) {
                        warn!(
                            attempt,
                            "Context overflow detected, suggested max_tokens: {}", suggested_max
                        );
                        signal =
                            RetrySignal::ContextOverflow { suggested_max_tokens: suggested_max };
                        // Don't retry on context overflow — caller must adjust.
                        return RetryResult { result, retries: attempt, signal };
                    }
                }

                // Other — non-retryable error, return immediately.
                if let Some(s) = status
                    && s < 500
                {
                    return RetryResult { result, retries: attempt, signal };
                }

                // 5xx (non-529) — retryable server error.
                if attempt < MAX_RETRIES {
                    let delay = retry_after_or_backoff(&result, attempt);
                    warn!(status, attempt, "Server error, retrying in {:?}", delay);
                    tokio::time::sleep(delay).await;
                    last_result = Some(result);
                    continue;
                }

                return RetryResult { result, retries: attempt, signal };
            }
            Err(err) => {
                let err_str = err.to_string();

                // Connection errors — signal stale connection.
                if is_connection_error(&err_str) {
                    warn!(attempt, "Connection error: {}, signalling stale connection", err_str);
                    signal = RetrySignal::StaleConnection;
                    if attempt < MAX_RETRIES {
                        let delay = backoff_delay(attempt);
                        tokio::time::sleep(delay).await;
                        last_result = Some(HttpResult::fail(err_str.clone(), true));
                        continue;
                    }
                    return RetryResult {
                        result: HttpResult::fail(err_str, false),
                        retries: attempt,
                        signal,
                    };
                }

                // Other transport errors — retry with backoff.
                if attempt < MAX_RETRIES {
                    let delay = backoff_delay(attempt);
                    warn!(attempt, "Transport error: {}, retrying in {:?}", err_str, delay);
                    tokio::time::sleep(delay).await;
                    last_result = Some(HttpResult::fail(err_str, true));
                    continue;
                }

                return RetryResult {
                    result: HttpResult::fail(err_str, false),
                    retries: attempt,
                    signal: RetrySignal::None,
                };
            }
        }
    }

    RetryResult {
        result: last_result.unwrap_or_else(|| HttpResult::fail("Retry exhaustion", false)),
        retries: MAX_RETRIES,
        signal,
    }
}

/// Determine delay from Retry-After header or fall back to exponential backoff.
fn retry_after_or_backoff(result: &HttpResult, attempt: u32) -> Duration {
    // Check Retry-After headers.
    if let Some(ref ra) = result.retry_after {
        if let Some(delay) = parse_retry_after_header(ra) {
            return delay.min(Duration::from_millis(MAX_DELAY_MS));
        }
    }
    if let Some(ref ra_ms) = result.retry_after_ms {
        if let Ok(ms) = ra_ms.parse::<u64>() {
            return Duration::from_millis(ms.min(MAX_DELAY_MS));
        }
    }
    backoff_delay(attempt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn immediate_success_no_retries() {
        let result =
            with_retry(|_| async { Ok(HttpResult::ok(200, serde_json::json!({"ok": true}))) })
                .await;
        assert_eq!(result.retries, 0);
        assert!(result.result.success);
        assert_eq!(result.signal, RetrySignal::None);
    }

    #[tokio::test]
    async fn retries_on_529_then_gives_model_overload_signal() {
        let mut call_count = 0u32;
        let result = with_retry(|_| {
            call_count += 1;
            async move {
                if call_count <= 3 {
                    Ok(HttpResult::retryable_status(529, None, None))
                } else {
                    Ok(HttpResult::ok(200, serde_json::json!({"ok": true})))
                }
            }
        })
        .await;
        assert_eq!(result.signal, RetrySignal::ModelOverloaded);
        // 3 overload retries, should signal after 3.
        assert!(call_count >= 3);
    }

    #[tokio::test]
    async fn auth_401_triggers_refresh_signal() {
        let result = with_retry(|_| async {
            Ok(HttpResult {
                success: false,
                status: Some(401),
                body: None,
                error: Some("Unauthorized".into()),
                interrupted: false,
                retryable: false,
                request_id: None,
                retry_after: None,
                retry_after_ms: None,
            })
        })
        .await;
        assert_eq!(result.signal, RetrySignal::AuthRefresh);
    }

    #[tokio::test]
    async fn context_overflow_detected() {
        let result = with_retry(|_| async {
            Ok(HttpResult {
                success: false,
                status: Some(400),
                body: None,
                error: Some("prompt is too long: maximum 100000 tokens".into()),
                interrupted: false,
                retryable: false,
                request_id: None,
                retry_after: None,
                retry_after_ms: None,
            })
        })
        .await;
        assert_eq!(result.signal, RetrySignal::ContextOverflow { suggested_max_tokens: 100000 });
    }

    #[tokio::test]
    async fn retries_on_429() {
        let mut call_count = 0u32;
        let result = with_retry(|_| {
            call_count += 1;
            async move {
                if call_count <= 2 {
                    Ok(HttpResult::retryable_status(429, None, None))
                } else {
                    Ok(HttpResult::ok(200, serde_json::json!({"ok": true})))
                }
            }
        })
        .await;
        assert!(result.result.success);
    }

    #[test]
    fn backoff_increases_with_attempt() {
        let d1 = backoff_delay(0);
        let d2 = backoff_delay(1);
        let d3 = backoff_delay(2);
        // With jitter, this statistical test should pass.
        assert!(d3 > d1 || d3.as_millis() > d1.as_millis() * 2);
        assert!(d2 > d1 || (d2.as_millis() as i64 - d1.as_millis() as i64).abs() < 200);
    }

    #[test]
    fn backoff_capped_at_32s() {
        let delay = backoff_delay(10);
        assert!(delay.as_millis() <= MAX_DELAY_MS as u128);
    }

    #[test]
    fn parse_retry_after_seconds() {
        let d = parse_retry_after_header("5").unwrap();
        assert_eq!(d.as_secs(), 5);
    }

    #[test]
    fn parse_retry_after_capped() {
        let d = parse_retry_after_header("100000").unwrap();
        assert!(d.as_secs() <= 32);
    }

    #[test]
    fn detect_context_overflow_anthropic() {
        let result = is_context_overflow("prompt is too long: maximum 100000 tokens");
        assert_eq!(result, Some(100000));
    }

    #[test]
    fn detect_context_overflow_openai() {
        let result = is_context_overflow("This model's maximum context length is 128000 tokens");
        assert_eq!(result, Some(64000)); // half for output
    }

    #[test]
    fn no_context_overflow_for_normal_error() {
        let result = is_context_overflow("Bad request: invalid parameter");
        assert!(result.is_none());
    }

    #[test]
    fn detect_connection_reset() {
        assert!(is_connection_error("broken pipe: connection reset by peer"));
        assert!(is_connection_error("ECONNRESET"));
        assert!(is_connection_error("Connection refused"));
        assert!(!is_connection_error("rate limit exceeded"));
    }
}
