# OpenDev Architecture

## Overview

OpenDev Desktop is an AI coding agent that follows a layered architecture with clear
separation between domain logic, infrastructure, application orchestration, and
user interfaces.

## Architecture Layers

```
┌──────────────────────────────────────────────────┐
│   React Components                               │
│         │                                        │
│   Stores (Zustand)                               │
│         │                                        │
│   Repositories                                   │
│         │                                        │
│   Transport (Platform-agnostic)                  │
├──────────────────────────────────────────────────┤
│   Desktop Interface (Tauri IPC)                  │
│   Commands · Events · Streams                    │
├──────────────────────────────────────────────────┤
│   Application Services                           │
│   Config · Session · Chat · Workflow · MCP       │
├──────────────────────────────────────────────────┤
│   Domain Model                                   │
│   Messages · Sessions · Tools · Events           │
├──────────────────────────────────────────────────┤
│   Infrastructure                                 │
│   HTTP Client · SQLite · MCP · File System       │
└──────────────────────────────────────────────────┘
```

## Communication Model

| Type | Mechanism | Use Case |
|------|-----------|----------|
| **Command** | `invoke()` → `Result<T>` | CRUD, config, status queries |
| **Event** | `listen()` → push | State changes, broadcasts |
| **Stream** | `Channel` → delta | Chat streaming, progress |

## Core Data Flow

```
User Input → React Component → Store → Repository
    → Transport.invoke("send_chat_query")
    → Desktop Interface (Tauri Command)
    → Application Service (ChatService)
    → Agent Runtime
    → Events streamed back via Transport.onEvent()
    → Store updates → React re-render
```

## Agent Loop (Detailed)

```
User Input → Agent Runtime
  → PromptComposer builds system prompt with context
  → AdaptedClient sends request to LLM provider via ProviderAdapter
  → LlmCaller parses response into tool calls or content
  → ReactLoop evaluates: continue, dispatch tool, or complete
  → ToolRegistry dispatches to BaseTool implementation
  → EventStore persists session events (JSONL + SQLite)
  → Response returned to user
```

## Key Design Decisions

### Provider Adapter Pattern

LLM providers are integrated through the `ProviderAdapter` trait. Adding a new
provider requires implementing request/response mapping without touching core
agent logic. See `opendev-http/src/adapters/`.

### Tool Registry

Tools implement the `BaseTool` trait and are registered globally in `ToolRegistry`.
The trait provides 20+ methods with sensible defaults — new tools typically only
override `name()`, `description()`, `parameter_schema()`, and `execute()`.

### Event Sourcing

Session history uses event sourcing via `SessionEvent` enum variants. The event
store (JSONL files + SQLite index) enables replay, snapshotting, and incremental
cost tracking.

### Agent Hierarchy

The main agent spawns subagents via `SubagentManager`. Each subagent runs its
own ReAct loop in an isolated context. Coordination happens through shared
state (TODO lists, file locks) and message passing.

## Crate Dependency Graph

```
opendev-models          (0 internal deps — pure domain)
  ├── opendev-config    (+models)
  ├── opendev-http      (+models, config)
  ├── opendev-history   (+models, config)
  ├── opendev-context   (+models, config)
  ├── opendev-mcp       (+models, config)
  ├── opendev-channels  (+models)
  ├── opendev-tools-core (+models, config)
  ├── opendev-agents    (+models, config, http, tools-core, context, runtime, memory)
  ├── opendev-runtime   (+models, config, history)
  ├── opendev-tools-impl (+tools-core, agents)
  ├── opendev-tui       (+models, agents, config, runtime, tools-core)
  ├── opendev-web       (+models, config, history, http, mcp)
  └── opendev-cli       (composition root — 15 internal deps)
```

## Known Architecture Debt

1. **opendev-tools-impl → opendev-agents dependency** — tool implementations import
   agent types, creating a layer violation. Planned fix: extract shared types to
   `opendev-models` in v0.3+.

2. **CLI God Object** — `opendev-cli` depends on 15 internal crates. Planned fix:
   split into thin binary + `opendev-app` composition root in v0.4+.

3. **opendev-runtime scope creep** — 83 files mixing permissions, secrets, event bus,
   and task management. Planned fix: split into focused crates in v0.5+.

4. **opendev-sandbox stubs** — entire crate is TODO placeholders. To be removed
   or feature-gated.

## Security Model

- API keys stored in system credential store when available
- Secrets detection via regex scanning of command output
- File access controlled by glob-based permission system
- WebFetch blocks private/internal IP ranges (SSRF protection)
- Session tokens use HMAC-SHA256 with configurable secret key
- Password hashing via Argon2 for web server auth
- Tool execution sandboxed (per-tool policies, path validation)

## Performance Characteristics

- SQLite with WAL mode for concurrent reads
- JSONL event store for append-only write performance
- Regexes compiled once via `LazyLock` (HTML converter, command preparation)
- Blocking I/O wrapped in `spawn_blocking` to prevent async runtime starvation
- Mutex poison recovery throughout the lock infrastructure

## Testing Strategy

- 3,183 unit tests across 326 files
- Integration tests for agent, history, plugins, config, and MCP modules
- Property-based testing via `proptest` for fuzzing-critical paths
- CI pipeline: `cargo fmt`, `cargo clippy`, `cargo test`, `cargo audit`, `cargo deny`
