# ADR-009: Multi-Platform Sandbox (opendev-exec)

## Status

Accepted 2026-06-28

## Context

OpenDev Desktop executes LLM-generated shell commands, git operations, LSP servers,
MCP transports, hooks, custom tools, formatters, web fetches, and browser screenshots.
Before Phase 2, the only protection was:

1. **BashTool env_filter** — strips API keys/tokens from child process environments
   (6 suffixes + 13 hard-coded names), applied only to BashTool foreground/background.
2. **Dangerous pattern regex** — 16 patterns blocking `rm -rf /`, `curl | sh`, etc.
3. **SSRF check** — `is_private_url` in `web_fetch` only.

Every other exec point (hooks, MCP servers, custom tools, LSP servers, git operations,
formatters, curl fetches, Chrome headless) inherited the FULL parent process environment,
including API keys. Chrome headless was launched with `--no-sandbox`.

The existing `opendev-sandbox` crate was 100% stubs, Linux-only (`#![cfg(target_os = "linux")]`),
and never produced a working sandbox. `opendev-runtime::sandbox::SandboxConfig` was a policy
module with `check_command`/`check_writable_path` that had zero call sites.

## Decision

Create a new `opendev-exec` crate as the unified execution sandbox layer with:

### Architecture

- **`ExecPolicy` trait** — evaluates whether a command should be allowed, denied, or
  require user approval. Six built-in policies: StrictPolicy, WorkspaceWritePolicy,
  ReadOnlyPolicy, DangerFullAccessPolicy, BashToolPolicy (with dangerous pattern
  detection + safe command allowlist).
- **`SandboxBackend` trait** — applies OS-level isolation (Landlock filesystem rules,
  Seatbelt sandbox profiles, bwrap namespaces, Windows Job Objects). Five backends:
  Landlock (Linux 5.13+), Seatbelt (macOS 12+), bwrap (Linux fallback), Windows
  (Job Object stub), NoneBackend (env_filter only).
- **`env_filter` module** — extracted from BashTool and expanded to 20 suffixes +
  38 exact matches + 21 protected prefixes. Applied to ALL exec points (17+).
- **`patterns` module** — 25 dangerous regex patterns (16 existing + 9 added:
  `chmod 777`, `chown -R`, `eval`, `exec`, `source`, SSH key injection, base64 pipe).
- **`net_filter` module** — SSRF protection extracted from `web_fetch` for reuse.
- **`capability` module** — rlimit/ulimit resource limits.
- **`HardenedProcess`** — applies env filter + process group isolation.

### Fail-Closed Principle

Any `SandboxBackend::apply()` error MUST result in `Decision::Deny`. No child process
is ever spawned without confirmed sandbox application. The only exception is init-time
detection falling back to `NoneBackend` with a visible UI warning.

### Exec Point Migration

All 17+ exec points migrated to use `opendev_exec::env_filter::apply()`:
- **P0 (5):** BashTool, hooks, MCP stdio, custom_tool, git apply (patch)
- **P1 (4):** custom_commands, LSP server, shadow git, worktree
- **P2 (4):** git_status, marketplace, instructions, discovery
- **P3 (4):** web_screenshot (--no-sandbox removed), open_browser, formatter, file_search

### Naming Cleanup

- `opendev-sandbox` → microVM feature-gate only (`#[cfg(feature = "microsandbox")]`)
- `opendev-runtime::sandbox` → `opendev-runtime::policy`
- `SandboxConfig` → `ExecPolicyConfig` (with `#[deprecated]` type alias)
- `opendev_models::config::SandboxConfig` → `ExecPolicy` (with deprecated alias)

## Alternatives

- **MicroVM (microsandbox):** Heavy, Linux-only, complex. Primary approach uses
  OS-native isolation; microVM retained as feature-gated experimental option.
- **Starlark DSL for policies:** Proposed for v2. V1 uses Rust pattern matching
  (simple, testable, maintainable for the current 25 patterns).
- **No OS isolation (env_filter only):** Rejected — env_filter cannot prevent
  filesystem access, network access, or resource exhaustion.
- **Docker containers:** Too heavy for desktop deployment; defeats the purpose
  of local-first architecture.

## Consequences

- **Protection:** All exec points now strip sensitive env vars. BashTool has full
  ExecPolicy + SandboxBackend integration with fail-closed enforcement.
- **Cross-platform:** Landlock (Linux), Seatbelt (macOS), bwrap (Linux fallback),
  Windows (Job Object stub) — all platforms have at least env_filter protection.
- **Backward compatible:** Deprecated type aliases preserve existing API surface.
  `opendev-sandbox` crate still exists (empty without microsandbox feature).
- **Code organization:** `opendev-exec` is an independent leaf crate. 8 crates
  gained `opendev-exec` dependency for env_filter.
- **Known gap:** Windows backend is a stub (env_filter only). Full Job Object
  integration deferred to future work.

## References

- Design: `docs/architecture/infrastructure-foundation-design.md` (§4)
- Recon: sandbox recon report (tool_f0bd4038a001)
- Crate: `crates/opendev-exec/`
- Primary backend: `crates/opendev-exec/src/backends/landlock.rs`, `seatbelt.rs`
- Exec policy: `crates/opendev-exec/src/policy.rs`
- Env filter: `crates/opendev-exec/src/env_filter.rs`
- BashTool integration: `crates/opendev-tools-impl/src/bash/`
- All exec points: see `opendev_exec::env_filter::apply()` call sites
