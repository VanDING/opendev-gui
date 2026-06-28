# ADR-011: Telemetry Architecture (opendev-telemetry)

## Status

Accepted 2026-06-28

## Context

Before Phase 4, OpenDev's observability was handled by `opendev-observability` (173 LoC).
It installed a `tracing-subscriber` with a daily-rotated file appender but had critical gaps:

- **5 write-only config fields:** `otlp_endpoint`, `export_perfetto_on_session_end`,
  `record_prompt_content`, `record_tool_args`, `record_file_location` were declared but never read.
- **No JSON output:** `tracing-subscriber`'s `json` feature was enabled but `.json()` was never called.
- **No log retention:** Files accumulated indefinitely — no cleanup, no max file count.
- **Tauri had no telemetry:** `OtelGuard::init` was never called in the desktop binary.
- **No panic handler:** Desktop app panics went to Tauri's default handler, not to a crash file.
- **No metrics:** Zero counters, no gauges, no histograms anywhere in the workspace.
- **No error reporting:** No Sentry, no bugsnag, no sentry-rust crate in tree.
- **SessionDebugLogger wrote full LLM bodies by default** with no redaction, `debug_logging` defaulted to `true`.
- **Memory facade logged full content:** `tracing::info!(content = %content, ...)` leaked memory content.
- **ErrorBoundary had no componentDidCatch:** React render errors were silently dropped.
- **50+ console.log calls** scattered across frontend with no centralized dispatcher.
- **No W3C TraceContext propagation:** Trace context stopped at the ReAct loop entry.

## Decision

Rename `opendev-observability` → `opendev-telemetry` and build a complete observability stack:

### Architecture

- **17-field TelemetryConfig** — ALL fields are honored (enabled, log_level, format, retention_days,
  log_dir, otlp_endpoint, otlp_protocol, record_prompt_content, record_tool_args, record_file_location,
  sentry_dsn, sentry_sample_rate, export_perfetto_on_session_end, perfetto_output_dir, debug_logging,
  include_full_payload).
- **Layer stack:** file (JSON, 14-day retention), redact (field-name-based), panic (crash dump),
  OTLP (feature-gated, default off), Sentry (feature-gated, opt-in via DSN).
- **Metric function stubs:** 15 v1 metrics recorded via `tracing::debug!` in the `metrics` target,
  ready for OTLP metrics exporter when the `metrics` feature is enabled.
- **W3C TraceContext:** `traceparent` header injected into all outgoing LLM HTTP requests.

### Key Design Points

**JSON output (always, not pretty):**
```json
{
  "timestamp": "2026-06-28T10:23:45.123Z",
  "level": "INFO",
  "target": "opendev_runtime::cost_tracker",
  "fields": {"provider": "anthropic", "tokens": 1234},
  "message": "usage recorded"
}
```

**14-day retention:**
`tracing_appender::rolling::Builder::new().max_log_files(14)` — implemented via the rolling
file appender's built-in cleanup.

**Redact layer:**
Field-name allowlist: `api_key`, `token`, `password`, `secret`, `bearer`, `client_secret`,
`bot_token`, `private_key`, `access_key`, `oauth`, `jwt`, etc. Matched values become `[REDACTED]`.

**Privacy-first:**
- No analytics — no user behavior data is collected.
- Sentry opt-in — empty DSN = off.
- OTLP default off — requires `OTEL_EXPORTER_OTLP_ENDPOINT`.
- Session debug logging default false (breaking change from v0.1.x).

**Tauri boot sequence:**
```
1. TelemetryGuard::init (FIRST — before everything)
2. install_crash_handler
3. Secrets → Sandbox → Protocol → Services → Run
```

**ErrorBudary componentDidCatch** captures errors and logs them via `console.error`.

**Frontend logger** centralized in `src/api/logger.ts` — `logger.info/debug/warn/error`.

### Startup Order

```
1. Init telemetry (with redaction)     ← this happens FIRST now
2. Install panic handler                ← crash dumps
3. Init secrets (keyring detect)
4. Init sandbox (OS support detect)
5. Init protocol
6. Build app services
7. Run
```

Error tolerance: Telemetry init failure → fallback to stderr + continue.

### Cargo Features

```toml
[features]
default = ["file", "redact", "panic"]
file = []
redact = []
panic = []
otlp = ["dep:opentelemetry", "dep:opentelemetry-otlp", "dep:tracing-opentelemetry", "dep:opentelemetry_sdk"]
sentry = ["dep:sentry"]
metrics = ["dep:metrics", "dep:metrics-exporter-otlp"]
all = ["file", "redact", "panic", "otlp", "sentry", "metrics"]
```

### V1 Metrics (15)

LLM: calls.total, tokens.input, tokens.output, duration.seconds
Tools: executed.total, duration.seconds
Sessions: active (gauge), created.total
Cost: usd.total
Errors: total
Sandbox: executions.total
Secrets: lookups.total
Approvals: requested.total, granted.total
Frontend: errors.total

## Alternatives

- **Pretty logging:** Not machine-parseable, breaks log aggregation.
- **No retention policy:** Disk fills unboundedly — 14 days gives ~1.4GB at 100MB/day.
- **SessionDebugLogger removed entirely:** Data shows it's useful for debugging, but must be
  opt-in with redaction.
- **No frontend logger:** 50+ console.log calls would remain scattered and unmanaged.
- **OpenTelemetry Agent:** Too heavy for a desktop app — lightweight function stubs are sufficient.

## Consequences

- All workspace crates can be observed through the unified `tracing` layer.
- Telemetry init failure does not prevent app startup (graceful degradation).
- Privacy controls are exposed in the UI (Settings → Privacy tab).
- Migration from v0.1.x: `debug_logging` defaults to `false` (was `true`).
- Crash dumps go to `~/.opendev/crash/crash-<timestamp>.log` (both CLI and desktop).
- W3C TraceContext enables correlation of LLM requests across the agent loop.

## References

- Design: `docs/architecture/infrastructure-foundation-design.md` (§6)
- Recon: telemetry recon report (tool_f0bdaa563001)
- Crate: `crates/opendev-telemetry/`
- Layers: `crates/opendev-telemetry/src/layers/`
- Metrics: `crates/opendev-telemetry/src/metrics.rs`
- Privacy UI: `src/components/Settings/PrivacySettings.tsx`
- Logger: `src/api/logger.ts`
- ErrorBoundary: `src/components/ErrorBoundary.tsx`
- Panic handler: `crates/opendev-telemetry/src/layers/panic.rs`
