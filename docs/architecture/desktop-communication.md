# Desktop Communication Architecture

**生效日期：** 2026-06-27  
**状态：** Accepted  
**ADR：** ADR-0005  
**范围：** Desktop Interface Layer Independence

---

## 一、通信模型

整个 Desktop 只允许三种通信机制：

| 类别 | 机制 | 适用场景 |
|------|------|---------|
| **Command** | `Transport.invoke()` → `Result<T>` | CRUD、配置、状态查询 |
| **Event** | `Platform.emit_event()` → `Transport.onEvent()` | 全局状态变更广播 |
| **Stream** | `Transport.openStream()` → 持续推送 | Chat 消息流、Workflow 执行、Tool 进度 |

### 1.1 Command

```
Frontend                    Desktop Interface                Application
   │                              │                              │
   ├── invoke("get_config") ──────►                              │
   │                              ├── ConfigService.get() ──────►│
   │                              │                              │
   │◄────── ConfigResponse ───────┤                              │
```

Command 是同步 Request/Response 模式。适用于：
- 配置读写
- 会话 CRUD
- 状态查询
- MCP 管理

### 1.2 Event

```
Application                    Desktop Interface                Frontend
   │                              │                              │
   │─── config.updated ──────────►│                              │
   │                              ├── onEvent("config.updated") ─►│
   │                              │                              │
```

Event 是一对多的广播模式。命名规范：`domain.object.action`。

**禁止：** `done`, `finished`, `message`, `update` 等裸命名。

### 1.3 Stream

```
Application                    Desktop Interface                Frontend
   │                              │                              │
   │── openStream(chat_query) ───►│                              │
   │   data: MessageChunk         ├── onData(chunk) ────────────►│
   │   data: MessageChunk         ├── onData(chunk) ────────────►│
   │   data: MessageComplete      ├── onData(complete) ─────────►│
   │                              │                              │
```

| 子类型 | 特征 | 示例 |
|--------|------|------|
| **Data Stream** | 累积型增量数据 | Chat 消息流、代码生成 |
| **State Stream** | 离散型状态变迁 | Workflow 执行、Tool 进度、Subagent 状态 |

---

## 二、层次职责

### 2.1 React Components

**允许：**
- UI 渲染
- 用户交互事件
- 调用 Store（Zustand）

**禁止：**
- `invoke()`
- `fetch()`
- `emit()` / `listen()`
- `WebSocket`
- 任何 Repository 调用

### 2.2 Store (Zustand)

**允许：**
- 维护状态
- 调用 Repository

**禁止：**
- `invoke()`
- `fetch()`
- `emit()`

### 2.3 Repository

Repository 是前端唯一的数据访问层。

**允许：**
- 调用 Transport

**禁止：**
- `fetch()`
- `new WebSocket()`
- `WebSocket`
- `invoke()`
- `emit()`
- 平台判断

### 2.4 Transport

Transport 是前端唯一知道平台的地方。

| 实现 | 对应场景 |
|------|---------|
| `TauriTransport` | Desktop — `invoke()`, `listen()`, `Channel` |
| `HttpTransport` | Web — `fetch()`, SSE, WebSocket |
| `DirectTransport` | CLI 内嵌 — 直接内存调用 |
| `MockTransport` | 测试 / Storybook |

```typescript
interface Transport {
  invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
  onEvent<T>(event: string, handler: (payload: T) => void): Promise<() => void>;
  openStream<T>(): StreamController<T>;
}
```

### 2.5 Desktop Interface (Rust)

Desktop Interface 是 Application 的 Port Adapter。

**允许：**
- DTO Mapping
- 调用 Application Service
- 事件转发

**禁止：**
- 业务逻辑
- 文件操作
- Agent 操作
- `if` / `match` 业务判断

### 2.6 Application Services

Application 是业务入口。

**允许：**
- 协调 Domain
- 调用 Infrastructure

**禁止依赖：**
- `tauri`
- `axum`
- `clap`
- 任何 Interface Framework

### 2.7 Core

**允许：**
- 业务规则
- Trait
- Domain Entity
- Policy

**禁止依赖：**
- Desktop
- HTTP
- CLI

---

## 三、事件命名规范

**格式：** `domain.object.action`

### Chat 域
```
chat.message.chunk
chat.message.completed
chat.tool.executing
chat.tool.completed
chat.thinking.block
chat.approval.required
chat.plan.approval.required
```

### Workflow 域
```
workflow.step.waiting
workflow.step.completed
workflow.plan.ready
```

### MCP 域
```
mcp.server.connected
mcp.server.disconnected
mcp.servers.updated
```

### Config 域
```
config.updated
```

### Session 域
```
session.activity
session.running
session.idle
```

### 禁止模式
```
❌ chat-update
❌ done
❌ finished
❌ message
❌ update
```

---

## 四、目录规范

### Rust 后端

```
interface/
├── desktop/
│   ├── contract/          # Desktop Interface 专用 DTO
│   │   ├── mod.rs
│   │   ├── config.rs
│   │   ├── session.rs
│   │   ├── chat.rs
│   │   ├── workflow.rs
│   │   ├── mcp.rs
│   │   ├── skills.rs
│   │   ├── files.rs
│   │   └── memory.rs
│   ├── commands/          # Command = DTO → Service
│   ├── events/
│   └── platform.rs
├── services.rs            # Application Service 注入
└── mod.rs

application/
├── config_service.rs
├── session_service.rs
├── chat_service.rs
├── workflow_service.rs
├── mcp_service.rs
├── skill_service.rs
├── file_service.rs
├── memory_service.rs
└── system_service.rs
```

### TypeScript 前端

```
src/
├── repositories/
│   ├── Transport.ts           # Transport 接口定义
│   ├── TauriTransport.ts      # Tauri IPC 实现
│   ├── configRepository.ts
│   ├── sessionRepository.ts
│   ├── chatRepository.ts
│   ├── workflowRepository.ts
│   ├── mcpRepository.ts
│   ├── skillRepository.ts
│   └── fileRepository.ts
├── stores/
│   ├── chat.ts
│   ├── status.ts
│   ├── todo.ts
│   ├── subagents.ts
│   └── fileChanges.ts
└── types/
    ├── index.ts
    └── generated/
```

---

## 五、Service 到 Command 映射

| Application Service | Desktop Commands |
|-------------------|-----------------|
| ConfigService | `get_app_config`, `update_app_config`, `list_model_providers`, `verify_model`, `set_operation_mode`, `set_autonomy_level` |
| SessionService | `list_sessions`, `get_current_session`, `create_session`, `delete_session`, `resume_session`, `get_session_messages`, `get_session_model`, `set_session_model`, `clear_session_model` |
| ChatService | `send_chat_query` (→ Data Stream), `interrupt_chat`, `clear_chat` |
| WorkflowService | `approve_tool`, `respond_to_ask`, `respond_to_plan` |
| MCPService | `list_mcp_servers`, `get_mcp_server`, `create_mcp_server`, `update_mcp_server`, `delete_mcp_server`, `connect_mcp_server`, `disconnect_mcp_server`, `test_mcp_server` |
| SkillService | `list_skills`, `toggle_skill_pin` |
| FileService | `list_workspace_files`, `verify_path`, `browse_directory` |
| SystemService | `get_git_branch`, `get_working_dir`, `health_check`, `get_bridge_info` |

---

## 六、约束规则

### 代码质量

所有新增 Rust 代码必须：
- `cargo fmt`
- `cargo clippy` 零新增 warning

所有新增 TypeScript 代码必须：
- 严格类型
- 避免 `any`

DTO 必须独立定义。禁止 `HashMap<String, Value>` 作为接口。

### 架构约束验证

```bash
# Command 中无业务逻辑
grep -r "std::fs\|tokio::fs" interface/desktop/commands/          → 0

# Component 无直接 invoke
grep -r "import.*invoke.*from.*tauri" src/components/             → 0

# Store 无直接 invoke
grep -r "import.*invoke.*from.*tauri" src/stores/                 → 0
```

### 删除验证

```bash
# 前端无 fetch, localhost, WebSocket, apiClient
grep -r "fetch(" src/ --include="*.ts" --include="*.tsx"          → 0
grep -r "localhost" src/ --include="*.ts" --include="*.tsx"       → 0
grep -r "API_BASE\|__OPENDEV_PORT__\|getPort" src/                → 0
grep -r "WebSocket\|ws://\|wsClient" src/                         → 0
grep -r "apiClient\." src/                                        → 0

# Rust Desktop 不启动 HTTP
cargo check -p opendev-desktop                                    → pass
strings target/debug/opendev-desktop | grep "axum\|cors\|127.0.0.1" → 0
```

---

## 七、与 AppState 的关系

在过渡完成后：
- `opendev-web::state::AppState` 被 `Application Services` 替代
- `WsBroadcast` 广播机制被 `DesktopPlatform::emit_event` 替代
- `WsMessageType` 枚举被统一事件命名规范替代
- `AppState` 中的 pending\_approvals / pending\_ask\_users 等管理职责进入 `WorkflowService`
- `AppState` 中的 agent\_executor / injection\_queues / running\_sessions 等执行管理职责进入 `ChatService`
- `AppState` 中的 ws\_tx / broadcast 功能被 `DesktopPlatform` 的事件系统替代

---

## 八、Stream 通信详解

### 8.1 Data Stream (Chat)

```
1. 前端调用 transport.invoke("send_chat_query", { message, session_id })
2. 后端 ChatService 开始处理，通过 stream 发送增量数据
3. 前端 StreamController.onData() 逐块接收
4. 完成后发送 final 信号

数据流:
  chat.message.start     → 消息开始
  chat.message.chunk     → 内容增量块 (多次)
  chat.tool.executing    → 工具调用开始
  chat.tool.completed    → 工具调用完成
  chat.message.chunk     → 更多内容
  chat.message.completed → 消息完成
```

### 8.2 State Stream (Workflow/Tool)

```
数据流:
  workflow.step.waiting    → 等待审批
  workflow.step.completed  → 步骤完成
  workflow.plan.ready      → 方案就绪
  workflow.approval.required → 需要审批
```
