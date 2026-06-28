//! W3C TraceContext propagation.
//!
//! Used by the ReAct loop to propagate trace headers to LLM providers.
//! Format: `traceparent: 00-<trace-id>-<span-id>-01`

use uuid::Uuid;

/// Generate a W3C traceparent header for the current span.
/// If no span is active, generates a new trace id.
pub fn generate_traceparent() -> String {
    let trace_id = Uuid::new_v4().to_string().replace('-', "")[..32].to_string();
    let span_id = Uuid::new_v4().to_string().replace('-', "")[..16].to_string();
    format!("00-{}-{}-01", trace_id, span_id)
}

/// Extract trace id from a traceparent header.
pub fn extract_trace_id(traceparent: &str) -> Option<String> {
    let parts: Vec<&str> = traceparent.split('-').collect();
    if parts.len() >= 2 { Some(parts[1].to_string()) } else { None }
}

/// Inject a traceparent header into the LLM request headers.
pub fn inject_trace_headers(headers: &mut std::collections::HashMap<String, String>) {
    let traceparent = generate_traceparent();
    headers.insert("traceparent".to_string(), traceparent);
}
