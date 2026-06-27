# Roadmap

> This roadmap describes **agreed next steps based on the current state of the codebase**.
> It addresses known technical debt, incompletions, and stability goals.
> No future features or unbuilt capabilities are discussed.

## Current Status (v0.1.9)

The coding agent is operational with:
- Multi-provider LLM support (OpenAI, Anthropic, Gemini, Bedrock, Groq, Mistral, Ollama)
- Agent/subagent architecture with ReAct loop
- Tool system with 20+ tools
- Event-sourced session persistence (JSONL + SQLite)
- Memory system (SQLite FTS5)
- Multiple interfaces: CLI, TUI, Web, Desktop, REPL
- Security hardening complete (SSRF, secrets, permissions, HMAC auth)
- CI/CD pipeline with fmt/deny blocking gates

Known issues from the current state:
1. `SqliteSessionStore` methods are stubs (return `Err("not yet implemented")`)
2. `opendev-runtime` scope creep (34 modules in one crate)
3. `opendev-cli` god object (depends on 15 internal crates)
4. `opendev-tools-impl` → `opendev-agents` layer violation
5. `opendev-sandbox` is entirely stubs
6. Internal crate versions stuck at 0.1.6 while workspace is 0.1.9
7. CI clippy has ~40 pre-existing warnings (report-only mode)
8. CI tests have known Linux flakiness (report-only mode)

## v0.2 — Infrastructure Completion

**Goal:** Make all existing infrastructure paths operational.

- Implement `SqliteSessionStore` stub methods → full SQLite-backed session persistence
- Complete `opendev-memory` integration into the agent loop
- Ship Tauri desktop app on macOS
- Reconcile crate versions (0.1.6 → 0.1.9)
- Enable clippy CI as blocking (fix all ~40 warnings)
- Resolve CI test flakiness on Linux

**Technical debt addressed:**
- SqliteSessionStore stubs (from `ARCHITECTURE.md`)
- Version mismatch in workspace dependencies
- CI reliability (clippy check, Linux test flakiness)

---

## v0.3 — Architectural Cleanup

**Goal:** Reduce technical debt in the core crate structure.

- Split `opendev-runtime` into focused crates (permissions, event bus, cost,
  task management as separate crates)
- Fix `opendev-tools-impl` → `opendev-agents` layer violation (extract shared
  types to `opendev-models`)
- Remove or feature-gate `opendev-sandbox` stubs
- Add audit checks for new clippy warnings

**Technical debt addressed:**
- opendev-runtime scope creep (#2 from ARCHITECTURE.md)
- opendev-tools-impl layer violation (#1 from ARCHITECTURE.md)
- opendev-sandbox stubs (#4 from ARCHITECTURE.md)

---

## v0.5 — Stability & Quality

**Goal:** Production-hardening for the desktop app.

- Split `opendev-cli` god object into thin binary + `opendev-app` composition root
- Security audit of all tool execution paths
- Error handling audit (all `unwrap()` reviewed)
- Performance benchmarking and optimization
- Documentation audit (all public API documented)
- Snapshot testing for key agent behaviors

**Technical debt addressed:**
- CLI god object (#3 from ARCHITECTURE.md)
- Production readiness (error handling, security audit, performance)

---

## v1.0 — Stable Release

**Goal:** First stable release with backward compatibility guarantees.

- All existing CI gates are blocking with zero warnings
- API stability guarantees for `opendev-models`, `opendev-tools-core`
  (semver policy established)
- Migration guide for v0.x → v1.0
- Release automation (crate publishing, changelog generation)
- Long-term support policy documented

## Guiding Principle

Each version targets specific known debt from the current codebase.
No version introduces new features not present in the current design documents.
Progress is measured by: fewer stubs, fewer warnings, simpler crate structure.
