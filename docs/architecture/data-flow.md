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

## IPC Protocol (Desktop → Frontend)

```
OpenDev Desktop (Tauri)                         React Frontend
  │                                              │
  │  Frontend invokes Tauri Command              │
  │◄─────────────────────────────────────────────│
  │  (e.g., send_chat_query, get_app_config)     │
  │                                              │
  │  Application Service processes               │
  │                                              │
  │  Events streamed via Tauri listen()           │
  │  (chat.message.chunk, tool_call,              │
  │   chat.thinking.block, status_update...)     │
  │─────────────────────────────────────────────►│
  │                                              │
  │  Store updates → React re-render              │
  │                                              │
```

## Event Naming Convention

All events follow `domain.object.action`:

```
chat.message.chunk
chat.message.completed
chat.tool.executing
chat.tool.completed
chat.thinking.block
chat.approval.required
session.activity
config.updated
mcp.server.connected
mcp.servers.updated
```

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
