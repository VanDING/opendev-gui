//! OTLP exporter layer (feature-gated).
//! Requires the "otlp" feature.

/// Build the OTLP tracing layer.
/// Returns Some(layer) if otlp_endpoint is configured, None otherwise.
#[cfg(feature = "otlp")]
pub fn build_otlp_layer(
    config: &crate::config::TelemetryConfig,
) -> Option<impl tracing_subscriber::Layer<tracing_subscriber::Registry>> {
    let endpoint = config.otlp_endpoint.as_ref()?;

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()
        .ok()?;

    let provider =
        opentelemetry_sdk::trace::TracerProvider::builder().with_batch_exporter(exporter).build();

    let tracer = provider.tracer("opendev");
    Some(tracing_opentelemetry::layer().with_tracer(tracer))
}

#[cfg(not(feature = "otlp"))]
pub fn build_otlp_layer(
    _config: &crate::config::TelemetryConfig,
) -> Option<tracing_subscriber::layer::Identity> {
    None
}

/// Stub for building OTLP metrics exporter.
#[cfg(feature = "otlp")]
pub fn build_otlp_metrics_exporter() {
    tracing::info!("OTLP metrics exporter initialized");
}

#[cfg(not(feature = "otlp"))]
pub fn build_otlp_metrics_exporter() {}
