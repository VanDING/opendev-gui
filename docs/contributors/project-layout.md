# Project Layout

## Top-Level

```
opendev-desktop/
├── Cargo.toml              Workspace manifest + shared dependencies
├── Cargo.lock              Locked dependency versions
├── rustfmt.toml            Rust formatting config
├── deny.toml               Cargo-deny policy (licenses, bans, sources)
├── .cargo/                 Cargo config (audit.toml)
├── .github/workflows/      CI pipeline
├── docs/                   Governance, architecture, engineering, contributors
├── crates/                 23 library crates
├── src/                    TypeScript/React frontend
├── src-tauri/              Tauri desktop shell
├── public/                 Static frontend assets
├── index.html              Frontend entry point
├── vite.config.ts          Vite configuration
├── package.json            Node.js dependencies
├── tsconfig.json           TypeScript configuration
└── README.md / CHANGELOG.md / ARCHITECTURE.md / DESIGN.md
```

## Frontend (`src/`)

```
src/
├── main.tsx                      React entry point
├── App.tsx                       Router + layout
├── index.css                     Global styles, CSS custom properties
├── pages/                        Route-level components
│   ├── ChatPage.tsx              Main chat interface
│   └── NotFoundPage.tsx          404
├── components/
│   ├── Chat/                     Chat components (MessageList, InputBox, etc.)
│   ├── Layout/                   Layout components (TopBar, Sidebar, Modals)
│   ├── Settings/                 Settings components (Model, MCP, Theme)
│   └── ui/                       Shared UI components (Button, Input, Modal, etc.)
├── api/                          API layer (client.ts, websocket.ts, mcp.ts)
├── stores/                       Zustand state stores (chat, fileChanges, etc.)
├── hooks/                        React hooks (useWorkspaces)
├── contexts/                     React contexts (ThemeContext)
├── types/                        TypeScript types (hand-written + generated)
└── constants/                    Constants (spinner)
```

## Backend (`crates/`)

```
crates/
├── opendev-models/               Core domain types (SessionEvent, ChatMessage, etc.)
├── opendev-config/               Config loading, paths, profiles
├── opendev-http/                 HTTP client, provider adapters (OpenAI, Anthropic, etc.)
├── opendev-context/              Context management, compaction, retrieval
├── opendev-history/              Event sourcing, SQLite session store, cost tracking
├── opendev-tools-core/           BaseTool trait, ToolRegistry, middleware
├── opendev-tools-impl/           Tool implementations (file_edit, bash, web_fetch, etc.)
├── opendev-tools-lsp/            LSP integration
├── opendev-tools-symbol/         Symbol refactoring
├── opendev-agents/               Main agent, ReAct loop, subagents, skills
├── opendev-runtime/              Approval, permissions, event bus, task mgmt
├── opendev-web/                  Axum HTTP server, WebSocket, routes
├── opendev-mcp/                  MCP client
├── opendev-channels/             Telegram channel
├── opendev-memory/               Memory system (SQLite FTS5)
├── opendev-observability/        Logging, tracing, OTLP, Perfetto
├── opendev-workflow/             Workflow pipeline/barrier/loop
├── opendev-hooks/                Lifecycle hooks
├── opendev-plugins/              Plugin marketplace
├── opendev-sandbox/              Sandbox (stubs)
├── opendev-cli/                  CLI binary entry point (composition root)
├── opendev-tui/                  Terminal UI (ratatui)
├── opendev-repl/                 Interactive REPL
└── opendev-sandbox/              see opendev-sandbox above
```

## Desktop Shell (`src-tauri/`)

```
src-tauri/
├── Cargo.toml                    Tauri dependencies
├── tauri.conf.json               Tauri configuration
├── capabilities/default.json     Tauri capabilities
├── build.rs                      Tauri build script
├── icons/                        App icons
└── src/
    ├── main.rs                   Tauri app entry (window setup, state mgmt)
    └── server.rs                 Embedded Axum server builder
```
