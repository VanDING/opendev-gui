# OpenDev Desktop Constitution

> Cabinet design principles — the foundational rules that govern the project's architecture,
> engineering, and evolution. These principles are **current agreed truth**, not aspirational.

---

## Principle 1: Layered Architecture

**The codebase is organized into four layers: Interfaces → Application Orchestration →
Domain Model → Infrastructure. Each layer depends only on layers below it.**

- Interfaces (Tauri Desktop, TUI, CLI, Web, REPL) depend on Application Orchestration.
- Application Orchestration (Agent Runtime, ReAct Loop, Subagents, Workflow) depends on
  Domain Model and Infrastructure.
- Domain Model (`opendev-models`) depends on nothing internal — it is the pure data foundation.
- Infrastructure (HTTP, SQLite, MCP, File System) implements abstractions defined by upper layers.

**Why it exists:** Layer violations are the #1 source of architectural decay. This rule keeps the
crate dependency graph acyclic and makes each crate independently testable.

---

### Foundation Layers (v0.2.0+)
Four new foundation crates provide cross-cutting capabilities:

- **opendev-protocol** — Unified app-server wire protocol (ADR-008). 30 methods + 18 events.
  5-end Transport trait (Tauri/TUI/Web/Workspace/Telegram). V1 frozen + V2 active dual-track.
- **opendev-exec** — Multi-platform sandbox execution (ADR-009). Landlock (Linux), Seatbelt (macOS),
  bwrap (Linux), Windows Job Objects. ExecPolicy trait + env_filter shared across 17+ exec points.
  Fail-closed by design.
- **opendev-secrets** — OS keyring + env + encrypted file secret store (ADR-010). SecretStore trait
  with chain resolution. SecretValue Display/Debug always prints [REDACTED].
- **opendev-telemetry** — JSON structured logging, 14-day retention, metrics, OTLP, Sentry opt-in,
  W3C TraceContext (ADR-011). Privacy-first: no analytics, opt-in error reporting.

---

## Principle 2: Traits Over Concrete Types

**Core behavior is defined by traits, not concrete structs. Traits provide sensible defaults
so implementors override only what they need.**

- `ProviderAdapter` trait for LLM providers — adding a provider means implementing request/response
  mapping without touching agent logic.
- `BaseTool` trait for tools — 20+ methods with sensible defaults; new tools typically override
  only `name()`, `description()`, `parameter_schema()`, and `execute()`.
- `MemoryProvider` trait for memory backends.

**Why it exists:** Trait-based abstraction decouples extension from modification. New providers,
tools, and memory backends can be added without changing existing code.

---

## Principle 3: Event Sourcing for Session History

**Session history is an append-only event stream. Events are the single source of truth.**

- `SessionEvent` enum variants represent every action in a session.
- Events are persisted as JSONL files (append-only) with a SQLite index for query.
- The event store supports replay, snapshotting, and incremental cost tracking.

**Why it exists:** Append-only event streams preserve full history, enable replay for debugging,
and provide a reliable foundation for cost tracking and audit.

---

## Principle 4: Single Composition Root

**Each executable binary has exactly one composition root where all dependencies are wired
together. No crate performs its own dependency injection.**

- `opendev-cli` is the composition root for CLI, TUI, Web, and REPL — it depends on 15 internal
  crates and wires everything together.
- `src-tauri` is the composition root for the desktop app — it depends on 6 internal crates
  and constructs the Axum server state.
- All other crates are library crates that accept their dependencies (no hidden wiring).

**Why it exists:** Composition roots localize the complexity of dependency wiring. Library crates
remain testable in isolation without mocking infrastructure.

---

## Principle 5: Async-First Runtime

**All I/O and long-running operations use `tokio` async. Blocking operations are wrapped
in `spawn_blocking` to prevent async runtime starvation.**

- The async runtime is `tokio` with full features.
- File I/O, regex compilation, and CPU-heavy operations use `spawn_blocking`.
- `Mutex` poison recovery is handled throughout — poisoned locks are unwrapped, not panicked.

**Why it exists:** A single-threaded blocking call stalls the entire async runtime. The
`spawn_blocking` pattern ensures that file system and compute operations do not block
concurrent agent processing.

---

## Principle 6: Registry Pattern for Extensible Systems

**Pluggable components (tools, providers, plugins) are registered in centralized registries
at startup, not discovered at runtime via filesystem scanning.**

- `ToolRegistry` — all tools are registered before the agent loop starts.
- Provider selection — the active provider is chosen by config, not auto-discovered.
- Plugins — registered by the plugin manager at startup.

**Why it exists:** Centralized registration makes the system's capabilities explicit and
predictable. No runtime surprises from filesystem changes or auto-discovery.

---

## Principle 7: Security by Default

**Security mechanisms are always-on, not opt-in. The system assumes untrusted input and
untrusted network boundaries.**

- ✅ API keys are stored in system credential store when available (opendev-secrets KeyringStore).
- ✅ Secrets are always [REDACTED] in Display/Debug (SecretValue constitution enforcement).
- ✅ SecretStore chain: env var > keyring > age-encrypted file fallback.
- ✅ env_filter strips 20+ sensitive env var patterns from ALL child processes (not just BashTool).
- ✅ Sandbox is fail-closed: any backend error prevents child process from spawning.
- ✅ Panic handler writes crash dumps to ~/.opendev/crash/ with 0o600 permissions.
- File access is controlled by glob-based permission system.
- SSRF protection blocks private/internal IP ranges in WebFetch.
- Session tokens use HMAC-SHA256 with a configurable secret key.
- Release builds require `OPENDEV_SECRET_KEY` environment variable.
- `openssl-sys` is banned from the dependency tree.

**Why it exists:** Security opt-ins are rarely enabled. Default-secure design means protection
is present even when the user hasn't thought about it.

---

## Principle 8: Surface Ladder Design System

**Visual hierarchy is communicated through surface color steps and hairlines, not drop shadows.
The design is dark-first, achromatic, and precision-driven.**

- Canvas (`#010102`) → Surface-1 → Surface-2 → Surface-3 → Surface-4 for elevation.
- 1px translucent hairlines replace multi-layer shadows.
- White/gray accent system — no chromatic brand color.
- Inter for body/display, JetBrains Mono for code.
- Unified radius: 8px interactive, 12px containers, 16px modals.

**Why it exists:** A consistent design system eliminates visual decision-making per component.
The dark-first achromatic palette is a developer tool convention that reduces visual fatigue.

---

## Principle 9: Workspace Monorepo

**All crates live in a single Cargo workspace. Shared dependencies are declared once
in the root `Cargo.toml` and referenced with `workspace = true` in sub-crates.**

- 24 workspace members (23 library crates + 1 Tauri binary).
- `[workspace.dependencies]` consolidates all external dependency versions.
- Every crate uses `version.workspace = true` and `edition.workspace = true`.
- Git dependencies are forbidden.

**Why it exists:** Monorepo with centralized dependency management ensures compatible versions
across all crates, simplifies CI, and makes cross-crate refactoring practical.

---

## Principle 10: Defensive Error Handling

**All fallible operations return `Result` with specific error types. Panic is reserved for
programming errors only. External failures (network, filesystem, user input) are always
surfaced as `Result`.**

- `thiserror` for crate-specific error enums.
- `anyhow` for composition roots and binary entry points.
- Mutex poisoning is handled with `unwrap_or_else(|e| e.into_inner())` — the lock is
  recovered, the program continues.
- Unrecoverable states (config corruption, DB migration failure) fail fast at startup.

**Why it exists:** Defensive handling of external failures prevents partial-state bugs.
Mutex poison recovery ensures the system survives transient lock contention.

---

## Principle 11: Testing at All Levels

**Every behavioral change is backed by tests. The test strategy is: unit tests for logic,
integration tests for boundaries, and CI gates for regression.**

- 3,000+ unit tests across 330+ test files.
- Integration tests for agent, history, plugins, config, MCP modules.
- CI pipeline: `cargo fmt` (blocking), `cargo clippy` (report-only), `cargo test`
  (report-only), `cargo audit` (report-only), `cargo deny` (blocking).
- Proptest was evaluated and removed — current testing approach is deterministic.

**Why it exists:** Tests are the executable specification. CI gates prevent regression.
Partial CI gating (report-only for clippy/test) acknowledges existing technical debt
while enforcing what can be enforced.

---

## Principle 12: Explicit Over Magic

**Behavior is configured explicitly, not inferred from context. No runtime auto-discovery,
no implicit defaults that hide behavior.**

- Provider credentials must be explicitly configured (env var or config file).
- Tool availability is determined by registration, not filesystem.
- Session storage uses explicit paths, not auto-detected locations.
- Config overrides are explicit (env vars match config keys).

**Why it exists:** Implicit behavior creates hidden dependencies and makes debugging harder.
Explicit configuration makes the system's operation fully reproducible from its configuration.

---

## Principle 13: Minimal Dependency Footprint

**External dependencies are carefully curated. The license allow-list is restrictive,
`openssl-sys` is banned, and git dependencies are forbidden.**

- Allowed licenses: MIT, Apache-2.0, BSD-2/3-Clause, 0BSD, ISC, Unicode-3.0, MPL-2.0,
  CDLA-Permissive-2.0, BSL-1.0, Zlib, CC0-1.0.
- `cargo-deny` enforces license and source policies in CI.
- No git dependencies allowed — all dependencies must come from crates.io.
- Known advisory ignores are documented and linked to upstream fixes.

**Why it exists:** Every dependency is a liability. Curated allow-list prevents license
surprises. Git dependency ban ensures reproducible builds. The advisory ignore list is
transparent and time-bound to upstream resolution.

---

## Principle 14: Interface Diversity, Unified Core

**Multiple user interfaces (Tauri Desktop, CLI, TUI, Web, REPL) share a single agent core
and domain model. No interface-specific logic lives in the core layers.**

- All interfaces use the same `MainAgent`, `ReactLoop`, `ToolRegistry`.
- Interface-specific code lives in the interface crate only.
- The Tauri desktop app is a thin shell: Tauri 2 + embedded Axum server.
- The CLI is the composition root for CLI, TUI, Web, and REPL.

**Why it exists:** Duplicating agent logic across interfaces creates a maintenance nightmare.
A single core with multiple shells keeps interfaces free to evolve independently while
the core stays consistent.

This is realized by the opendev-protocol crate (ADR-008):
- 5 Transport implementations: TauriTransport, TuiInProcessTransport, WebSocketTransport,
  UnixSocketTransport/NamedPipeTransport, TelegramTransport (v2).
- 30 typed RPC methods, 18 typed events — same wire format for all clients.
- V1 frozen (bug fixes only) + V2 active development.
- ts-rs auto-generates TypeScript bindings for all client types.
