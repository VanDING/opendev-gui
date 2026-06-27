# Crate Layering

## Layer Structure

The 24 workspace members are organized into 4 architectural layers:

```
Layer 0 — Domain Model
  opendev-models             Pure data types — no internal deps

Layer 1 — Infrastructure
  opendev-config             Configuration loading, paths
  opendev-http               HTTP client, provider adapters, auth
  opendev-history            Event sourcing, SQLite session store
  opendev-context            Context management, compaction, retrieval
  opendev-mcp                MCP protocol client
  opendev-memory             Memory system (SQLite FTS5)
  opendev-observability      Observability (OTLP, Perfetto)
  opendev-hooks              Lifecycle hooks
  opendev-plugins            Plugin manager
  opendev-sandbox            Sandbox (stubs)

Layer 2 — Application Orchestration
  opendev-tools-core         BaseTool trait, ToolRegistry, middleware
  opendev-tools-lsp          LSP integration tools
  opendev-tools-symbol       Symbol refactoring tools
  opendev-tools-impl         Concrete tool implementations
  opendev-agents             Agent runtime, ReAct loop, subagents, skills
  opendev-runtime            Approval, permissions, event bus, task management
  opendev-workflow           Workflow pipeline/barrier/loop patterns
  opendev-channels           Telegram channel

Layer 3 — Interfaces
  opendev-web                Axum HTTP server, WebSocket protocol
  opendev-tui                Terminal UI (ratatui)
  opendev-repl               Interactive REPL
  opendev-cli                CLI binary entry point (composition root)
  src-tauri                  Tauri desktop shell (composition root)
```

## Dependency Rules

1. **Layer N can depend on Layer N and Layer N-1 only.** No skipping layers.
2. **No crate in Layer 1-2 may depend on a crate in Layer 3.**
3. **No circular dependencies between crates in any layer.**
4. **`opendev-models` has zero internal dependencies** — it is the pure data foundation.

## Current Layer Violations

From `ARCHITECTURE.md`:

1. **`opendev-tools-impl` → `opendev-agents`** (Layer 2 → Layer 2, but in the wrong
   direction): Tool implementations import agent types. Since both are Layer 2, the
   violation is within the layer rather than across layers. Fix: extract shared types
   to `opendev-models`.

2. **`opendev-cli` god object**: Depends on 15 internal crates, making it a de facto
   composition root. This is by design (ADR-004: Single Composition Root) but creates
   a fragile crate that breaks on any upstream change.

3. **`opendev-runtime` scope creep**: 34 public modules mixing permissions, secrets,
   event bus, task management, and cost tracking. The crate has grown beyond its original
   scope and should be split.

4. **`opendev-sandbox` stubs**: Entire crate is TODO placeholders. It exists in the
   dependency tree but provides no functionality.

## Dependency Graph (Current)

```
opendev-models
  ├── opendev-config    (+models)
  ├── opendev-http      (+models, config)
  ├── opendev-history   (+models, config)
  ├── opendev-context   (+models, config)
  ├── opendev-mcp       (+models, config)
  ├── opendev-tools-core (+models, config)
  ├── opendev-channels  (+models)
  ├── opendev-agents    (+models, config, http, tools-core, context, runtime, memory)
  ├── opendev-runtime   (+models, config, history)
  ├── opendev-tools-impl (+tools-core, agents)  ← layer violation
  ├── opendev-tui       (+models, agents, config, runtime, tools-core)
  ├── opendev-web       (+models, config, history, http, mcp)
  └── opendev-cli       (composition root — 15 internal deps)
```

## Adding a New Crate

1. Identify which layer the crate belongs to.
2. Check that all dependencies are from the same or lower layer.
3. Add to the workspace `[workspace.members]` list.
4. Add to `[workspace.dependencies]` if it will be used by other crates.
5. Update the dependency graph in this document.
6. Run `cargo check --workspace` to verify no circular dependencies.
7. Write an ADR for the new crate.
