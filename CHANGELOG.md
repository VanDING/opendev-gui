# Changelog

All notable changes to OpenDev Desktop.

## [0.1.9] — 2026-06-26

### Security

- **SSRF fix**: WebFetch tool now blocks requests to private, loopback, link-local, multicast, and unspecified IP addresses (CWE-918)
- **LIKE injection fix**: SQLite search queries now escape `%`, `_`, and `\` wildcard characters (CWE-89 variant)
- **Cookie Secure flag**: Session cookies now set the `Secure` flag in release builds, preventing MITM theft over HTTP (CWE-614)
- **HMAC key hardening**: Release builds panic at startup if `OPENDEV_SECRET_KEY` environment variable is not set, replacing the hardcoded default key `"change-me-in-production"` (CWE-312)
- **SAFETY comments**: All production `unsafe` blocks now have `// SAFETY:` documentation comments, meeting Rust safety compliance standards

### Added

- **Bedrock SigV4 signing**: AWS Bedrock adapter now implements full SigV4 request signing (AWS4-HMAC-SHA256) with canonical request, string-to-sign, signing key derivation, and authorization headers
- Bedrock adapter gracefully falls back with `X-Bedrock-SigV4-Error` diagnostic header when AWS credentials are not configured
- `X-Amz-Security-Token` support for temporary AWS credentials via `AWS_SESSION_TOKEN`

### Performance

- WebFetch HTML converter: ~17 regex patterns now compiled once via `LazyLock` (10-50x improvement per call)
- Bash `prepare_command`: regex pattern cached as `static LazyLock` (~100x improvement on the hot path)

### Reliability

- 100+ `Mutex::lock().unwrap()` calls replaced with `unwrap_or_else(|e| e.into_inner())` poison recovery pattern, eliminating potential crash sites
- File I/O operations wrapped in `tokio::task::spawn_blocking` to prevent async runtime starvation
- `std::sync::RwLock` usage in async contexts documented with SAFETY INVARIANT comments

### Engineering

- CI/CD pipeline established (GitHub Actions): `cargo fmt`, `cargo clippy`, `cargo test`, `cargo audit`, `cargo deny`
- `deny.toml` configured for license compliance and duplicate dependency detection
- `.cargo/audit.toml` configured for security advisory scanning
- `rustfmt.toml` added (edition 2024, max_width 100)
- `README.md` added with project overview and quick start guide
- `ARCHITECTURE.md` added with layered architecture documentation

### Changed

- `proptest` dependency removed (was unused)
- Orphaned `proptest` test modules removed from `bash/helpers_tests`, `bash/patterns_tests`, and `patch/tests`

### Breaking Changes

- Release builds now require the `OPENDEV_SECRET_KEY` environment variable to be set. Debug builds are unaffected and continue to use a development default.

## [0.1.8] — 2026-06-24

### Initial Release

- Multi-provider LLM support (OpenAI, Anthropic, Google Gemini, AWS Bedrock, Groq, Mistral, Ollama)
- Multiple interfaces: Tauri desktop app, terminal UI, CLI, and web server
- Agent orchestration with subagent spawning
- File editing with read/write/edit/diff preview
- Shell command execution with process group management
- MCP protocol client for external tool servers
- Long-term and short-term memory with FTS5 SQLite
- Plugin marketplace system
- Telegram channel remote interaction
- Event-sourced session persistence with cost tracking
