//! Experimental protocol features.
//! Methods marked #[experimental] are unstable and may change without notice.

/// Marker attribute for experimental features.
/// In Rust, we use a doc-comment convention; on the wire, these methods
/// carry an `"experimental": true` flag.
pub fn is_experimental_method(method: &str) -> bool {
    // V1 has no experimental methods. V2 may add them.
    let _ = method;
    false
}

/// List of experimental method names (empty in v1).
pub fn experimental_methods() -> &'static [&'static str] {
    &[]
}
