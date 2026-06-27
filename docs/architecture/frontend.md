# Frontend Architecture

## Overview

The frontend is a TypeScript + React application built with Vite. It communicates with
the Rust backend via **Tauri IPC** (invoke + events), following the Desktop Native
Communication Architecture (ADR-0005).

## Technology Stack

| Concern | Choice |
|---|---|
| Build tool | Vite 5 |
| UI framework | React 19 |
| State management | Zustand stores |
| Styling | CSS custom properties + index.css |
| Type generation | `ts-rs` from Rust models |
| Desktop shell | Tauri 2 (communication via IPC) |
| Transport | `TauriTransport` (invoke/listen/Channel) |

## Architecture Flow

```
React Components
        │
Stores (Zustand)
        │
Repositories
        │
Transport (TauriTransport)
        │
        ├── invoke() → Tauri IPC Commands
        ├── onEvent() → Tauri Events
        └── openStream() → Tauri Channel
        │
═════════════════════════════════════ (Process boundary)
        │
Desktop Interface (Rust)
        │
        ├── Commands (DTO Mapping only)
        ├── Events (domain.object.action)
        └── Streams
        │
Application Services
        │
Core + Infrastructure
```

## Component Tree

```
App
├── ThemeContext.Provider
├── ErrorBoundary
├── Routes
│   ├── / → ChatPage
│   │   ├── ChatInterface
│   │   │   ├── TopBar
│   │   │   ├── MessageList
│   │   │   │   └── MessageItem (per message)
│   │   │   │       ├── MarkdownContent
│   │   │   │       ├── ToolCallMessage
│   │   │   │       ├── ThinkingBlock
│   │   │   │       ├── DiffViewer
│   │   │   │       ├── BashPreview
│   │   │   │       └── ...
│   │   │   ├── InputBox
│   │   │   │   ├── FileMentionDropdown
│   │   │   │   ├── QuickActions
│   │   │   │   └── CommandPalette
│   │   │   ├── StatusBar
│   │   │   ├── ProgressIndicator
│   │   │   ├── QueueBar
│   │   │   ├── DetailPanel
│   │   │   │   ├── FileChangesButton
│   │   │   │   ├── SubagentTree
│   │   │   │   └── TodoPanel
│   │   │   ├── LandingPage / WelcomeScreen
│   │   │   ├── PlanApprovalDialog
│   │   │   ├── AskUserDialog
│   │   │   └── StatusDialog
│   │   └── SessionsSidebar
│   │       ├── Breadcrumb
│   │       ├── NewSessionModal
│   │       ├── DeleteConfirmModal
│   │       └── SessionModelModal
│   ├── /settings → SettingsModal
│   │   ├── ModelSettings (ModelSlot)
│   │   ├── MCPSettings (MCPServerCard, AddMCPServerModal,
│   │   │             EditMCPServerModal, MCPToolsModal)
│   │   └── ThemeSettings
│   └── * → NotFoundPage
```

## State Management

Five Zustand stores manage frontend state:

| Store | File | Purpose |
|---|---|---|
| `useChatStore` | `stores/chat.ts` | Messages, input, connection, session |
| `useFileChangesStore` | `stores/fileChanges.ts` | File diff state for DiffViewer |
| `useSubagentsStore` | `stores/subagents.ts` | Subagent tree state |
| `useTodoStore` | `stores/todo.ts` | TODO list state |
| `useStatusStore` | `stores/status.ts` | Connection status, loading states |

Stores call **Repositories** (never Transport/IPC directly).

## Layer Structure

### Components → Stores

Components call Stores only. They never call Repository, Transport, or IPC directly.

### Stores → Repositories

Stores call Repository methods. They maintain local UI state.

### Repositories → Transport

Repositories use the `Transport` interface. They don't know which platform they run on.

### Transport

The `Transport` interface abstracts IPC mechanism:

| Implementation | Platform |
|---|---|
| `TauriTransport` | Desktop (Tauri invoke + listen + Channel) |
| `HttpTransport` | Web (fetch + SSE) |
| `MockTransport` | Testing |

## Repository Layer

| Repository | File | Commands |
|---|---|---|
| `configRepository` | `repositories/configRepository.ts` | get_app_config, update_app_config, list_model_providers, verify_model, set_operation_mode, set_autonomy_level |
| `sessionRepository` | `repositories/sessionRepository.ts` | list_sessions, create_session, get_session, delete_session, resume_session, get_session_messages, get_session_model, update_session_model, clear_session_model |
| `chatRepository` | `repositories/chatRepository.ts` | send_chat_query, interrupt_chat, clear_chat, get_chat_messages |
| `workflowRepository` | `repositories/workflowRepository.ts` | approve_tool, respond_to_ask, respond_to_plan |
| `mcpRepository` | `repositories/mcpRepository.ts` | list_mcp_servers, get_mcp_server, create_mcp_server, update_mcp_server, delete_mcp_server, connect_mcp_server, disconnect_mcp_server |
| `skillRepository` | `repositories/skillRepository.ts` | list_skills, toggle_skill_pin |
| `fileRepository` | `repositories/fileRepository.ts` | browse_directory, verify_path, list_workspace_files |

## Event System

Events follow the `domain.object.action` naming convention:

```
chat.message.chunk
chat.message.completed
chat.tool.executing
chat.thinking.block
chat.approval.required
session.activity
config.updated
mcp.server.connected
mcp.servers.updated
```

Events are received via `Transport.onEvent()` (backed by Tauri `listen()`).

## Design System

The visual design is defined in `DESIGN.md` and implemented via CSS custom properties
in `src/index.css`. Components consume CSS variables (`--color-*`, `--radius-*`,
`--space-*`) — no hardcoded values.
