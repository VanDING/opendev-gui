# Logging & Observability

## Current Setup

### Tracing

The project uses the `tracing` crate ecosystem for structured logging:

| Crate | Purpose |
|---|---|
| `tracing` | Core instrumentation |
| `tracing-subscriber` | Log formatting and output |
| `opendev-observability` | Production observability (OTLP, Perfetto) |

### Default Configuration

In the CLI entry point (`opendev-cli/src/helpers.rs`):

```rust
pub fn init_tracing(paths: &Paths) -> ObservabilityGuard {
    let config = ObservabilityConfig {
        level: std::env::var("OPENDEV_LOG").unwrap_or_else(|_| "info".into()),
        otlp_endpoint: std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok(),
        perfetto_enabled: std::env::var("OPENDEV_PERFETTO").is_ok(),
        ..Default::default()
    };
    ObservabilityGuard::init(config, paths)
        .expect("Failed to initialize observability")
}
```

### Environment Variables

| Variable | Effect |
|---|---|
| `OPENDEV_LOG` | Log level (default: `info`). Values: `error`, `warn`, `info`, `debug`, `trace` |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | Enable OTLP export to this endpoint |
| `OPENDEV_PERFETTO` | Enable Perfetto tracing when set |

## Logging Conventions

- Use structured fields: `tracing::info!(session_id = %id, "Session started")`.
- Use appropriate levels:
  - `error!` — unrecoverable failure
  - `warn!` — unexpected but recoverable
  - `info!` — significant lifecycle events (session start/end, tool execution)
  - `debug!` — detailed operational info
  - `trace!` — very detailed (per-LLM-token, per-event)
- Do not log sensitive data:
  - API keys
  - Session tokens
  - File contents
  - Environment variables containing secrets

## Observability Crate

The `opendev-observability` crate provides:
- `ObservabilityConfig` — configuration for tracing, OTLP, Perfetto.
- `ObservabilityGuard` — RAII guard that flushes and shuts down observability
  on drop. Call `guard.shutdown()` during app teardown.

Logs are written to `{data_dir}/logs/` when file output is configured.
