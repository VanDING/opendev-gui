# OpenDev Desktop — 基础设施底座设计方案 (Protocol / Sandbox / Keyring / Telemetry)

> 编制日期: 2026-06-28
> 状态: **方案设计 (Design) — 待评审**
> 对应决策类别: **第一类 — 必须做 (工程基础)**
> 输入材料:
> - 4 份专项 recon (协议 / 沙箱 / 凭据 / 遥测)
> - `docs/architecture/comparison-codex-vs-opendev.md`
> - `docs/AUDIT_REPORT.md` + `docs/REMEDIATION_PLAN.md`
> - `docs/constitution.md` (14 条冻结原则)
> - 现有 ADR 001-007 + adr-0005

---

## 0. TL;DR

**这 4 件事不是功能,是底座。**

| 类别 | 推理 | 时机 |
|---|---|---|
| **Protocol (app-server 协议)** | Desktop / CLI / Web / Telegram / Workspace 都在长出独立客户端,若不先定协议,后续每一端都是一遍手撕 | 越早越便宜 |
| **Sandbox (沙箱)** | 安全是地基,不是装饰。零信任 (Zero Trust) 必须从第一天设计,后期补的成本是指数级 | 越早越便宜 |
| **Keyring (凭据存储)** | Constitution 第 7 条已声明 "API keys are stored in system credential store when available",但代码从未实现。constitution 落后于现实的债要先还 | 越早越便宜 |
| **Telemetry (遥测)** | 用户出现后才有 "产品健康度" 这个问题。但观测能力必须在用户出现**之前**就位,否则第一次崩就是盲飞 | 越早越便宜 |

**本方案的 4 条不可妥协原则:**

1. **架构,不是产品** — 不为新功能而建,为新功能能**建得更快**而建。
2. **底座要平** — 4 个底座互相不阻塞,可大致并行推进,但有依赖序(见 §2)。
3. **现有代码先服务,后清理** — 不发起革命。底层换装,业务层零变更。
4. **观测先行** — 每一件底层改造都必须**可被观测**,否则就是盲改。

**建议的工程顺序 (sequential,非 blocking):**

```
Phase 0  ┌─ Protocol  ──┐   Phase A  ┌─ Sandbox   ──┐   Phase B  ┌─ Keyring   ─┐   Phase C  ┌─ Telemetry ──┐
         │ 定义 wire     │            │ BashTool +    │            │ SecretStore │            │ OTel + JSON  │
         │ protocol v1   │            │ 其它 exec     │            │ + migration │            │ + Sentry opt │
         └───────────────┘            └───────────────┘            └─────────────┘            └──────────────┘
         现在 → +3 月                  +3 → +5 月                  +4 → +6 月                  +5 → +7 月
```

**本文档不做的事:**

- ❌ 不写实现代码
- ❌ 不发起任何 ADR (待评审后,按你的指示再写 ADR-008 ~ ADR-011)
- ❌ 不评估 "Realtime voice / Image gen / Cloud / SDK / Connectors" 等第二类功能 (你的判断:暂缓)
- ❌ 不拆 `opendev-runtime` "kitchen sink" (这是 P2/P3,与本方案正交)

---

## 1. 哲学 — 为什么是底座,不是功能

### 1.1 你的原话回顾

> **protocol** — "你未来: Desktop / CLI / Web / Telegram / Workspace 都会越来越依赖统一协议。这是:架构。不是:产品。"
>
> **sandbox** — "安全:不是功能。而是:底座。"
>
> **keyring** — "如果以后有用户。"
>
> **telemetry** — "如果以后有用户。"

### 1.2 底座 vs 功能的判别

| 判别维度 | 功能 (Feature) | 底座 (Foundation) |
|---|---|---|
| 用户能看到吗? | ✅ 看到 | ❌ 看不到(或刻意不可见) |
| 缺了会怎样? | "少个能力" | "多件事做不了" |
| 后期补可行吗? | ✅ 可分批补 | ❌ 越晚越贵,常常要重写 |
| 谁来消费? | 终端用户 | 其它代码 |
| 示例 | "Realtime voice 输入" | "统一的客户端-服务端协议" |
| 商业成熟度信号? | ✅ 强 | ❌ 弱 |

**对照 4 个待办:**

| 类别 | 用户能看到吗? | 缺了会怎样? | 后期补可行? | 判别 |
|---|---|---|---|---|
| Protocol | ❌ (API 形态而已) | 多端集成每次手撕,无法演化 | ❌ 重写几乎全部 client | **底座** |
| Sandbox | ❌ (后台执行) | LLM 跑出 `rm -rf /` 时已晚 | ❌ 已写下的 tool 一律要返工 | **底座** |
| Keyring | ❌ (存储层) | 凭据泄漏 → 用户的真金白银 | ❌ 用户数据已泄漏 | **底座** |
| Telemetry | ❌ (后台收集) | 第一次 OOM 时盲飞 | ❌ 数据回溯需要重做 | **底座** |

→ **全部是底座**。✓

### 1.3 与现有 constitution 的关系

Constitution 7 (Security by Default) 已有:

> - API keys are stored in system credential store when available (never logged).
> - Secrets are redacted from logs.

**当前是空头支票。** Keyring + Telemetry 方案直接落地这两条。完成后,constitution 从 "愿景" 变成 "现状"。

Constitution 1 (Layered Architecture) + 4 (Async-first) + 11 (Testing at all levels) + 12 (Explicit over magic):

→ Protocol 方案直接对应 "Layered" 和 "Explicit";Sandbox 方案对应 "Async-first"(spawn 在 async runtime 受控)和 "Testing at all levels"(沙箱策略必须可单测)。

Constitution 8 (Surface Ladder Design) **不适用** — 那是前端视觉系统,本方案不动前端像素。

### 1.4 不是什么

- ❌ 不是 "Codex 我们没有的功能清单" 的反向模仿
- ❌ 不是 "为了好看加的脚手架"
- ❌ 不是 "重构借口" (本方案不重命名 crate,不调换依赖方向)
- ❌ 不是 "性能优化" (性能可能更慢,换的是**可观察性 + 可演化性**)

---

## 2. 总体路径 — 依赖序与并行

### 2.1 依赖图

```
            ┌─────────────────────────┐
            │   opendev-models         │   (已有, 域类型)
            └─────────────────────────┘
                       ▲
                       │ 复用现有 FrontendEvent,新增 ProtocolEvent
   ┌───────────────────┼───────────────────┐
   │                   │                   │
┌──┴─────────┐  ┌──────┴──────┐  ┌─────────┴────────┐
│ opendev-   │  │ opendev-    │  │  opendev-        │
│  secrets   │  │  sandbox    │  │  telemetry       │
│ (NEW)      │  │ (REPLACE)   │  │  (REPLACE)       │
└──┬─────────┘  └──────┬──────┘  └─────────┬────────┘
   │                   │                   │
   │  被所有 tool      │  被所有 tool     │  被所有 crate
   │  通过 SecretStore │  通过 spawn hook │  通过 tracing
   │  查询 API key     │  调 sandbox     │  自动收集
   ▼                   ▼                   ▼
   ┌─────────────────────────────────────────┐
   │     opendev-protocol (NEW)              │   独立 crate,wire types
   │     ─────────────────────────────       │
   │     1. 统一定义 5 端共享的 wire format  │
   │     2. 通过 ts-rs 自动出 TS 绑定        │
   │     3. v1 frozen + v2 active 双轨       │
   └─────────────────────────────────────────┘
                       ▲
                       │ 5 个客户端各自实现 Transport
   ┌──────────┬────────┼────────┬───────────┐
   │ Tauri    │ TUI    │ Web    │ Telegram  │ Workspace
   │ (现 desktop)│(现 ratatui)│(现 opendev-web)│(现 channels)│(新)
   └──────────┴────────┴────────┴───────────┴──────────
```

**关键依赖关系:**

- **Protocol 是根** — 它定义 wire format,所有 client 实现 Transport 接口
- **Secrets / Sandbox / Telemetry 是 leaf** — 各自独立,可并行
- **5 个客户端都依赖 Protocol** — 但每个客户端可独立迁移
- **Secrets + Sandbox + Telemetry 都通过 Protocol 暴露配置** — 协议是它们的"窗口"

### 2.2 4 个工作的先后

| 顺序 | 类别 | 理由 |
|---|---|---|
| **1st** | Protocol | 是其它三件事的"窗口"。先有协议,后面三件事能优雅地暴露给用户 |
| **2nd** | Sandbox | 改动 tool exec 路径,影响面大。Protocol 到位后,沙箱配置走 protocol 暴露给用户 |
| **3rd** | Keyring | 改动凭据解析路径,需要 Protocol 帮它暴露 settings 视图 |
| **4th** | Telemetry | 改动可观测性,本身需要协议携带"诊断"事件类型;可后置 |

**为什么不是 Sandbox 第一?** 因为 Sandbox 改动会穿透 BashTool + 其它 17 个 exec 点;Protocol 不改业务逻辑,只改 wire format,影响面更可控。

**为什么 Keyring 第三?** 因为 Keyring 需要跟 settings UI 协作,UI 走 Protocol。先有 Protocol,Keyring 才能"配出来就能用"。

**为什么 Telemetry 最后?** 因为它的"诊断"事件要复用 Protocol 的事件 envelope。先有 Protocol,Telemetry 才能复用同一通道。

**备选:Telemetry 提到 Keyring 之前 (Oracle review 提示)**

论据:Telemetry 提供 audit / tracing,Sandbox 和 Keyring 都依赖它做"安全可观测" (e.g. "谁何时读了哪个 secret"、"哪个 tool exec 被 policy deny 了")。先建 Telemetry,后两件都自带审计。

**反驳 (保留原序):**
- Telemetry 的 redact layer 依赖 `opendev-runtime::secrets::redact_secrets` 函数,该函数目前是 2 调用点的"工具输出脱敏",不是 tracing 路径脱敏。改造为 telemetry layer 是 Telemetry 自身的工作,跟 Keyring 无关。
- 审计需求 (谁读了 key) 可以等 Keyring 上线后,通过 `SecretStore::audit()` 加,不阻塞 Keyring 设计。
- Telemetry v1 的核心价值是 "Tauri 也有日志了",这跟 Sandbox/Keyring 没有依赖关系。
- 真正依赖序:Protocol 是根,其它 3 件互不依赖(都从 Protocol 取 config 暴露入口)。

**结论:** 保留 Protocol → Sandbox → Keyring → Telemetry 顺序。**但** 实施时,Telemetry 的 redact layer (Phase 4B) 可以从 Keyring 设计 (Phase 3A) 提前借 `SecretValue::Display` 实现,**两件并行**。

### 2.3 工程量估算

| 类别 | 新增 LoC (估) | 修改 LoC (估) | 新增 crate | 人月 |
|---|---|---|---|---|
| Protocol | ~3000 (含测试) | ~500 (TS 部分) | `opendev-protocol` (NEW) | 0.8-1.0 |
| Sandbox | ~2500 (含测试 + 3 后端) | ~400 (BashTool 等) | 重写 `opendev-sandbox` | 1.0-1.2 |
| Keyring | ~1200 (含测试 + migration) | ~300 (AppConfig 等) | `opendev-secrets` (NEW) | 0.5-0.7 |
| Telemetry | ~1500 (含测试 + OTLP 集成) | ~400 (Tauri 入口等) | `opendev-telemetry` (重命名) | 0.5-0.8 |
| **合计** | **~8200** | **~1600** | **2 新 + 1 重命名 + 1 重写** | **2.8-3.7 人月** |

**前置准备:** 1 周设计冻结 + 1 周脚手架 = 2 周。合计 **3-4 个月** 跑完第一类全部底座。

**对比第二类 (暂缓) 的体量:** Realtime voice + Image gen + Cloud tasks + Remote control + TS SDK + Python SDK + Connectors + Mobile,粗估 12+ 人月。本方案性价比高得多。

---

## 3. 协议 (Protocol) — 架构的底座

### 3.1 现状摘要 (来自 recon)

**3 个并行的协议面,做同一件事:**

1. **`FrontendEvent`** (opendev-models) — 17 变体 tagged union,`ts-rs` 出 TS,**但 live wire 永远不用它**。它只活在文档中。
2. **`DesktopEvent`** (src-tauri) — 6 子枚举 17 变体,`domain.object.action` 命名,**但从不实例化**。命名约定活在文档中。
3. **`WsMessageType`** (opendev-web) — 40 变体,`serde(rename)` 到字符串,**但 Tauri 不走它**。WS path 走它。
4. **`WSMessage` (TS)** (src/types/index.ts) — 32 字符串字面量 union,**实际跑的协议**。
5. **`AppEvent`** (opendev-tui) — 60+ 变体,进程内,**不跨网**。

**最致命的问题 — 命名混用 4 套:**

```
mcp_servers_updated  ← opendev-web 真实 broadcast 的名字
mcp:servers_updated  ← opendev-web WsMessageType 序列化出来的名字 (但实际不 broadcast)
mcp.server.connected ← DesktopEvent 文档 (但实际不实例化)
mcp_status_update    ← TS WSMessage 旧 alias (死代码)
```

**其它致命问题:**

- ❌ `server.rs` 临时桥接 (`src-tauri/src/server.rs:1-7` 注释明说"will be fully removed")
- ❌ 8/34 Tauri 命令返回 `serde_json::Value` (untyped escape hatch)
- ❌ Tauri `send_chat_query` 是 no-op (chat 走 legacy AppState bridge,不走新命令)
- ❌ 无 `protocol_version` 字段,无版本协商
- ❌ `WsBroadcast.seq` 在 Tauri 路径上**被丢弃**,前端无法检测丢包
- ❌ 两个 `OperationMode` 枚举 (opendev-web vs src-tauri) 并存
- ❌ 三个 `AgentExecutor` trait (opendev-channels / opendev-web / opendev-cli) 并存
- ❌ `ts-rs` 输出去重但**两份**,且**前端不引用**

**好消息:**

- ✅ `Transport` 抽象已就位 (`src/repositories/Transport.ts`)
- ✅ `FrontendEvent` 17 变体可用作"想要的协议形状"参考
- ✅ Constitution 5 (Tool trait with defaults) 已就位
- ✅ ADR-0005 desktop-communication 已经定下了"六边形架构 + 协议驱动"的总方向

### 3.2 设计目标

**主目标:** **5 端共享 1 个 wire protocol,带版本,可演化。**

**子目标:**

1. **可发现** — 任何 client 拿到协议后,能"自描述"自己会发什么事件、会调什么方法
2. **可验证** — 编译期阻止"字段拼错" / "事件名错" 类 bug
3. **可演化** — 加新方法 / 字段不破坏现有 client
4. **可观测** — 所有 wire 流量可被统一 telemetry 层捕获
5. **可移植** — 任意语言能实现 Transport (Rust / TS / Python / Go)
6. **可测试** — 协议测试不需要起 server

### 3.3 设计方案

#### 3.3.1 新 crate: `opendev-protocol`

**职责:** 唯一定义 wire format,ts-rs 出 TS 绑定,无业务逻辑。

**目录结构:**

```
crates/opendev-protocol/
├── Cargo.toml
├── src/
│   ├── lib.rs                       # 重新导出
│   ├── version.rs                   # PROTOCOL_VERSION = "1.0"
│   ├── envelope.rs                  # WireEnvelope { v, id, src, dst, kind, payload }
│   ├── methods.rs                   # 所有 RPC 方法名(常量)
│   ├── events.rs                    # 所有事件名(常量)
│   ├── v1/                          # V1 协议(frozen)
│   │   ├── mod.rs
│   │   ├── session.rs               # session/start, session/resume, session/list, ...
│   │   ├── turn.rs                  # turn/start, turn/interrupt, turn/steer
│   │   ├── message.rs               # 消息 chunking
│   │   ├── tool.rs                  # tool/call, tool/result
│   │   ├── approval.rs              # approval/required, approval/respond
│   │   ├── fs.rs                    # fs/read, fs/write, fs/list
│   │   ├── mcp.rs                   # mcp/server/list, mcp/server/connect, mcp/tool/call
│   │   ├── skill.rs
│   │   ├── config.rs
│   │   ├── workspace.rs
│   │   └── error.rs
│   ├── v2/                          # V2 协议(active dev)
│   │   ├── mod.rs
│   │   └── (增量)
│   └── experimental.rs              # #[experimental("...")] 标记
└── tests/
    ├── wire_compat.rs
    ├── versioning.rs
    └── (snapshot tests)
```

**关键设计:**

```rust
// src/version.rs
pub const PROTOCOL_VERSION_MAJOR: u16 = 1;
pub const PROTOCOL_VERSION_MINOR: u16 = 0;
pub const PROTOCOL_VERSION_PATCH: u16 = 0;
pub const PROTOCOL_VERSION: &str = "1.0.0";

// src/envelope.rs
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WireEnvelope<P: Payload> {
    Request(RequestFrame<P>),       // client → server
    Response(ResponseFrame<P>),     // server → client
    Notification(NotificationFrame<P>),  // server → client (no ack)
    Error(ErrorFrame),              // 错误帧
}

pub struct RequestFrame<P> {
    pub v: ProtocolVersion,
    pub id: RequestId,           // UUID v7 (时间序)
    pub method: Method,          // e.g. "session/start"
    pub params: P,
}

pub struct NotificationFrame<P> {
    pub v: ProtocolVersion,
    pub seq: u64,                // 单调递增,**这就是 WsBroadcast.seq 该有的归宿**
    pub event: Event,            // e.g. "message/chunk"
    pub data: P,
}
```

**命名约定** (写在 `docs/architecture/protocol-naming.md`):

- **方法:** `<domain>/<verb>` (snake_case) — `session/start`, `turn/interrupt`, `tool/approve`
- **事件:** `<noun>/<past-tense>` — `message/chunked`, `tool/started`, `approval/required`
- **字段:** snake_case (wire) → camelCase (TS via ts-rs)
- **ID:** 字符串 UUID v7 (时间序,前端可排序)

**版本约定:**

- **V1 (frozen):** v0.2 之前冻结,只修 bug,不加新方法/字段。
- **V2 (active):** v0.3 开始开发,新方法/字段;V1 client 仍能连。
- **V1 → V2 升级路径:** client 启动时协商 `protocol_version`,server 回应 `min_supported` + `max_supported`。Client 选择兼容版本。

#### 3.3.2 Transport 接口(5 端各自实现)

**统一形态:**

```rust
// crates/opendev-protocol/src/transport.rs (新)
#[async_trait]
pub trait Transport: Send + Sync {
    /// 发送 RPC 请求,等待响应
    async fn call<P: Payload, R: Payload>(&self, method: Method, params: P) -> Result<R, ProtocolError>;
    /// 订阅事件
    async fn subscribe(&self, event: Event) -> Result<EventStream, ProtocolError>;
    /// 取消订阅
    async fn unsubscribe(&self, handle: EventHandle) -> Result<(), ProtocolError>;
    /// 探测 server 协议版本
    async fn negotiate(&self) -> Result<NegotiatedVersion, ProtocolError>;
}
```

**5 端实现:**

| 客户端 | Transport 实现 | 后端协议 |
|---|---|---|
| Tauri 桌面 | `TauriTransport` (Tauri `invoke` + `Channel<T>`) | Tauri IPC + JSON |
| TUI (ratatui) | `TuiInProcessTransport` | tokio mpsc 进程内 |
| Web (`opendev-web` 模式) | `WebSocketTransport` (axum WS) | RFC 6455 + JSONL |
| Telegram | `TelegramTransport` | HTTP long-poll + JSON |
| Workspace (新) | `UnixSocketTransport` (mac/Linux) / `NamedPipeTransport` (Win) | UDS / NP + JSONL |

**Tauri 2 的 `Channel<T>`** 终于能给 `Transport::subscribe` 用上 (recon §13.8 提到这是空头支票)。

#### 3.3.3 v1 方法表 (首批,28 个)

参考 `FrontendEvent` 17 变体 + Codex 80+ 方法 + 当前 34 个 Tauri 命令,提取去重后,v1 应提供:

**Session (5):**
- `session/list` — list past sessions
- `session/start` — create new
- `session/resume` — resume existing
- `session/delete` — delete
- `session/turns` — list turns in a session

**Turn (3):**
- `turn/start` — send user input, kicks off agent loop
- `turn/interrupt` — cancel running turn
- `turn/steer` — inject mid-turn (Codex 独有)

**Tool (3):**
- `tool/list` — get available tool schemas (with deferral support)
- `tool/search` — activate deferred tools
- `tool/approve` — respond to approval request

**MCP (7):**
- `mcp/server/list` — list configured servers
- `mcp/server/get` — get one server's config
- `mcp/server/create` — add
- `mcp/server/update` — update
- `mcp/server/delete` — remove
- `mcp/server/connect` — start
- `mcp/server/disconnect` — stop

**Skill (2):**
- `skill/list`
- `skill/pin` — toggle pin

**Config (5):**
- `config/get`
- `config/update`
- `config/mode/set` — operation mode (normal/plan)
- `config/autonomy/set`
- `config/model/verify`

**File (3):**
- `fs/browse`
- `fs/verify-path`
- `fs/list-workspace`

**Workspace (2):**
- `workspace/list` — list workspaces (groups of sessions)
- `workspace/get` — get one

**总计 30 个 v1 方法**(比当前 34 Tauri 命令少 4 个,合并了等价项)。

**v1 事件表 (首批, 18 个):**

| 事件 | 触发时机 | payload |
|---|---|---|
| `message/start` | 助手消息开始 | `{ session_id, message_id }` |
| `message/chunked` | 文本流式 chunk | `{ session_id, message_id, content, seq }` |
| `message/completed` | 消息完成 | `{ session_id, message_id, role }` |
| `thinking/block` | 推理块 | `{ session_id, content, block_start }` |
| `tool/started` | tool 开始执行 | `{ session_id, tool_call_id, name, arguments }` |
| `tool/completed` | tool 完成 | `{ session_id, tool_call_id, success, output }` |
| `subagent/spawned` | 子 agent 启动 | `{ session_id, subagent_id, agent_type }` |
| `subagent/completed` | 子 agent 完成 | `{ subagent_id, success }` |
| `nested/tool/call` | 嵌套 tool call | `{ subagent_id, name, depth, arguments }` |
| `nested/tool/result` | 嵌套 tool result | `{ subagent_id, name, depth, success }` |
| `status/update` | 状态更新 (token/cost) | `{ session_id, model, input_tokens, ... }` |
| `progress/update` | 进度 | `{ session_id, status, message }` |
| `approval/required` | tool 需批准 | `{ session_id, request_id, tool_name, description }` |
| `ask/required` | 询问用户 | `{ session_id, request_id, question, options }` |
| `plan/required` | 计划需批准 | `{ session_id, request_id, plan_content }` |
| `session/activity` | 会话状态 | `{ session_id, running }` |
| `mcp/server/connected` | MCP server 状态变化 | `{ server_name }` |
| `error/raised` | 错误 | `{ session_id, message, code }` |

**对比当前 FrontendEvent 17 变体**:多 1 个(`progress/update` 之前是 todo 类的)。命名全部从 snake_case 改成 `<noun>/<past-tense>`,带 namespace。

#### 3.3.4 兼容层 (migration shim)

**v1 协议上线后,旧协议路径必须保留至少 1 个 release**,保证:
- 老 binary 能连新 server (server 解析旧 `WsBroadcast` 名字)
- 新 binary 能连老 server (client 解析旧 `mcp:status_changed` 等)

**实现位置:** `src-tauri/src/server.rs` 的 shim 层,加一个 `legacy_event_name_to_v1()` 函数,把 4 套命名都映射到 v1 事件名。

**移除时机:** v0.4 (即 v1 协议上线 1 个 minor 之后)。

### 3.4 实施阶段

| Phase | 内容 | 估时 | 风险 |
|---|---|---|---|
| 1A | 新建 `opendev-protocol` crate,定义 `WireEnvelope` + `version.rs` + `transport.rs` | 1 周 | 低 |
| 1B | v1 方法 + 事件的所有类型 (28 方法 + 18 事件) | 2 周 | 中 (类型多) |
| 1C | ts-rs 绑定 + 前端 `@opendev/protocol-types` 包 | 1 周 | 低 |
| 1D | 5 端 Transport trait 抽象 + 1 个端到端 (Tauri) 实现 | 1 周 | 中 (Tauri Channel 集成) |
| 1E | migration shim + 旧路径 1-release 兼容 | 1 周 | 中 |
| 1F | 端到端测试 (snapshot + integration) | 1 周 | 低 |
| **合计** | | **~7 周** | |

**v1 不做的事 (留给 v2):**
- Realtime 语音 (Codex 那种)
- WebSocket 多客户端同步 (Codex App 那种)
- 反向通知 (server → client broadcast for server-initiated actions)
- 实验性方法 (`#[experimental(...)]`)

**Frontend migration path (Oracle review 确认):**

**当前状态:** `src/stores/chat.ts` 有 25+ `eventBridge.on('message_start', ...)` 监听 snake_case 字符串 (recon §5.4)。server shim 能把 v1 `message/start` 事件映射回 `message_start` 字符串给老 client,但**前端 25 个 handler 的订阅名也要同时迁**。

**双轨期方案 (v0.2.0 → v0.3.0):**
- (a) **v0.2.0**: server 端 dual-emit — 同一事件发 v1 名 (`message/start`) + 老名 (`message_start`)。前端 25 个 handler 继续工作。
- (b) **v0.2.x**: 前端逐步迁,每次发 PR 改 1-2 个 handler,改成订阅 v1 名 (e.g. `'message/start'`)
- (c) **v0.3.0**: server 端停止 dual-emit,只发 v1 名。前端所有 handler 已迁完。
- (d) **v0.4.0**: 删 server 端 shim,删 `WSMessage.type` union 里的所有老名 (`mcp_status_update` 等)

**前端 migration 工具:**
- `scripts/migrate-event-names.ts` — 扫 `src/`,把所有 `eventBridge.on('xxx', ...)` 提到 `eventNames.ts` 常量,统一替换
- 25 个 handler 的订阅名集中到 `src/api/eventNames.ts` 一处定义,避免散落
- 加 ESLint rule: `eventBridge.on` 第一参数必须用 import 的常量,不允许 magic string

**老 chat store 重构:**
- v0.2 期间: `chat.ts` 5 个 store 都保留 snake_case 订阅,加 deprecation 注释
- v0.3: 全部迁完,删 deprecation 注释
- v0.4: 老 alias 类型 (`mcp_status_update` 等) 从 `WSMessage.type` union 移除

### 3.5 关键决策 (需用户输入)

| # | 决策 | 我的推荐 | 备选 |
|---|---|---|---|
| 3-A | **wire 格式选哪个?** | JSON (over JSON-RPC 2.0-like 风格) | MessagePack / CBOR / bincode |
| 3-B | **transport 抽象层放在哪?** | `opendev-protocol::transport` (供所有 crate 依赖) | 每个 crate 各自实现 |
| 3-C | **v1 命名约定 `domain/object/action` vs `<noun>/<past-tense>`?** | `<noun>/<past-tense>` (类似 Codex 风格,事件性质) | 沿用 constitution 推荐的 `domain.object.action` |
| 3-D | **V1 frozen 时机?** | v0.2.0 GA 时冻结 (而非 v0.1.9) | v0.1.10 即冻结(激进) |
| 3-E | **Telegram 是否在 v1 必做?** | 否 (v1 覆盖 Tauri/TUI/Web,Workspace;Telegram 走 v2) | v1 必做 |
| 3-F | **保留 server.rs 临时桥接多久?** | v1 上线后保留 1 个 minor (v0.3) | 立即删除(激进) |

---

## 4. 沙箱 (Sandbox) — 安全的底座

### 4.1 现状摘要 (来自 recon)

**两套"沙箱",都是空头:**

1. **`opendev-sandbox` 顶层 crate (1,036 LoC, 9 文件)**
   - `#![cfg(target_os = "linux")]` — macOS / Windows 完全排除编译
   - 100% stub:`MicroSandbox::create()` 返回 `Ok(Self { started: false })`,`run_code()` 返回 `Ok(String::new())`
   - 唯一真实代码:`runtime.rs` 的 `msb` 二进制发现 + 健康检查
   - 唯一真实模块:`parser.rs` 的 `FINAL()` 正则提取
   - 真正执行 LLM 生成的 Python → sandbox microVM → loop 的整条链路从未跑通
   - **微 VM (microsandbox) 设计偏离主流**:Codex 走 Landlock/Seatbelt/bwrap,不走 microVM

2. **`opendev-runtime::sandbox` 内部模块 (163 LoC + 127 LoC 测试)**
   - `SandboxConfig { enabled, allowed_commands: Vec<String>, writable_paths: Vec<String> }`
   - `for_project()` 预设 27 个白名单命令
   - `check_command()` 和 `check_writable_path()` 函数**完整实现**
   - **但是 — 0 个调用点**。`grep` 全 workspace 找不到任何生产代码调用它。`BashTool` 不知道它存在。

**BashTool — 唯一真实的 exec 表面,有几个补丁,没有真正的沙箱:**

- ✅ `setpgid(0, 0)` 新建进程组 (便于 kill -pgid)
- ✅ `env_clear()` + `filtered_env()` (按后缀过滤 6 类 + 13 硬编码名字)
- ✅ 双超时 (idle 60s, max 600s) + cancel token
- ✅ 危险命令正则 (16 个)
- ❌ **无 Landlock / seccomp / AppArmor / Seatbelt** — 完整 syscall 权限
- ❌ **无 chroot / namespace / setuid / capability drop** — 跑在用户权限下
- ❌ **无 rlimit** — 可 fork bomb
- ❌ **无文件系统写限制** — `writable_paths` 没被用
- ❌ **无网络命名空间** — 可任意 curl

**另外 17 个 exec 点 (来自 recon §5):** git、hooks、MCP stdio transport、custom tool、`!`cmd``、schedule、sound、web screenshot (用 `--no-sandbox`!)、open_browser、file search、formatter、curl fallback、microsandbox server — **全部无沙箱**。

**BashTool env filter 的覆盖盲区** (recon §5.5 + §13.6):
- ✅ 在 BashTool foreground/background 应用
- ❌ hooks (`opendev-hooks/executor.rs:90-96`) — 无 env clear
- ❌ MCP stdio (`opendev-mcp/src/transport/stdio.rs:100-109`) — 用 config env
- ❌ custom tool (`opendev-tools-impl/src/custom_tool.rs:127-132`) — 无 env clear
- ❌ patch (`opendev-tools-impl/src/patch/mod.rs:102,127`) — 继承父进程 env
- ❌ worktree (git) — 继承父进程 env
- ❌ snapshot (shadow git) — 继承父进程 env
- ❌ LSP server spawn — 继承父进程 env

→ LLM 起的 hook / MCP server / custom tool 能直接读 `OPENAI_API_KEY` 等。

**SSRF precedent (`web_fetch/mod.rs:33-67`)** 是当前**唯一**的"输入端网络隔离"实现:loopback / private / link-local / multicast / fc00::/7 全部 fail-closed。这是新沙箱层可以复用的范式。

### 4.2 设计目标

**主目标:** **任何 tool exec 都不再裸跑在用户 host 上。**

**子目标:**

1. **跨平台** — macOS / Linux / Windows 都有真实隔离(不要重蹈 `opendev-sandbox` Linux-only 覆辙)
2. **真用上** — 新抽象必须**所有** exec 点都调用,否则就是 `opendev-runtime::sandbox` 第二
3. **可测试** — CI 跑得动,跨平台;policy engine 可 mock
4. **可降级** — 系统不支持时,优雅退化到 "no-sandbox + 显式提示"
5. **可观测** — 每个 spawn 都 emit `tracing` 事件,记策略
6. **保守起步** — v1 用 Rust pattern matching (够用),v2 再考虑 Starlark DSL

### 4.3 设计方案

#### 4.3.1 重命名与新 crate

**解决命名冲突** (recon §14.4 建议):

| 旧名 | 新名 | 理由 |
|---|---|---|
| `opendev-sandbox` (顶层 crate) | `opendev-sandbox` (保留,作为 microVM 后端,**feature-gate**) | 不破名,但要 `#![cfg(feature = "microsandbox")]` |
| `opendev-runtime::sandbox` | `opendev-runtime::policy` | 跟 `opendev-agents` 的 `policy` 概念一致 |
| `opendev_models::config::SandboxConfig` | `opendev_models::config::ExecPolicy` | 跟"执行策略"语义一致 |
| 新抽象 (wraps `Command::new`) | `opendev-exec` (顶层 crate, NEW) | 名字清晰:"执行层",不做 microVM |

**新 crate 拓扑:**

```
crates/opendev-exec/                     ← NEW
├── src/
│   ├── lib.rs                           # 公共 API
│   ├── policy.rs                        # ExecPolicy (trait)
│   ├── backend.rs                       # Backend (trait) — 不同 OS 后端
│   ├── backends/
│   │   ├── mod.rs
│   │   ├── landlock.rs                  # Linux Landlock (主推)
│   │   ├── bwrap.rs                     # Linux bubblewrap (备选)
│   │   ├── seatbelt.rs                  # macOS sandbox-exec
│   │   ├── windows.rs                   # Windows restricted token / job
│   │   └── none.rs                      # 降级 (no sandbox)
│   ├── process.rs                       # HardenedProcess: env filter + pre_exec + 监控
│   ├── env_filter.rs                    # 共享 env 过滤 (从 BashTool 提取)
│   ├── pattern.rs                       # 共享 dangerous-pattern 检测
│   └── capability.rs                    # 资源限制 (rlimit / ulimit)
```

#### 4.3.2 ExecPolicy trait

**核心抽象:**

```rust
// crates/opendev-exec/src/policy.rs
pub trait ExecPolicy: Send + Sync {
    /// 评估一个 command 描述,返回 Decision
    fn evaluate(&self, request: &ExecRequest) -> Result<Decision, PolicyError>;

    /// 名称 (用于 tracing + debugging)
    fn name(&self) -> &'static str;

    /// 是否启用了 syscall 隔离 (vs 仅 env filter)
    fn has_os_isolation(&self) -> bool;
}

pub struct ExecRequest {
    pub tool: ToolKind,                   // Bash / Custom / Hook / MCP / Git / ...
    pub command: String,                  // 原始命令
    pub argv: Vec<String>,                // 解析后
    pub cwd: PathBuf,
    pub env: HashMap<String, String>,
    pub requested_paths: Vec<PathBuf>,    // 试图访问的路径
    pub requested_net: Option<Url>,
    pub capabilities: RequiredCapabilities,
}

pub enum Decision {
    Allow,                                 // 直接放行
    AllowWith(PolicyVerdict),              // 放行但记录 verdict
    Deny { reason: String },               // 拒绝
    Prompt { reason: String, ttl: Duration }, // 需用户批准
}

pub struct RequiredCapabilities {
    pub read: Vec<PathBuf>,
    pub write: Vec<PathBuf>,
    pub network: bool,
    pub subprocess: bool,
    pub max_memory_mb: Option<u64>,
    pub max_cpu_secs: Option<u64>,
    pub max_open_fds: Option<u64>,
    pub max_file_size_mb: Option<u64>,
}
```

**v1 内置 policies:**

1. `StrictPolicy` — 全 deny,只有 hard-coded 读命令可过 (`ls`, `cat`, `head`, `grep`, `git status`, ...)
2. `WorkspaceWritePolicy` — 默认:可写 cwd + tmp,只读 ~/Library/Application Support,网络 opt-in
3. `ReadOnlyPolicy` — 完全只读,无网络
4. `DangerFullAccessPolicy` — 不推荐,显式 opt-in
5. `BashToolPolicy` — BashTool 专用:复用 16 个 dangerous regex + 27 个 safe command 白名单 + read-only 启发

**policy 决议 = pattern matching** (Rust,无 DSL):

```rust
// 例:BashToolPolicy
impl ExecPolicy for BashToolPolicy {
    fn evaluate(&self, req: &ExecRequest) -> Result<Decision, PolicyError> {
        if is_dangerous_command(&req.command) {
            return Ok(Decision::Deny { reason: "matches dangerous pattern".into() });
        }
        if !self.is_safe_command(&req.command) {
            return Ok(Decision::Prompt { reason: "not in safe list", ttl: 5.minutes() });
        }
        Ok(Decision::Allow)
    }
}
```

**v2 考虑 Starlark DSL** (deferred)— 当 pattern 数量超过 50 / 出现用户自定义 policy 需求时再评估。

#### 4.3.3 Backend trait

```rust
// crates/opendev-exec/src/backend.rs
pub trait SandboxBackend: Send + Sync {
    fn name(&self) -> &'static str;
    fn supported(&self) -> bool;          // 探测 OS/kernel 支持
    fn apply(&self, cmd: &mut Command, request: &ExecRequest) -> Result<(), BackendError>;
    fn post_spawn_check(&self, child_pid: u32) -> Result<(), BackendError>;
}
```

**v1 后端实现:**

| Backend | 平台 | 隔离强度 | 依赖 | 实施优先级 |
|---|---|---|---|---|
| `LandlockBackend` | Linux 5.13+ | 文件系统路径白名单 (细粒度) | `landlock = "0.4"` | **1st (主推)** |
| `SeatbeltBackend` | macOS 12+ | sysctl + fs + net (细粒度) | 系统 `sandbox-exec` | **1st** |
| `BwrapBackend` | Linux 任意 | 完整 namespace 隔离 (粗粒度) | 系统 `bwrap` 或 vendored | 2nd (高级用) |
| `WindowsBackend` | Win 10+ | Job object + restricted token | `windows-sys` | 3rd |
| `NoneBackend` | 所有 | 仅 env filter | (无) | 降级 (默认兜底) |

**BashTool 应用示意:**

```rust
// crates/opendev-tools-impl/src/bash/foreground.rs (改造后)
fn run_foreground(...) {
    let policy = BashToolPolicy::new(working_dir);
    let backend = SandboxBackend::detect(); // 自动选最佳可用后端
    
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(&command).current_dir(working_dir);
    
    // env filter
    let safe_env = env_filter::filter(std::env::vars());
    cmd.env_clear().envs(&safe_env);
    
    // 沙箱注入
    let request = ExecRequest { tool: ToolKind::Bash, command: command.clone(), ... };
    let decision = policy.evaluate(&request)?;
    match decision {
        Decision::Deny { reason } => return Ok(ToolResult::fail(format!("Denied: {reason}"))),
        Decision::Prompt { reason, ttl } => return Ok(ToolResult::needs_approval(reason, ttl)),
        _ => {}
    }
    // ← 关键 (Oracle review): 沙箱 apply 失败必须 fail-closed
    // 任何 backend.apply() 错误 (Landlock EOPNOTSUPP / bwrap ENOENT / Seatbelt
    // 拒绝) 都转化为 Decision::Deny,**不能让 child 跑在无沙箱状态**
    if let Err(e) = backend.apply(&mut cmd, &request) {
        tracing::error!(error = %e, backend = backend.name(), "sandbox apply failed; refusing to spawn");
        return Ok(ToolResult::fail(format!(
            "Sandbox backend '{}' failed to apply: {}. Command not executed (fail-closed).",
            backend.name(), e
        )));
    }
    
    // 进程组
    #[cfg(unix)]
    unsafe { cmd.pre_exec(|| { libc::setpgid(0, 0); Ok(()) }); }
    
    // ... spawn + monitor 同前
}
```

**Fail-closed 原则 (Oracle review 确认):**
- 沙箱 backend 在 runtime 失败 (kernel 突然不支持、namespace 用尽) → `Decision::Deny`,child 不 spawn
- 任何 fallback 都不允许绕过 policy
- 仅在 init 时 (Phase 1 detect) 允许降级到 `NoneBackend` (带显式 UI 警告)
- Fail-closed 写进 `opendev-exec::BACKEND_FAIL_CLOSED` 常量,代码 review 必查

#### 4.3.4 env_filter 提升为共享模块

**`opendev-exec/src/env_filter.rs`** 统一所有 exec 点的 env 过滤:

- 扩展现有 6 后缀 + 13 硬编码名字
- 新增 (recon §13.6 列出):
  - `*_PRIVATE_KEY` / `*_CLIENT_SECRET` / `*_ACCESS_KEY` (AWS)
  - `OAUTH_*` / `JWT_*` / `BEARER_*` / `*_CONNECTION_STRING`
  - `DATABASE_URL` / `REDIS_URL` / `POSTGRES_*`
  - `*_PASS` / `*_KEYFILE` / `*_CERT` / `*_TLS_*`
- 提供 `apply(cmd: &mut Command)` 方法,所有 exec 路径必须调用

#### 4.3.5 全 exec 点迁移

**优先级 (按风险排):**

| 优先级 | exec 点 | 当前状态 | 改造 |
|---|---|---|---|
| P0 | `BashTool` (foreground + background) | env filter only | 加 policy + backend |
| P0 | `opendev-hooks/executor.rs` | 无 env clear | 加 env_filter + policy |
| P0 | `opendev-mcp/transport/stdio.rs` | config env | 加 env_filter (config env 在此 filter 后传入) — **但保留 opt-out** |

**MCP env exception (Oracle review 指出):** 不是所有 MCP server child 进程都应该 strip API key。

**对策:**
- `McpServerConfig` 加 `passthrough_env: Vec<String>` 字段(显式 opt-in 的 "this MCP server needs these env vars")
- env_filter 默认 strip,但 `passthrough_env` 列表里的 name 保留
- `opendev mcp doctor` 命令检查每个 server 的 config,提示 "你配了 passthrough_env,确保这是有意的"
- 文档强制声明:MCP server 自身应使用 `SecretStore` 取 key,不要靠 parent env;`passthrough_env` 仅作为过渡期 hack

**示例:**
```json
// mcp_servers.json
{
  "weather-mcp": {
    "command": "weather-mcp-server",
    "args": [],
    "passthrough_env": ["WEATHER_API_KEY"],
    "envs": {}
  }
}
```
| P0 | `opendev-tools-impl/src/custom_tool.rs` | 无 env clear | 加 env_filter + policy |
| P0 | `opendev-tools-impl/src/patch/mod.rs` (git apply) | 继承父 env | 加 env_filter |
| P1 | `opendev-runtime/src/custom_commands/expansion.rs` (`!`cmd``) | 同步阻塞 + 继承 | 加 env_filter + 改异步 |
| P1 | `opendev-tools-lsp/src/handler.rs` (LSP server) | 继承 | 加 env_filter |
| P1 | `opendev-runtime/src/snapshot.rs` (shadow git) | 继承 | 加 env_filter |
| P1 | `opendev-tools-impl/src/worktree.rs` (git worktree) | 继承 | 加 env_filter |
| P2 | `opendev-agents/src/attachments/collectors/git_status.rs` | 继承 | 加 env_filter |
| P2 | `opendev-plugins/src/marketplace.rs` (git clone) | 继承 + 无 URL allow-list | 加 env_filter + URL allowlist |
| P2 | `opendev-context/src/environment/instructions.rs` (curl) | 继承 | 加 env_filter + URL allowlist |
| P2 | `opendev-agents/src/skills/discovery.rs` (curl) | 继承 | 加 env_filter + URL allowlist |
| P3 | `opendev-tools-impl/src/web_screenshot.rs` | `--no-sandbox` (!!) | **去掉 --no-sandbox**,加 backend |
| P3 | `opendev-tools-impl/src/open_browser.rs` | 继承 | 加 env_filter |
| P3 | `opendev-tools-impl/src/formatter.rs` | config env | 加 env_filter |
| P3 | `opendev-tools-impl/src/file_search/{grep_tool,backends}.rs` | 继承 | 加 env_filter |

**"加 env_filter" 改动量:** 每个点 1-3 行(调用 `env_filter::apply(&mut cmd)`)。
**"加 policy" 改动量:** 高风险点(P0)10-20 行,新 policy 决议。

#### 4.3.6 SSRF 提升为共享

`is_private_url` 提到 `opendev-exec::net_filter::is_private_url` (或更合适的新 crate `opendev-net`),在以下地方复用:

- `WebFetchTool` (已有)
- `WebScreenshotTool` (Chrome URL,recon §13.24)
- `OpenBrowserTool`
- `opendev-plugins/marketplace.rs` (git clone URL)
- `opendev-context/environment/instructions.rs` (curl)
- `opendev-agents/skills/discovery.rs` (curl)

**不要在 `opendev-exec` 里** — 网络过滤是协议无关的,放新 crate 或 `opendev-models` 都行。我建议放 `opendev-exec` 的一个子模块 `opendev-exec::net_filter`,因为它跟"exec"语义紧。

### 4.4 实施阶段

| Phase | 内容 | 估时 | 风险 |
|---|---|---|---|
| 2A | 新建 `opendev-exec` crate,定义 trait + env_filter 提取 | 1 周 | 低 |
| 2B | 4 个后端 (Landlock/Seatbelt/Bwrap/Windows),先各做"hello world" | 2 周 | 中 (OS API 差异) |
| 2C | `BashToolPolicy` 实现 + 集成到 BashTool | 1 周 | 中 |
| 2D | P0 exec 点全迁移 (5 个) | 1 周 | 中 |
| 2E | P1 + P2 exec 点迁移 (8 个) | 1 周 | 低 |
| 2F | P3 exec 点迁移 + 去掉 `web_screenshot --no-sandbox` | 1 周 | 中 |
| 2G | `opendev-sandbox` 重写 (microVM 单独 feature-gate) | 1 周 | 低 |
| 2H | 测试 + 文档 (每后端 cross-platform CI) | 1 周 | 中 |
| **合计** | | **~9 周** | |

### 4.5 关键决策 (需用户输入)

| # | 决策 | 我的推荐 | 备选 |
|---|---|---|---|
| 4-A | **microVM (microsandbox) 还要不要?** | **不要**(Codex 也不走 microVM,效果差,复杂度高)。`opendev-sandbox` 重写为 Landlock/Seatbelt/bwrap + feature-gate microVM | 保留 microVM 路线,集成 microsandbox |
| 4-B | **BashTool 当前的 16 个 dangerous regex + 27 个 safe command 是否扩充?** | v1 不动,加 5 个 (chmod 777 file / chown -R / eval / source / > ~/.ssh) | 大幅扩展到 30+ 个 |
| 4-C | **policy engine 走 Rust pattern matching 还是 Starlark DSL?** | **v1 走 Rust pattern** (简单、可测、好维护);v2 再考虑 Starlark | v1 直接上 Starlark |
| 4-D | **降级策略**:kernel 不支持 Landlock 时怎么办? | 默认 `NoneBackend` + UI 显眼横幅警告 "沙箱不可用,仅 env filter" | panic 退出 |
| 4-E | **`opendev-runtime::sandbox` 重命名时机?** | v0.2 改名 (与 Protocol v1 一起) | 单独 ADR-008 立即改名 |
| 4-F | **`web_screenshot --no-sandbox` 是否本次必去?** | **必去** (v0.2 + sandbox 上线同期) | 保留 --no-sandbox |
| 4-G | **Workspace 模式 (Codex 那种自动 worktree)** 是不是 sandbox 必含? | 否,Workspace 模式放在 v2 | 是 (v1) |

---

## 5. 凭据 (Keyring) — 隐私的底座

### 5.1 现状摘要 (来自 recon)

**4 套凭据存储,各管各的:**

| 存储 | 位置 | 权限 | 用途 | 使用情况 |
|---|---|---|---|---|
| `settings.json` | `~/.opendev/settings.json` | 无 0600 | LLM API key / Telegram token | **唯一活跃的存** |
| `auth.json` | `~/.opendev/auth.json` | 0600 (per `auth.rs:182-188` + 启动时收紧) | LLM API key + OAuth token | **死代码** (`CredentialStore` 0 调用点) |
| `users.json` | `~/.opendev/users.json` (CLI) / `<data>/users/users.json` (Tauri) | 0600 | Web UI 用户密码 (Argon2) | 活跃 (但 Tauri 不门禁) |
| `mcp.json` | `~/.opendev/mcp.json` | 0600 | MCP OAuth `client_secret` | 活跃 |

**API key 实际解析路径** (`AppConfig::get_api_key_with_env`,recon §7):

```
1. models.dev registry env var (e.g. ZHIPU_API_KEY)   ←
2. builtin env var (14 providers)                       ← env var 总是赢
3. convention-based env (PROVIDER_API_KEY)
4. AppConfig.api_key in settings.json                   ← 当前主路径
5. OPENAI_API_KEY fallback
```

→ Tauri desktop 走 settings.json,CLI 走 env var,**两条路径,不统一**。

**重大问题清单** (recon §14):

- ❌ `CredentialStore` 死代码 (240 行 + 96 行测试,**0 调用点**)
- ❌ `AuthProfileManager` 死代码 (240 行 + 106 行测试,**0 调用点**)
- ❌ API key 在 `settings.json` 是**明文**,**无 0600** (recon §7.4 验证)
- ❌ Telegram bot token 在 `settings.json` 是**明文**
- ❌ HMAC session key (`OPENDEV_SECRET_KEY`) `Box::leak` 到 `&'static [u8]`,**永远不 zeroize** (recon §9.3)
- ❌ debug build 默认 `secret_key = "change-me-in-production"` (release panic,debug 接受)
- ❌ 整个 workspace **0 个 `zeroize` 使用**,0 个 `secrecy::SecretString` 使用 (recon §13)
- ❌ `redact_secrets` 只在 2 个 tool 输出位置用,**不在 tracing 路径上**
- ❌ **Tauri 命令零认证** (recon §10.2)
- ❌ **WebSocket handler 零认证** (recon §10.3)
- ❌ **opendev-web 路由零认证** (除 `/api/auth/me`)
- ❌ Constitution 7 说"API keys in system credential store when available",**完全是空头**

**好消息 (recon §13.2):**

- ✅ `keyring = "3.6.3"` 已经在 `Cargo.lock` 里(通过 `opendev-sandbox → microsandbox` 传递)
- ✅ `zeroize = "1.9.0"` 已经在 `Cargo.lock` 里(通过 `keyring` / `aws-lc-rs` / `rustls`)
- ✅ macOS `security-framework = "2.11.1" / "3.7.0"` 已在
- ✅ Linux `secret-service = "4.0.0"` / `dbus-secret-service = "4.1.0"` 已在

→ **直接加 `keyring` / `secrecy` 到 `opendev-http` 的 `Cargo.toml` 零编译成本**(后端都已在)。

### 5.2 设计目标

**主目标:** **让 constitution 第 7 条从"愿景"变成"现实"。**

**子目标:**

1. **OS keyring 优先** — macOS Keychain / Linux Secret Service / Windows Credential Manager
2. **环境变量覆盖** — env var 永远最高优先(方便 CI / Docker / 临时场景)
3. **明文文件兜底** — keyring 不可用时 (headless server / CI),fallback 到 加密文件 (`age` 或对称 AES)
4. **零明文 on disk** — settings.json / auth.json / mcp.json 全部**不存明文 secret**
5. **类型级保护** — `secrecy::SecretString` + `zeroize::Zeroize` 双保险
6. **激活死代码** — `CredentialStore` / `AuthProfileManager` 真的被用,而不是测试通过就完事
7. **migration 工具** — 现有用户的 `settings.json` 明文 key 一次性迁移

### 5.3 设计方案

#### 5.3.1 新 crate: `opendev-secrets`

**职责:** 唯一定义 secret 存储抽象,提供 `SecretStore` trait,实现 KeyringStore + FileStore + EnvStore 三个 backend。

```
crates/opendev-secrets/                  ← NEW
├── Cargo.toml                           # keyring 3, secrecy 0.10, zeroize 1
├── src/
│   ├── lib.rs                           # 重新导出
│   ├── error.rs                         # SecretError
│   ├── key.rs                           # SecretKey (类型,避免 string 错传)
│   ├── value.rs                         # SecretValue (类型,newtype SecretString)
│   ├── provider.rs                      # SecretProvider (LLM provider 名 → key 命名空间)
│   ├── store.rs                         # SecretStore trait
│   ├── backends/
│   │   ├── mod.rs
│   │   ├── keyring.rs                   # macOS / Linux / Windows
│   │   ├── file.rs                      # 加密文件 fallback
│   │   └── env.rs                       # env var 覆盖
│   ├── resolver.rs                      # 多 backend 链式解析
│   ├── audit.rs                         # 访问审计 (用于 telemetry)
│   ├── migration.rs                     # 从 settings.json / auth.json 迁移
│   └── rotate.rs                        # AuthProfileManager 复活
└── tests/
```

#### 5.3.2 类型设计

**避免 "string 错传" 是第一步:**

```rust
// src/key.rs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SecretKey {
    namespace: Namespace,    // LLM / Telegram / Mcp / Hmac / WebUser
    account: String,         // e.g. "openai" / "telegram/bot" / "hmac/session"
}

pub enum Namespace {
    Llm,                     // LLM provider API keys
    Telegram,                // Telegram bot tokens
    Mcp,                     // MCP OAuth secrets
    Hmac,                    // Web session signing keys
    WebUser,                 // Web UI user passwords (already Argon2 hashed)
}

impl SecretKey {
    pub fn llm(provider: &str) -> Self { ... }
    pub fn telegram() -> Self { ... }
    pub fn hmac_session() -> Self { ... }
}

// src/value.rs
pub struct SecretValue(secrecy::SecretString);

impl SecretValue {
    pub fn new(s: String) -> Self { Self(secrecy::SecretString::new(s)) }
    pub fn expose(&self) -> &str { self.0.expose_secret() }
}

// 重要:Display 和 Debug 都打印 [REDACTED]
impl fmt::Display for SecretValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}
impl fmt::Debug for SecretValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretValue([REDACTED])")
    }
}
```

→ tracing 自动 redact(`%secret` 走 Display)。**这是 constitution 7 "never logged" 的真实实现**。

#### 5.3.3 SecretStore trait

```rust
#[async_trait]
pub trait SecretStore: Send + Sync {
    async fn get(&self, key: &SecretKey) -> Result<Option<SecretValue>, SecretError>;
    async fn set(&self, key: &SecretKey, value: &SecretValue) -> Result<(), SecretError>;
    async fn delete(&self, key: &SecretKey) -> Result<bool, SecretError>;
    async fn list(&self, namespace: Namespace) -> Result<Vec<SecretKey>, SecretError>;
    fn backend_name(&self) -> &'static str;
}
```

**Chain resolver:**

```rust
// src/resolver.rs
pub struct ChainedSecretStore {
    stores: Vec<Arc<dyn SecretStore>>,    // 优先级: env → keyring → file
}

impl ChainedSecretStore {
    pub fn new() -> Self {
        Self {
            stores: vec![
                Arc::new(EnvStore::new()),
                Arc::new(KeyringStore::new()),
                Arc::new(FileStore::new(/* path */)),
            ],
        }
    }

    async fn get(&self, key: &SecretKey) -> Result<Option<SecretValue>, SecretError> {
        for store in &self.stores {
            if let Some(v) = store.get(key).await? {
                return Ok(Some(v));
            }
        }
        Ok(None)
    }
}
```

#### 5.3.4 Backend 实现

**EnvStore (env var 覆盖,永远最优先):**

```rust
pub struct EnvStore;
impl SecretStore for EnvStore {
    async fn get(&self, key: &SecretKey) -> Result<Option<SecretValue>, SecretError> {
        let env_var = key.to_env_var();  // "OPENAI_API_KEY" / "TELEGRAM_BOT_TOKEN" / ...
        match std::env::var(&env_var) {
            Ok(v) if !v.is_empty() => Ok(Some(SecretValue::new(v))),
            _ => Ok(None),
        }
    }
    // set / delete 都是 no-op (env var 是只读的)
}
```

**KeyringStore (主存储,生产):**

```rust
pub struct KeyringStore {
    service: String,                     // "com.opendev.desktop"
}

impl SecretStore for KeyringStore {
    async fn get(&self, key: &SecretKey) -> Result<Option<SecretValue>, SecretError> {
        let entry = keyring::Entry::new(&self.service, &key.account)?;
        match entry.get_password() {
            Ok(v) => Ok(Some(SecretValue::new(v))),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(SecretError::Backend(e.into())),
        }
    }
    async fn set(&self, key: &SecretKey, value: &SecretValue) -> Result<(), SecretError> {
        let entry = keyring::Entry::new(&self.service, &key.account)?;
        entry.set_password(value.expose())?;
        Ok(())
    }
}
```

**FileStore (加密 fallback,CI / headless):**

```rust
pub struct FileStore {
    path: PathBuf,                       // ~/.opendev/secrets.age
    key: SecretValue,                    // 从 keyring 或 env 或 auto-generated 拿
}

impl SecretStore for FileStore {
    // set / get / delete 全部用 age (https://github.com/rustsec/rustsec) 加密
    // 加密算法:age X25519 + scrypt
}
```

**keyring 不可用时的降级顺序:**

```
尝试 keyring → 成功:用 keyring
            → 失败 (e.g. headless Linux no D-Bus):
               1. 试 file store (需要 master key)
                  1a. master key 在 env:用
                  1b. master key 在 settings.json (自动生成,一次性):用
                  1c. 都没有:报错,告诉用户 "set OPENDEV_MASTER_KEY or `opendev setup`"
```

#### 5.3.5 集成到 AppConfig::get_api_key_with_env

**替换现有实现:**

```rust
// opendev-models/src/config/mod.rs (新)
impl AppConfig {
    pub async fn get_api_key(&self, registry_env_var: Option<&str>, secrets: &ChainedSecretStore) -> Result<SecretValue, String> {
        // 1. 优先:SecretStore 链 (env → keyring → file)
        let key = SecretKey::llm(&self.model_provider);
        if let Some(v) = secrets.get(&key).await.map_err(|e| e.to_string())? {
            return Ok(v);
        }
        // 2. 兜底:AppConfig.api_key (existing, deprecated path)
        if let Some(ref k) = self.api_key {
            tracing::warn!("AppConfig.api_key is deprecated; migrate to keyring");
            return Ok(SecretValue::new(k.clone()));
        }
        Err(format!("No API key for provider {}", self.model_provider))
    }
}
```

**Shadow-key UX 问题 (Oracle review 指出):** 当前实现是 env 永远赢。如果用户已经在 `OPENAI_API_KEY` env var 设了 key,然后通过 settings UI 试图改 keyring,新 keyring 写入后**永远不被读**(env 仍赢)。这会让用户困惑。

**对策:**
- (a) `SecretStore::set` 在 KeyringStore 上写成功后,UI 立即提示 "已写入 keyring,但 env var 仍生效,如需 keyring 接管请 unset env"
- (b) 提供 `opendev secret doctor` 命令,扫描所有已配的 secret,列出 "env shadowed" 的 key
- (c) 在 settings UI 旁显示 "🔒 env 覆盖" 小图标

**AppConfig.api_key deprecation 时间表:**
- v0.2.0: 引入 SecretStore,新 UI 写 keyring。`AppConfig.api_key` 仍可用,但每次读 emit `tracing::warn`。
- v0.2.x 期间: 启动时检测明文 key,弹窗提示迁移 (可推迟,但推迟超过 30 天则强制迁移)。
- v0.3.0: 强制 migration 工具跑一次,清空 `AppConfig.api_key` 字段。
- v0.4.0: 移除 `AppConfig.api_key` 字段 (hard break,changelog 标注)。任何在 v0.3 没迁的用户必须手动跑 `opendev secret migrate`。

**Tauri / CLI 启动时统一初始化:**

```rust
// src-tauri/src/main.rs (改造)
let secrets = ChainedSecretStore::new();   // 自动 detect 最佳 backend
let services = build_services(config, session_manager, model_registry, working_dir, secrets);
// 传给所有 service
```

#### 5.3.6 Migration 工具

**一次性执行,v0.2 GA 时强制弹窗:**

```rust
// crates/opendev-secrets/src/migration.rs
pub fn migrate_settings_json(path: &Path, secrets: &ChainedSecretStore) -> Result<MigrationReport, MigrationError> {
    let raw = std::fs::read_to_string(path)?;
    let mut value: serde_json::Value = serde_json::from_str(&raw)?;
    let mut report = MigrationReport::default();

    // api_key
    if let Some(key) = value.get("api_key").and_then(|v| v.as_str()).map(String::from) {
        if !key.is_empty() {
            secrets.set(&SecretKey::llm(&value["model_provider"].as_str().unwrap_or("unknown")), &SecretValue::new(key.clone()))?;
            value.as_object_mut().unwrap().remove("api_key");
            value.as_object_mut().unwrap().insert("api_key_ref".into(), json!("llm/<provider>"));
            report.moved.push("api_key → keyring".into());
        }
    }

    // Telegram bot token
    if let Some(token) = value.pointer("/channels/telegram/bot_token").and_then(|v| v.as_str()).map(String::from) {
        if !token.is_empty() {
            secrets.set(&SecretKey::telegram(), &SecretValue::new(token.clone()))?;
            value.pointer_mut("/channels/telegram").unwrap().as_object_mut().unwrap().remove("bot_token");
            report.moved.push("telegram.bot_token → keyring".into());
        }
    }

    // HMAC key (only if not in env)
    if std::env::var("OPENDEV_SECRET_KEY").is_err() {
        // generate or migrate
    }

    std::fs::write(path, serde_json::to_string_pretty(&value)?)?;
    Ok(report)
}
```

**触发点:** `opendev setup` (首次) + 启动时检测到明文 key 时提示用户。

#### 5.3.7 激活死代码

**`AuthProfileManager` 复活** — 加 keyring 索引:

```rust
// opendev-secrets/src/rotate.rs
pub struct AuthProfileManager {
    provider: String,
    secrets: Arc<dyn SecretStore>,
}

impl AuthProfileManager {
    pub async fn from_env_then_keyring(&self) -> Result<Vec<SecretValue>, SecretError> {
        // 1. env var
        // 2. keyring entries: "openai/account-1", "openai/account-2", ...
        // 3. file store
    }
    // get_active_key / mark_success / mark_failure 沿用现有 cooldown 逻辑
}
```

**Cooldown 保持不变** (recon §5.2): 429=30s, 401=300s, 403=600s, 5xx=30-60s。

**`CredentialStore` 复活** — 作为 `KeyringStore` 的别名 facade,保留向后兼容的 `get_key` / `set_key` API,但底层委托给 `SecretStore`。

### 5.4 实施阶段

| Phase | 内容 | 估时 | 风险 |
|---|---|---|---|
| 3A | 新建 `opendev-secrets` crate + 类型 + trait | 1 周 | 低 |
| 3B | EnvStore + KeyringStore 实现 | 1 周 | 中 (keyring 平台差异) |
| 3C | FileStore (age 加密) 实现 | 1 周 | 中 |
| 3D | 替换 `AppConfig::get_api_key_with_env` 走 SecretStore | 0.5 周 | 低 |
| 3E | Migration 工具 + 启动检测 + 提示 | 1 周 | 中 |
| 3F | 激活 AuthProfileManager (keyring 索引) | 0.5 周 | 低 |
| 3G | 全 workspace `String` → `SecretString` 替换 (Bedrock signing key 等) | 1 周 | 中 |
| 3H | 文档 + 测试 + 跨平台 CI (Linux 无 D-Bus 场景) | 1 周 | 中 |
| **合计** | | **~7 周** | |

### 5.5 关键决策 (需用户输入)

| # | 决策 | 我的推荐 | 备选 |
|---|---|---|---|
| 5-A | **`keyring` crate 版本?** | `keyring = "3"` (default features) | `keyring = "2"` (更老但更稳) |
| 5-B | **FileStore 加密算法?** | `age` X25519 + scrypt | AES-256-GCM (手写) |
| 5-C | **`secrecy::SecretString` 替换粒度?** | 全部 secret 字段 (Bedrock signing key / Argon2 hash / session token 等) | 仅 LLM API key + Telegram token |
| 5-D | **HMAC key 走 keyring?** | **走** + 加 `OPENDEV_MASTER_KEY` env 兜底 | 保持 env var (现状) |
| 5-E | **Migration 时机?** | v0.2 GA 强制弹窗 (一次性) | v0.2 软提示,v0.3 强制 |
| 5-F | **Tauri 命令零认证是否本次一并修?** | **不在本次范围** (单独 ADR 修),但 secrets 设计要留 hook | 一并修 |
| 5-G | **`CredentialStore` 死代码复活 / 删?** | **复活** (作为 SecretStore 的 compat facade) | 删 (hard break) |
| 5-H | **是否把 `OPENDEV_SECRET_KEY` 默认值 `"change-me-in-production"` 删掉?** | **删**,release panic + debug 随机生成 | 保留 (现状) |

---

## 6. 遥测 (Telemetry) — 可观测的底座

### 6.1 现状摘要 (来自 recon)

**1 个小 crate (173 LoC),做着 1 件事 (本地文件 appender):**

```
crates/opendev-observability/  (173 LoC, 4 文件)
├── src/lib.rs        (3 行,pub use)
├── src/config.rs     (77 行,TelemetryConfig 7 字段,5 个是 write-only)
├── src/guard.rs      (71 行,OtelGuard::init 安装 fmt subscriber)
└── src/error.rs      (18 行,OtelError 2 变体,1 个死代码)
```

**实际工作的部分 (很少):**
- ✅ `tracing-subscriber::fmt()` + daily-rotated file appender + `RUST_LOG` EnvFilter
- ✅ `OtelGuard::init` 安装一次性的全局 subscriber

**声明但从未实现的 (recon §15 全部验证):**
- ❌ `TelemetryConfig::otlp_endpoint` 字段写入但**从不读**
- ❌ `TelemetryConfig::export_perfetto_on_session_end` 字段写入但**从不读**
- ❌ `TelemetryConfig::record_prompt_content` / `record_tool_args` / `record_file_location` — 隐私 flag,声明但**从不强制**
- ❌ `TelemetryConfig::enabled` — 字段写入但**不 honor**
- ❌ `tracing-perfetto-writer = "0.3"` 在 Cargo.toml,`0` 个使用点
- ❌ `tracing-subscriber` `json` feature 开了,但 `OtelGuard::init` **从不调 `.json()`** → 输出是 pretty,不是 JSON
- ❌ `OPENDEV_LOG` env var 文档化,但代码读 `RUST_LOG` (recon §6)
- ❌ `OPENDEV_PERFETTO` env var 文档化,完全无 reader
- ❌ `OTEL_EXPORTER_OTLP_ENDPOINT` env var 文档化,完全无 reader

**关键架构问题:**

- ❌ **Tauri binary 从不调 `OtelGuard::init`** (recon §2.8) → 桌面用户**完全无日志文件**
- ❌ **Tauri binary 无 panic handler** → panic 走 Tauri 默认,不去 `~/.opendev/crash/`
- ❌ **无 metrics layer** — 0 个 `prometheus` / `statsd` / OTel metrics / `metrics` crate
- ❌ **无 error reporting** — 0 个 Sentry / bugsnag / sentry-rust
- ❌ **无 analytics** — 0 个 posthog / mixpanel / amplitude
- ❌ **`SessionDebugLogger` 默认 `debug_logging = true`** → 写**完整 LLM 请求/响应体**到 `~/.opendev/sessions/<id>.debug`,**不 redact** (recon §7.5)
- ❌ **`opendev-memory/src/facade.rs:147`**:`tracing::info!(content = %content, ...)` → 把**整个 memory 内容**打 info 级
- ❌ **`redact_secrets` 不在 tracing 路径上** (recon §7) → 任何 `tracing` 输出都不脱敏
- ❌ **Frontend `ErrorBoundary` 无 `componentDidCatch`** → React render 错误**静默丢弃** (recon §13)
- ❌ **50+ `console.log` 在 React 端** → dev only,无 level filter,无远程传输
- ❌ **Settings UI 无 "Diagnostics" / "Privacy" tab** → 用户**无法 opt-out**
- ❌ **`opendev-workflow` 声明 `tracing` dep 但 0 使用** → 死依赖

**好消息:**

- ✅ `tracing = "0.1.44"` 已在 workspace 全面使用 (22/24 crate 声明)
- ✅ `tracing-subscriber = "0.3.23"` + `tracing-appender = "0.2.5"` 已在
- ✅ Span 系统已经在 6 个地方使用 (`react_loop`, `llm_call`, `tool_execution`)
- ✅ `panic_handler` + crash dump 在 CLI 已有 (recon §8)
- ✅ 22/24 workspace crate 已有 `tracing` 依赖 (基建到位)

### 6.2 设计目标

**主目标:** **让"系统在跑"这件事可被观测。**

**子目标:**

1. **Tauri 桌面也跑 telemetry** — 不再是"只有 CLI 写日志,桌面零日志"
2. **真 JSON 输出** — 文件 appender 用 JSON,不是 pretty
3. **OTLP exporter 可选** — 配 `OTEL_EXPORTER_OTLP_ENDPOINT` 就工作
4. **有 metrics** — 至少 counter + gauge + histogram
5. **有 W3C TraceContext** — trace 跨 ReAct loop
6. **有 Sentry 风格错误上报 (opt-in)** — 崩了能收到
7. **无 analytics** — privacy-first,默认不收集任何用户行为
8. **settings UI 暴露开关** — 用户能看到、能 opt-out

### 6.3 设计方案

#### 6.3.1 重命名 + 拆 crate

**解决"声明 vs 实现"差距:**

| 旧 | 新 | 理由 |
|---|---|---|
| `opendev-observability` | `opendev-telemetry` (重命名) | 名字准确:telemetry = metrics + traces + logs,不只是 "observability 配置" |
| 173 LoC 全在一个 crate | 拆为多层 | 让 layer 独立,OTLP / Sentry / file 是独立 feature |

**新 crate 拓扑:**

```
crates/opendev-telemetry/                ← 重命名自 opendev-observability
├── src/
│   ├── lib.rs                           # 重新导出
│   ├── config.rs                        # TelemetryConfig (修 5 个 write-only 字段)
│   ├── guard.rs                         # TelemetryGuard (重命名自 OtelGuard)
│   ├── layers/
│   │   ├── mod.rs
│   │   ├── file.rs                      # 日志层 (JSON)
│   │   ├── otlp.rs                      # OTLP 层 (feature-gated)
│   │   ├── sentry.rs                    # Sentry 层 (feature-gated)
│   │   ├── redact.rs                    # 自动 redact 层 (基于 redact_secrets)
│   │   └── panic.rs                     # panic hook → 写 crash dump
│   ├── metrics.rs                       # counter / gauge / histogram helper
│   ├── trace_context.rs                 # W3C TraceContext 注入/提取
│   ├── retention.rs                     # 日志保留策略 (janitor task)
│   └── shutdown.rs                      # graceful shutdown
└── tests/

crates/opendev-tracing/                  ← 也许拆出来,看是否值得
├── (helper macros: span_event!, metric_counter!, ...)
```

**Cargo features:**

```toml
[features]
default = ["file", "redact", "panic"]
file = []                                # JSON 文件 appender
redact = ["dep:opendev-runtime"]         # 自动 redact secrets
panic = []                               # panic → crash dump
otlp = ["dep:opentelemetry", "dep:opentelemetry-otlp", "dep:tracing-opentelemetry"]
sentry = ["dep:sentry"]
metrics = ["dep:metrics", "dep:metrics-exporter-otlp"]
all = ["file", "redact", "panic", "otlp", "sentry", "metrics"]
```

#### 6.3.2 修 TelemetryConfig (5 个 write-only 字段全部 honor)

```rust
// src/config.rs
pub struct TelemetryConfig {
    pub enabled: bool,                              // ← 必须 honor,默认 true
    pub log_level: LogLevel,
    pub log_dir: PathBuf,                           // ← 显式,不再依赖 Paths::global_logs_dir()
    
    // 输出
    pub format: LogFormat,                          // ← NEW: Json | Pretty
    pub retention_days: u32,                        // ← NEW: 默认 14
    
    // OTLP
    pub otlp_endpoint: Option<String>,              // ← 真的读
    pub otlp_protocol: OtlpProtocol,                // ← NEW: Grpc | Http
    
    // 隐私
    pub record_prompt_content: bool,                // ← 真的强制 (默认 false)
    pub record_tool_args: bool,                     // ← 真的强制 (默认 false)
    pub record_file_location: bool,                 // ← 真的强制 (默认 true)
    
    // Sentry
    pub sentry_dsn: Option<String>,                 // ← NEW: 显式 DSN
    pub sentry_sample_rate: f32,                    // ← NEW: 默认 0.1 (10%)
    
    // Perfetto
    pub export_perfetto_on_session_end: bool,       // ← 真的实现 (默认 false,advanced)
    pub perfetto_output_dir: Option<PathBuf>,       // ← NEW
}
```

#### 6.3.3 JSON 输出 + 保留策略

**文件层 (默认):**

```rust
// src/layers/file.rs
pub fn build_file_layer(config: &TelemetryConfig) -> impl Layer<S> {
    let file_appender = tracing_appender::rolling::Builder::new()
        .rotation(Rotation::DAILY)
        .max_log_files(config.retention_days as usize)  // ← 真的限制
        .filename_prefix("opendev")
        .filename_suffix("log")
        .build(&config.log_dir)
        .expect("failed to create log file appender");

    // JSON formatter (不再 pretty)
    tracing_subscriber::fmt::layer()
        .json()
        .with_current_span(true)
        .with_span_list(false)
        .with_writer(file_appender)
}
```

**JSON 格式示例:**

```json
{
  "timestamp": "2026-06-28T10:23:45.123Z",
  "level": "INFO",
  "target": "opendev_agents::react_loop::phases::llm_call",
  "span": { "name": "llm_call", "iteration": 3, "model": "claude-opus-4" },
  "fields": {
    "provider": "anthropic",
    "input_tokens": 1234,
    "duration_ms": 2345
  },
  "message": "llm call completed"
}
```

#### 6.3.4 OTLP 导出 (feature-gated)

**添加依赖:**

```toml
otlp = ["dep:opentelemetry", "dep:opentelemetry-otlp", "dep:tracing-opentelemetry", "dep:opentelemetry_sdk"]
```

**实现:**

```rust
// src/layers/otlp.rs
pub fn build_otlp_layer(config: &TelemetryConfig) -> Option<impl Layer<S>> {
    let endpoint = config.otlp_endpoint.as_ref()?;
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()
        .ok()?;
    let provider = opentelemetry_sdk::trace::TracerProvider::builder()
        .with_batch_exporter(exporter)
        .build();
    let tracer = provider.tracer("opendev");
    Some(tracing_opentelemetry::layer().with_tracer(tracer))
}
```

**与现有 tracing 集成:**

```rust
// src/guard.rs
pub fn init(config: &TelemetryConfig) -> TelemetryGuard {
    let registry = tracing_subscriber::registry()
        .with(env_filter(config.log_level))
        .with(layers::file::build(config));
    
    if let Some(otlp) = layers::otlp::build(config) {
        registry.with(otlp).init();
    } else {
        registry.init();
    }
    TelemetryGuard { ... }
}
```

#### 6.3.5 Redact layer (privacy-by-default)

**从 `opendev-runtime::secrets::redact_secrets` 提到 `opendev-telemetry::layers::redact`:**

```rust
// src/layers/redact.rs
pub fn build_redact_layer() -> impl Layer<S> {
    // 拦截所有 events,对每个 field value 跑 redact_secrets
    // 注意:不能 redact 整个 message (开销大),只 redact 已知的 sensitive fields
    // (e.g. `api_key`, `token`, `password`, `bot_token`, `secret`, `client_secret`)
}
```

**实现策略:** 注册一个 field-name allowlist (`api_key`, `token`, `password`, `secret`, `bearer`, `key` 等),这些字段的值在写入前走 `redact_secrets`。其他字段不动。

**这是 constitution 7 "never logged" 的真实实现。**

#### 6.3.6 Metrics 层 (轻量,先 counter + gauge)

**添加依赖:**

```toml
metrics = ["dep:metrics", "dep:metrics-exporter-otlp"]
```

**Helper macros:**

```rust
// src/metrics.rs
counter!("opendev.llm.calls.total", "provider" => provider, "model" => model);
counter!("opendev.llm.tokens.input", "provider" => provider).increment(tokens as u64);
counter!("opendev.llm.tokens.output", "provider" => provider).increment(tokens as u64);
counter!("opendev.tools.executed.total", "tool" => tool_name, "result" => result);
gauge!("opendev.sessions.active").set(active_count as f64);
histogram!("opendev.llm.duration.seconds").record(duration.as_secs_f64());
```

**Exporter:** 默认 OTLP (复用 otlp_endpoint),可选 stdout (开发用)。

**v1 metrics:**

- `opendev.llm.calls.total` (counter, labels: provider, model, status)
- `opendev.llm.tokens.input` (counter, labels: provider)
- `opendev.llm.tokens.output` (counter, labels: provider)
- `opendev.llm.duration.seconds` (histogram, labels: provider)
- `opendev.tools.executed.total` (counter, labels: tool, result)
- `opendev.tools.duration.seconds` (histogram, labels: tool)
- `opendev.sessions.active` (gauge)
- `opendev.sessions.created.total` (counter)
- `opendev.cost.usd.total` (counter, labels: provider)
- `opendev.errors.total` (counter, labels: code, surface)
- `opendev.sandbox.executions.total` (counter, labels: tool, decision)
- `opendev.secrets.lookups.total` (counter, labels: namespace, result)
- `opendev.approvals.requested.total` (counter, labels: tool)
- `opendev.approvals.granted.total` (counter, labels: tool, decision)

→ **不收集 PII,只收集系统健康指标。**

#### 6.3.7 W3C TraceContext

**为 ReAct loop 加 trace propagation:**

```rust
// crates/opendev-agents/src/react_loop/execution.rs (改造)
async fn run_iteration(...) {
    let span = info_span!("react_loop.iteration", iteration = self.iteration, session_id = %self.session_id);
    let _enter = span.enter();
    
    // 注入 W3C TraceContext 到 outgoing LLM request headers
    let trace_headers = opentelemetry::global::get_text_map_propagator(|p| {
        p.extract(&HashMapContext::new())   // 实际从 incoming request 提取
    });
    let traceparent = trace_headers.get("traceparent").cloned();
    
    // 调 LLM 时把 traceparent 放到 header
    let response = self.llm_call(model, prompt, traceparent).await;
    ...
}
```

**W3C spec 格式:** `traceparent: 00-<trace-id>-<span-id>-<flags>` (16-byte trace-id hex + 8-byte span-id hex + 1-byte flags)

#### 6.3.8 Sentry 错误上报 (opt-in)

**v1 只做 "error 上报",不做 "session replay" / "performance monitoring"。**

**实现:**

```rust
// src/layers/sentry.rs
pub fn build_sentry_layer(config: &TelemetryConfig) -> Option<impl Layer<S>> {
    let dsn = config.sentry_dsn.as_ref()?;
    let guard = sentry::init(sentry::ClientOptions {
        dsn: Some(dsn.parse().ok()?),
        sample_rate: config.sentry_sample_rate,
        release: Some(env!("CARGO_PKG_VERSION").into()),
        ..Default::default()
    });
    Some(Arc::new(guard))
}
```

**触发点:**

- `OtelGuard::init` 初始化时 `sentry::init(...)`
- panic hook 自动 `sentry::capture_message(...)` (sentry-rust 内置)
- 关键 error event (`tracing::error!`) 用 `sentry::capture_error(...)` 包装 (轻量,只在 sample rate 内)

**Opt-in 路径:** settings.json 新增 `telemetry.sentry.dsn` 字段,空 = 关闭。

**v1 不做:** release tagging、source map upload、profiling、session replay。

#### 6.3.9 Tauri 入口修复

**最大 bug 是桌面无 telemetry。修复一行:**

```rust
// src-tauri/src/main.rs (改造)
fn main() {
    // 1. Init telemetry FIRST
    let _telemetry_guard = opendev_telemetry::TelemetryGuard::init(&TelemetryConfig {
        enabled: true,
        log_level: LogLevel::Info,
        log_dir: Paths::new(None).global_logs_dir(),
        format: LogFormat::Json,
        retention_days: 14,
        ..Default::default()
    });
    
    // 2. Install panic handler
    opendev_telemetry::layers::panic::install_crash_handler();
    
    // 3. Init secrets
    let secrets = ChainedSecretStore::new();
    
    // 4. Build services
    let services = build_services(config, session_manager, model_registry, working_dir, secrets);
    
    // 5. Run Tauri
    tauri::Builder::default()
        .manage(services)
        .invoke_handler(tauri::generate_handler![...])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**`install_crash_handler`:** 复制现有 `opendev-cli/src/helpers.rs:75-154` 的逻辑,加 `redact_secrets` (在 panic message 上),加 Sentry 捕获 (if configured)。

#### 6.3.10 Privacy settings UI

**SettingsModal 新增第 5 个 tab: "Privacy"**

- "Send anonymous error reports" toggle (on by default = false,explicit opt-in)
- "Allow performance metrics" toggle (off by default)
- "View local logs" button (opens `~/.opendev/logs/` in OS file manager)
- "Clear all local data" button (destructive, requires confirmation)

**前端:**

```tsx
// src/components/Settings/PrivacySettings.tsx
const [sentryEnabled, setSentryEnabled] = useState(false);
const [metricsEnabled, setMetricsEnabled] = useState(false);

const handleSentryToggle = async (enabled: boolean) => {
    await configRepository.updateTelemetryConfig({ sentry_dsn: enabled ? '<placeholder>' : null });
    setSentryEnabled(enabled);
};
```

**对应 protocol:** (Phase A 已上) `config/telemetry/update` 方法。

#### 6.3.11 `SessionDebugLogger` 修复

**当前 bug:** 默认 `debug_logging = true` 写**完整 LLM body**。修复:

- (a) **默认改为 `false`** (breaking change,需 changelog)
- (b) 启用时强制走 `redact_secrets`
- (c) 加 retention (默认 7 天)
- (d) 加 "include full payload" 显式 toggle (用户知情的 opt-in)

**或者:** 用 `tracing` layer 替代 `SessionDebugLogger` — 每个 session 一个 `tracing-appender` writer,filter by `session_id`,content 走 redact layer。**推荐这个**。

### 6.4 实施阶段

| Phase | 内容 | 估时 | 风险 |
|---|---|---|---|
| 4A | 重命名 crate + 修 `TelemetryConfig` 5 个 write-only 字段 + JSON 输出 + 保留策略 | 1 周 | 低 |
| 4B | Redact layer (从 opendev-runtime::secrets 提到 telemetry) | 0.5 周 | 中 |
| 4C | Tauri 入口 + panic handler 集成 | 0.5 周 | 低 |
| 4D | `SessionDebugLogger` 改用 tracing layer + redact + 默认 false | 1 周 | 中 |
| 4E | Metrics layer + 15 个 v1 counter/gauge/histogram | 1 周 | 低 |
| 4F | OTLP exporter (feature-gated) | 1 周 | 中 |
| 4G | W3C TraceContext 跨 ReAct loop | 0.5 周 | 中 |
| 4H | Sentry 集成 (opt-in) | 0.5 周 | 中 |
| 4I | Privacy settings UI tab | 0.5 周 | 低 |
| 4J | Frontend logger 集中化 (替换 50+ console.log) | 0.5 周 | 低 |
| 4K | 文档 + 测试 | 0.5 周 | 低 |
| **合计** | | **~7 周** | |

### 6.5 关键决策 (需用户输入)

| # | 决策 | 我的推荐 | 备选 |
|---|---|---|---|
| 6-A | **Sentry 是否本期必做?** | **必做 opt-in**(empty DSN = off),代码量小、价值大 | 跳过,只做 OTLP |
| 6-B | **Metrics 用哪个 crate?** | `metrics` + `metrics-exporter-otlp` (Codex 风格) | 自写 counter |
| 6-C | **`SessionDebugLogger` 默认改 false?** | **是** (v0.2 改,changelog 标注 breaking) | 保持 true,加更显眼的提示 |
| 6-D | **Privacy UI tab 是否必做?** | **必做** (合规要求) | 后置 |
| 6-E | **Perfetto 是否实现?** | **实现** (代码量小,debug 有用) | 移除 `tracing-perfetto-writer` 死依赖 |
| 6-F | **OTLP 默认开吗?** | **默认关** (需配 `OTEL_EXPORTER_OTLP_ENDPOINT` 才开) | 默认开 (隐私风险) |
| 6-G | **Log 保留期默认多少天?** | 14 天 (按 disk 100MB/天算,14 天 = 1.4GB) | 7 天 / 30 天 |

---

## 7. 跨切关注点

### 7.1 4 件事如何互相影响

```
            ┌─────────────────────────────────────────┐
            │           Protocol (Layer 1)            │
            │  - v1 wire format                       │
            │  - Transport trait (5 端实现)          │
            │  - ts-rs → TS bindings                  │
            └─────────────┬───────────────────────────┘
                          │ 暴露配置 / 事件 / 命令
       ┌──────────────────┼──────────────────┐
       ▼                  ▼                  ▼
┌─────────────┐   ┌──────────────┐   ┌──────────────┐
│ Sandbox     │   │ Keyring      │   │ Telemetry    │
│ (Layer 2)   │   │ (Layer 2)    │   │ (Layer 2)    │
│             │   │              │   │              │
│ - ExecPolicy│   │ - SecretStore│   │ - Traces     │
│ - Backend   │   │ - keyring    │   │ - Metrics    │
│ - env filter│   │   / file     │   │ - Redact     │
│ - SSRF      │   │   / env      │   │ - Sentry     │
└─────────────┘   └──────────────┘   └──────────────┘
       │                  │                  │
       └──────────────────┴──────────────────┘
                          │
                  都消费 Protocol
                  都互相 emit 事件
```

**具体的互相影响:**

| 类别 | 依赖 | 被依赖 |
|---|---|---|
| **Protocol** | 无 | 其它 3 类 (暴露配置 / 事件) |
| **Sandbox** | 协议 (v1 的 `tool/*` 事件携带 policy 决策) | 协议 (暴露 sandbox 配置 UI) |
| **Keyring** | 协议 (v1 的 `config/secret/*` 方法) | 沙箱 + 遥测 (env filter / redact) |
| **Telemetry** | 协议 (v1 的 `event/error` 携带 trace_id) | 其它 3 类 (tracing 自动捕获) |

### 7.2 Protocol 如何暴露其它 3 类

| 类别 | Protocol 方法 | Protocol 事件 |
|---|---|---|
| Sandbox | `sandbox/policy/get` / `sandbox/policy/update` / `sandbox/policy/list` | `sandbox/policy/violated` / `sandbox/backend/status` |
| Keyring | `secret/list` / `secret/get` / `secret/set` / `secret/delete` / `secret/migrate` | `secret/stored` / `secret/deleted` / `secret/migrated` |
| Telemetry | `telemetry/config/get` / `telemetry/config/update` / `telemetry/logs/list` | `telemetry/metric/recorded` (可选,内部用) / `telemetry/error/reported` |

→ 5 端统一通过 protocol 看到/配置所有底座。**架构层面的一致性**。

### 7.3 启动顺序

**首次启动(冷启动):**

```
1. 读 config (settings.json, 包含 telemetry + sandbox + secret 配置)
2. Init telemetry (with redaction)        ← 任何后续 log 都被 redact
3. Init secrets (keyring detect)           ← 任何后续 code 都能 ask "do we have openai key?"
4. Init sandbox (OS support detect)        ← 任何后续 tool exec 都被 policy 评估
5. Init protocol (read v1)                 ← 任何后续 client 都能 connect
6. Build app services (consume 1-5)
7. Run Tauri / CLI / TUI / Web
```

**错误降级顺序(任意一步失败):**

```
Telemetry init 失败  → stderr 输出 warning,继续 (no telemetry)
Secrets init 失败    → 提示用户"keyring 不可用,用 OPENDEV_MASTER_KEY 或 env var",继续
Sandbox init 失败    → 提示用户"sandbox 不可用,只有 env filter",继续 (降级到 NoneBackend)
Protocol init 失败   → panic (致命)
```

### 7.4 配置 (`AppConfig` 扩展)

```rust
// opendev-models/src/config/mod.rs (扩展)
pub struct AppConfig {
    // ... 现有 ~30 字段
    pub telemetry: TelemetryConfig,            // NEW (从 opendev-telemetry 引入)
    pub secrets: SecretsConfig,                // NEW (从 opendev-secrets 引入)
    pub sandbox: SandboxConfig,                // 重命名/扩展自 opendev-runtime::sandbox::SandboxConfig
}
```

**迁移:** 现有 `~/.opendev/settings.json` 不动,新增字段全部 `#[serde(default)]` 兼容。

### 7.5 与 constitution 14 条原则的关系

| 原则 | 影响 | 处理 |
|---|---|---|
| 1. Layered architecture | Protocol = Layer 1;Sandbox/Secret/Telemetry = Layer 2;App services = Layer 3 | ✓ 一致 |
| 2. Traits over concrete | ExecPolicy / SecretStore / Transport / SandboxBackend 全是 trait | ✓ |
| 3. Event sourcing | Protocol 事件 = canonical | ✓ 与现有 `SessionEvent` 共存 |
| 4. Async-first | Sandbox spawn / secret IO / OTLP 全 async | ✓ |
| 5. Single composition root | `src-tauri/main.rs` / `opendev-cli/main.rs` 持有 init 顺序 | ✓ 强制 |
| 6. Registry pattern | ToolRegistry / PolicyRegistry 复用 | ✓ |
| 7. Security by default | Sandbox + Keyring + Redact 直接落地 | ✓ **本方案核心** |
| 8. Surface ladder design | 不影响 (前端) | n/a |
| 9. Workspace monorepo | 2 新 crate (opendev-protocol, opendev-secrets) + 1 重命名 (opendev-telemetry) + 1 重写 (opendev-exec) | ✓ workspace dep 一致 |
| 10. Defensive error handling | Sandbox 降级 / Secret fallback / Telemetry 容错 | ✓ 强制 |
| 11. Testing at all levels | Sandbox 跨平台 CI / Secret mock backend / Protocol snapshot | ✓ |
| 12. Explicit over magic | Protocol 显式 version 字段 / Secret 显式 namespace | ✓ |
| 13. Minimal dependency footprint | 复用 0 编译成本的 transitives (keyring / zeroize) | ✓ |
| 14. Interface diversity, unified core | 4 件事统一通过 protocol 暴露,核心 1 个 | ✓ **本方案核心** |

→ 本方案与 constitution 14 条**全部对齐**,其中第 7 条 (Security) 和第 14 条 (Interface diversity) 是**直接落地**。

### 7.6 与现有 ADR 001-007 的关系

| ADR | 影响 | 处理 |
|---|---|---|
| 001 Rust 2024 | 无影响 | n/a |
| 002 Event-sourced sessions | Protocol 事件 + SessionEvent 共存;前者 wire,后者 persistence | v1 protocol 不重写 event-sourcing;v2 整合 |
| 003 Provider adapter pattern | Secret store 用于 `ProviderAdapter` 取 API key | 直接调用 |
| 004 Workspace monorepo | 2 新 + 1 重命名 + 1 重写,workspace 拓扑变 | workspace.dependencies 自动同步 |
| 005 BaseTool trait with sensible defaults | Sandbox 改造 BaseTool.execute 注入 pre_exec | v1 不改 trait,只改实现 |
| 006 Agent/subagent architecture | Sandbox 同样作用于 subagent 的 tool call | 一致 |
| 007 SQLite persistence | Secret store 不存 SQLite (走 keyring / file) | 不影响 |
| adr-0005 Desktop communication | **本方案是 adr-0005 的执行版本** | v1 protocol 是 adr-0005 v2 |

**新增 ADR (待评审后):**
- ADR-008: App-Server Protocol v1
- ADR-009: Multi-Platform Sandbox (opendev-exec)
- ADR-010: OS Keyring as Primary Secret Store (opendev-secrets)
- ADR-011: Telemetry Architecture (opendev-telemetry)

---

## 8. 总体 Sequencing & Phasing

### 8.1 推荐时序 (4 月跑完)

```
Week  1  2  3  4  5  6  7  8  9  10 11 12 13 14 15 16
      ├──── Protocol A ────┤
                          ├──── Sandbox A ────────────┤
                                                  ├──── Keyring A ─────┤
                                                              ├──── Telemetry A ─┤
```

但实际上,4 个工作**部分可并行**(各 team 1 人):

```
Team A  (1 人) Protocol 全程
Team B  (1 人) Sandbox 全程
Team C  (1 人) Keyring 全程
Team D  (1 人) Telemetry 全程
                      ↓
                  4 人 3-4 个月 = 12-16 人月
```

但通常 OpenDev 是 1-2 人团队,串行更现实:

```
Single dev (1 人):
Week  1-7   Protocol v1 (priority 1)
Week  8-16  Sandbox (priority 2)
Week  17-23 Keyring (priority 3)
Week  24-30 Telemetry (priority 4)
Total: 30 weeks = 7.5 months
```

**如果 2 人:**

```
Person 1: Protocol (1-7) → Sandbox (8-16) → Telemetry (17-23)
Person 2: Sandbox parallel (1-9) → Keyring (10-16) → Telemetry support (17-23)
                                                              ↓
                                                       Merge + polish
Total: 23 weeks = 5.7 months
```

### 8.2 每个阶段的 Definition of Done

**Protocol v1 (7 weeks):**

- [ ] `opendev-protocol` crate 上线
- [ ] 28 方法 + 18 事件类型 + `WireEnvelope` + `version.rs`
- [ ] ts-rs 出 TS,前端 `@opendev/protocol-types` 包能用
- [ ] 5 端 `Transport` trait 实现,至少 Tauri + TUI 跑通
- [ ] 旧路径 1-release 兼容 (`server.rs` 改名 shim)
- [ ] snapshot tests + cross-crate integration tests
- [ ] ADR-008 文档化

**Sandbox v1 (9 weeks):**

- [ ] `opendev-exec` crate 上线
- [ ] 4 个 backend (Landlock / Seatbelt / Bwrap / Windows),至少 Landlock + Seatbelt 真用上
- [ ] BashTool 改造,集成 ExecPolicy + backend
- [ ] P0 (5) + P1 (8) + P2 (4) = 17 个 exec 点全部走 exec 抽象
- [ ] env_filter 提到共享模块,所有 exec 点应用
- [ ] SSRF 提到共享模块,所有 fetch 工具应用
- [ ] `opendev-sandbox` 重写为 microVM feature-gate
- [ ] cross-platform CI (Linux + macOS + Windows runner)
- [ ] ADR-009 文档化

**Keyring v1 (7 weeks):**

- [ ] `opendev-secrets` crate 上线
- [ ] SecretStore trait + EnvStore + KeyringStore + FileStore (age)
- [ ] `AppConfig::get_api_key` 走 SecretStore
- [ ] Migration 工具 + 启动检测 + 提示
- [ ] `AuthProfileManager` 复活走 SecretStore
- [ ] `CredentialStore` 复活为 compat facade
- [ ] HMAC key 走 SecretStore
- [ ] 全 workspace `String` → `SecretString` 替换关键字段
- [ ] ADR-010 文档化

**Telemetry v1 (7 weeks):**

- [ ] `opendev-observability` → `opendev-telemetry` 重命名
- [ ] 修 5 个 write-only 字段全部 honor
- [ ] JSON 输出 + 14 天 retention
- [ ] Tauri 入口 + panic handler 集成
- [ ] Redact layer (从 secrets 提)
- [ ] `SessionDebugLogger` 改用 tracing layer,默认 off
- [ ] Metrics 15 个 counter/gauge/histogram
- [ ] OTLP exporter (feature-gated)
- [ ] W3C TraceContext 跨 ReAct loop
- [ ] Sentry 集成 (opt-in, DSN 配置)
- [ ] Privacy settings UI tab
- [ ] ADR-011 文档化

### 8.3 关键里程碑

| 时间点 | 事件 | 影响 |
|---|---|---|
| v0.1.10 | (本设计评审通过) | 设计冻结,开发启动 |
| v0.2.0 | Protocol v1 上线,`server.rs` 改名 shim | 旧 client 仍能用 |
| v0.2.5 | Sandbox BashTool + 4 后端 + Landlock 启用 | 真安全底座 |
| v0.2.8 | Keyring 上线 + migration 弹窗 | constitution 7 兑现 |
| v0.3.0 | Telemetry 上线 + Privacy UI + Sentry opt-in | 4 类底座完成 |
| v0.3.5 | 旧 shim + 旧路径全部移除 | 干净状态 |
| v0.4.0 | v2 protocol (Realtime voice / 高级 workspace / 额外 metrics) | 进入第二类功能 |

---

## 9. 关键决策 (需用户输入汇总)

按 4 件事汇总 (已在 §3.5, §4.5, §5.5, §6.5 详述,此处仅列最关键的 5 个):

| # | 决策 | 我的推荐 | 备选 | 影响范围 |
|---|---|---|---|---|
| **0-1** | **整体时序:4 件事并行还是串行?** | **串行 1 人 30 周** | 4 人并行 12 周 | 项目周期 |
| **0-2** | **是否本批次必做 Telemetry?** | **必做** (你已确认"如果以后有用户") | 仅协议 + 沙箱 | 用户出现时的盲飞风险 |
| **0-3** | **是否本批次必做 Keyring?** | **必做** (constitution 7 兑现) | 推到 v0.3 | 安全债延期 |
| **0-4** | **Protocol v1 是否包含 Telegram?** | **否** (v1 = Tauri + TUI + Web,Workspace 5 端;Telegram v2) | 是 | 4 周 |
| **0-5** | **是否启用 v0.2.0 GA 时强制 Keyring migration 弹窗?** | **是** (一次性,用户可推迟) | 仅警告,后台迁移 | constitution 兑现时间 |

**次要决策 (待 0-1 到 0-5 决定后细化):**

- wire 格式 JSON vs MessagePack (3-A)
- microVM (microsandbox) 是否保留 (4-A)
- policy engine Rust pattern vs Starlark (4-C)
- `keyring` crate 版本 (5-A)
- FileStore 加密算法 age vs AES-GCM (5-B)
- HMAC key 是否走 keyring (5-D)
- Sentry 是否本期必做 (6-A)
- Metrics crate 选型 (6-B)
- `SessionDebugLogger` 默认 false (6-C)
- Privacy UI tab 必做 (6-D)
- Perfetto 是否实现 (6-E)
- OTLP 默认开关 (6-F)
- 日志保留期 (6-G)

→ 我等你对 0-1 到 0-5 的决定后,再细化次要决策;或你给指示,全部按我的推荐走。

---

## 10. 风险与开放问题

### 10.1 主要风险

| # | 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|---|
| R1 | **Protocol v1 设计错了**,后期重写 | 中 | 高 (5 端都要改) | 走 v1/v2 双轨,允许 v2 增量 |
| R2 | **Landlock/Seatbelt 在某 OS 版本上炸** | 中 | 高 (沙箱失效) | 降级到 NoneBackend + UI 警告 |
| R3 | **Keyring 跨平台行为不一致** (e.g. headless Linux) | 高 | 中 (功能降级) | file backend 兜底 |
| R4 | **`SessionDebugLogger` 默认改 false 破坏用户调试流程** | 中 | 中 | 显式 changelog + 文档 |
| R5 | **Tauri 2 `Channel<T>` 集成坑** | 中 | 中 | v1 用 `onEvent`,Channel v2 |
| R6 | **migration 工具 bug 导致用户数据丢失** | 低 | 高 (用户信任崩) | 写前先备份 + dry-run 模式 |
| R7 | **5 端同步迁移某端卡住** | 中 | 中 | 每端独立可切回旧路径 |
| R8 | **OTLP exporter 性能开销** | 中 | 中 | 批处理 + 默认 off |
| R9 | **W3C TraceContext 增加 LLM request 体积** | 低 | 低 | 16+8 byte,可忽略 |
| R10 | **macOS Keychain 弹窗打断用户** | 高 | 中 | 写明文档,提前告知 |
| R11 | **Frontend 25+ handler 迁移不同步** | 中 | 中 (老 handler 收不到事件) | 6.4 节的"双轨期方案": server dual-emit (v0.2) → 前端逐步迁 → v0.3 停止 dual-emit |
| R12 | **Sandbox 跨平台 CI 成本** | 中 | 中 | 沙箱测试在 Linux runner 跑 90%, macOS 跑 9%, Windows 跑 1% (按 commit 频率分配);月预算 < 2000 CI 分钟 |

### 10.2 开放问题 (本次不解决,留 v2)

| # | 问题 | 备注 |
|---|---|---|
| Q1 | **Tauri 命令零认证** | 不在本批次;留 ADR-012 |
| Q2 | **WebSocket handler 零认证** | 同上 |
| Q3 | **OAuth 流程 (ChatGPT 等)** | Codex 风格 OAuth 是大工程,留 v2 |
| Q4 | **设备码登录** | 同上 |
| Q5 | **`opendev-runtime` "kitchen sink" 拆分** | P2/P3,与本方案正交 |
| Q6 | **Starlark DSL policy** | v1 不上,v2 评估 |
| Q7 | **Realtime voice / Image gen / Cloud tasks / Remote control** | **第二类,你已确认暂缓** |
| Q8 | **TS / Python SDK** | **第二类,暂缓** |
| Q9 | **Connectors / Apps** | **第二类,暂缓** |
| Q10 | **Mobile (iOS / Android)** | **第二类,暂缓** |

### 10.3 何时停止本批次,转入第二类

判断标准:

- [ ] v0.3.0 GA (4 件事全部上线)
- [ ] constitution 7/14 全部兑现
- [ ] 旧路径 (`server.rs` shim / v0.1.x key 格式) 全部移除
- [ ] 跨平台 CI 全绿
- [ ] 文档 + 教程 + ADR 全部完成

→ 此时可宣告 "底座完整",开始评估第二类 (Realtime voice / SDK / Cloud) 的 ROI。

---

## 11. 附录 — 引用文件

### 11.1 4 份 recon (本会话产出)

- `~/.local/share/opencode/tool-output/tool_f0bd5566e001P8AfU8uosCWK0j` — Protocol/IPC 调研
- `~/.local/share/opencode/tool-output/tool_f0bd4038a001J29QOiC38cvY4I` — Sandbox 调研
- `~/.local/share/opencode/tool-output/tool_f0bd7c7a6001EV9DxYP5mmGDSd` — Keyring/Credential 调研
- `~/.local/share/opencode/tool-output/tool_f0bdaa563001gNFvpads2j6Y7O` — Telemetry/Observability 调研

### 11.2 项目内引用

- `docs/architecture/comparison-codex-vs-opendev.md` (上一轮产出)
- `docs/architecture/adr-0005-desktop-native-communication-architecture.md` (本方案的 architecture 根)
- `docs/architecture/desktop-communication.md` (本方案的 implementation 根)
- `docs/architecture/data-flow.md`
- `docs/architecture/security-model.md`
- `docs/architecture/crate-layering.md`
- `docs/constitution.md` (14 条冻结原则)
- `docs/AUDIT_REPORT.md` (v0.1.8 audit, 504 行)
- `docs/REMEDIATION_PLAN.md` (90 天 plan, 702 行)
- `docs/roadmap.md` (v0.1.9 → v1.0)
- `docs/engineering/logging-observability.md`
- `docs/engineering/error-handling.md`
- `docs/engineering/coding-standards.md`
- `docs/engineering/testing.md`
- ADR 001-007

### 11.3 关键代码引用 (用于实施时回看)

**Protocol:**

- `crates/opendev-models/src/frontend_event.rs` (17 变体,做 v1 参考)
- `src-tauri/src/interface/desktop/events/mod.rs` (6 子枚举 17 变体)
- `src-tauri/src/interface/desktop/commands/` (34 commands,做 v1 方法参考)
- `src-tauri/src/server.rs` (临时桥接,待改名 shim)
- `crates/opendev-web/src/protocol.rs` (`WsMessageType` 40 变体,做 v1 命名参考)
- `src/types/index.ts` (32 字符串字面量 union)
- `src/repositories/Transport.ts` + `TauriTransport.ts` + `eventBridge.ts`
- `crates/opendev-web/src/state/mod.rs` (AppState 16 字段)
- `crates/opendev-web/src/websocket.rs` (handle_socket + handle_client_message)

**Sandbox:**

- `crates/opendev-sandbox/src/lib.rs` (Linux-only, 重写起点)
- `crates/opendev-runtime/src/sandbox.rs` (SandboxConfig 死代码,改名 policy)
- `crates/opendev-tools-impl/src/bash/mod.rs` + `foreground.rs` + `background.rs` + `helpers.rs` + `patterns.rs` (BashTool 全栈)
- `crates/opendev-runtime/src/permissions/mod.rs` (PermissionRuleSet)
- `crates/opendev-tools-impl/src/file_edit.rs` (per-file lock)
- `crates/opendev-tools-impl/src/path_utils.rs` (is_sensitive_file)
- `crates/opendev-tools-impl/src/web_fetch/mod.rs` (SSRF precedent)
- `crates/opendev-runtime/src/secrets.rs` (8 patterns detect+redact)
- `crates/opendev-tools-impl/src/schedule.rs` (无 runner,需 sandbox)
- `crates/opendev-mcp/src/transport/stdio.rs` (env 注入点)
- `crates/opendev-tools-lsp/src/handler.rs` (LSP server spawn)
- `crates/opendev-hooks/src/executor.rs` (shell subprocess 风险)

**Keyring:**

- `crates/opendev-http/src/auth.rs` (CredentialStore 死代码,复活起点)
- `crates/opendev-http/src/rotation.rs` (AuthProfileManager 死代码,复活起点)
- `crates/opendev-http/src/user_store.rs` (Argon2 user store,加 SecretString 包装)
- `crates/opendev-web/src/routes/auth.rs` (Argon2 + HMAC + cookie)
- `crates/opendev-models/src/config/mod.rs:323-374` (AppConfig::get_api_key_with_env)
- `crates/opendev-models/src/config/channels.rs` (Telegram token)
- `crates/opendev-models/src/user.rs` (User struct,加 SecretString 包装)
- `crates/opendev-channels/src/telegram/mod.rs:50-58` (Telegram token resolution)
- `crates/opendev-config/src/loader/` (settings.json 写入)
- `src-tauri/src/application/config_service.rs:71-77` (mask_api_key)
- `src/components/Settings/ModelSettings.tsx:48-52` (UI 输入)

**Telemetry:**

- `crates/opendev-observability/src/lib.rs` (重命名起点)
- `crates/opendev-observability/src/guard.rs` (OtelGuard → TelemetryGuard)
- `crates/opendev-observability/src/config.rs` (5 个 write-only 字段)
- `src-tauri/src/main.rs:35-126` (无 telemetry init, 修)
- `crates/opendev-cli/src/helpers.rs:8-28, 75-154` (init_tracing + install_panic_handler, 搬到 telemetry)
- `crates/opendev-runtime/src/debug_logger.rs` (SessionDebugLogger, 改 tracing layer)
- `crates/opendev-runtime/src/cost_tracker.rs` + `crates/opendev-history/src/cost/tracker.rs` (CostTracker,加 metric)
- `crates/opendev-agents/src/react_loop/execution.rs:49-51, 81` (W3C TraceContext 起点)
- `crates/opendev-memory/src/facade.rs:147` (content 泄漏, 修)
- `src/components/ErrorBoundary.tsx` (无 componentDidCatch, 修)
- `src/components/Settings/SettingsModal.tsx:25-50` (加 Privacy tab)
- `src/stores/chat.ts:171, 176, 534, ...` (50+ console.log, 集中化)

---

## 12. 评审清单 (reviewer checklist)

请评审者重点关注:

- [ ] §0 TL;DR 是否概括到位
- [ ] §1 哲学判断 (4 类 = 底座) 是否同意
- [ ] §2 总体路径 (4 件时序 + 依赖) 是否同意
- [ ] §3 Protocol 设计 (v1 vs v2 双轨、命名约定、5 端 Transport) 是否同意
- [ ] §4 Sandbox 设计 (Landlock/Seatbelt 优先级、policy engine 选型) 是否同意
- [ ] §5 Keyring 设计 (keyring 优先 + file 兜底、migration 时机) 是否同意
- [ ] §6 Telemetry 设计 (Sentry opt-in、JSON 输出、Privacy tab) 是否同意
- [ ] §7 跨切关注点 (启动顺序、降级路径) 是否同意
- [ ] §8 总体 sequencing (3-4 月 vs 7.5 月) 是否同意
- [ ] §9 关键决策 0-1 到 0-5 是否批准推荐方向
- [ ] §10 风险是否识别到位

— 完 —
