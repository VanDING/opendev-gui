# OpenDev Desktop 项目状态报告

> **生成日期**：2026-06-27
> **面向读者**：ChatGPT（无法直接查看代码）
> **报告目的**：共享完整项目状态，供开发计划讨论
> **基于审计**：`docs/AUDIT_REPORT.md`（2026-06-26）、`docs/REMEDIATION_PLAN.md`

---

## 一、项目概览

**OpenDev Desktop** 是一个由 LLM（大语言模型）驱动的 AI 编程智能体桌面应用，遵循分层架构，将领域逻辑、基础设施、应用编排和用户界面严格分离。

- **项目定位**：AI 编程智能体（AI Coding Agent）的桌面 GUI 形态
- **核心能力**：通过 ReAct 循环（Reasoning + Acting 循环）执行代码生成、文件编辑、命令执行、Web 搜索等任务
- **支持模型供应商**：OpenAI、Anthropic、Google Gemini、AWS Bedrock、Groq、Mistral、Ollama
- **多端形态**：CLI / TUI（终端 UI）/ Web Server / Tauri Desktop 四种用户界面共用同一核心运行时
- **核心子系统**：子智能体（subagent）、技能（skill）、插件市场、长期记忆（SQLite FTS5）、MCP 工具集成
- **远程交互**：Telegram 频道桥接
- **当前版本**：`v0.1.9`
- **许可证**：MIT
- **仓库地址**：`https://github.com/opendev-to/opendev`
- **Rust 版本要求**：`1.94`
- **代码规模**：892 个 `.rs` 文件、24 个 workspace crate、约 7.2 MB Rust 源码

---

## 二、技术栈

### 2.1 Rust 后端

**Tauri 配置：**

| 配置项 | 值 |
|--------|----|
| Tauri 版本 | 2.x（`tauri = { version = "2" }`） |
| 启用的 feature | `custom-protocol`（默认） |
| 编译期依赖 | `tauri-build = { version = "2" }` |
| 桌面 binary 名 | `opendev-desktop`（version `0.1.0`） |

**Workspace 公共依赖（关键项）：**

| 类别 | 依赖 | 版本 / Feature |
|------|------|----------------|
| 序列化 | `serde` | `1` + `derive` |
| 序列化 | `serde_json` | `1` |
| 序列化 | `serde_yaml` | `0.9` |
| 序列化 | `chrono` | `0.4` + `serde` |
| 序列化 | `uuid` | `1` + `v4`, `serde` |
| 序列化 | `strum` | `0.28` + `derive` |
| 异步 | `tokio` | `1` + `full` |
| 异步 | `tokio-util` | `0.7` |
| 异步 | `async-trait` | `0.1` |
| HTTP | `reqwest` | `0.13` + `json`, `rustls`, `stream` |
| HTTP | `axum` | `0.8` + `ws`（仅 opendev-web 使用） |
| HTTP | `tower` / `tower-http` | `0.5` / `0.6` + `cors`, `fs` |
| HTTP | `futures` / `http-body-util` | `0.3` / `0.1` |
| 日志 | `tracing` | `0.1` |
| 日志 | `tracing-subscriber` | `0.3` + `env-filter`, `json` |
| 日志 | `tracing-appender` | `0.2` |
| 错误 | `thiserror` / `anyhow` | `2` / `1` |
| 系统 | `libc` | `0.2` |
| 正则 | `regex` | `1` |
| 文件系统 | `ignore` / `glob` | `0.4` / `0.3` |
| TUI | `ratatui` | `0.30` |
| TUI | `crossterm` | `0.29` + `event-stream` |
| TUI | `unicode-width` | `0.2` |
| CLI | `clap` | `4` + `derive` |
| CLI | `dirs-next` | `2` |
| 类型生成 | `ts-rs` | `10` + `serde-json-impl`, `chrono-impl` |
| 测试 | `tempfile` | `3` |
| 数据库 | `sqlx` | `0.8` + `sqlite`, `runtime-tokio`, `chrono` |
| 哈希 | `sha2` / `hex` | `0.10` / `0.4` |
| URL | `url` | `2` |

**src-tauri 特有依赖：** `tauri = "2"`、`tokio = "1"`（`full`）、`async-trait = "0.1"`，并按需依赖 `opendev-web`、`opendev-config`、`opendev-history`、`opendev-models`、`opendev-http`、`opendev-runtime`、`opendev-agents`、`opendev-mcp`。

### 2.2 TypeScript / React 前端

| 类别 | 依赖 | 版本 |
|------|------|------|
| UI 框架 | `react` / `react-dom` | `^19.2.7` |
| 状态管理 | `zustand` | `^5.0.14` |
| 路由 | `react-router-dom` | `^7.18.0` |
| 工具提示 / Toast | `sonner` | `^2.0.7` |
| Markdown | `markdown-it` + `@types/markdown-it` | `^14.2.0` |
| 代码高亮 | `shiki` | `^4.2.0` |
| 流程图 | `@xyflow/react` | `^12.10.1` |
| 图标 | `lucide-react` | `^1.21.0` |
| 对话框 | `@radix-ui/react-dialog` | `^1.1.17` |
| Tauri 客户端 | `@tauri-apps/api` | `^2.0.0` |
| Tauri CLI | `@tauri-apps/cli` | `^2.0.0` |
| 构建工具 | `vite` | `^8.1.0` |
| 构建工具 | `@vitejs/plugin-react` | `^6.0.3` |
| 样式 | `tailwindcss` + `@tailwindcss/vite` | `^4.3.1` |
| 类型 | `typescript` | `^6.0.3` |
| 类型 | `@types/react` / `@types/react-dom` | `^19.2.17` / `^19.2.3` |
| Lint | `eslint` + `@eslint/js` | `^9.0.0` |
| Lint | `eslint-plugin-react-hooks` / `eslint-plugin-react-refresh` | `^7.0.0` / `^0.4.0` |
| Lint | `typescript-eslint` | `^8.0.0` |

---

## 三、架构总览

### 3.1 分层架构

```
┌──────────────────────────────────────────────────┐
│   React Components                                │
│         │                                        │
│   Stores (Zustand) — 5 stores                    │
│         │                                        │
│   Repositories — 7 个领域仓储                     │
│         │                                        │
│   Transport — 平台无关 (TauriTransport)          │
├──────────────────────────────────────────────────┤  ← 进程边界
│   Desktop Interface (Tauri)                       │
│     Commands · Events · Streams                  │
│         │                                        │
│   Application Services — 8 个服务                 │
│         │                                        │
│   Core — 领域类型、Trait                          │
│         │                                        │
│   Infrastructure — HTTP / SQLite / MCP / FS       │
└──────────────────────────────────────────────────┘
```

### 3.2 通信模型（ADR-0005 规范）

整个 Desktop 只允许三种通信机制：

| 类别 | 机制 | 适用场景 |
|------|------|---------|
| **Command** | `Transport.invoke()` → `Result<T>` | CRUD、配置、状态查询（同步） |
| **Event** | `Platform.emit_event()` → `Transport.onEvent()` | 全局状态变更广播（事件） |
| **Stream** | `Transport.openStream()` → 持续推送 | Chat 消息流、Workflow 执行、Tool 进度 |

**事件命名规范**：`domain.object.action`（如 `chat.message.chunk`、`mcp.server.connected`）。禁止使用 `done`、`finished`、`message`、`update` 等裸命名。

---

## 四、Rust 后端详细结构

### 4.1 24 个 Workspace Crates

| # | Crate | 内部依赖数 | 职责 |
|---|-------|-----------|------|
| 1 | `opendev-models` | 0 | 核心数据模型（`Session`、`ChatMessage`、`AppConfig`、`ToolCall`、`FrontendEvent`），ts-rs TypeScript 类型生成源 |
| 2 | `opendev-config` | 1 | 配置加载、路径解析、`Paths`、`ModelRegistry`、`ConfigLoader`、配置迁移 |
| 3 | `opendev-http` | 2 | LLM HTTP 客户端、`ProviderAdapter` trait、`CircuitBreaker`、`KeyRotation`、`UserStore` |
| 4 | `opendev-context` | 2 | 上下文管理、压缩、环境指令解析 |
| 5 | `opendev-history` | 2 | `SessionManager`（JSONL + SQLite）、会话事件存储、快照、文件检查点 |
| 6 | `opendev-tools-core` | 2 | 工具 trait 定义（`BaseTool`，20+ 方法，18 个默认实现）、注册表、策略、中间件 |
| 7 | `opendev-tools-impl` | 9 ⚠️ | 具体工具实现：bash、文件读写、Web fetch/search、agent、TODO、VLM |
| 8 | `opendev-tools-lsp` | 3 | LSP 集成工具 |
| 9 | `opendev-tools-symbol` | 4 | 符号导航工具 |
| 10 | `opendev-agents` | 8 | 智能体运行时（`MainAgent`、`ReactLoop`、`SubagentManager`、`AgentEventCallback`、`SkillLoader`、`PromptComposer`、`LlmCaller`） |
| 11 | `opendev-web` | 5 | Web/Event 基础设施（`AppState`、`WsBroadcast`、axum routes、websocket handler） |
| 12 | `opendev-repl` | — | 交互式 REPL |
| 13 | `opendev-cli` | 15 ⚠️ | CLI 入口（`clap`）、`WebAgentExecutor`、`WebEventCallback`、TUI runner（**God Object**） |
| 14 | `opendev-mcp` | 2 | MCP 协议客户端（`McpTransport` trait、stdio/http/sse、`McpManager`） |
| 15 | `opendev-channels` | 1 | 多通道支持（`telegram/` 模块），可扩展 Discord/Slack |
| 16 | `opendev-tui` | 5 | Ratatui 终端 UI |
| 17 | `opendev-runtime` | 2 | 运行时服务（`EventBus`、任务管理器、审批系统、密钥检测、TODO、权限）— 83 文件的"杂货袋" |
| 18 | `opendev-hooks` | 0 | Git 钩子 / 生命周期钩子集成 |
| 19 | `opendev-plugins` | 1 | 插件管理器 + 市场 |
| 20 | `opendev-sandbox` | 2 | 代码执行沙箱 — **100% TODO 桩代码** |
| 21 | `opendev-memory` | 0 | 长期/短期记忆（SQLite FTS5）、`MemoryFacade`、`WriteGate`、`CascadeBuffer` |
| 22 | `opendev-observability` | 1 | 遥测 / tracing 配置 |
| 23 | `opendev-workflow` | 0 | 工作流引擎（Pipeline / Barrier / Loop） |
| 24 | `src-tauri` | 6 | Tauri 桌面 binary（`application` + `interface` + `server`） |

**依赖倒置问题**：`opendev-tools-impl → opendev-agents` 形成潜在循环依赖隐患，计划在 v0.3+ 提取共享类型到 `opendev-models`。

### 4.2 src-tauri 内部结构

```
src-tauri/src/
├── main.rs                    — 入口，注册 34 个 commands，启动 Tauri
│                                + spawn_event_bridge() 将 WsBroadcast
│                                  桥接到 Tauri emit
├── server.rs                  — 临时模块：保留 AppState 初始化用于
│                                agent event broadcast，不启动 HTTP listener
├── application/
│   ├── mod.rs                 — AppServices 聚合体（依赖注入容器）
│   ├── config_service.rs      — ConfigService
│   ├── session_service.rs     — SessionService
│   ├── chat_service.rs        — ChatService（实现 AgentExecutor trait）
│   ├── workflow_service.rs    — WorkflowService（通过 oneshot 通道处理审批）
│   ├── mcp_service.rs         — MCPService
│   ├── skill_service.rs       — SkillService
│   ├── file_service.rs        — FileService
│   └── system_service.rs      — SystemService
└── interface/
    ├── mod.rs                 — `pub mod desktop; pub mod services;`
    ├── services.rs            — build_services() 依赖注入工厂
    └── desktop/
        ├── mod.rs             — Desktop 适配器根
        ├── platform.rs        — DesktopPlatform trait + TauriPlatform 实现
        │                        + StreamSender / StreamReceiver
        ├── contract/          — 7 个 DTO 模块
        │   ├── config.rs
        │   ├── session.rs
        │   ├── chat.rs        — ChatStreamEvent enum
        │   ├── workflow.rs
        │   ├── mcp.rs
        │   ├── skills.rs
        │   └── files.rs
        ├── commands/          — 7 个 command 模块（DTO → Service）
        │   ├── config.rs      — 6 commands
        │   ├── session.rs     — 9 commands
        │   ├── chat.rs        — 4 commands
        │   ├── workflow.rs    — 3 commands
        │   ├── mcp.rs         — 7 commands
        │   ├── skills.rs      — 2 commands
        │   └── files.rs       — 3 commands
        └── events/
            └── mod.rs         — Event enums（domain.object.action 命名）
```

### 4.3 Application Services 聚合

`AppServices` 结构包含 8 个服务实例（`main.rs` 通过 `app.manage(services)` 注册为 Tauri managed state）：

```rust
pub struct AppServices {
    pub config: ConfigService,
    pub session: SessionService,
    pub chat: ChatService,
    pub workflow: WorkflowService,
    pub mcp: MCPService,
    pub skill: SkillService,
    pub file: FileService,
    pub system: SystemService,
}
```

**Application Services 的不变量**（代码注释中明确）：
- 不知道 Desktop / HTTP / CLI / 任何平台
- 不依赖 `tauri` / `axum` / `clap`
- 协调 Domain + 调用 Infrastructure

### 4.4 DesktopPlatform trait

`interface/desktop/platform.rs` 定义了**唯一**抽象层：

```rust
#[async_trait]
pub trait DesktopPlatform: Send + Sync + 'static {
    fn manage<T: Send + Sync + 'static>(&self, state: T);
    fn emit_event(&self, event: &str, payload: impl Serialize + Clone);
    fn create_stream<T: Serialize + Send + 'static + Clone>(
        &self,
    ) -> (StreamSender<T>, StreamReceiver<T>);
    fn app_handle(&self) -> Option<tauri::AppHandle> { None }
}
```

`TauriPlatform` 是其 Tauri 实现。注释明确指出：未来可能实现 `SlintPlatform`、`WryPlatform` 等。

### 4.5 34 个 Tauri Commands（完整列表）

**注**：`main.rs` 中实际注册的命令数为 **34 个**（不是 outline 描述的 31 个），分布如下：

#### Config（6 个）

| # | Command | Request | Response |
|---|---------|---------|----------|
| 1 | `get_app_config` | — | `ConfigResponse` |
| 2 | `update_app_config` | `UpdateConfigRequest` | `void` |
| 3 | `set_operation_mode` | `ModeUpdateRequest` | `void` |
| 4 | `set_autonomy_level` | `AutonomyUpdateRequest` | `void` |
| 5 | `list_model_providers` | — | `Vec<ProviderInfo>` |
| 6 | `verify_model` | `VerifyModelRequest` | `VerifyModelResponse` |

#### Session（9 个）

| # | Command | Request | Response |
|---|---------|---------|----------|
| 7 | `list_sessions` | — | `Vec<SessionInfoResponse>` |
| 8 | `create_session` | `CreateSessionRequest` | `CreateSessionResponse` |
| 9 | `get_session` | `id` | `serde_json::Value` |
| 10 | `delete_session` | `id` | `void` |
| 11 | `resume_session` | `id` | `String` |
| 12 | `get_session_messages` | `id` | `Vec<serde_json::Value>` |
| 13 | `get_session_model` | `id` | `serde_json::Value` |
| 14 | `update_session_model` | `id`, `SessionModelUpdateRequest` | `void` |
| 15 | `clear_session_model` | `id` | `void` |

#### Chat（4 个）

| # | Command | Request | Response |
|---|---------|---------|----------|
| 16 | `send_chat_query` | `ChatQueryRequest` | `ChatActionResponse` |
| 17 | `interrupt_chat` | — | `ChatActionResponse` |
| 18 | `clear_chat` | `workspace?` | `ChatActionResponse` |
| 19 | `get_chat_messages` | — | `Vec<serde_json::Value>` |

#### Workflow（3 个）

| # | Command | Request | Response |
|---|---------|---------|----------|
| 20 | `approve_tool` | `ApprovalResponse` | `WorkflowActionResult` |
| 21 | `respond_to_ask` | `AskUserResponse` | `WorkflowActionResult` |
| 22 | `respond_to_plan` | `PlanApprovalResponse` | `WorkflowActionResult` |

#### MCP（7 个）

| # | Command | Request | Response |
|---|---------|---------|----------|
| 23 | `list_mcp_servers` | — | `MCPServerListResponse` |
| 24 | `get_mcp_server` | `name` | `serde_json::Value` |
| 25 | `create_mcp_server` | `CreateMCPServerRequest` | `MCPActionResponse` |
| 26 | `update_mcp_server` | `name`, `UpdateMCPServerRequest` | `MCPActionResponse` |
| 27 | `delete_mcp_server` | `name` | `MCPActionResponse` |
| 28 | `connect_mcp_server` | `name` | `MCPActionResponse` |
| 29 | `disconnect_mcp_server` | `name` | `MCPActionResponse` |

#### Skills（2 个）

| # | Command | Request | Response |
|---|---------|---------|----------|
| 30 | `list_skills` | — | `Vec<SkillResponse>` |
| 31 | `toggle_skill_pin` | `name` | `TogglePinResponse` |

#### Files（3 个）

| # | Command | Request | Response |
|---|---------|---------|----------|
| 32 | `browse_directory` | `BrowseDirectoryRequest` | `BrowseDirectoryResponse` |
| 33 | `verify_path` | `VerifyPathRequest` | `serde_json::Value` |
| 34 | `list_workspace_files` | `query?` | `serde_json::Value` |

**Command 文件中的不变量**（代码注释明确）：
1. 接收反序列化 DTO（来自 Tauri invoke）
2. 映射为 Service 输入
3. 调用 Application Service
4. 返回 DTO Response

Commands **禁止**：业务逻辑、文件操作、Agent 操作、`if` / `match` 业务判断。

### 4.6 server.rs 的临时状态

`src-tauri/src/server.rs` 保留 AppState 初始化（用于 agent event broadcast），但**不再启动 HTTP listener**。文件头注释明确说明：

> "This module will be fully removed when agent events flow directly through Application Services."

当前流程：
1. `server::setup_event_broadcast()` 创建 `EventBridgeHandle`
2. `main.rs` 调用 `spawn_event_bridge(app.handle(), handle.broadcast_rx)`
3. 桥接器订阅 `tokio::sync::broadcast::Receiver<WsBroadcast>`
4. 收到消息后调用 `app.emit(&msg.msg_type, msg.data)` — 用 `msg_type` 作为 Tauri 事件名
5. 当 agent events 直接通过 Application Services 路由时，此文件可完全删除

---

## 五、前端详细结构

### 5.1 目录结构

```
src/
├── main.tsx                     — 入口（注释明确：Tauri IPC 始终可用，
│                                  无需 connection step，无 wsClient.connect()）
├── App.tsx                      — BrowserRouter：
│                                  /chat → ChatPage，/ → /chat，* → NotFoundPage
├── api/
│   └── eventBridge.ts           — Tauri event → {type, data} 桥接
│                                  （取代旧的 wsClient）
├── repositories/
│   ├── Transport.ts             — Transport 接口定义
│   ├── TauriTransport.ts        — Tauri IPC 实现
│   ├── configRepository.ts      — 6 个方法
│   ├── sessionRepository.ts     — 9 个方法
│   ├── chatRepository.ts        — 5 个方法
│   ├── workflowRepository.ts    — 3 个方法
│   ├── mcpRepository.ts         — 7 个方法
│   ├── skillRepository.ts       — 2 个方法
│   ├── fileRepository.ts        — 3 个方法
│   └── index.ts                 — 单例导出（全部 wired 到 tauriTransport）
├── stores/
│   ├── chat.ts                  — useChatStore（per-session state，25+ event handlers）
│   ├── status.ts                — useStatusStore（状态栏数据）
│   ├── todo.ts                  — useTodoStore（agent TODO 项）
│   ├── subagents.ts             — useSubagentStore（子智能体树）
│   └── fileChanges.ts           — useFileChangesStore（diff viewer 数据）
├── types/
│   ├── index.ts                 — Message, Session, Config, WSMessage, types
│   ├── mcp.ts                   — MCP 特定类型
│   └── generated/               — ts-rs 从 Rust 模型自动生成
│                                  （如 FrontendEvent.ts 等）
├── components/
│   ├── Chat/                    — 22 个组件
│   │   ├── ChatInterface.tsx, InputBox.tsx, MessageList.tsx, MessageItem.tsx
│   │   ├── ToolCallMessage.tsx, ThinkingBlock.tsx, SubagentTree.tsx
│   │   ├── ApprovalDialog.tsx, AskUserDialog.tsx, PlanApprovalDialog.tsx
│   │   ├── StatusBar.tsx, StatusDialog.tsx, ProgressIndicator.tsx
│   │   ├── LandingPage.tsx, WelcomeScreen.tsx, QuickActions.tsx
│   │   ├── FileChangesButton.tsx, DetailPanel.tsx, TodoPanel.tsx
│   │   ├── CommandPalette.tsx, FileMentionDropdown.tsx, QueueBar.tsx
│   │   ├── MarkdownContent.tsx, BashPreview.tsx, DiffViewer.tsx
│   │   └── MatrixRain.tsx
│   ├── Layout/
│   │   ├── TopBar.tsx, SessionsSidebar.tsx, Breadcrumb.tsx
│   │   ├── NewSessionModal.tsx, DeleteConfirmModal.tsx, SessionModelModal.tsx
│   ├── Settings/
│   │   ├── SettingsModal.tsx, ModelSettings.tsx, ModelSlot.tsx
│   │   ├── MCPSettings.tsx, MCPServerCard.tsx
│   │   ├── AddMCPServerModal.tsx, EditMCPServerModal.tsx, MCPToolsModal.tsx
│   │   ├── SkillsSettings.tsx, ThemeSettings.tsx
│   └── ui/
│       ├── Button.tsx, IconButton.tsx, Input.tsx
│       ├── Modal.tsx, SegmentedControl.tsx, SelectField.tsx, HaloSpinner.tsx
├── hooks/
│   └── useWorkspaces.ts
├── contexts/
│   └── ThemeContext.tsx
├── constants/
│   └── spinner.ts
└── pages/
    ├── ChatPage.tsx
    └── NotFoundPage.tsx
```

### 5.2 关键 TypeScript 类型

来自 `src/types/index.ts`：

```typescript
// 工具调用信息（支持嵌套）
export interface ToolCallInfo {
  id: string;
  name: string;
  parameters: Record<string, any>;
  result?: string | null;
  error?: string | null;
  result_summary?: string | null;
  approved?: boolean | null;
  nested_tool_calls?: ToolCallInfo[] | null;
}

// 消息（6 种 role）
export interface Message {
  role: 'user' | 'assistant' | 'system' | 'tool_call' | 'tool_result' | 'thinking';
  content: string;
  timestamp?: string;
  tool_call_id?: string;
  tool_name?: string;
  tool_args?: Record<string, any>;
  tool_result?: any;
  tool_args_display?: string | null;
  tool_summary?: string | string[] | null;
  tool_success?: boolean;
  tool_error?: string | null;
  tool_calls?: ToolCallInfo[];
  metadata?: Record<string, any>;
  depth?: number;
  parent_tool_call_id?: string;
  thinking_trace?: string | null;
  reasoning_content?: string | null;
  isOptimistic?: boolean;
  optimisticId?: string;
}

// 会话
export interface Session {
  id: string;
  working_dir?: string;
  working_directory?: string;  // 后端实际返回的字段名
  created_at: string;
  updated_at: string;
  message_count: number;
  token_usage?: Record<string, number>;
  title?: string;
  has_session_model?: boolean;
}

// 配置
export interface Config {
  model_provider: string;
  model: string;
  api_key: string | null;
  temperature: number;
  enable_bash: boolean;
  working_directory: string;
}

// WebSocket 兼容事件（30+ 类型联合）
export interface WSMessage {
  type: 'user_message' | 'message_start' | 'message_chunk' | 'message_complete'
      | 'tool_call' | 'tool_result' | 'approval_required' | 'approval_resolved'
      | 'error' | 'pong'
      | 'mcp_status_update' | 'mcp_servers_update'
      | 'mcp:status_changed' | 'mcp:servers_updated'
      | 'connected' | 'disconnected'
      | 'thinking_block' | 'status_update'
      | 'ask_user_required' | 'ask_user_resolved'
      | 'session_activity'
      | 'plan_approval_required' | 'plan_approval_resolved' | 'plan_content'
      | 'subagent_start' | 'subagent_complete'
      | 'parallel_agents_start' | 'parallel_agents_done'
      | 'task_completed' | 'progress'
      | 'nested_tool_call' | 'nested_tool_result'
      | 'full_sync';
  data: any;
}

// 状态栏
export interface StatusInfo {
  mode: 'normal' | 'plan';
  autonomy_level: 'Manual' | 'Semi-Auto' | 'Auto';
  thinking_level?: 'Off' | 'Low' | 'Medium' | 'High';
  model?: string;
  model_provider?: string;
  working_dir?: string;
  git_branch?: string | null;
  session_cost?: number;
  context_usage_pct?: number;
}

// 工具审批
export interface ApprovalRequest {
  id: string;
  tool_name: string;
  arguments: Record<string, any>;
  description: string;
  preview?: string;
}

// 询问用户
export interface AskUserOption { label: string; description: string; }
export interface AskUserQuestion {
  question: string;
  header: string;
  options: AskUserOption[];
  multi_select: boolean;
}
export interface AskUserRequest {
  request_id: string;
  questions: AskUserQuestion[];
}

// 计划审批
export interface PlanApprovalRequest {
  request_id: string;
  plan_content: string;
}

// Per-session state（支持并发 session 切换）
export interface PerSessionState {
  messages: Message[];
  isLoading: boolean;
  error: string | null;
  pendingApproval: ApprovalRequest | null;
  pendingAskUser: AskUserRequest | null;
  pendingPlanApproval: PlanApprovalRequest | null;
  progressMessage: string | null;
  queuedMessages: string[];
  optimisticMessages: Map<string, Message>;
}
```

### 5.3 Transport 接口

```typescript
// src/repositories/Transport.ts
export interface StreamController<T> {
  onData(handler: (data: T) => void): () => void;
  close(): void;
}

export interface Transport {
  invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
  onEvent<T>(event: string, handler: (payload: T) => void): Promise<() => void>;
  openStream<T>(): StreamController<T>;
}
```

`TauriTransport` 的实现要点：
- `invoke` 直接转发到 `@tauri-apps/api/core` 的 `tauriInvoke`
- `onEvent` 包装 `@tauri-apps/api/event` 的 `tauriListen`，返回 unsubscribe
- `openStream` 返回 `TauriStreamController`，支持多个 handler

**Repository 设计**：所有 Repository 工厂函数接收 `Transport` 作为参数（`createXxxRepository(transport)`），由 `src/repositories/index.ts` 用 `tauriTransport` 实例化单例。

---

## 六、五大 Zustand Store 详解

### 6.1 useChatStore（`stores/chat.ts`，约 904 行）

**State（ChatState 接口）：**
- `sessionStates: Record<string, PerSessionState>` — 每会话状态表（并发 session 切换核心）
- `currentSessionId: string | null` — 当前激活会话
- `isConnected: boolean`
- `hasWorkspace: boolean`
- `status: StatusInfo | null`
- `thinkingLevel: 'Off' | 'Low' | 'Medium' | 'High'`
- `runningSessions: Set<string>`
- `sessionListVersion: number`
- `sidebarCollapsed: boolean`

**Actions：**
- `loadSession(sessionId)` — 切换会话（带缓存，无消息时 fetch）
- `sendMessage(content)` — 创建乐观 user message 并发送
- `clearChat()` / `respondToApproval(...)` / `respondToAskUser(...)` / `respondToPlanApproval(...)`
- `toggleMode()` / `cycleAutonomy()` / `cycleThinkingLevel()`
- `sendInterrupt()` / `bumpSessionList()` / `toggleSidebar()` / `setSidebarCollapsed(...)`
- `setConnected(...)` / `setHasWorkspace(...)` / `setStatus(...)`

**Event handlers（通过 `eventBridge.on(...)` 订阅）：** 25+ 个，覆盖以下事件类型
- `connected` / `disconnected`
- `user_message` / `message_start` / `message_chunk` / `message_complete`
- `error` / `task_completed` / `full_sync` / `pong`
- `tool_call` / `tool_result` / `nested_tool_call` / `nested_tool_result`
- `thinking_block`
- `approval_required` / `approval_resolved`
- `ask_user_required` / `ask_user_resolved`
- `plan_approval_required` / `plan_approval_resolved`
- `subagent_start` / `subagent_complete`
- `status_update` / `session_activity` / `progress`

**核心辅助函数：**
- `expandToolCalls(toolCalls, timestamp, depth)` — 递归展开嵌套工具调用为扁平 Message 列表
- `expandMessages(rawMessages)` — 展开原始 API 消息（含 `thinking_trace` / `reasoning_content` / `tool_calls`）为带 role 的扁平 Message 流
- `patchSession(state, sessionId, patch)` — 不可变更新 per-session state

### 6.2 useStatusStore（`stores/status.ts`）

- State：`StatusBarData`（model、provider、tokens、costs、git branch、autonomy、MCP counts、file changes）
- 主要由 `status_update` 事件驱动

### 6.3 useTodoStore（`stores/todo.ts`）

- State：`TodoItem[]`、`planName`、`visible`
- 监听 `tool_result`（识别 `write_todos`、`update_todo` 工具调用）和 `status_update`

### 6.4 useSubagentStore（`stores/subagents.ts`）

- State：`Map<subagentId, SubagentState>`（name、task、activeTools、completedTools）
- 监听 `subagent_start`、`nested_tool_call`、`nested_tool_result`、`subagent_complete`、`status_update`

### 6.5 useFileChangesStore（`stores/fileChanges.ts`）

- State：`FileChange[]`、`FileChangesSummary`
- 当前通过 `sessionRepository.getSessionMessages()` 加载（事件驱动增强尚未完成）

---

## 七、事件系统

### 7.1 事件命名规范

**目标命名规范**：`domain.object.action`

**当前后端实际使用的事件名**（从 `WSMessage['type']` 联合类型与 `spawn_event_bridge` 的 `msg.msg_type` 转发推导）：

| 域 | 事件名 | 用途 |
|----|--------|------|
| 连接 | `connected`, `disconnected`, `pong` | 连接生命周期 |
| Chat | `user_message`, `message_start`, `message_chunk`, `message_complete` | 消息流 |
| Tool | `tool_call`, `tool_result`, `nested_tool_call`, `nested_tool_result` | 工具执行 |
| Thinking | `thinking_block` | 思考块 |
| Approval | `approval_required`, `approval_resolved` | 工具审批 |
| Ask | `ask_user_required`, `ask_user_resolved` | 用户询问 |
| Plan | `plan_approval_required`, `plan_approval_resolved`, `plan_content` | 计划审批 |
| Status | `status_update`, `progress`, `task_completed` | 状态/进度 |
| Subagent | `subagent_start`, `subagent_complete`, `parallel_agents_start`, `parallel_agents_done` | 子智能体 |
| Session | `session_activity` | 会话活动 |
| Sync | `full_sync` | 全量同步 |
| MCP | `mcp:status_changed`, `mcp:servers_updated`（冒号分隔，**与文档规范不一致**） | MCP 状态 |
| Error | `error` | 错误 |

**命名不一致（已知问题）**：
- 设计规范要求 `domain.object.action`（如 `mcp.server.connected`）
- 实际后端事件使用 `mcp:status_changed` / `mcp:servers_updated`（冒号分隔、动名词不分）
- 类型定义同时保留了 `mcp_status_update` / `mcp_servers_update`（下划线分隔）的别名 — 表明存在迁移中状态

### 7.2 事件流转链路

```
Agent Loop
  → AgentEventCallback (trait)
    → WebEventCallback 实现（opendev-cli / opendev-web）
      → AppState.broadcast()
        → tokio::sync::broadcast::channel<WsBroadcast>
          → spawn_event_bridge()  ←  src-tauri/main.rs
            → app.emit(msg.msg_type, msg.data)
              → Tauri event system
                → Transport.onEvent()  ←  src/repositories/TauriTransport.ts
                  → eventBridge.on()  ←  src/api/eventBridge.ts
                    → Store handlers
                      → React re-render
```

### 7.3 `WsBroadcast` 数据结构（opendev-web/src/state）

```rust
pub struct WsBroadcast {
    pub msg_type: String,  // 事件名（直接作为 Tauri emit name）
    pub data: serde_json::Value,  // 事件 payload
}
```

---

## 八、数据流详解

### 8.1 Chat Query 完整链路

1. **用户输入** → `InputBox.tsx` 组件
2. **Store action** → `useChatStore.sendMessage(content)`
   - 创建乐观 user message（`isOptimistic: true`）
   - 调用 `chatRepository.sendQuery(message, sessionId)`
3. **Repository** → `tauriTransport.invoke("send_chat_query", { req: { message, session_id } })`
4. **Tauri IPC** → `src-tauri` command `send_chat_query`
5. **Command** → `interface/desktop/commands/chat.rs` 仅做 DTO 映射
6. **Application Service** → `ChatService` 验证 session、标记 running、fire `WebAgentExecutor`
7. **Agent 运行时** → `MainAgent` / `ReactLoop`：
   - `PromptComposer` 构建 system prompt
   - `AdaptedClient` 通过 `ProviderAdapter` 调 LLM
   - `LlmCaller` 解析为 tool calls / content
   - `ReactLoop` 决策：continue / dispatch tool / complete
   - `ToolRegistry` → `BaseTool::execute()` → 工具结果回环
8. **事件回流** → `WebEventCallback` → `AppState.broadcast()` → `tokio::broadcast`
9. **桥接到 Tauri** → `spawn_event_bridge` → `app.emit("message_chunk", ...)`
10. **前端接收** → `TauriTransport.onEvent()` → `eventBridge` → `useChatStore` 处理器
11. **Store 更新** → `sessionStates[id].messages` 追加 → React `MessageList` 重渲染

### 8.2 Config 读取链路

1. `useChatStore` / 其他 store 调用 `configRepository.getConfig()`
2. `configRepository` → `tauriTransport.invoke("get_app_config")`
3. Tauri command → `ConfigService.get_config()` → 读取 `RwLock<AppConfig>`
4. 返回 `ConfigResponse`（**API keys 被掩码**）

### 8.3 配置加载初始化链路（main.rs setup）

```
working_dir = current_dir()
  → Paths::new(Some(working_dir))
    → ConfigLoader::load(paths.global_settings(), paths.project_settings())
    → SessionManager::new(paths.global_sessions_dir())
    → ModelRegistry::new()
    → build_services(config, session_manager, model_registry, working_dir)
      → AppServices { config, session, chat, workflow, mcp, skill, file, system }
    → app.manage(services)
  → server::setup_event_broadcast(&working_dir)
    → EventBridgeHandle { broadcast_rx }
  → spawn_event_bridge(app.handle(), handle.broadcast_rx)
```

---

## 九、当前状态与已知问题

### 9.1 Sprint v0.2 已完成项（Desktop Native IPC 重构）

- ✅ Desktop 完全 Native IPC（**零 HTTP 依赖**）
- ✅ 34 个 Tauri Commands 注册（实际计数，非 31）
- ✅ 8 个 Application Services 提取
- ✅ 7 个前端 Repository 全部 wired 到 `tauriTransport`
- ✅ 所有 `fetch()` / `apiClient` 已删除
- ✅ WebSocket 删除，由 Tauri events 取代
- ✅ Axum HTTP server 从 desktop 移除
- ✅ `__OPENDEV_PORT__` / port 注入机制已删除
- ✅ `cargo check` 0 errors

### 9.2 关键架构清理结果

| 项 | Before | After |
|----|--------|-------|
| 进程间通信 | HTTP + WebSocket | Tauri IPC + Tauri Events |
| 前端 HTTP 客户端 | `apiClient` / `fetch` | `TauriTransport.invoke` |
| 前端 WS 客户端 | `wsClient` | `TauriTransport.onEvent` |
| 后端 HTTP 监听 | axum listener | **无** |
| 配置传输 | 端口注入 | 直接 invoke |
| 数据流 | HTTP request/response | 进程内 DTO 映射 |

### 9.3 来自 AUDIT_REPORT.md / REMEDIATION_PLAN.md 的已知问题

#### Critical（必须立即处理）

| ID | 问题 | CWE | 位置 |
|----|------|-----|------|
| SEC-01 | **SSRF 漏洞**：`WebFetch` 无私有 IP 过滤（127.0.0.1、10.x、192.168.x、169.254.x） | CWE-918 | `tools-impl/src/web_fetch/mod.rs:92-99` |
| REL-04 | **Bedrock 适配器 SigV4 签名缺失**（5 个 TODO） — AWS 请求会 403 | — | `http/src/adapters/bedrock/` |

#### High（近期处理）

| ID | 问题 | 位置 |
|----|------|------|
| ENG-01 | **无 CI/CD**：无 `.github/workflows`、无 `Makefile`、无 `rustfmt.toml`、`clippy.toml`、`deny.toml` | 仓库根 |
| REL-02 | `std::sync::RwLock` 用在 async 上下文（task_manager、team_manager、team_task_list） | opendev-runtime |
| PERF-02 | **无 `spawn_blocking`** — 文件 I/O、SQLite、bash 进程在 async 工作线程上 | 5+ 文件 |
| REL-01 | Mutex 中毒 → 进程崩溃（25+ 处 `.expect("poisoned")`） | 多个文件 |
| PERF-01 | HTML 转换器每次调用重编译 ~17 个正则 | `web_fetch/html_converter.rs` |

#### Medium（季度内处理）

- Mutex 中毒恢复、unsafe 块 SAFETY 注释、HTML 正则缓存、SQLite LIKE 通配符注入、Cookie Secure flag

#### Architecture Debt（计划 v0.3+ 处理）

| 债务项 | 影响 | 计划 |
|--------|------|------|
| `opendev-tools-impl → opendev-agents` 依赖倒置 | 工具层不可复用 | v0.3+ 提取共享类型 |
| CLI God Object（15 个内部依赖） | 编译时间增加 | v0.4+ 拆分为薄二进制 + `opendev-app` 组合根 |
| `opendev-runtime` 杂货袋（83 文件） | 模块边界模糊 | v0.5+ 拆分 |
| `opendev-sandbox` 100% 桩代码 | 编译时间 + 二进制体积 | feature-gate 或移除 |
| 双 SQLite（memory + history） | 连接池竞争 | 共享 `opendev-storage` |

#### 事件命名不匹配（本次重构发现）

- **前端 `WSMessage.type` 联合类型**同时包含 `mcp_status_update`/`mcp_servers_update`（下划线）和 `mcp:status_changed`/`mcp:servers_updated`（冒号）
- **后端**实际广播冒号分隔的事件名（来自 `WsBroadcast.msg_type` 字符串拼接）
- **设计规范**要求 `domain.object.action`（如 `mcp.server.connected`）— 实际命名两套都不符合
- 状态：迁移中，前后端需要统一

#### 其他

- 965 个间接依赖的 crate（`Cargo.lock`）
- 3,621 个 `unwrap()`（~90% 在测试中）
- 153 个 `expect()`（生产代码中 25+ 处是 "lock poisoned"）
- 5 个生产 `unsafe` 块（3 个缺 `// SAFETY:` 注释）
- `dirs` v5 + v6 在不同 crate 中并存
- `proptest` 在 `opendev-tools-impl` 声明但未使用

### 9.4 如何运行

```bash
# 必须使用 cargo tauri dev — 不能只跑 npm run dev
# 因为 Tauri 事件系统需要 Rust 端注册 handler 与事件桥接
cargo tauri dev

# 仅前端 Vite 开发（无 Rust 后端，事件系统失效，不推荐）
npm run dev

# 类型生成（从 Rust 模型生成 TypeScript）
cargo test -p opendev-models export_frontend_types

# 构建
cargo tauri build
```

### 9.5 开发命令

```bash
# Rust
cargo check
cargo build
cargo test                 # 3,183 个测试，326 个测试文件
cargo fmt --check          # CI 强制
cargo clippy               # 尚未 CI 强制

# 前端
npm run dev                # Vite
npm run build              # tsc --noEmit && vite build
npm run lint               # eslint
```

---

## 十、开发工作流与规范

### 10.1 代码质量工具链

| 工具 | 状态 | 说明 |
|------|------|------|
| `cargo fmt` | CI 强制 | 统一格式 |
| `cargo clippy` | **未 CI 强制** | 建议设为 `-D warnings` |
| `cargo test` | 手动 | 3,183 个测试 |
| `cargo audit` | **未集成** | 扫描已知漏洞 |
| `cargo deny` | **未集成** | 许可证合规 |
| `tsc --noEmit` | 在 `npm run build` 中 | TypeScript 类型检查 |
| `eslint` | `npm run lint` | 前端 Lint |

### 10.2 类型生成（TypeScript ← Rust）

通过 `ts-rs` 从 `opendev-models` 自动生成 TypeScript 类型到 `src/types/generated/`：

```bash
cargo test -p opendev-models export_frontend_types
```

### 10.3 架构合规验证命令

```bash
# Command 中无业务逻辑
grep -r "std::fs\|tokio::fs" interface/desktop/commands/  →  0

# Component 无直接 invoke
grep -r "import.*invoke.*from.*tauri" src/components/      →  0

# Store 无直接 invoke
grep -r "import.*invoke.*from.*tauri" src/stores/          →  0

# 前端无 fetch / localhost / WebSocket / apiClient
grep -r "fetch(" src/ --include="*.ts" --include="*.tsx"   →  0
grep -r "localhost" src/ --include="*.ts" --include="*.tsx" →  0
grep -r "API_BASE\|__OPENDEV_PORT__\|getPort" src/         →  0
grep -r "WebSocket\|ws://\|wsClient" src/                  →  0
grep -r "apiClient\." src/                                →  0

# Rust Desktop 不启动 HTTP
cargo check -p opendev-desktop                              →  pass
strings target/debug/opendev-desktop | grep "axum\|cors\|127.0.0.1"  →  0
```

### 10.4 整体工程评分（来自 AUDIT_REPORT.md）

| 维度 | 分数 | 关键发现 |
|------|------|---------|
| 架构 | 68/100 | 领域模型和适配器模式优秀；tools→agents 倒置、CLI God Object、Sandbox 空壳拖累 |
| 代码质量 | 72/100 | 错误类型健壮、测试覆盖好；SQLite sync-over-async 脆弱、14 参数方法、expect("poisoned") 普遍 |
| Rust 最佳实践 | 65/100 | Arc\<dyn Trait\> 惯用、enum 状态机；async 路径 std RwLock、无 spawn_blocking、async_trait on sync trait |
| 安全 | 78/100 | 基础扎实（Argon2、HMAC、参数化 SQL、敏感文件屏蔽）；SSRF 漏洞、Mutex 中毒、unsafe 无注释 |
| 性能 | 72/100 | WAL 模式、LazyLock 使用合理；HTML 正则重编译、无 spawn_blocking、无界通道 |
| 测试 | 72/100 | 3,183 测试、326 文件；proptest 未使用、无 fuzz、测试 unwrap 过量 |
| 可维护性 | 60/100 | 无 CI/CD、无文档、无 lint 自动化 |

### 10.5 Top 10 待解决问题（按影响×紧急度）

| # | 问题 | 严重度 | 紧急度 | 优先级 |
|---|------|-------|-------|--------|
| 1 | SSRF：WebFetch 无私有 IP 过滤 | HIGH | HIGH | P0 |
| 2 | Bedrock SigV4 签名缺失 | HIGH | HIGH | P0 |
| 3 | Sandbox crate 100% 桩代码 | MEDIUM | HIGH | P0 |
| 4 | 无 CI/CD（clippy/test/audit） | HIGH | MEDIUM | P1 |
| 5 | std::sync::RwLock in async | MEDIUM | HIGH | P1 |
| 6 | 无 spawn_blocking | MEDIUM | HIGH | P1 |
| 7 | Mutex 中毒 → 进程崩溃（25+ 处） | MEDIUM | MEDIUM | P1 |
| 8 | HTML 转换器正则重编译 | MEDIUM | MEDIUM | P1 |
| 9 | tools-impl → agents 依赖倒置 | MEDIUM | LOW | P2 |
| 10 | CLI God Object（15 deps） | LOW | LOW | P3 |

### 10.6 REMEDIATION_PLAN 总结

- **4 个 P0 项目**：建立 CI/CD、修复 SSRF、实现 Bedrock SigV4、添加 cargo-deny
- **7 个 P1 项目**：HTML 正则缓存、spawn_blocking、Mutex poison 恢复、无界通道替换、prepare_command 正则、unsafe SAFETY 注释、Cookie Secure flag
- **6 个 P2 项目**：std→tokio RwLock、test env→temp_env、LIKE 转义、HMAC 强化、API key zeroize、unwrap on JSON
- **13 个 P3 项目**（视资源决定）：block_on hack 修复、sandbox 决策、args_map 优化、README、ARC-01~08 架构重构
- **总工时估算**：18-24 人天（含验证）
- **核心原则**："避免为了架构漂亮而进行无价值重构"

---

## 十一、下一步讨论焦点

以下议题适合与 ChatGPT 共同讨论：

1. **v0.2 → v0.3 路线图**：是先打 P0 安全补丁，还是同步进行架构债务清理（tools→agents 解耦）？
2. **事件命名统一**：是彻底迁移到 `domain.object.action`（推荐）还是保留冒号分隔（向后兼容）？
3. **`server.rs` 临时状态**：何时彻底删除？（依赖 Application Services 直接接收 agent events）
4. **CI 策略**：是否在 CI 启用 `cargo clippy -- -D warnings`（现有代码可能有大量 warning）？
5. **架构重构 ROI**：`opendev-runtime` 拆分、`opendev-cli` 拆分、tools→agents 解耦的成本/收益评估
6. **Sandbox crate 处理**：移除 / feature-gate / 实现？产品是否需要 sandbox 功能？
7. **类型生成策略**：`ts-rs` 是 feature-gate 还是继续作为默认依赖？
8. **测试策略**：proptest 实际启用 / cargo-fuzz 引入 / 测试中 unwrap 减少？
9. **文档补全**：README + 架构图 + 贡献指南的优先级？
10. **多端形态长期方向**：CLI / TUI / Web / Desktop 四个前端未来如何演进？是否统一到 Transport 抽象层？

---

**报告生成完毕。** 所有数据均直接来自代码、架构文档和审计报告。可结合 `docs/AUDIT_REPORT.md`、`docs/REMEDIATION_PLAN.md`、`ARCHITECTURE.md`、`docs/architecture/*.md` 进一步讨论。
