# ADR-010: OS Keyring as Primary Secret Store (opendev-secrets)

## Status

Accepted 2026-06-28

## Context

API keys, bot tokens, and OAuth secrets are the most sensitive data in OpenDev Desktop.
Before Phase 3, these were stored as:

1. **`settings.json`** — `api_key: Option<String>` in plaintext, no 0600 permission
2. **`auth.json`** — `CredentialStore` (dead code, 0 call sites) with 0600 permission
3. **`mcp.json`** — `client_secret: String` in plaintext (file is 0600)
4. **`users.json`** — `password_hash: String` (already Argon2 hash, but no zeroize)

Constitution 7 states: "API keys are stored in system credential store when available
(never logged)." This was completely aspirational — no code path used any OS keyring.
The `keyring 3.6.3` crate existed only as a transitive dependency of the unused
`opendev-sandbox` → `microsandbox` path. No `zeroize` usage anywhere in the workspace.

`CredentialStore` and `AuthProfileManager` were fully implemented with tests (240+ lines
each) but had **zero production call sites**. Actual API key resolution went through
`AppConfig::get_api_key_with_env` which looked up env vars first, then `settings.json`
— never the OS keyring.

## Decision

Create a new `opendev-secrets` crate as the unified secret storage layer with:

### Architecture

- **`SecretStore` trait** — async `get`/`set`/`delete`/`list` with three backends:
  1. `EnvStore` — env var override (always wins, set/delete are no-ops)
  2. `KeyringStore` — OS keyring via `keyring` crate (macOS Keychain, Linux Secret Service,
     Windows Credential Manager), service name `"com.opendev.desktop"`
  3. `FileStore` — age X25519 + scrypt encrypted file fallback (for headless/CI)
- **`ChainedSecretStore`** — resolves in priority order: env → keyring → file.
  Set/delete propagate to all writable stores.

### Type-Level Protection

- **`SecretValue`** — newtype over `secrecy::SecretString` with `Display`/`Debug` printing
  `[REDACTED]` (Constitution 7 "never logged" implementation).
- **`SecretKey`** — typed key with `Namespace` enum + `account` string, preventing string
  confusion errors. Constructors: `SecretKey::llm("openai")`, `SecretKey::telegram()`, etc.
- **`zeroize::ZeroizeOnDrop`** on `SecretValue`.

### Dead Code Revival

- **`CredentialStore`** — rewritten as compat facade over `ChainedSecretStore`. All methods
  now async, delegate to SecretStore chain. Backward-compatible API preserved.
- **`AuthProfileManager`** — rewritten with SecretStore backing. Loads keys from
  SecretStore chain (single + multi-key account-1..9). Cooldown: 429=30s, 401=300s,
  403=600s, 5xx=30-60s.

### SecretString Field Migration

Key fields changed from `String` to `secrecy::SecretString`:

| Field | Location | Before | After |
|-------|----------|--------|-------|
| `api_key` | `AppConfig` | `Option<String>` | `Option<String>` (deprecated, will be removed in v0.4) |
| `password_hash` | `User` | `String` | `secrecy::SecretString` |
| `bot_token` | `TelegramChannelConfig` | `String` | `secrecy::SecretString` |
| `client_secret` | `McpOAuthConfig` | `String` | `secrecy::SecretString` |

### AppConfig.api_key Deprecation Timeline

| Version | Behavior |
|---------|----------|
| v0.2.0 | SecretStore introduced, new UI writes keyring. `AppConfig.api_key` still works but emits `tracing::warn` on read |
| v0.2.x | Startup detection, migration prompt (deferrable up to 30 days) |
| v0.3.0 | Force migration, clear `AppConfig.api_key` field |
| v0.4.0 | Remove `AppConfig.api_key` field (hard break) |

### New Resolution Path

`AppConfig::get_api_key_via_secrets()`:
1. SecretStore chain (env → keyring → file)
2. Legacy env var resolution (backward compat)
3. Deprecated `self.api_key` with warning
4. `OPENAI_API_KEY` last resort fallback

## Alternatives

- **Continue with plaintext `settings.json`** — violates Constitution 7, exposes API keys
  on disk and in backups.
- **AES-256-GCM file encryption instead of age** — age has better key derivation (X25519 +
  scrypt) and is audited.
- **Keyring only, no file fallback** — headless CI environments cannot use OS keyring
  (no D-Bus, no Keychain), so age-encrypted file is necessary.
- **Single key shared across all providers** — each API key should be independently
  storable/rotatable.

## Consequences

- Constitution 7 is now implemented: API keys are stored in OS keyring when available,
  Display/Debug always show `[REDACTED]`.
- Zeroize is used throughout for secret fields (first zeroize usage in the workspace).
- Zero-cost dependency: `keyring 3`, `zeroize 1`, `secrecy 0.10` were already compiled
  as transitive deps.
- Existing `auth.json` and `settings.json` with plaintext keys need migration (one-time
  operation at v0.2.0 GA).
- Shadow-key UX: when env var overrides a keyring value, UI shows "🔒 env overrides"
  indicator + doctor command to diagnose.
- `CredentialStore` and `AuthProfileManager` are no longer dead code.
- **Known limitation:** `KeyringStore::list()` returns empty list — the `keyring` crate
  API does not support key enumeration. Applications must track stored keys themselves.
  This is acceptable for v1 (apps know which keys they've stored).

## References

- Design: `docs/architecture/infrastructure-foundation-design.md` (§5)
- Recon: keyring recon report (tool_f0bd7c7a6001)
- Crate: `crates/opendev-secrets/`
- Backends: `crates/opendev-secrets/src/backends/env.rs`, `keyring.rs`, `file.rs`
- Chain: `crates/opendev-secrets/src/resolver.rs`
- Migration: `crates/opendev-secrets/src/migration.rs`
- Dead code revival: `crates/opendev-http/src/auth.rs`, `rotation.rs`
- Constitution 7: `docs/constitution.md`
