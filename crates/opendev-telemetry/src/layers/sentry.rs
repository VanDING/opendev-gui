//! Sentry error reporting layer (opt-in, feature-gated).

/// Build the Sentry layer.
/// Only initializes if config.sentry_dsn is set and non-empty.
#[cfg(feature = "sentry")]
pub fn build_sentry_layer(
    config: &crate::config::TelemetryConfig,
) -> Option<sentry::ClientInitGuard> {
    let dsn = config.sentry_dsn.as_ref()?;
    if dsn.is_empty() {
        return None;
    }
    let guard = sentry::init(sentry::ClientOptions {
        dsn: Some(dsn.parse().ok()?),
        sample_rate: config.sentry_sample_rate,
        release: Some(env!("CARGO_PKG_VERSION").into()),
        ..Default::default()
    });
    tracing::info!("Sentry error reporting enabled (sample_rate: {})", config.sentry_sample_rate);
    Some(guard)
}

#[cfg(not(feature = "sentry"))]
pub fn build_sentry_layer(_config: &crate::config::TelemetryConfig) -> Option<()> {
    None
}
