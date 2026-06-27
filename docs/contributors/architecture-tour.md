# Architecture Tour

## Key Types

Start with these types to understand the data model:

| Type | Crate | Purpose |
|---|---|---|
| `SessionEvent` | `opendev-models` | Foundation of event-sourced persistence |
| `ChatMessage` | `opendev-models` | Message in the agent conversation |
| `ToolCall` | `opendev-models` | Tool invocation from LLM |
| `Session` | `opendev-models` | Session metadata and state |
| `BaseTool` trait | `opendev-tools-core` | All tools implement this |
| `ProviderAdapter` trait | `opendev-http` | LLM provider integration |
| `AgentDefinition` | `opendev-agents` | Agent type configuration |
| `SubAgentSpec` | `opendev-agents` | Subagent spawning spec |
| `LoadedSkill` | `opendev-agents` | Loaded skill with metadata |
| `MemoryEntry` | `opendev-memory` | Memory persistence unit |

## Entry Points

| Interface | Entry Point | File |
|---|---|---|
| CLI | `opendev-cli/src/main.rs` | Binary entry, CLI parsing |
| Desktop | `src-tauri/src/main.rs` | Tauri app setup |
| Desktop server | `src-tauri/src/server.rs` | Embedded Axum server |
| Web server | `opendev-web/src/server.rs` | Standalone HTTP server |
| Agent | `opendev-agents/src/main_agent.rs` | Main agent entry |
| ReAct loop | `opendev-agents/src/react_loop/mod.rs` | Core agent loop |

## Core Data Flow

```
User Input → PromptComposer → AdaptedClient (LLM) → LlmCaller (parse)
  → ReactLoop (evaluate) → ToolRegistry → BaseTool.execute()
  → EventStore (persist) → Response
```

See `docs/architecture/data-flow.md` for the detailed flow.

## Layer Map

```
Interfaces:          src-tauri, opendev-web, opendev-tui, opendev-cli, opendev-repl
Application Layer:   opendev-agents, opendev-tools-*, opendev-runtime, opendev-workflow
Domain Model:        opendev-models
Infrastructure:      opendev-config, opendev-http, opendev-history, opendev-mcp,
                     opendev-memory, opendev-context, opendev-observability,
                     opendev-hooks, opendev-plugins, opendev-sandbox, opendev-channels
```

## Key Patterns

- **Trait-based abstraction**: `BaseTool`, `ProviderAdapter`, `MemoryProvider` — define
  behavior, implement for each variant.
- **Event sourcing**: All session events are appended to an event stream. See ADR-002.
- **Registry pattern**: Tools, providers, and plugins are registered at startup.
- **Composition root**: `opendev-cli` and `src-tauri` are the only places where all
  dependencies are wired together.
