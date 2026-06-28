use crate::key::SecretKey;

/// Audit logging for secret access.
/// 
/// Records when secrets are accessed, by what key, and whether
/// the access succeeded. Used by the telemetry layer.
pub fn record_access(key: &SecretKey, success: bool) {
    if success {
        tracing::debug!(
            target: "opendev_secrets::audit",
            key = %key,
            namespace = %key.namespace().as_str(),
            "Secret access granted"
        );
    } else {
        tracing::warn!(
            target: "opendev_secrets::audit",
            key = %key,
            namespace = %key.namespace().as_str(),
            "Secret access denied"
        );
    }
}

/// Record a secret mutation (set or delete).
pub fn record_mutation(key: &SecretKey, operation: &str) {
    tracing::info!(
        target: "opendev_secrets::audit",
        key = %key,
        operation = operation,
        namespace = %key.namespace().as_str(),
        "Secret mutation"
    );
}
