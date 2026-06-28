# Core Data Flow

## Primary Flow: User Request → Response

```
User Input
    │
    ▼
PromptComposer
  - Builds system prompt with context (MemoryCollector, SkillLoader)
  - Compresses context if token budget exceeded
    │
    ▼
AdaptedClient (via ProviderAdapter)
  - Sends request to LLM provider
  - Handles auth, retry, circuit breaker
  - Returns streamed or complete response
    │
    ▼
LlmCaller
  - Parses LLM response into tool calls or text content
  - Handles thinking blocks, refusal messages
    │
    ▼
ReactLoop (evaluates)
  ├── Continue → PromptComposer → AdaptedClient (next turn)
  ├── Dispatch Tool → ToolRegistry → BaseTool.execute()
  │     │
  │     ▼
  │   Tool Result (file change, shell output, search results, etc.)
  │     │
  │     ◄──── Back to ReactLoop
  │
  └── Complete → Return final response
```

## Event Flow (Persistence)

```
Every turn produces SessionEvent:
  UserMessage → EventStore.append()
  AssistantMessage → EventStore.append()
  ToolCall → EventStore.append()
  ToolResult → EventStore.append()
  LLMUsage → CostTracker.record()

EventStore:
  → JSONL file (append-only, per session)
  → SQLite index (for query/search)
  → Projector (maintains queryable views)
```

## Subagent Flow

```
MainAgent decides to parallelize
    │
    ▼
SubagentManager.spawn_subagents(specs)
    │
    ▼
For each subagent:
  ├── Create isolated ReactLoop
  ├── Inject subagent prompt + context
  ├── Run independently (async)
  ├── Report progress → SubagentEventBridge
  └── Return SubagentRunResult
    │
    ▼
MainAgent collects results, continues main loop
```

## Memory Flow

```
Agent turn completes
    │
    ▼
MemoryFacade.push_turn(user_msg, assistant_msg, tool_results)
    │
    ├── ShortTermMemory (in-memory, session-local)
    │     │
    │     ▼
    ├── WriteGate (noise filter — drops low-importance entries)
    │     │
    │     ▼
    ├── CascadeBuffer (batches entries for async persistence)
    │     │
    │     ▼
    └── LongTermMemory (SQLite FTS5)

On next user message:
  MemoryFacade.recall_within_budget(query, project, token_budget)
    → matches entries via FTS5
    → scores by recency, importance, relevance
    → returns top entries within token budget
```

## IPC Protocol (Desktop → Frontend) — opendev-protocol v1

Since v0.2.0, the app-server protocol is defined in the `opendev-protocol` crate
(see ADR-008). All 5 client surfaces (Tauri, TUI, Web, Telegram, Workspace)
implement the unified `Transport` trait.

### Wire Format

JSON-RPC 2.0-like envelope with fixed field order (`v`, `id`/`src`/`dst`, `kind`, `payload`).
Methods use `<domain>/<verb>` naming (30 methods). Events use `<noun>/<past-tense>` naming (18 events).

### Tauri IPC Path

```
OpenDev Desktop (Tauri)                         React Frontend
  │                                              │
  │  Frontend invokes via Transport.invoke()     │
  │◄─────────────────────────────────────────────│
  │  (e.g., turn/start, config/get)              │
  │                                              │
  │  Application Service processes               │
  │                                              │
  │  Events streamed via Transport.onEvent()     │
  │  (message/chunked, tool/started,              │
  │   status/updated, approval/required...)      │
  │─────────────────────────────────────────────►│
  │                                              │
  │  Store updates → React re-render              │
  │                                              │
```

### Dual-Emit Period (v0.2.0 → v0.3.0)

Server emits both legacy names (`message_chunk`) and v1 names (`message/chunked`)
during migration. Frontend handlers migrate incrementally. See `src-tauri/src/server.rs`
for the `legacy_event_name_to_v1()` mapping shim.

### Protocol Versioning

- **V1 (frozen):** v0.2.0 GA — bug fixes only, no new methods/events.
- **V2 (active):** v0.3.0+ — new methods/events; V1 clients remain compatible.
- **Negotiation:** Client sends `protocol_version`; server returns `min_supported` + `max_supported`.

## Event Naming Convention (v1 Protocol)

All events follow `<noun>/<past-tense>` naming per ADR-008:

```
message/started
message/chunked
message/completed
thinking/chunked
tool/started
tool/completed
subagent/spawned
subagent/completed
status/updated
approval/required
session/activity
mcp/server/connected
error/raised
```

See `docs/architecture/protocol-naming.md` for the complete 30-method + 18-event reference table
and legacy name mappings.

## Architecture Compliance

### Layer Rules

| Layer | Allowed | Forbidden |
|-------|---------|-----------|
| **Component** | UI, user interaction, call Store | `invoke()`, `fetch()`, `listen()`, `WebSocket`, `emit()` |
| **Store** | Maintain state, call Repository | `invoke()`, `fetch()`, `emit()` |
| **Repository** | Call Transport | `fetch()`, `WebSocket`, `invoke()`, platform checks |
| **Transport** | Platform-specific IPC | Only abstraction boundary |
| **Command** | DTO Mapping, call Service | `if`/`match` business logic, file ops, state mgmt |
| **Application** | Coordinate Domain, call Infra | Depend on `tauri`, `axum`, `clap` |
| **Core** | Business rules, Traits, Entities | Depend on Desktop, HTTP, CLI |

### Communication Path

```
React → Store → Repository → Transport.invoke()
                                         ↓
                              Desktop Interface (Tauri Command)
                                         ↓
                              Application Service → Core / Infra
                                         ↓
                              Events → Transport.onEvent() → Store → React
```
