# Security Model

## Principles

1. **Default-secure**: Security mechanisms are always-on, not opt-in.
2. **Defense in depth**: Multiple layers of protection (permissions, secrets detection,
   SSRF protection, token auth).
3. **Least privilege**: Tools and agents operate with minimum necessary permissions.

## Security Mechanisms

### API Key Management (v0.2.0+)

- **OS keyring is primary:** API keys stored in macOS Keychain / Linux Secret Service /
  Windows Credential Manager via `opendev-secrets::KeyringStore` (ADR-010).
- **Environment variable override:** env var ALWAYS wins (for CI/Docker/temp scenarios).
- **Encrypted file fallback:** age X25519 + scrypt encryption for headless/CI environments.
- **Type-level protection:** All secret fields use `secrecy::SecretString` + `zeroize::ZeroizeOnDrop`.
  Display/Debug always print `[REDACTED]` — Constitution 7 is now concretely enforced.
- **Migration path:** `opendev secret migrate` moves plaintext keys from settings.json to keyring.
  AppConfig.api_key deprecated: v0.2 warn → v0.3 force → v0.4 remove.
- **Shadow-key UX:** When env var overrides keyring, UI shows "🔒 env overrides" indicator.
  `opendev secret doctor` CLI diagnoses all secrets.

### Sandbox (v0.2.0+)

- **ExecPolicy trait** evaluates every command before execution (6 built-in policies).
- **SandboxBackend** applies OS-level isolation (Landlock/Seatbelt/bwrap/Windows).
- **Fail-closed:** Any backend.apply() error → child process not spawned. NEVER run unsandboxed.
- **env_filter** applied to ALL 17+ exec points (BashTool, hooks, MCP, custom tools, git, LSP, etc.)
- **Dangerous patterns:** 25 regex patterns block rm -rf /, curl | sh, eval, chmod 777, base64 pipe, etc.
- **SSRF protection:** is_private_url() shared module for all fetch tools (web_fetch, web_screenshot, etc.)
- **Resource limits:** rlimit/ulimit for memory, CPU, file descriptors, process count.
- **Only env_filter fallback:** NoneBackend with UI warning banner on unsupported systems.

### Redaction (v0.2.0+)

- **Field-name-based redact layer** in opendend-telemetry: api_key, token, password, secret, bearer,
  client_secret, bot_token, oauth, jwt, etc. are automatically redacted from all tracing output.
- **SessionDebugLogger** defaults to `false`. When enabled, content is redacted before writing.
- **Memory content** is no longer logged at info level (fixed facade.rs:147).

## References

- ADR-008: App-Server Protocol v1 (`docs/adr/008-app-server-protocol.md`)
- ADR-009: Multi-Platform Sandbox (`docs/adr/009-multi-platform-sandbox.md`)
- ADR-010: OS Keyring Secret Store (`docs/adr/010-os-keyring-secret-store.md`)
- ADR-011: Telemetry Architecture (`docs/adr/011-telemetry-architecture.md`)
- Constitution: `docs/constitution.md`
- Data Flow: `docs/architecture/data-flow.md`
