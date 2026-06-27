# Security Model

## Principles

1. **Default-secure**: Security mechanisms are always-on, not opt-in.
2. **Defense in depth**: Multiple layers of protection (permissions, secrets detection,
   SSRF protection, token auth).
3. **Least privilege**: Tools and agents operate with minimum necessary permissions.

## Security Mechanisms

### API Key Management

- API keys are stored in system credential store when available.
- Environment variable fallback (`OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, etc.).
- Keys are never logged or exposed in error messages.
- Key rotation support via `CredentialStore`.

### Secrets Detection

- Regex scanning of command output for credential patterns.
- Detected secrets are redacted before display.
- Pattern list in `crates/opendev-runtime/src/secrets.rs`.

### File System Permissions

- Glob-based permission system controls file access.
- Read/write/execute permissions per path pattern.
- Configuration in `~/.config/opendev/permissions.toml`.
- Implementation in `crates/opendev-runtime/src/permissions/`.

### SSRF Protection

- WebFetch tool blocks requests to private/internal IP ranges.
- Blocks: 127.0.0.0/8, 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16, ::1/128.
- Implementation in `crates/opendev-tools-impl/src/web_fetch/`.

### Session Authentication

- Session tokens use HMAC-SHA256 with configurable secret key.
- Release builds require `OPENDEV_SECRET_KEY` environment variable.
- Token verification on every API request.
- Implementation in `crates/opendev-runtime/src/session_model.rs`.

### Web Server Auth

- Password hashing via Argon2 for web server authentication.
- Session-based auth with secure cookies.
- Implementation in `crates/opendev-web/src/routes/auth.rs`.

### Tool Execution Policies

- Per-tool permission policies.
- Path validation for file-related tools.
- Approval flow for destructive operations.
- Implementation in `crates/opendev-runtime/src/approval/`.

## Known Gaps

- `opendev-sandbox` is stubs — no sandboxing of tool execution.
- No network-level sandboxing (separate network namespace).
- No capability-based security for plugins.
