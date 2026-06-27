# ADR-0005 — Desktop Native Communication Architecture

**状态：** Accepted  
**日期：** 2026-06-27  
**替代：** Web-First Architecture（隐式，无正式 ADR）  
**Sprint 代号：** Cabinet v0.2 — Interface Layer Independence  
**范围：** Desktop Interface Architecture + 前端 Transport Layer

---

## 一、核心决策

> Cabinet Desktop 不应将 Tauri 作为中心。Tauri 只是当前 Desktop Platform 的一个 Adapter。
>
> Application 永远不知道是谁在调用它——Desktop、CLI、HTTP、MCP、SDK、Automation 调用的是同一套 Services。

### 被拒绝的方案

| 方案 | 拒绝理由 |
|------|---------|
| 渐进式 HTTP → IPC 迁移 | 保留 HTTP/WS 共存，产生双通道技术债 |
| Tauri Commands 为中心 | 绑定特定 Desktop Framework，违背 Ports & Adapters |
| 保留 HTTP Server 双通道 | 维护两套 Interface，行为不一致 |

### 采纳方案

**Hexagonal Architecture，一次 Sprint 完成。**

---

## 二、最终分层架构

```
React Components
        │
Stores (Zustand)
        │
Repositories
        │
Transport (Platform-agnostic abstraction)
        │
        ├── invoke()
        ├── onEvent()
        └── openStream()
        │
═════════════════════════════════════ Process boundary
        │
Desktop Interface
        │
        ├── Commands (DTO Mapping only, 2-3 lines)
        ├── Events   (domain.object.action)
        └── Streams  (Data Stream / State Stream)
        │
Interface Contract (shared DTOs)
        │
Application Services
        ├── ConfigService         ├── SessionService
        ├── ChatService           ├── WorkflowService
        ├── MCPService            ├── SkillService
        ├── FileService           ├── MemoryService
        └── SystemService
        │
Core (Domain types, Traits)
        │
Infrastructure (File System, Git, LLM HTTP)
```

### 关键约束

| 层 | 禁止 |
|----|------|
| **Component** | 禁止 `invoke()`, `fetch()`, `emit()`, `listen()` — 必须通过 Store |
| **Store** | 禁止 `invoke()`, `fetch()`, `emit()` — 必须通过 Repository |
| **Repository** | 禁止 `fetch()`, `new WebSocket()` — 必须通过 Transport |
| **Command** | 禁止 `if`/`match` 业务判断、文件操作、状态管理 — 仅 DTO Mapping |
| **Interface** | 禁止包含 Agent 逻辑 — Agent 属于 Core |
| **Application** | 禁止依赖任何 Interface（`axum`, `tauri`, `clap`） |

---

## 三、前端 Transport Layer

### 3.1 Transport 接口（平台无关抽象）

```typescript
// repositories/Transport.ts

/**
 * Transport — 前端唯一知道平台的地方。
 *
 * 实现者：
 *   ├─ TauriTransport      (Desktop — invoke/event/channel)
 *   ├─ HttpTransport       (Web — fetch/sse)
 *   ├─ DirectTransport     (CLI embed — 直接调用)
 *   └─ MockTransport       (Testing)
 */
interface Transport {
  /** 同步 Request/Response */
  invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;

  /** 全局事件监听 → 返回 unsubscribe */
  onEvent<T>(event: string, handler: (payload: T) => void): Promise<() => void>;

  /** 长生命周期数据流 → 返回 StreamController */
  openStream<T>(): StreamController<T>;
}

interface StreamController<T> {
  onData(handler: (data: T) => void): void;
  close(): void;
}
```

### 3.2 实现者

| 实现 | 对应场景 |
|------|---------|
| `TauriTransport` | `invoke()` + `listen()` + `Channel` |
| `HttpTransport` | `fetch()` + SSE + WebSocket（未来 Web） |
| `DirectTransport` | 直接内存调用（CLI 内嵌） |
| `MockTransport` | 测试/Storybook |

### 3.3 Repository 使用 Transport

```typescript
// repositories/configRepository.ts
import type { Transport } from './Transport';

export function createConfigRepository(transport: Transport) {
  return {
    getConfig:      () => transport.invoke<AppConfig>('get_app_config'),
    updateConfig:   (dto: ConfigUpdate) => transport.invoke<void>('update_app_config', { config: dto }),
    listProviders:  () => transport.invoke<Provider[]>('list_model_providers'),
    verifyModel:    (p: string, m: string) => transport.invoke<VerifyResult>('verify_model', { provider: p, model: m }),
  };
}

// 生产环境单一实例
export const configRepository = createConfigRepository(tauriTransport);
```

**关键：** Repository 不知道 `Transport` 是什么。换平台 = 换 Transport 实现，Repository 不变。

---

## 四、Rust Desktop Platform 抽象

### 4.1 DesktopPlatform Trait

```rust
// interface/desktop/platform.rs

/// Desktop Platform 抽象。
/// 实现者不限于 Tauri——未来任何 Desktop Framework 都可以实现。
///
/// 命名选择：`DesktopPlatform` 而非 `DesktopRuntime`，
/// 因为 Tauri/Slint/Wry 是 Desktop Framework（Platform Adapter），
/// 真正的 Runtime 是 Rust Process。
#[async_trait]
pub trait DesktopPlatform: Send + Sync + 'static {
    fn manage<T: Send + Sync + 'static>(&self, state: T);
    fn emit_event(&self, event: &str, payload: impl Serialize);
    fn create_stream<T: Serialize>(&self) -> (StreamSender<T>, StreamReceiver<T>);
}

/// Tauri 实现
pub struct TauriPlatform {
    app: tauri::AppHandle,
}

impl DesktopPlatform for TauriPlatform {
    fn manage<T: Send + Sync + 'static>(&self, state: T) {
        self.app.manage(state);
    }
    fn emit_event(&self, event: &str, payload: impl Serialize) {
        let _ = self.app.emit(event, payload);
    }
    // ...
}
```

### 4.2 Platform Registry

```rust
// 未来扩展
pub struct SlintPlatform { /* ... */ }
pub struct WryPlatform { /* ... */ }
```

---

## 五、Interface Contract（共享 DTO 层）

### 5.1 设计原则

> Command 不是独立的 API 定义。Command 是 Contract 的薄映射。
>
> Desktop、HTTP、CLI 共享同一份 Request/Response DTO。

### 5.2 目录结构

```
interface/
├── desktop/
│   ├── contract/              # Desktop Interface 专用 Contract
│   │   ├── mod.rs
│   │   ├── config.rs          # UpdateConfigRequest, ConfigResponse
│   │   ├── session.rs         # CreateSessionRequest, SessionListResponse
│   │   ├── chat.rs            # ChatQueryRequest, ChatStreamEvent
│   │   ├── workflow.rs        # ApproveRequest, AskUserResponse
│   │   ├── mcp.rs             # MCPServerRequest, MCPServerResponse
│   │   ├── skills.rs          # SkillResponse, TogglePinRequest
│   │   ├── files.rs           # FileEntry, BrowseDirectoryResponse
│   │   └── memory.rs          # (reserved)
│   ├── commands/              # Command = Contract → Service
│   ├── events/
│   └── platform.rs
├── services.rs                # Application Service 注入
└── mod.rs
```

### 5.3 Contract 示例

```rust
// interface/desktop/contract/config.rs
use serde::{Deserialize, Serialize};

/// 发送给 `update_app_config` 的请求
#[derive(Debug, Deserialize)]
pub struct UpdateConfigRequest {
    pub model_provider: Option<String>,
    pub model: Option<String>,
    pub temperature: Option<f64>,
    pub api_key: Option<String>,
    pub api_base_url: Option<String>,
}

/// `get_app_config` 返回的响应
#[derive(Debug, Serialize)]
pub struct ConfigResponse {
    pub model_provider: String,
    pub model: String,
    pub api_key: Option<String>,      // masked: "sk-****...****ab12"
    pub api_base_url: Option<String>,
    pub temperature: f64,
}
```

**未来 HTTP Interface 可直接复用这些 Contract。**

---

## 六、Command 设计规范

### 6.1 Command = DTO Mapping Only

```rust
// interface/desktop/commands/config.rs
impl ConfigCommands {
    /// ✅ 正确：仅 DTO Mapping，无业务逻辑
    #[tauri::command]
    async fn get_app_config(state: State<'_, AppServices>) -> Result<ConfigResponse, String> {
        state.config.get_config().await.map_err(|e| e.to_string())
    }

    #[tauri::command]
    async fn update_app_config(
        state: State<'_, AppServices>,
        req: UpdateConfigRequest,
    ) -> Result<(), String> {
        state.config.update_config(req).await.map_err(|e| e.to_string())
    }
}
```

### 6.2 禁止模式

```rust
// ❌ 禁止：Command 中直接访问文件系统
fn create_session(req) {
    let dir = format!("/tmp/sessions/{}", req.id);
    std::fs::create_dir_all(&dir)?;       // 这是 Service 的职责
}

// ❌ 禁止：Command 中包含业务判断
fn set_mode(req) {
    if req.mode == "plan" {
        // 业务逻辑
    }
}

// ❌ 禁止：Command 操作 Agent
fn start_agent(req) {
    agent.start(...);                     // Agent 逻辑在 Core，不在 Interface
}
```

---

## 七、通信分类与事件命名规范

### 7.1 三类通信机制

| 类别 | 机制 | 适用 |
|------|------|------|
| **Command** | `Transport.invoke()` → `Result<T>` | 配置读写、CRUD、状态查询 |
| **Event** | `Transport.onEvent()` ← `Platform.emit_event()` | 全局状态变更广播 |
| **Stream** | `Transport.openStream()` | 持续数据推送 |

### 7.2 Stream 子类型

| 子类型 | 特征 | 示例 |
|--------|------|------|
| **Data Stream** | 累积型增量数据（delta → delta → done） | Chat 消息流、代码生成 |
| **State Stream** | 离散型状态变迁（Waiting → Running → Finished） | Workflow 执行、Tool 进度、Subagent 状态 |

**收益：** 前端 Store 对 Data Stream 做 append，对 State Stream 做 replace。

### 7.3 事件命名规范

**格式：** `domain.object.action`

```
# Chat
chat.message.chunk
chat.message.completed
chat.tool.executing
chat.tool.completed
chat.thinking.block
chat.approval.required

# Workflow
workflow.step.waiting
workflow.step.completed
workflow.approval.required
workflow.plan.ready

# MCP
mcp.server.connected
mcp.server.disconnected
mcp.servers.updated

# Memory
memory.updated
memory.index.complete

# Workspace
workspace.index.completed

# Config
config.updated

# Session
session.activity
```

**禁止：** 裸命名（`chat-update`、`done`、`finished`、`message`）。

---

## 八、Application Service 组织

### 8.1 命名约定

| 旧命名 | 新命名 |
|--------|--------|
| `ConfigApplicationService` | `ConfigService` |
| `ChatApplicationService` | `ChatService` |
| `WorkflowApplicationService` | `WorkflowService` |

**原因：** Architecture 已经分层。`application/` 目录下的类型不需要前缀重复。

### 8.2 服务清单

```
application/
├── config_service.rs       → ConfigService
├── session_service.rs      → SessionService
├── chat_service.rs         → ChatService
├── workflow_service.rs     → WorkflowService
├── mcp_service.rs          → MCPService
├── skill_service.rs        → SkillService
├── file_service.rs         → FileService
├── memory_service.rs       → MemoryService
└── system_service.rs       → SystemService
```

### 8.3 Interface Layer 禁止 Agent 逻辑

```
Interface Layer:
  ✅ Transport → DTO → Application Service

Interface Layer 永远不知道:
  ❌ Agent 是谁
  ❌ Planner 如何决策
  ❌ Scheduler 如何调度
  ❌ Workflow 如何编排

原因: Cabinet 未来扩展到 Work Agent 时，
      Agent 逻辑会显著复杂化。
      Interface 只负责通信，不负责智能。
```

---

## 九、Service 到 Command 映射

```
ConfigService     ← get_app_config, update_app_config, list_model_providers,
                     verify_model, set_operation_mode, set_autonomy_level

SessionService    ← list_sessions, get_current_session, create_session,
                     delete_session, resume_session, get_session_messages,
                     get_session_model, set_session_model, clear_session_model

ChatService       ← send_chat_query (→ Data Stream), interrupt_chat, clear_chat

WorkflowService   ← approve_tool, respond_to_ask, respond_to_plan

MCPService        ← list_mcp_servers, get_mcp_server, create_mcp_server,
                     update_mcp_server, delete_mcp_server, connect_mcp_server,
                     disconnect_mcp_server, test_mcp_server

SkillService      ← list_skills, toggle_skill_pin

FileService       ← list_workspace_files, verify_path, browse_directory

MemoryService     ← (reserved for future)

SystemService     ← get_git_branch, get_working_dir, health_check, get_bridge_info
```

---

## 十、最终 Release Checklist

### 前端清理验证

```bash
grep -r "fetch(" src/ --include="*.ts" --include="*.tsx"          → 0
grep -r "localhost" src/ --include="*.ts" --include="*.tsx"       → 0
grep -r "API_BASE\|__OPENDEV_PORT__\|getPort" src/                → 0
grep -r "WebSocket\|ws://\|wsClient" src/                         → 0
grep -r "apiClient\." src/                                        → 0
```

### Rust 清理验证

```bash
cargo check -p opendev-desktop                                    → pass
strings target/debug/opendev-desktop | grep "axum\|cors\|127.0.0.1" → 0
```

### 架构约束验证

```bash
# Command 中无业务逻辑
grep -r "std::fs\|tokio::fs" interface/desktop/commands/          → 0

# Component 无直接 invoke
grep -r "import.*invoke.*from.*tauri" src/components/             → 0

# Store 无直接 invoke
grep -r "import.*invoke.*from.*tauri" src/stores/                 → 0
```

### 性能基线

| 指标 | Sprint 前 | Sprint 后 |
|------|----------|----------|
| 配置读取 | 5-50ms (TCP + Axum) | <1ms (IPC) |
| 会话列表 | 5-50ms | <1ms |
| Chat 首字 | 10-100ms | <5ms |
| 启动端口绑定 | 有 `127.0.0.1:XXXX` | **无** |
| `lsof -i` 输出 | 1 条 localhost | **0 条** |

### 删除行长统计

| 删除项 | 行数 |
|--------|------|
| `src/api/client.ts` | 229 |
| `src/api/mcp.ts` | 116 |
| `src/api/websocket.ts` | 192 |
| `src-tauri/src/server.rs` (Axum 启动) | 118 |
| `crates/opendev-web/src/server.rs` | 98 |
| `crates/opendev-web/src/websocket.rs` | 388 |
| `crates/opendev-web/src/protocol.rs` | 233 |
| CORS / API_BASE / Port Injection | ~50 |
| **合计** | **~1,424 行删除** |

### 新增行长统计

| 新增项 | 行数 |
|--------|------|
| `interface/desktop/contract/*.rs` (8 文件) | ~600 |
| `interface/desktop/commands/*.rs` (8 文件) | ~400 |
| `interface/desktop/platform.rs` | ~100 |
| `interface/desktop/events/` | ~300 |
| `interface/services.rs` | ~80 |
| `src/repositories/*.ts` (8 文件) | ~500 |
| `src/repositories/Transport.ts` | ~60 |
| `src/repositories/TauriTransport.ts` | ~80 |
| `docs/architecture/desktop-communication.md` | ~200 |
| **合计** | **~2,320 行新增** |

---

## 十一、Sprint Task 清单

| Task | 名称 | 工作量 | 风险 |
|------|------|--------|------|
| **T0** | Architecture Standard (`desktop-communication.md`) | 小 | 低 |
| T1 | Extract Application Services | 大 | 中 |
| T2 | Desktop Interface Layer (`platform.rs` + `contract/`) | 大 | 中 |
| T3 | Desktop Commands (DTO Mapping only) | 大 | 中 |
| T4 | Event System (命名规范 + 三类通信) | 大 | 高 |
| T5 | Frontend `Transport` + `TauriTransport` | 中 | 低 |
| T6 | Repository Layer (依赖 Transport) | 中 | 低 |
| T7 | Migrate all `fetch()` → Repository | 中 | 中 |
| T8 | Chat Stream → Channel (Data Stream) | 大 | 高 |
| T9 | Delete WebSocket | 小 | 低 |
| T10 | Delete HTTP Server | 小 | 低 |
| T11 | Delete Port Injection + `__OPENDEV_PORT__` | 小 | 低 |
| T12 | Final Cleanup (`interface-http` extraction) | 中 | 低 |

### 依赖图

```
T0 ──→ T1 ──→ T2 ──→ T3 ──→ T4 ──→ T8 ──→ T9
                    │       │
                    └──→ T5 ──→ T6 ──→ T7 ──→ T10 ──→ T11 ──→ T12
```

---

## 附录 A：当前架构审计摘要

Sprint 前现状（来自 2026-06-27 代码审计）：

- **38 个 HTTP Route**（`crates/opendev-web/src/routes/`）
- **1 个 WebSocket**（`/ws`）
- **0 个 `#[tauri::command]`**
- **3 种不一致的 `API_BASE` 构建方式**（`client.ts` 绝对路径 / `mcp.ts` 相对路径 / `SkillsSettings.tsx` 重复绝对路径）
- **26 个 `fetch()` 调用** 在 `client.ts`
- **31 个 `apiClient.*` 调用** 在 11 个组件/store 中
- **6 个 `apiClient` 方法无前端调用**（可能死代码）
- **MCP WebSocket 事件名不匹配**（前端 `mcp_status_update` / 后端 `mcp:status_changed`）
- **`/api/config/thinking` 端点不存在于后端** 但前端有调用

## 附录 B：已发现的 Bug

| # | 问题 | 严重度 |
|---|------|--------|
| 1 | `client.ts` 调用 `POST /api/config/thinking` — Rust 端无此路由 | 高 |
| 2 | MCPSettings 订阅 `mcp_status_update` — 实际事件名为 `mcp:status_changed` | 高 |
| 3 | SessionsSidebar 使用相对路径 `/api/sessions/:id` DELETE — 与 client.ts 不一致 | 中 |
| 4 | 6 个 apiClient 方法无调用（`sendQuery`, `getMessages`, `getCurrentSession`, `exportSession`, `verifyPath`, `health`） | 低 |
