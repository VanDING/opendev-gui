# OpenDev Desktop vs OpenAI Codex — 全方位深度对比

> 编制日期: 2026-06-28
> 目标项目: 本仓库 `opendev-desktop` (workspace `0.1.9`, Rust `1.94` edition 2024)
> 对标项目: [`openai/codex`](https://github.com/openai/codex) (Rust CLI 当前 `rust-v0.142.3` / 预发布 `0.143.0-alpha.29`, TypeScript 遗留版已归档)
> 文档定位: 独立、逐层、逐功能、逐模块的对比报告,服务于架构选型、功能对标、改进路线设计。

---

## 0. TL;DR — 5 句话总结

| 维度 | OpenDev Desktop | OpenAI Codex |
|---|---|---|
| **产品形态** | 单仓库、Tauri 桌面优先 + 同核驱动 CLI/REPL/Web/TUI/Telegram | 单仓库 monorepo、CLI 优先 + 官方 Codex App (macOS/Win) + VSCode 扩展 + Web/Cloud + iOS/Android |
| **架构风格** | 严格 4 层 (React→Stores→Repositories→Transport ↔ Desktop↔Application↔Core↔Infrastructure) + 严格 Cargo workspace 分层,composition root 单点 | 二级分核: `core` + `app-server`(JSON-RPC) + `tui`/`exec`/`cli` 多面,经由 `protocol` 共享 wire types |
| **能力广度** | 80+ 用户功能,7 LLM provider,24 provider 路由,40+ 工具,LSP/符号/AST/MCP/Memory/Skills/Subagents/Channels/Plan mode 全套 | 75+ crate,~50+ 用户功能,6+ provider (OpenAI/Azure Responses/Bedrock/Anthropic-via-Bedrock/Ollama/LM Studio/custom),核心工具 (shell/apply_patch/web_search/view_image/update_plan) + MCP + Guardian + Skills + Plugins + Connectors + Realtime/voice + Codex Cloud |
| **核心差异** | "广度优先" — 多通道(Telegram)、LSP 集成、AST 检索、Subdir 指令系统、Memory + Skill 双轨、Plan mode 5 阶段、team/mailbox 多 agent 协作 | "深度优先" — App-server JSON-RPC 公共协议 (`thread/*`/`turn/*`/`fs/*`/`process/*`/`command/*`/`realtime/*` v1+v2 双版本,Sticky routing, 80+ 种 RPC 方法)、Guardian 自动化审批、Realtime WebRTC 语音、Realtime voice,Memories + Codex Cloud Tasks |
| **成熟度** | 工程评分 68-78/100,正在 v0.1.9 → v0.2 → v0.3 重构期 (P0 已修, P1 进行中) | 生产级(94.1k stars、887 个 releases、OpenAI 官方商业产品),通过 frozen `protocol` + app-server v1/v2 双轨制保证向后兼容 |

**一句话定位**:
- **OpenDev Desktop** = "Tauri 桌面优先 + 多通道广度型本地 agent",类似但更本地化、更广泛、面向多端/多 channel 的工程平台
- **OpenAI Codex** = "CLI 优先 + JSON-RPC 协议化、云端桌面全产品线、深度集成 OpenAI 商业生态的商用 agent"

---

## 1. 项目元数据对比

| 维度 | OpenDev Desktop | OpenAI Codex |
|---|---|---|
| 仓库地址 | 本地 `/Users/van/projects/opendev-desktop` | `https://github.com/openai/codex` |
| 主语言 | Rust (edition 2024) + TypeScript (前端) | Rust 96.5% + Python 2.7% + Starlark 0.2% (Bazel) + TypeScript 0.2% (遗留) |
| Rust 版本 | 1.94 | 未强制 (通常使用最新 stable) |
| Workspace 规模 | 23 个 lib crate + 1 个 bin crate (Tauri) | 约 75+ 个 crate (`codex-rs/Cargo.toml`) |
| 代码量 | 923 个 .rs 文件 / ~164,060 LoC + 121 个 .ts/.tsx / ~11,504 LoC | 多万行级 (核心 `core/` 95 文件 + `tui/` 70 文件 + 60+ 周边) |
| 许可证 | (未在 README 顶部声明,`deny.toml` 限定 MIT/Apache-2.0/BSD-2/BSD-3/0BSD/ISC/Unicode-3.0/MPL-2.0/CDLA-Permissive-2.0/BSL-1.0/Zlib/CC0-1.0) | Apache-2.0 |
| 当前版本 | workspace `0.1.9` / 各内部 crate `0.1.6` | `rust-v0.142.3` (稳定) / `rust-v0.143.0-alpha.29` (预发) |
| 文档完备度 | 高 (14 份架构/工程/治理/ADR/路线图/AUDIT/REMEDIATION 文档) | 高 (开发者站 + `AGENTS.md` 编程规约 + 每个子 crate 的 README) |
| 工程治理 | `docs/constitution.md` 14 条冻结原则 + 7 份 ADR | AGENTS.md 多份 (顶层/核心/tools/app-server/protocol) |
| 商业形态 | 开源无商业产品 | OpenAI 官方商业产品线:Codex CLI / App / Web / Cloud Tasks / Mobile / GitHub Action / IDE 插件 |

---

## 2. 顶层仓库结构对比

### 2.1 OpenDev Desktop 顶层布局

```
opendev-desktop/
├── .cargo/audit.toml
├── .github/workflows/ci.yml          # 5 个 CI job
├── ARCHITECTURE.md / CHANGELOG.md / DESIGN.md / README.md
├── Cargo.toml / Cargo.lock           # workspace + 965 个 transitive 依赖
├── crates/                            # 23 个库 crate
├── deny.toml / rustfmt.toml
├── docs/                              # 治理 + 架构 + 工程 + ADR
├── index.html / package.json / vite.config.ts / tsconfig*.json
├── public/                            # 静态资源
├── src/                               # React 19 前端
└── src-tauri/                         # Tauri 2 桌面壳
```

### 2.2 OpenAI Codex 顶层布局

```
codex/
├── .codex/                            # 项目自身的 AGENTS.md (dogfooding)
├── .devcontainer/
├── .github/                           # workflows + issue templates
├── .vscode/
├── bazel/ / MODULE.bazel / BUILD.bazel
├── codex-cli/                         # 遗留 TS CLI (现为 npm 包装壳)
├── codex-rs/                          # Rust workspace
├── docs/                              # 全部链接到 developers.openai.com/codex
├── patches/                           # 第三方补丁
├── scripts/                           # 顶层构建脚本
├── sdk/                               # TypeScript + Python SDK
│   ├── typescript/                    # @openai/codex-sdk
│   └── python/                        # Python Codex SDK
├── third_party/                       # vendored 代码
├── tools/                             # 工具脚本
├── justfile / flake.nix
└── pnpm-workspace.yaml / package.json
```

### 2.3 关键差异

| 维度 | OpenDev Desktop | OpenAI Codex |
|---|---|---|
| 多语言混合 | "Rust 主导 + 极小 TS 前端" | "Rust 主导 + 大量工具链 (Bazel/Just/Nix/Python/TS)" |
| 前端位置 | `src/` (与 Tauri 共享根目录) | **无内置 Web/桌面前端** (前端为独立产品: Codex App、Codex Web、Codex Mobile) |
| 桌面壳 | Tauri 2 + 自写 React 19 (chakra 替代品: 自研组件 + Radix + lucide + shiki + xyflow) | 不在本仓库;由 `cli/src/desktop_app/` 触发;另外 `cli/src/app_cmd.rs` 启 Codex App |
| 协议定义 | `opendev-models` (单一 crate, `ts-rs` 导出 TS) | `protocol/` (wire types) + `app-server-protocol/` (RPC 协议, v1 + v2) |
| 多端集成 | 通过共享 `MainAgent` 单 core 驱动 4 个 UI | 通过共享 `core` + `app-server` 驱动 N 个独立产品 |

---

## 3. Crate 对 Crate 映射

OpenDev Desktop 23 个 crate 映射到 Codex 的对应 crate (按层次分组):

### 3.1 域模型 / 数据层

| OpenDev | 职责 | Codex 对应 |
|---|---|---|
| `opendev-models` (3,285 LoC) | 域类型,零内依赖,`ts-rs` 自动生成 TS,包含 `AppConfig`/`Session`/`ChatMessage`/`ToolCall`(递归)/`FrontendEvent`(23 变体)/`Operation`/`ValidateTransition` | `protocol/` (~30 文件,1k+ LoC),极简依赖,`ts-rs 11` 导出 TS,包含 `Op`/`EventMsg`(50+ 变体)/`SessionId`/`ThreadId`/`AgentPath`/`SandboxPolicy`/`AskForApproval`/`MemoryCitation` |
| — | (无对应) | `core-api/` — `core` 内部 API surface |
| — | (无对应) | `app-server-protocol/` — v1 frozen + v2 active JSON-RPC 协议 |
| — | (无对应) | `app-server-test-client/` + `mcp_test_support` + `core_test_support` |

**对比**:
- OpenDev 把所有 wire types + domain models 收在一个 crate (`opendev-models`);Codex 拆成 3 层 (`protocol` 内 wire types + `core-api` 内部 API + `app-server-protocol` 外部 RPC),概念边界更清晰
- OpenDev 的 `FrontendEvent` 是 TUI/Web/Desktop 三端共享的事件协议;Codex 用 `EventMsg` 作为 TUI/app-server 共享事件,再用 `app-server-protocol/v2` 暴露给外部
- Codex 的 `Op`/`EventMsg` 变体更多(50+),涵盖 Realtime (RealtimeConversationStart/Audio/Text/Speech/Close/ListVoices)、Guardian (GuardianAssessment)、Hook 等 OpenDev 没有的事件类型

### 3.2 配置 / 路径 / 注册表

| OpenDev | 职责 | Codex 对应 |
|---|---|---|
| `opendev-config` (4,048 LoC) | 分层 config 加载、path 管理 (XDG + legacy `~/.opendev/`)、models.dev 同步、迁移、profile、watcher | `config/` (在 `codex-rs/core/src/config/`) + `codex-home/` (`$CODEX_HOME` 解析) + `models-manager/` (目录获取) + `secrets/` (auth.json + keyring) + `keyring-store/` (OS keyring) |
| — | (无对应) | `network-proxy/` — 系统 / PAC / WPAD 代理解析 |
| — | (无对应) | `feature`/`features`/`codex-features` — 特性开关 |
| — | (无对应) | `external-agent-migration` — 从 Claude Code 等导入配置 |
| — | (无对应) | `collaboration-mode-templates` — 协作模式 preset JSON |

**对比**:
- OpenDev 把配置、路径、模型注册表、迁移都收在一个 `opendev-config`;Codex 拆得更细
- OpenDev 使用 `directories 6` XDG;Codex 用自有 `home-dir` + `$CODEX_HOME` env override
- OpenDev 使用 `~/.opendev/` legacy 路径;Codex 用 `~/.codex/` (单一稳定路径)
- OpenDev 用 `deny.toml` (cargo-deny 0.19) 治理依赖;Codex 没有显式 deny 配置(依赖 OpenAI 内部审核)
- Codex 有专门的 OS keyring 集成 (`keyring-store`),使用 `keyring` crate;OpenDev 的 `CredentialStore` 实际未使用 system credential store (TODO: 改用 `zeroize`)

### 3.3 HTTP / Provider / Auth / Streaming

| OpenDev | 职责 | Codex 对应 |
|---|---|---|
| `opendev-http` (9,113 LoC) | 7 provider 适配器 (OpenAI/Anthropic/Gemini/Bedrock/Groq/Mistral/Ollama/Azure) + reqwest 客户端 + auth + circuit breaker + streaming + rotation | `model-provider/` + `model-provider-info/` + `codex-api/` + `codex-client/` + `aws-auth/` + `ollama/` + `lmstudio/` |
| — | (无) | `responses-api-proxy/` — Node 实现的 Responses API 代理 |
| — | (无) | `backend-client/` + `codex-backend-openapi-models/` — 后端 OpenAPI 客户端 |

**对比**:
- OpenDev 使用 Chat Completions API 适配 (OpenAI 路径);Codex 偏向 Responses API (`responses_*`),`codex-api` 内部直接支持 Responses + Compact + Memories + Realtime
- OpenDev 有 `CircuitBreaker` (Closed/Open/HalfOpen) + `AuthProfileManager` (429/401/403/5xx 自动 cooldown 轮转);Codex 没有显式熔断, 但有 `responses_retry` + sticky routing `x-codex-turn-state`
- OpenDev 有 `AdaptedClient::with_curl_transport()`(对 DashScope 这种拒绝 reqwest 的端点的兜底);Codex 没有
- OpenDev Bedrock 用手写 `aws4-hmac-sha256` + `X-Amz-Security-Token` + `X-Bedrock-SigV4-Error` 诊断 fallback (v0.1.9 新增);Codex 用官方 `aws-config` + `aws-smithy-runtime` + `aws-sigv4`
- Codex 支持的 provider:OpenAI、Azure Responses、Bedrock、Anthropic-via-Bedrock、Ollama、LM Studio、custom;OpenDev 多:OpenAI、Anthropic、Gemini、**Bedrock**、Groq、Mistral、Ollama、Azure,以及通过 models.dev 解析的 20+ provider

### 3.4 上下文工程 / 压缩 / 检索

| OpenDev | 职责 | Codex 对应 |
|---|---|---|
| `opendev-context` (7,712 LoC) | 压缩、验证、context picker、env/memory、检索、subdir 指令、validated message list、worktree | `core/src/context/` + `core/src/context_manager/` + `compact.rs` + `compact_remote.rs` + `compact_remote_v2.rs` + `compact_token_budget.rs` + `context-fragments/` (ContextualUserFragment trait) + `agents_md.rs` + `agents_md_manager.rs` + `session_prefix.rs` |
| — | (无) | `realtime_context.rs` + `realtime_conversation.rs` + `realtime_prompt.rs` — Realtime 专用 context |

**对比**:
- OpenDev 的 `ContextCompactor` 有 5 档阈值 (70/80/85/90/99%);Codex 的 `compact_token_budget` 是 token 预算驱动
- OpenDev 强调 "ValidatedMessageList"(写时强制消息对不变量);Codex 用 `<context_window>` + `ContextualUserFragment` 模式
- OpenDev 8 并行 `std::thread::scope` git/dir 扫描;Codex 用 `agents_md_manager` + `git-utils` (gix-based)
- OpenDev 有 `SubdirInstructionTracker` 显式跟踪多层指令;Codex 也有 `agents_md` 但实现更轻
- Codex 多出 Realtime 子系统 (WebRTC 音频上下文);OpenDev 无
- OpenDev 强制 "上下文增量构建、避免频繁变更引起 cache miss"(原则 3, constitution.md);Codex 的 `responses_request_properties_match` 函数和"no history rewrite"硬性规约是同源思路

### 3.5 持久化 / 历史

| OpenDev | 职责 | Codex 对应 |
|---|---|---|
| `opendev-history` (9,219 LoC) | Session 持久化 (JSON+JSONL+SQLite)、`SessionEvent`(10 变体 + Tombstone)、`EventEnvelope`、`Projector`、`SnapshotManager`(shadow git)、`FileCheckpointManager`、`FileLock`、`CostTracker`、`SqliteSessionStore` (WAL) | `rollout/` (JSONL 持久化) + `rollout-trace/` (追踪上下文) + `state/` (SQLite via sqlx) + `thread-store/` + `message-history/` + `memories/{read,write}/` (long-term memory) |
| `opendev-memory` (2,091 LoC) | 短+长记忆,SQLite FTS5,decay scoring,WriteGate,`CascadeBuffer`,`ShortTermMemory`,`MemoryDecay`,`auto_nudge`,`MEMORY.md` | `ext/memories/` + `memories/{read,write}/` + `core/src/memory_citation.rs` (`MemoryCitation`) |

**对比**:
- OpenDev 的 event-sourcing 10 个 `SessionEvent` 变体 + Tombstone + `undo_to_seq` 是更"严肃"的实现;Codex 用 JSONL rollout + `state` SQLite,模式更轻
- OpenDev SQLite 由 `opendev-history` 和 `opendev-memory` 各起一个 (技术债, 计划合并到 `opendev-storage`);Codex 用单一 `state` crate 集中管理
- OpenDev `SqliteSessionStore::block_on` 是用 `OnceLock<Runtime>` 桥接 sync→async 的 hack (标记为技术债);Codex 统一 async + `sqlx 0.9` runtime-tokio
- Codex 有专门的 `rollout-trace`(inference-trace + compaction-trace);OpenDev 无对应

### 3.6 工具层

| OpenDev | 职责 | Codex 对应 |
|---|---|---|
| `opendev-tools-core` (5,528 LoC) | `BaseTool` trait (20+ 方法, 18 默认)、`ToolRegistry`(中间件、JSON Schema 校验、超时、同 turn dedup、alias、core 标记 + 延迟加载)、`ToolMiddleware`、`ToolPolicy`、`ToolResultSanitizer`(Head/Tail/HeadTail)、`Normalizer` | `tools/` (提取的 host-side 工具机制) + `core/src/tools/`(编排) + `tools/src/{tool_spec,tool_config,tool_definition,tool_discovery,tool_executor,tool_search,tool_call,tool_output,tool_payload,dynamic_tool,code_mode,mcp_tool,request_plugin_install,response_history,responses_api,image_detail,json_schema,function_call_error}.rs` |
| `opendev-tools-impl` (24,326 LoC) | 40+ 工具:Bash (双超时 + read-only 启发 + dangerous patterns)、File Edit (9-pass fuzzy + per-file lock)、File Write (atomic)、File Read (binary detect + suggestions)、Grep/AstGrep、File List (depth 3)、WebFetch (SSRF 保护 + HTML→markdown)、WebSearch (DuckDuckGo)、WebScreenshot、Browser、OpenBrowser、LspQuery、McpBridge、MemoryTool、NotebookEdit (Jupyter)、MultiEdit、PatchTool、DiffPreview、InsertBefore/AfterSymbol、WorktreeManager、InvokeSkill、RunWorkflow、PresentPlan、Message、TaskComplete、Todo (6 个变体)、ScheduleTool、CustomTool、Session/PastSessions、VLM、AskUser、Agents/SpawnSubagent/SpawnTeammate、CheckMailbox、ToolSearch、CreateTeam/DeleteTeam/SendMessage/TeamAddTask/ClaimTask/CompleteTask/ListTasks、ChannelProgressCallback | 核心工具 (在 `core/src/tools/`):shell (含 `unified_exec/` 交互式 PTY 模式)、`apply_patch`、`view_image`、`request_user_input`、`request_permissions`、`web_search` (via `ext/web-search`)、`image_generation` (via `ext/image-generation`)、`update_plan` (`plan_tool.rs`)、`list_dir`/`read_file`/`write_file`/`grep_files`、MCP tools (via `mcp_tool_call.rs`)、Dynamic tools (`dynamic_tools.rs`)、Code-mode tools (`code_mode.rs`) |
| `opendev-tools-lsp` (2,800 LoC) | LSP server 管理、JSON-RPC、`SymbolCache`、`LspWrapper`、`WorkspaceSymbolTool`、`DiagnosticsDebouncer` | (无独立 crate;LSP 不在 Codex 范围内) |
| `opendev-tools-symbol` (1,132 LoC) | find_symbol / find_references / rename / replace_body via LSP | (无对应) |
| `opendev-patch` (在 `opendev-tools-impl/src/patch/`) | `PatchTool` (unified-diff application) | `apply-patch/` 独立 crate,自包含 parser + applier + streaming parser,作为独立子二进制 (`--codex-run-as-apply-patch` arg0 trick) |

**对比**:
- **Codex 的 `apply_patch` 是独立 crate**;OpenDev 的 `PatchTool` 在 `opendev-tools-impl` 内
- OpenDev 工具集 **远比 Codex 丰富** (40+ vs ~10 个核心工具),特别是:
  - **LSP 集成** (OpenDev 独有)
  - **符号级编辑** (find/references/rename/replace_body) (OpenDev 独有)
  - **AST-grep 工具** (OpenDev 独有)
  - **Web 截图/系统浏览器**(OpenDev 独有)
  - **多 agent 协作工具** (SpawnSubagent、CheckMailbox、CreateTeam、TeamAddTask 等) (OpenDev 独有,Codex 用 `codex_delegate.rs` + `core/src/agent/`)
  - **Todo 工具的 6 个变体** (write/update/complete/list/clear + 统一 `TodoTool`)(OpenDev 独有)
  - **Memory 工具** (OpenDev `MemoryTool`)(Codex 的 `memories/{read,write}` 是子 crate,不是 tool)
  - **Session 搜索** (OpenDev `PastSessionsTool`)(Codex 用 `app-server` `thread/list` 替代)
  - **VLM 工具**(OpenDev 独有)
  - **Schedule 工具**(OpenDev 独有)
  - **Custom 工具** (用户定义) (OpenDev 独有)
- Codex 独有:
  - **`view_image` 工具** (本地图片 → 模型上下文)
  - **`image_generation` 工具** (生成图,via `ext/image-generation`)
  - **`update_plan` 工具** (Codex 的 plan 模式是 tool 形式)
  - **`request_permissions` 工具** (请求额外权限)
  - **Dynamic tools** (host 注入)
  - **Code-mode** (`code_mode.rs` + `code-mode/` + `code-mode-host/` + `code-mode-protocol/` + `v8-poc/`) — V8 沙箱执行 JS 编排工具
  - **`unified_exec`** — 交互式 PTY 工具
  - **`apply_patch` 作为独立二进制 + streaming parser** (OpenDev 缺失)

### 3.7 Agent 运行时

| OpenDev | 职责 | Codex 对应 |
|---|---|---|
| `opendev-agents` (20,439 LoC) | `MainAgent` (composition-based)、`ReactLoop` (ReAct 迭代: reason→tool→observe)、`LlmCaller`、`LlmCallConfig`、`PromptComposer` (3 cache policies: Static/Cached/Uncached)、`SubagentManager` (6 内置 + 自定义)、`DoomLoopDetector` (3-cycle window, Nudge→StepBack→CompactContext)、`memory_consolidation`、`skills` (builtin/curator/discovery/loader/metadata/parsing/visibility)、`agent_types`、`attachments`、`response_cleaner`、`workflow_spawner`、`traits` (`BaseAgent` async trait + 10 个 event callback) | `core/` (95 文件,核心域) + `core/src/codex.rs` (`Codex` orchestrator) + `core/src/codex_thread.rs` (`CodexThread` per-thread facade) + `core/src/codex_delegate.rs` (multi-agent) + `core/src/agent/` (agent runtime) + `core/src/session/` + `core/src/state/` + `core/src/tasks/` + `core/src/compact.rs` + `core/src/guardian/` (Guardian) + `core/src/plugins/` + `core/src/skills/` |
| — | (无) | `codex-delegate` 等同于 OpenDev `SubagentManager` 部分职责 |

**对比**:
- OpenDev 显式 `MainAgent` + `ReactLoop` ReAct 循环;Codex 核心是 `Codex` orchestrator + `CodexThread` per-thread facade,内部使用 `Op`/`EventMsg` SQ/EQ pattern(Submission Queue / Event Queue)
- Codex 的 **`Codex::Op` 变体(~25 个)** + **`EventMsg` 变体(50+ 个)** 是更抽象的"协议驱动"模式,所有交互都通过 `Op::UserInput` 提交,`EventMsg` 流式返回;OpenDev 的 `FrontendEvent` (23 变体) + `AgentEventCallback` (10 event) 是同思路但粒度更细
- OpenDev 的 `DoomLoopDetector` (3-cycle window, 3-rep threshold) 是独立模块;Codex 没有显式 doom-loop 检测模块
- OpenDev 的 `PromptComposer` 有 3 档 cache policy;Codex 通过 `responses_request_properties_match` + WebSocket 增量复用实现 cache 友好
- OpenDev 的 `SubagentManager` 内置 6 个 agent:code_explorer / planner / general / build / verification / project_init;Codex 通过 `codex_delegate` 支持 multi-agent,但**没有内建角色**,靠用户/插件定义
- OpenDev 的 `skills` 子系统 (builtin/curator/discovery/loader/metadata/parsing/visibility) 7 个子模块;Codex 有 `core/src/skills/` + `skills/` + `ext/skills/` + `core-skills/` 多 crate 拆分,功能相似但架构更分散
- Codex 有 Guardian (`core/src/guardian/` + `ext/guardian/` + `protocol::GuardianAssessment`) — 自动化审批审核器,带风险等级分类;**OpenDev 无对应**
- Codex 的 `agent/` 子目录有 `agent-graph-store`(directed graph store of agent relationships) — OpenDev 无

### 3.8 运行时服务 (审批/权限/事件总线/快照/计划/Worktree/Mailbox/团队/任务)

| OpenDev | 职责 | Codex 对应 |
|---|---|---|
| `opendev-runtime` (14,466 LoC, "kitchen sink") | 34 个子模块:Approval、Permissions、Secrets (8 patterns detect+redact)、EventBus、SnapshotManager、TaskManager、Todos、Worktree、Mailbox、PlanApproval、PlanIndex、CustomCommands、TeamManager、TeamTaskList、Schedule、StateSnapshot、FileWatcher、Sandbox stubs、InterruptToken、ActionSummarizer、ToolSummarizer、DebugLogger、LazyInit、FSM、PlanNames、Constants、Errors、AskUserChannel、ToolApprovalChannel、CostTracker、SessionModel | 部分分散在 `core/` + `protocol/` + `app-server/` + `runtime/`(无独立 "kitchen-sink" crate);`core/src/safety.rs` + `core/src/guardian/` + `protocol/src/approvals.rs` + `core/src/permissions.rs`(无独立 Permissions crate) + `core/src/event_bus` (无显式 event bus) |
| `opendev-hooks` (1,568 LoC) | 生命周期 hook(PreToolUse, PostToolUse, SessionStart, ...)以 shell 子进程方式执行,exit code 协议 (0=ok, 2=block),JSON 输出解析 | `hooks/` + `tui/src/hooks_rpc.rs` + `tui/src/startup_hooks_review.rs` + `app-server/README.md` (`hooks/list`) |
| `opendev-plugins` (1,798 LoC) | 插件管理 + marketplace catalog (HTTP 拉取) | `plugin/` + `core-plugins/` + `cli/src/plugin_cmd.rs` + `cli/src/marketplace_cmd.rs` + `ext/{extension-api,connectors}/` + `ext/connectors/`(ChatGPT connectors) |

**对比**:
- OpenDev 的 `opendev-runtime` 是 "kitchen sink" 90 文件/14466 LoC,被审计报告标记为"需拆分" (P2/P3 技术债);Codex 没有类似的"瑞士军刀"crate,关注点分散在多个边界清晰的 crate 中
- OpenDev 的 `EventBus` + `RuntimeEvent`(pub/sub with topics);Codex 没有独立 event bus — 用 `EventMsg` 协议流代替
- OpenDev 的 `TeamManager` + `TeamTaskList` + `Mailbox` 是显式多 agent 协作子系统;Codex 用 `codex_delegate` 隐式多 agent
- OpenDev 的 `PlanApproval`(5 阶段工作流);Codex 用 `update_plan` tool + `PlanApproval` 协议
- OpenDev 的 `InterruptToken`(`Arc<InterruptInner>` = `AtomicBool` + `tokio::CancellationToken` + `background` soft yield flag) 设计精致;Codex 通过 `Op::Interrupt` + 协议级 cancellation
- OpenDev 的 `secrets::detect_secrets`(8 patterns:Anthropic/OpenAI/Groq/Google/GitHub API keys + Bearer + password + base64) + `redact_secrets`;Codex 用 `secrets/` + `keyring-store` (OS keyring) 但 redact pattern 较少
- Codex 的 `Guardian` 比 OpenDev 的 `ApprovalRulesManager` 复杂得多:有 `GuardianRiskLevel`、`GuardianAssessmentOutcome`、`GuardianUserAuthorization`、单次否认豁免(`Op::ApproveGuardianDeniedAction`)

### 3.9 MCP (Model Context Protocol)

| OpenDev | 职责 | Codex 对应 |
|---|---|---|
| `opendev-mcp` (4,633 LoC) | 客户端:`McpTransport` trait + 3 transports (Stdio/HTTP/SSE) + `McpManager` + OAuth + `McpOAuthConfig` + JSON-RPC 协议类型 | `mcp-server/` (作为 MCP server 暴露) + `rmcp-client/` (基于官方 `rmcp 1.8` SDK) + `codex-mcp/` (内联适配器,暴露 Codex 为 tool) + `ext/mcp/` + `core/src/{mcp,mcp_tool_call,mcp_openai_file,mcp_skill_dependencies,mcp_tool_approval_templates,mcp_tool_exposure}.rs` |

**对比**:
- OpenDev 是纯**客户端**;Codex 同时是 **MCP server** + MCP 客户端 + Codex-as-MCP-tool (3 个不同角色)
- Codex 用官方 `rmcp 1.8` SDK;OpenDev 手写 JSON-RPC
- Codex 的 MCP 集成深入到: skill 依赖 (`mcp_skill_dependencies`)、approval templates (`mcp_tool_approval_templates`)、tool exposure (`mcp_tool_exposure`)、`mcp_openai_file`(OpenAI 文件上传);OpenDev 仅暴露 `McpBridgeTool`
- Codex 在 `0.142.2` 起默认开启 **MCP tool search**(延迟暴露 MCP tool 列表);OpenDev 通过 `ToolSearch` 实现类似机制

### 3.10 多通道 / Web

| OpenDev | 职责 | Codex 对应 |
|---|---|---|
| `opendev-channels` (3,363 LoC) | 多通道路由器 + Telegram bot (polling + remote_claim + pair/unpair) + `ChannelAdapter` trait | (无;Telegram/Discord/Slack 不在 Codex 范围) |
| `opendev-web` (5,512 LoC) | Axum 0.8 + WebSocket + Argon2 + HMAC-SHA256 + 11 route 模块 (`auth`/`chat`/`commands`/`config`/`mcp`/`sessions`/`skills`) | (无;Web 入口走 `app-server` 的 stdio/websocket/uds 协议,不暴露 HTTP) |
| `opendev-repl` (4,059 LoC) | 交互式 REPL loop + `QueryProcessor` + `QueryEnhancer` + `ToolExecutor` + `HandlerRegistry` | (无;Codex 的交互入口是 TUI) |
| `opendev-cli` (7,674 LoC) | CLI binary `opendev` (clap 4, 模式: REPL / 非交互 `-p` / replay `--replay JSONL` / remote (Telegram)) | `cli/` (顶层 `codex` 命令分发) + `exec/` (非交互 `codex exec`) + `tui/` (交互 TUI) |

**对比**:
- OpenDev 把 CLI / REPL / Web / TUI / Tauri 当作**4 个并列 UI** 共享一个 `MainAgent`;Codex 把 CLI / exec / TUI / app-server / mcp-server / login / sandbox 作为**6+ 个并列子命令**共享一个 `core`
- OpenDev 的 Web 模式用 HTTP + WebSocket + Argon2 鉴权(可被外部访问);Codex 没有 Web UI 入口,所有外部集成走 `app-server` JSON-RPC
- OpenDev 的 Telegram bot 是 first-class;Codex 无对应
- Codex 独有 `doctor` (诊断) + `marketplace` (marketplace 管理) + `plugin` + `remote-control`(远程控制) + `state-db-recovery` + `debug-sandbox`

### 3.11 TUI / 桌面

| OpenDev | 职责 | Codex 对应 |
|---|---|---|
| `opendev-tui` (26,735 LoC) | Ratatui 0.30 + Crossterm 0.29,153 文件,22 widgets + 25 controllers + 15 managers + 9 themes + custom commands + slash commands + Markdown 渲染 + spinner + subagent display + todo panel + tool display | `tui/` (70 文件, ~15-25k LoC) — Ratatui 0.29 (patched) + Crossterm 0.28 (patched) + `pulldown-cmark 0.10` + `syntect 5` + `portable-pty 0.9` + `arboard 3` |
| `src/` (前端, 11,504 LoC TS/TSX) + `src-tauri/` (2,995 LoC Rust) | React 19 + Zustand 5 + react-router-dom 7 + shiki 4 + sonner 2 + @xyflow/react 12 + Radix Dialog + Vite 8 + Tailwind v4;Tauri 2 (custom-protocol),34 个 `#[tauri::command]`,`DesktopPlatform` trait,9 themes,Hexagonal 架构 | (桌面端 Codex App 不在仓库内,由 `cli/src/desktop_app/` 触发;VSCode 扩展在 marketplace 单独发布) |

**对比**:
- OpenDev 有**两套 UI**:TUI(26,735 LoC)+ 桌面 React/Tauri(11,504 + 2,995 LoC);Codex 只有 TUI 完整在仓库内
- OpenDev TUI 的 22 widgets/25 controllers/15 managers 比 Codex TUI 的 ~70 个 .rs 文件粒度更细
- Codex TUI 的 markdown 渲染(`markdown.rs` + `markdown_render.rs` + `markdown_stream.rs` + `markdown_text_merge.rs` + `markdown_render_tests.rs`)是工业级的流式 markdown 合并,带 snapshot 测试;OpenDev 用 `markdown-it 14` JS 库在前端渲染
- Codex TUI 有 pets (`tui/src/pets/`)、shimmer、ascii_animation 装饰;OpenDev 有 `MatrixRain.tsx`(前端)
- Codex TUI 的 `selection_list`、`pager_overlay`、`resume_picker`、`onboarding/` 是用户体验精修;OpenDev 同样有但实现不同
- Codex TUI 集成 `app_server_session`(把 TUI 接到 app-server 而不是直接 core);OpenDev TUI 直接连 core(通过共享 `MainAgent`)

### 3.12 沙箱

| OpenDev | 职责 | Codex 对应 |
|---|---|---|
| `opendev-sandbox` (1,036 LoC) | **100% stubs**,Linux-only (`#![cfg(target_os = "linux")]`),microVM microsandbox wrapper 计划中 | `sandboxing/` (统一跨平台,200+ 文件中的 8 个) — `seatbelt.rs` (macOS `sandbox-exec`) + `landlock.rs` (Linux) + `bwrap.rs` (Linux,首选) + `windows.rs` (Win) + `policy_transforms.rs` + `manager.rs` + 嵌入的 `.sbpl` 模板 |
| — | (无) | `linux-sandbox/` — 独立 `codex-linux-sandbox` 二进制,arg0 重新 exec 触发 Landlock |
| — | (无) | `bwrap/` — vendored bubblewrap 启动器 |
| — | (无) | `execpolicy/` + `execpolicy-legacy/` — Starlark 写的可执行策略 |
| — | (无) | `process-hardening/` — seccomp-style 进程硬化 |
| — | (无) | `exec-server/` + `exec-server-protocol/` — 本地 exec/IPC server(给 apply-patch 子系统 + Windows exec) |
| `opendev-runtime/src/sandbox` (内部) | 简单 stub 包装 | — |

**对比**:
- OpenDev **沙箱是 100% stubs**(audit 报告 P3);Codex 沙箱是**生产级跨平台**:macOS Seatbelt + Linux Landlock/bwrap + Windows elevated/unelevated 双后端
- Codex 用 **Starlark DSL** 编写可执行策略(`execpolicy/`);OpenDev 用 `ApprovalRulesManager` 模式匹配
- Codex 沙箱支持 **split filesystem policy**(read-only carveouts + denied sub-paths);OpenDev 无
- Codex 通过 arg0 trick 复用同一个二进制;OpenDev 无此模式

### 3.13 观测 / 记忆 / 工作流 / Observability

| OpenDev | 职责 | Codex 对应 |
|---|---|---|
| `opendev-observability` (173 LoC) | `OtelGuard` + daily-rotated file appender + Perfetto writer | `otel/` + `core/src/otel_init.rs` + `rollout-trace/` + `analytics/` + `app-server/src/app_server_tracing.rs` + `sentry 0.46.0` |
| `opendev-workflow` (352 LoC) | workflow engine: pipeline / barrier / loop patterns | (无独立 crate;`core/src/tasks/` + `core/src/agent/` 涵盖) |
| `opendev-memory` (2,091 LoC) | 短+长记忆,SQLite FTS5,decay,WriteGate | `ext/memories/` + `memories/{read,write}/` + `core/src/memory_citation.rs` (`MemoryCitation`) |
| — | (无) | `core/src/apps.rs` + `app-server/README.md` (`app/list`) — ChatGPT Apps 集成 |
| — | (无) | `realtime-webrtc/` + `protocol::RealtimeEvent` + `RealtimeVoice` enum — Realtime 语音 |
| — | (无) | `code-mode/` + `code-mode-host/` + `code-mode-protocol/` + `v8-poc/` — V8 JS 沙箱执行 |
| — | (无) | `connectors/` + `ext/connectors/` — ChatGPT Connectors |
| — | (无) | `ext/remote_control` — 远程控制(手机/Chrome 驱动 Codex) |
| — | (无) | `attestation` (TEE) — `core/src/attestation.rs` + `app-server/src/attestation.rs` |
| — | (无) | `agent-graph-store` — 有向图 agent 关系 |
| — | (无) | `codex-experimental-api-macros` — proc-macro 标记实验性 API |
| — | (无) | `external-agent-sessions` — 外部会话接入 |

**对比**:
- Codex 的 `sentry 0.46.0` 错误上报;OpenDev 无
- Codex 的 `analytics` (合规 / telemetry);OpenDev 无
- Codex 的 **Realtime WebRTC** 语音通道(`realtime-webrtc/` + `RealtimeVoice` 枚举: Alloy/Arbor/.../Verse);OpenDev 无
- Codex 的 **Code-mode** (V8 JS 沙箱 + `code-mode-protocol`) 允许模型写 JS 编排工具;OpenDev 无
- Codex 的 **Connectors**(ChatGPT connectors);OpenDev 无
- Codex 的 **attestation** (TEE 证明);OpenDev 无
- Codex 的 **remote control**(手机/Chrome 驱动 Codex);OpenDev 无
- Codex 的 **AGENTS.md** 显式管理 + `agents_md_manager`;OpenDev 的 `SubdirInstructionTracker` 是更通用的多源方案 (AGENTS.md/CLAUDE.md/.opendev/instructions.md + frontmatter + include)

---

## 4. 核心抽象对比

### 4.1 Agent 核心 (Codex::Codex vs opendev::MainAgent)

| 维度 | OpenDev `MainAgent` | Codex `Codex`/`CodexThread` |
|---|---|---|
| 位置 | `opendev-agents/src/main_agent.rs` | `codex-rs/core/src/codex.rs` + `codex_thread.rs` |
| 模式 | 组合式:`MainAgent::new(config, deps)` + 显式 `ReactLoop` ReAct 迭代 | 协议式:`Op` 提交队列 + `EventMsg` 事件队列 (SQ/EQ pattern) |
| 通信抽象 | `BaseAgent` async trait + `AgentEventCallback` (10 events) | `Op` (25+ variants) + `EventMsg` (50+ variants) 流 |
| 编排单元 | 单 `ReactLoop` 反复迭代 reason→tool→observe | 单 `Codex` orchestrator 持续消费 `Op`,内部 `codex_thread` per-thread facade |
| 多 agent | `SubagentManager` 显式管理 6 内置 + 自定义,每个独立 `ReactLoop` | `codex_delegate.rs` + `core/src/agent/` + `agent-graph-store` |

### 4.2 工具系统 (BaseTool vs ToolSpec + ConfiguredToolSpec)

| 维度 | OpenDev `BaseTool` | Codex `ToolSpec` |
|---|---|---|
| 位置 | `opendev-tools-core/src/traits.rs:66+` | `tools/src/tool_spec.rs` + `core/src/tools/` |
| 接口粒度 | async trait, 20+ 方法 18 默认 | `ToolSpec` (静态) + `ConfiguredToolSpec` (运行时配置) + `LoadableToolSpec` (可加载) + `ResponsesApiNamespace` (per-namespace 打包) |
| 注册 | `ToolRegistry` 持 `RwLock<HashMap<String, Arc<dyn BaseTool>>>` | `core/src/tools/` + `core/src/tools/handlers.rs` 类似 |
| 加载策略 | core 工具必发送, deferred 工具走 `ToolSearch` 激活 | 默认 + MCP tool search (0.142.2 起) |
| 中间件 | `ToolMiddleware` trait 管道 | (无独立 middleware,但有 `RequestPluginInstall` 流程) |
| 超时 | `ToolTimeoutConfig` per-tool (idle + max) | `core/src/tools/handlers.rs` 内置 |
| 去重 | same-turn dedup `Mutex<HashMap<String, ToolResult>>` per turn | (无) |
| 别名 | alias table `tool_names.rs` + fuzzy match | (无,codex 用 `ToolName` typed enum) |
| Sanitization | `ToolResultSanitizer` (Head/Tail/HeadTail) + overflow dir | `utils/output-truncation/` + 协议层 |
| 校验 | JSON Schema `validation.rs` | `tools/src/json_schema.rs` + `schemars` |

### 4.3 MCP 集成

| 维度 | OpenDev | Codex |
|---|---|---|
| 角色 | 仅 client (3 transports) | server + client + Codex-as-tool (3 个 crate) |
| SDK | 手写 JSON-RPC | 官方 `rmcp 1.8` |
| 协议 | JSON-RPC over stdio/HTTP/SSE | JSON-RPC over stdio + OAuth 完整 + WebSocket 远端 stdio |
| OAuth | `McpOAuthConfig` | `mcpServer/oauth/login` + `oauthLogin/completed` |
| 工具暴露 | `McpBridgeTool`(一个 bridge tool) | 每个 MCP tool 直接成为 model-visible tool(`mcp_tool_call.rs` + `mcp_tool_exposure.rs`) |
| Tool search | `ToolSearch` 工具 | 默认开启 (0.142.2+) |
| 深度集成 | 浅 | 深: skill 依赖、approval templates、OpenAI file 上传、namespace 暴露 |

### 4.4 沙箱 + 审批

| 维度 | OpenDev | Codex |
|---|---|---|
| macOS | 无 | `seatbelt.rs` + `.sbpl` 模板 + `restricted_read_only_platform_defaults.sbpl` |
| Linux | stub | Landlock (`landlock 0.4.4`) + bubblewrap (首选) + vendored `bwrap` + arg0 trick `codex-linux-sandbox` |
| Windows | 无 | 2 后端:elevated (完整 fs) + unelevated (restricted token + split fs policy) |
| 策略语言 | `ApprovalRulesManager` pattern 匹配 | **Starlark DSL** (`execpolicy/`) + legacy 兼容 |
| Approval | `AskForApproval` enum + `ApprovalRule`(Allow/Deny/Prompt) | `AskForApproval::UnlessTrusted/OnRequest/Granular/Never` + `GranularApprovalConfig` (sandbox_approval, rules, skill_approval, request_permissions, mcp_elicitations) |
| Guardian | 无 | `core/src/guardian/` + `ext/guardian/` + `GuardianAssessment` 事件 + `Op::ApproveGuardianDeniedAction` 单次豁免 |
| Permission profile | `PermissionRuleSet` glob-based | `protocol::PermissionProfile` + `FileSystemSandboxPolicy` + `FileSystemSandboxKind` (read-only carveouts, denied sub-paths) |
| Split fs policy | 无 | 有 (read-only 写洞, denied sub-paths) |
| Network policy | 无 | `NetworkSandboxPolicy` + `NetworkAccess` + `NetworkPolicyAmendment` |

### 4.5 会话/线程管理

| 维度 | OpenDev | Codex |
|---|---|---|
| 持久化 | JSON + JSONL + SQLite + Event Sourcing | JSONL rollout + SQLite `state` |
| Event sourcing | 10 变体 `SessionEvent` + Tombstone + `undo_to_seq` | 较轻,主要用 rollout JSONL |
| ID 模型 | `Session` (id: String) | `SessionId` + `ThreadId` + `AgentPath` 三种 ID |
| 快照 | `SnapshotManager` (shadow git) + `FileCheckpointManager` + `StateSnapshot` | (rollout JSONL 是唯一真理) |
| Resume | `SessionManager::resume` + `resume_picker` TUI | `app-server thread/resume` + `tui/resume_picker.rs` |
| Fork | `SessionEvent::SessionForked` + `get_forked_title` | `app-server thread/fork` |
| Archive | `SessionEvent::SessionArchived/Unarchived` | `app-server thread/archive/unarchive` |
| Rollback | Tombstone + undo (有限) | `app-server thread/rollback` (deprecated) + `ThreadRolledBack` event |
| 目标 (goal) | 无 | `app-server thread/goal/{set,get,clear}` + `ThreadGoalUpdated` notification |

### 4.6 提示工程

| 维度 | OpenDev `PromptComposer` | Codex |
|---|---|---|
| 位置 | `opendev-agents/src/prompts/composer/` (4 子模块) | `core/src/prompts/` + `protocol/src/prompts/base_instructions/` + `codex-prompts` |
| 分区 | `PromptSection` 数组,带 `priority: i32` + `condition: Option<ConditionFn>` + `content_provider: Option<ContentProviderFn>` + `cache_policy: CachePolicy` | **XML 包装器**: `<user_instructions>` / `<environment_context>` / `<apps_instructions>` / `<skills_instructions>` / `<plugins_instructions>` / `<collaboration_mode>` / `<multi_agent_mode>` / `<realtime_conversation>` / `<context_window>` / `<context_window_guidance>` |
| Cache 策略 | 3 档:Static / Cached / Uncached (Anthropic prompt caching 优化) | `responses_request_properties_match` (增量 WebSocket deltas);`<context_window>` 增量构建 |
| 嵌入式 | `include_str!` 内嵌 + 文件系统回退 | `include_dir 0.7.4` + 文件系统回退 |
| 调试 | (无) | `core/src/prompt_debug.rs` — 提示调试快照 |
| 加载器 | `opendev-agents/src/prompts/loader.rs` | `core/src/prompts/loader.rs`(推测) |

**核心差异**: OpenDev 用**程序化** `PromptSection` 组合;Codex 用**XML 包装器**做增量缓存。这两种方案各有优劣 — 前者更类型安全,后者对 cache 友好(改不改一眼能看出来)。

---

## 5. 跨切关注点对比

### 5.1 配置 / 路径

| 维度 | OpenDev | Codex |
|---|---|---|
| 主路径 | `~/.opendev/` (legacy) 或 `~/.config/opendev/` (XDG) | `~/.codex/`(单一稳定) |
| Env override | `OPENDEV_DIR`, `OPENDEV_SESSION_DIR`, `OPENDEV_LOG_DIR`, `OPENDEV_CACHE_DIR` | `$CODEX_HOME` (单一 env 覆盖整目录) |
| 加载优先级 | defaults → global settings → project settings → env vars → migrations → validation | ConfigToml 多层 + `$CODEX_HOME/config.toml` + 项目级 `.codex/config.toml` |
| Schema | `opendev-models::AppConfig` 122+ 行包含所有配置 | `config::Config` + `ConfigToml` + `feature` flags |
| 模式 | TOML (Rust struct via serde) | TOML (`toml 0.9.5` + `toml_edit 0.24.0`) + JSON Schema generation (`just write-config-schema`) |
| 热重载 | `ConfigWatcher` (poll-based) | `app-server` `config/mcpServer/reload` + `Op::ReloadUserConfig` |
| Model 目录 | `models.dev/api.json` 同步到 `~/.opendev/cache/models_dev/` | `models-manager/` 持续同步 + `model-provider-info/` 列表 |
| Plugin/Marketplace | `~/.opendev/plugins/known_marketplaces.json` + `installed_plugins.json` + `marketplaces/` | `plugin/` + `marketplace` + `ext/extension-api/` + `cli/src/marketplace_cmd.rs` |

### 5.2 鉴权

| 维度 | OpenDev | Codex |
|---|---|---|
| Web UI 鉴权 | Argon2 密码 + HMAC-SHA256 session token + cookie expiry | (无 Web UI) |
| Tauri desktop 鉴权 | 无(`AppServices` 不对命令做身份检查) | (无 — TUI 不鉴权) |
| 关键依赖 | `OPENDEV_SECRET_KEY`(release build 必填) | `keyring-store` (OS keyring) + `secrets/` (auth.json 0600) |
| LLM API key 解析 | `AppConfig::get_api_key_with_env`:models.dev env → builtin env → `{PROVIDER}_API_KEY` → config → `OPENAI_API_KEY` fallback | `secrets::OPENAI_API_KEY` lookup + keyring store + `auth.json` |
| OAuth | (无) | `login/` crate: ChatGPT OAuth + device-code + API key + PKCE + local HTTP callback server (`tiny_http 0.12`) |
| 多账号轮转 | `AuthProfileManager`:429=30s, 401=300s, 403=600s, 5xx=30-60s | (无 — 单账号, 走 OAuth 刷新) |
| Telegram pairing | `opendev-channels`:pair/unpair 命令 | (无) |
| Channel DM policy | `DmPolicy` 枚举 | (无) |
| Attestation | 无 | `core/src/attestation.rs` + `app-server/src/attestation.rs` (TEE 证明头) |

### 5.3 错误处理

| 维度 | OpenDev | Codex |
|---|---|---|
| 库内错误 | `thiserror 2` per-crate | `thiserror 2.0.17` per-crate |
| 顶层 | `anyhow 1` 在 `opendev-cli` + `src-tauri` | `anyhow 1` + `color-eyre 0.6.3` |
| Mutex poison | 100+ 处 `.unwrap_or_else(|e| e.into_inner())` (v0.1.9 修复) | (使用标准 `parking_lot` + 较少 mutex) |
| unwrap/expect | 3,621 unwrap / 153 expect / 54 panic (panic 全在 test) | 较少,使用 `expect` + 错误传播 |
| unsafe | 5 生产 + 33 测试 (生产:libc::kill ×2, pre_exec+setpgid ×2, libc::kill ×3) | 5+ 生产 (Landlock/seccomp/prctl/bwrap 调用) |
| spawn_blocking | 标记 P1 待办 | 大量使用 `tokio::task::spawn_blocking` |
| bounded channel | 5 个 `mpsc::unbounded_channel` 待替换为 bounded | 大量使用 bounded mpsc |

### 5.4 日志 / 观测

| 维度 | OpenDev | Codex |
|---|---|---|
| 框架 | `tracing 0.1` + `tracing-subscriber 0.3` (env-filter, json) + `tracing-appender 0.2` (daily rotated) | 同上 + `tracing-opentelemetry 0.32.0` + `opentelemetry 0.31.0` + `opentelemetry-otlp` + `sentry 0.46.0` |
| 文件输出 | `~/.opendev/logs/opendev.log` daily rotated | (无 daily rotated,走 OTLP) |
| OTLP | `OTEL_EXPORTER_OTLP_ENDPOINT` | `OTEL_EXPORTER_OTLP_ENDPOINT` 完整 |
| Perfetto | `OPENDEV_PERFETTO` (v0.1.9 新增) | (无) |
| Web dashboard | (无) | `app-server` 提供 introspection |
| Analytics | (无) | `analytics/` + `feedback/upload` + telemetry on/off + OpenAI compliance |
| Error tracking | (无) | `sentry 0.46.0` |
| Tracing context | (无 W3C TraceContext) | `protocol::W3cTraceContext` + `rollout-trace` |

### 5.5 测试 / CI

| 维度 | OpenDev | Codex |
|---|---|---|
| Unit tests | 3,183 个 test across 326 文件 | 大量,无明确统计 |
| Integration | 12 个 crate 有 `tests/integration.rs` | `core/tests/common` + `app-server/tests/common` + `mcp-server/tests/common` + `app_test_support` |
| Snapshot | (未明确) | `insta 1.46.3`(`markdown_render_tests.rs` 等) |
| Fuzz | (无) | (无,部分 crate 用 `proptest` 临时) |
| Proptest | v0.1.9 移除 (声明未用) | (未明确) |
| Mock | `wiremock` for HTTP | `wiremock 0.6` + `assert_cmd 2` + `predicates 3` + `mockall`(推测) |
| Bench | criterion: `agent_bench.rs` + `render_bench.rs` | `divan 0.1.21` + criterion |
| CI | GitHub Actions:5 job (fmt blocking, clippy/test/audit report-only, deny blocking) | GitHub Actions:多个 workflow(包括 `rust-ci.yml`、`python-sdk-release.yml`、`python-runtime-build.yml`、`python-runtime-release.yml`、`cargo-audit.yml` 等) |
| Build system | cargo only | cargo + Bazel (`bazel/`, `MODULE.bazel`, `BUILD.bazel`, `.bazelrc`) + Just (`justfile`) + Nix (`flake.nix`) |
| 工具 | cargo-deny 0.19 + rustsec/audit-check | cargo-deny (无明确配置) |
| Lint policy | rustfmt edition 2024 + clippy 警告 ~40 (待修) | rustfmt + clippy + `argument-comment-lint`(自研工具) |
| Dep policy | `deny.toml` + `.cargo/audit.toml` 17 ignored advisories | 无显式 deny.toml |

---

## 6. 前端 / 桌面 / TUI 对比

### 6.1 桌面应用

| 维度 | OpenDev Desktop | Codex App |
|---|---|---|
| 技术 | Tauri 2 + React 19 + Vite 8 + Tailwind v4 + Zustand 5 | (仓库外)Electron(?) 桌面 |
| 入口 | `src-tauri/src/main.rs` (126 行) 注册 34 个 command | `cli/src/desktop_app/` 触发 + `codex app` 子命令 |
| Bridge | `opendev-web` 复用 + `WsBroadcast` 桥接 + Tauri emit | `app-server` over Unix-domain socket (`$CODEX_HOME/app-server-control/app-server-control.sock`) |
| 协议 | Tauri event 名 + JSON payload(经过 `eventBridge.ts` 适配) | `app-server` JSON-RPC v1 + v2 (TUI/VSCode/Codex App 共享) |
| 状态 | 5 个 Zustand stores (chat/fileChanges/subagents/todo/status) | app-server 持有 `thread_state` + `thread_status` |
| 主题 | 9 themes (cyberpunk/dark-default/light-default/warm/geek/sumi-e/synthwave/techno/brutalism) | (TUI 内的多 theme) |
| 设计 | 自研 "Surface Ladder" 设计系统 (1px hairline、achromatic、Inter+JetBrains Mono) | 简单,主要依赖 Codex brand |
| 组件库 | 自研 + Radix Dialog + lucide + shiki + sonner + xyflow | N/A |
| 子模块 | Chat (25) + Layout (6) + Settings (10) + ui (7) | N/A |

### 6.2 TUI

| 维度 | OpenDev `opendev-tui` (26,735 LoC) | Codex `tui/` (~15-25k LoC) |
|---|---|---|
| 框架 | Ratatui 0.30 + Crossterm 0.29 | Ratatui 0.29 (patched) + Crossterm 0.28 (patched) + `ansi-to-tui 7.0.0` + `syntect 5` + `pulldown-cmark 0.10` + `portable-pty 0.9` + `arboard 3` |
| 文件数 | 153 .rs | ~70 .rs |
| 架构 | 22 widgets + 25 controllers + 15 managers + 9 themes | module-per-feature,大量子目录 |
| Markdown | (前端 `markdown-it 14` 在 Tauri 中) | `markdown.rs` + `markdown_render.rs` + `markdown_stream.rs` + `markdown_text_merge.rs` (流式合并) + snapshot tests |
| 装饰 | 22 widgets 含 MatrixRain 等 | pets (`tui/src/pets/`) + shimmer + ascii_animation |
| 输入 | (无明确 $EDITOR 集成) | `external_editor.rs` ($EDITOR 唤起) |
| Clipboard | (无) | `clipboard_copy.rs` + `clipboard_paste.rs` (arboard) |
| Resize | 基础 | `resize_reflow_cap.rs`(性能优化) |
| Selection | 基础 | `selection_list.rs` + `oss_selection.rs` + `pager_overlay.rs` |
| Notification | (无) | `notifications/`(桌面通知) |
| Onboarding | (无) | `onboarding/`(首次使用向导) |
| Keymap | 基础 | `keymap.rs` + `keymap_setup/`(可定制) |
| Session log | (无) | `session_log.rs` |
| Branch summary | (无) | `branch_summary.rs` + `git_action_directives.rs` |
| Goal display | (无) | `goal_display.rs` + `goal_files.rs` |
| Mention codec | (无 `@`-mention 文件选择) | `mention_codec.rs` + `mention_syntax.rs` (`@file`, `@symbol`) |
| App-server session | 无 (直接 core) | `app_server_session.rs`(TUI 经 app-server) |
| Multi-agents UI | `subagents` widget | `multi_agents.rs` + `collaboration_modes.rs` |
| IDE context | (无) | `ide_context.rs` (从 VSCode 接收) |

### 6.3 CLI

| 维度 | OpenDev `opendev-cli` | Codex `cli/` + `exec/` |
|---|---|---|
| 主二进制 | `opendev` (clap 4) | `codex` (clap 4) |
| 子命令 | `setup` / `config` / `mcp` / `run` / `session` / `channel` / `remote` | `exec` / `app-server` / `sandbox` / `mcp` / `marketplace` / `plugin` / `remote-control` / `app` / `doctor` / `login` / `debug-sandbox` |
| 模式 | REPL / 非交互 `-p` / replay `--replay JSONL` / remote (Telegram) | TUI (默认) / `codex exec` 非交互 / `codex debug-sandbox` / `codex doctor` |
| Doctor | 无 | `cli/src/doctor.rs` + `cli/src/doctor/` (系统健康检查) |
| WSL 路径处理 | 无 | `cli/src/wsl_paths.rs` |
| State DB recovery | 无 | `cli/src/state_db_recovery.rs` |
| Exit status | 自定义 | `cli/src/exit_status.rs` |

### 6.4 端到端事件流对比

**OpenDev 桌面事件流:**
```
Agent Loop (Rust)
  → AgentEventCallback trait
    → WebEventCallback implementation
      → AppState.broadcast() [in opendev-web, used by Tauri bridge]
        → tokio::sync::broadcast::channel<WsBroadcast>
          → spawn_event_bridge() [src-tauri/src/main.rs:14-33]
            → app.emit(msg.msg_type, msg.data)
              → Tauri event system
                → TauriTransport.onEvent() [src/repositories/TauriTransport.ts:16-21]
                  → eventBridge.on() [src/api/eventBridge.ts:22-56]
                    → Store handlers [src/stores/chat.ts]
                      → React re-render
```
**问题**: 事件名不统一(`mcp:status_changed` vs `mcp_status_update` vs `mcp.server.connected` 3 种命名混用),`server.rs` 是临时桥接,标记 "This module will be fully removed when agent events flow directly through Application Services"。

**Codex 事件流:**
```
Core 内部: Codex orchestrator
  → Op 提交队列 (SQ)
  → EventMsg 事件队列 (EQ)
    → app-server transport (stdio/WS/uds)
      → JSON-RPC 2.0 消息
        → TUI / VSCode / Codex App / SDK 客户端
```
**优势**: 单一稳定的 wire protocol(v1 + v2 双轨),`app-server-protocol` 用 `ts-rs` 自动生成 TS 类型,`cursor pagination` 标准,`#[experimental(...)]` proc-macro 标记实验性方法。

---

## 7. 功能清单 (User-Facing Features)

> 80+ features from opendev (per feature inventory §7 of recon) vs ~50+ from codex (per feature inventory §7 of codex recon).

### 7.1 核心 agent 功能

| Feature | OpenDev | Codex |
|---|---|---|
| ReAct loop 交互 | ✅ `MainAgent` + `ReactLoop` | ✅ `Codex` + `CodexThread` ReAct |
| 多 provider LLM | ✅ 7+ 适配器,models.dev 解析 20+ | ✅ 5+ (OpenAI/Azure/Bedrock/Anthropic/Ollama/LMStudio/custom) |
| Doom-loop 检测 | ✅ `DoomLoopDetector` (3-cycle, 3-rep) | ❌ (无显式检测) |
| Stream / chunked response | ✅ `FrontendEvent::MessageChunk` | ✅ `ResponseEvent` + `StreamCallback` |
| Reasoning content | ✅ `message.reasoning_content` | ✅ `AgentReasoning` + `AgentReasoningRawContent` + `AgentReasoningSectionBreak` |
| Interrupt / cancel | ✅ `InterruptToken` (sync+async+soft) | ✅ `Op::Interrupt` + `TurnAborted` |
| Subagent / multi-agent | ✅ `SubagentManager` (6 内置) | ✅ `codex_delegate` + `core/src/agent/` |
| Team / mailbox | ✅ `TeamManager` + `Mailbox` + `TeamTaskList` | ❌ (无 mailbox) |
| Workflow engine | ✅ `opendev-workflow` (pipeline/barrier/loop) | ❌ |
| Task scheduler | ✅ `opendev-runtime::task_scheduler` | ❌ |
| Background tasks | ✅ `background_agents` + `background_tasks` | ✅ `app-server` `thread/backgroundTerminals/{clean,list,terminate}` |
| Guardian auto-approval | ❌ | ✅ `core/src/guardian/` + `GuardianAssessment` |
| Realtime (voice) | ❌ | ✅ `realtime-webrtc/` + `RealtimeVoice` |

### 7.2 文件 / 编辑

| Feature | OpenDev | Codex |
|---|---|---|
| File read | ✅ `FileReadTool` (binary + suggestions) | ✅ `list_dir` / `read_file` tools |
| File write (atomic) | ✅ `FileWriteTool` | ✅ `write_file` tool |
| File edit (fuzzy) | ✅ `FileEditTool` (9-pass + per-file lock) | ✅ `apply_patch` (unified-diff, fuzzy Unicode-dash) |
| Multi-edit | ✅ `MultiEditTool` | ❌ (无独立 tool,apply_patch 可多次) |
| Patch tool | ✅ `PatchTool` | ✅ `apply_patch` (独立 crate + streaming parser) |
| Diff preview | ✅ `DiffPreviewTool` | ✅ `tui/src/diff_render.rs` + `diff_model.rs` + `get_git_diff.rs` |
| Notebook (Jupyter) | ✅ `NotebookEditTool` | ❌ |
| Symbol find | ✅ `opendev-tools-symbol` (find/references/rename/replace_body) | ❌ |
| LSP integration | ✅ `opendev-tools-lsp` (JSON-RPC + SymbolCache + WorkspaceSymbolTool) | ❌ |
| AST-grep | ✅ `AstGrepTool` | ❌ |
| Grep | ✅ `GrepTool` | ✅ `grep_files` tool |
| File list (glob) | ✅ `FileListTool` (depth 3) | ✅ `list_dir` tool |
| Fuzzy file search | ❌ (有文件列表,无 fuzzy) | ✅ `file-search/` + `tui/src/file_search.rs` + `app-server/src/fuzzy_file_search.rs` |
| File watcher | ✅ `FileWatcher` (notify) | ✅ `file-watcher/` + `app-server/src/fs_watch.rs` |
| Worktree isolation | ✅ `WorktreeManager` (adjective-noun) | ✅ `app-server` `runtimeWorkspaceRoots` |
| File checkpoint / undo | ✅ `FileCheckpointManager` + `SnapshotManager` (shadow git) | ❌ (靠 rollout 重建) |

### 7.3 Shell / 执行

| Feature | OpenDev | Codex |
|---|---|---|
| Bash exec | ✅ `BashTool` (fg/bg, dual-timeout, dangerous patterns, read-only heuristic) | ✅ `shell` tool + `unified_exec/` 交互式 PTY |
| Background process | ✅ `BashTool` background mode | ✅ `unified_exec` 持久 shell |
| Process spawn (raw) | ❌ | ✅ `app-server` `process/spawn` + `process/{writeStdin,resizePty,kill,outputDelta,exited}` |
| Command exec (raw) | ❌ | ✅ `app-server` `command/exec` + `command/exec/{write,resize,terminate,outputDelta}` |
| !cmd user shell | ❌ (无) | ✅ `Op::RunUserShellCommand` + `tui/src/exec_command.rs` |

### 7.4 网络 / 浏览器

| Feature | OpenDev | Codex |
|---|---|---|
| Web fetch | ✅ `WebFetchTool` (HTML→markdown, SSRF 保护 + 17+ regex) | ✅ (via `core/src/tools/`) |
| Web search | ✅ `WebSearchTool` (DuckDuckGo HTML, 256KB body limit) | ✅ `ext/web-search/` + `protocol::WebSearchAction` |
| Web screenshot | ✅ `WebScreenshotTool` | ❌ |
| Browser automation | ✅ `BrowserTool` | ❌ |
| Open browser | ✅ `OpenBrowserTool` (系统浏览器) | ❌ (有 `webbrowser 1.0` dep) |

### 7.5 MCP

| Feature | OpenDev | Codex |
|---|---|---|
| MCP client (stdio) | ✅ `StdioTransport` | ✅ via `rmcp 1.8` |
| MCP client (HTTP/SSE) | ✅ `HttpTransport` + `SseTransport` | ✅ via `rmcp 1.8` |
| MCP remote (over WS) | ❌ | ✅ via `responses-api-proxy` |
| MCP OAuth | ✅ `McpOAuthConfig` | ✅ `mcpServer/oauth/login` |
| Codex as MCP server | ❌ | ✅ `mcp-server/` |
| Codex as MCP tool | ❌ | ✅ `codex-mcp/` |
| MCP tool search | ✅ `ToolSearch` (通用) | ✅ (default since 0.142.2) |
| Skill 依赖 MCP | ❌ | ✅ `mcp_skill_dependencies.rs` |
| MCP tool approval | ❌ | ✅ `mcp_tool_approval_templates.rs` |
| MCP file upload (OpenAI) | ❌ | ✅ `mcp_openai_file.rs` |

### 7.6 Skills / Plugins / Connectors

| Feature | OpenDev | Codex |
|---|---|---|
| Skills (Markdown+frontmatter) | ✅ `opendev-agents/src/skills/` 7 子模块 | ✅ `core/src/skills/` + `skills/` + `ext/skills/` + `core-skills/` |
| Skills discovery | ✅ 4 sources: builtin + local dirs + remote URL indexes + config paths | ✅ `ext/skills/` (similar) |
| Skills pin | ✅ | ✅ |
| Plugin manager | ✅ `opendev-plugins` + `MarketplaceCatalog` (HTTP 拉取) | ✅ `plugin/` + `core-plugins/` + `ext/extension-api/` |
| Marketplace | ✅ `known_marketplaces.json` + `installed_plugins.json` | ✅ `cli/src/marketplace_cmd.rs` + `marketplace/{add,remove,upgrade}` |
| Plugin install | ✅ | ✅ `app-server` `plugin/install` + `plugin/uninstall` |
| Plugin read | ✅ | ✅ `app-server` `plugin/read` + `plugin/skill/read` |
| Connectors (ChatGPT) | ❌ | ✅ `connectors/` + `ext/connectors/` |
| Apps (ChatGPT) | ❌ | ✅ `core/src/apps.rs` + `app/list` |
| Hooks (lifecycle) | ✅ 10 events shell exec | ✅ `hooks/` + `hooks_rpc.rs` + `app-server hooks/list` |
| External agent migration | ❌ | ✅ `external-agent-migration` (从 Claude Code 导入) |
| External agent sessions | ❌ | ✅ `external-agent-sessions` |
| Plugin list (CLI) | ✅ `opendev plugin list` | ✅ `codex plugin list` |

### 7.7 Memory / 上下文

| Feature | OpenDev | Codex |
|---|---|---|
| Short-term memory | ✅ `ShortTermMemory` | ❌ (用 rollout 重建) |
| Long-term memory (FTS5) | ✅ `opendev-memory` (decay, WriteGate) | ✅ `ext/memories/` + `memories/{read,write}/` |
| Memory consolidation | ✅ `memory_consolidation.rs` (后台 worker) | ❌ |
| Project memory `MEMORY.md` | ✅ (200-line / 25KB cap) | ✅ `AGENTS.md` + `agents_md_manager` |
| AGENTS.md discovery | ✅ `SubdirInstructionTracker` (7 sources) | ✅ `core/src/agents_md.rs` + `agents_md_manager` |
| CLAUDE.md support | ✅ (同 SubdirInstructionTracker) | ❌ (无显式, 走 AGENTS.md) |
| Context compaction | ✅ `ContextCompactor` (5 阈值) | ✅ `compact.rs` + `compact_remote.rs` + `compact_remote_v2.rs` + `compact_token_budget.rs` |
| Compact remote | ❌ | ✅ 委托给 server `/responses/compact` |
| Context picker | ✅ `ContextPicker` | ✅ `context-fragments/` + `ContextualUserFragment` trait |
| Codebase retrieval | ✅ `CodebaseIndexer` + `EntityExtractor` + `ContextRetriever` | ❌ (无独立 retrieval, 走 RAG via `connectors`) |
| Context token monitor | ✅ `ContextTokenMonitor` | ✅ `TokenCount` event |
| Validated message list | ✅ `ValidatedMessageList` (写时强制) | ❌ |

### 7.8 Persistence / History

| Feature | OpenDev | Codex |
|---|---|---|
| Event-sourced sessions | ✅ `SessionEvent` 10 变体 + Tombstone | ❌ (JSONL only) |
| JSONL persistence | ✅ `SessionEvent` JSONL | ✅ `rollout/` JSONL |
| SQLite persistence | ✅ `SqliteSessionStore` (WAL) | ✅ `state` crate (`sqlx 0.9` + WAL) |
| Session listing | ✅ `SessionManager::list` | ✅ `app-server thread/list` + `thread/loaded/list` + `thread/turns/list` + `thread/items/list` |
| Session resume | ✅ `SessionManager::resume` | ✅ `app-server thread/resume` |
| Session fork | ✅ `SessionEvent::SessionForked` | ✅ `app-server thread/fork` |
| Session archive | ✅ `SessionEvent::SessionArchived` | ✅ `app-server thread/archive/unarchive` |
| Session export | ✅ `export_markdown` | ❌ |
| Snapshot / rollback | ✅ `SnapshotManager` (shadow git) + Tombstone undo | ✅ `app-server thread/rollback` (deprecated) + `ThreadRolledBack` |
| Sidechain transcript | ✅ `SidechainReader/Writer` | ❌ |
| Cost tracking | ✅ `CostTracker` + `TokenUsage` (cache_read/creation) | ❌ (无显式 cost tracking) |
| File diff snapshot | ✅ `FileCheckpointManager` | ❌ |
| Topic detection | ✅ `TopicDetector` | ❌ |

### 7.9 审批 / 权限 / 计划

| Feature | OpenDev | Codex |
|---|---|---|
| Tool approval flow | ✅ `ApprovalRulesManager` + `ApprovalDialog` | ✅ `AskForApproval` + `app-server` `ExecApprovalRequest` / `ApplyPatchApprovalRequest` |
| Permission rules | ✅ `PermissionRuleSet` (glob, priority, directory_scope) | ✅ `protocol::PermissionProfile` + `FileSystemSandboxPolicy` |
| Sensitive file check | ✅ `is_sensitive_file` (.env*/credentials/id_rsa/.npmrc/.pypirc) | ✅ 同等, 默认拒绝 |
| Secrets detection | ✅ 8 patterns detect+redact | ✅ `secrets/` + `keyring-store` |
| Approval policy: never | ✅ (无单独) | ✅ `AskForApproval::Never` |
| Approval policy: on-request | ✅ 默认 | ✅ `AskForApproval::OnRequest` |
| Approval policy: granular | ❌ | ✅ `AskForApproval::Granular { sandbox_approval, rules, skill_approval, request_permissions, mcp_elicitations }` |
| Approval policy: unless-trusted | ❌ | ✅ `AskForApproval::UnlessTrusted` |
| Guardian auto-approval | ❌ | ✅ 风险等级评估 |
| Plan mode (5-phase) | ✅ `PlanApprovalRequest` + `PresentPlanTool` | ✅ `update_plan` tool (`PlanUpdate` event) |
| Ask user (multi-Q) | ✅ `AskUserTool` + `AskUserRequest` (multi_select + "other") | ✅ `request_user_input` tool (`RequestUserInput` event) |
| Permission request (runtime) | ❌ | ✅ `request_permissions` tool |
| Tool approval response | ✅ `approve_tool` Tauri command | ✅ `Op::ExecApproval` / `Op::PatchApproval` / `Op::RequestPermissionsResponse` |

### 7.10 Todo / Plan / Goal

| Feature | OpenDev | Codex |
|---|---|---|
| Todo write/update/complete/list/clear | ✅ 6 个独立 tool | ❌ (无 todo tool) |
| Unified todo tool | ✅ `TodoTool` | ❌ |
| Todo list UI (TUI) | ✅ `TodoPanelWidget` | ❌ (无) |
| Subagent (per-subagent todo) | ✅ | ❌ |
| Task complete (final) | ✅ `TaskCompleteTool` | ❌ |
| Per-thread goal | ❌ | ✅ `app-server thread/goal/{set,get,clear}` + `ThreadGoalUpdated` notification |
| Plan update (mid-turn) | ✅ `PresentPlanTool` | ✅ `update_plan` tool |
| Review mode | ❌ | ✅ `Op::Review` + `EnteredReviewMode` / `ExitedReviewMode` events + `app-server review/start` |

### 7.11 视觉 / 多模态

| Feature | OpenDev | Codex |
|---|---|---|
| Image input (read) | ✅ `FileReadTool` binary | ✅ `view_image` tool (`ViewImageToolCall` event) |
| Image analysis (VLM) | ✅ `VlmTool` | ❌ (无独立 VLM tool) |
| Image generation | ❌ | ✅ `ext/image-generation/` + `image_generation` tool |
| Image upload (remote) | ❌ | ✅ `mcp_openai_file.rs` + `app-server/src/image_url.rs` |
| Web screenshot | ✅ | ❌ |
| Vision detail | ❌ | ✅ `protocol::ImageDetail` |
| Realtime audio (WebRTC) | ❌ | ✅ `realtime-webrtc/` + `RealtimeVoice` (Alloy/Arbor/.../Verse) + `RealtimeOutputModality::{Text,Audio}` |
| Personality | ❌ | ✅ `protocol::Personality` + `personality_migration` |

### 7.12 通知 / 装饰

| Feature | OpenDev | Codex |
|---|---|---|
| Sound | ✅ `opendev-runtime/src/sound` | ❌ |
| Desktop notification | ❌ | ✅ `tui/notifications/` |
| Toast (前端) | ✅ `sonner 2.0.7` | N/A |
| Spinner | ✅ `SpinnerState` (TUI) + `HaloSpinner` (前端) | ✅ `tui/spinner` + `ascii_animation` |
| Matrix rain | ✅ `MatrixRain.tsx` | ❌ (有 pets + shimmer) |
| Pets animation | ❌ | ✅ `tui/pets/` |
| Shimmer | ❌ | ✅ `tui/shimmer.rs` |
| Themes | 9 (前端) + 多个 TUI | 多个 TUI |

### 7.13 通道 / 多端

| Feature | OpenDev | Codex |
|---|---|---|
| Telegram bot | ✅ `opendev-channels/src/telegram/` (polling + remote_claim + pair/unpair) | ❌ |
| Web mode (Axum) | ✅ `opendev-web` (Argon2 + HMAC) | ❌ |
| WebSocket (前端) | ✅ | ❌ (走 app-server WS) |
| HTTP auth | ✅ Argon2 + HMAC-SHA256 + cookie | ❌ |
| Remote control (phone) | ❌ | ✅ `ext/remote_control` + `app-server remoteControl/{enable,disable,status/read,pairing/start,pairing/status,client/list,client/revoke,status/changed}` |
| iOS / Android | ❌ | ✅ Codex Mobile (via ChatGPT app) |
| VSCode extension | ❌ | ✅ `openai.chatgpt` (marketplace) |
| Cursor / Windsurf | ❌ | ✅ (走同一扩展) |
| GitHub Action | ❌ | ✅ `codex exec` 包装 |

---

## 8. 配置 / 设置 / 平台能力

| Feature | OpenDev | Codex |
|---|---|---|
| Multi-model (per-slot) | ✅ 4 slots: normal / thinking / vision / compact | ✅ `core-plugins` + 协作模式 + `model/list` |
| Per-session model override | ✅ `update_session_model` Tauri command | ✅ `app-server thread/settings/update` + `ThreadSettingsOverrides` |
| Model provider auto-detect | ✅ `detect_provider_from_key` (sk-ant-/sk-/gsk_/AIza) | ✅ `model-provider-info` |
| Model catalog sync | ✅ `models.dev` 拉取 | ✅ `models-manager` + `models_refresh_worker` |
| Capability queries | (无) | ✅ `app-server modelProvider/capabilities/read` + `experimentalFeature/list` |
| Service tier | (无) | ✅ `service_tier_resolution` |
| Sandbox mode UI | ✅ `set_operation_mode` Tauri command | ✅ `codex sandbox` + `debug-sandbox` |
| Operation mode (Plan) | ✅ `OperationMode::Normal / Plan` | ✅ `MultiAgentMode` + `CollaborationMode` |
| Autonomy level | ✅ `set_autonomy_level` Tauri command | ✅ `AskForApproval` + `GranularApprovalConfig` |
| Reasoning effort | ✅ `reasoning_effort` in `AppConfig` | ✅ `ReasoningEffort` in `openai_models.rs` |
| Verbosity | (无) | ✅ `Verbosity` in `openai_models.rs` |
| Reasoning summary | (无) | ✅ `ReasoningSummary` in `config_types.rs` |
| Streaming on/off | ✅ `enable_streaming` adapter | ✅ `supports_streaming` adapter + Responses streaming |
| Format / formatter | ✅ `FormatterConfig` | ❌ |
| Color scheme | ✅ `color_scheme` in `AppConfig` | (无, 自动检测) |
| Show token count | ✅ `show_token_count` | ✅ `TokenCount` event |
| Enable sound | ✅ `enable_sound` | ❌ |
| Verbose | ✅ `verbose` | ❌ |
| Debug logging | ✅ `debug_logging` | ❌ (走 `RUST_LOG`) |
| Auto-save | ✅ `auto_save_interval` | (无, 实时保存) |
| Max undo history | ✅ `max_undo_history` | ❌ |
| Topic detection | ✅ `topic_detector` | ❌ |
| Plan mode explore agent count | ✅ 3 | ❌ |
| Plan mode plan agent count | ✅ 1 | ❌ |
| Plan mode explore variant | ✅ enabled | ❌ |
| Default agent | ✅ `default_agent` | ❌ (无 agent 角色概念) |
| Agents (inline config) | ✅ `agents: HashMap<String, AgentConfigInline>` | ❌ |
| Model variants | ✅ `model_variants: HashMap<String, ModelVariant>` | ❌ (走 model_provider_info) |
| Skill paths | ✅ `skill_paths` | ✅ `app-server skills/extraRoots/set` |
| Skill URLs | ✅ `skill_urls` (remote index) | ❌ |
| Instructions | ✅ `instructions: Vec<String>` (类似 system prompt) | ✅ `<user_instructions>` + `<apps_instructions>` + `<plugins_instructions>` |
| Instruction excludes | ✅ | ❌ |
| Max context tokens | ✅ `max_context_tokens` | ✅ `<context_window>` (动态) |
| Max tokens | ✅ `max_tokens` | ✅ `ModelInfo::max_tokens` |
| Temperature | ✅ | ✅ |
| Working directory | ✅ | ✅ |
| Allowed extensions | ✅ `allowed_extensions` | ❌ |
| Max file size | ✅ `max_file_size` | ❌ |
| Backup before edit | ✅ `backup_before_edit` | ❌ |
| Bash timeout | ✅ `bash_timeout` | ✅ (per-tool idle + max) |
| Enable bash | ✅ `enable_bash` | ❌ (always on) |
| Show diffs | ✅ `show_diffs` | ✅ (TUI 一直显示) |
| Auto mode | ✅ `AutoModeConfig` (max_operations, require_confirmation_after, dangerous_operations_require_approval) | ❌ (auto-mode 不在概念里) |
| Sandbox config | ✅ `SandboxConfig` (stub) | ✅ 完整 4 类:`DangerFullAccess`, `ReadOnly { network_access }`, `WorkspaceWrite { writable_roots, network_access, exclude_tmpdir_env_var, exclude_slash_tmp }`, `ExternalSandbox { network_access }` |
| Channels (Telegram) | ✅ `ChannelsConfig` (telegram + dm_policy) | ❌ |

---

## 9. SDK / 集成 / 分发

| 维度 | OpenDev | Codex |
|---|---|---|
| TypeScript SDK | ❌ (无) | ✅ `@openai/codex-sdk` (Node 18+),spawn CLI + JSONL over stdin/stdout,`Codex` class,`startThread`/`resumeThread`/`forkThread`,`run`/`runStreamed`,JSON Schema structured output |
| Python SDK | ❌ (无) | ✅ Python Codex SDK + `sdk/python-runtime/` + `python-sdk-release` workflow |
| Rust client | (无,内嵌 core 即可) | ✅ `codex-app-server-client` |
| 桌面 App | ✅ Tauri 自带 | ✅ Codex App (macOS/Windows), `codex app` + `cli/src/desktop_app/` 触发 |
| VSCode 扩展 | ❌ | ✅ `openai.chatgpt` (marketplace),`clientInfo: codex_vscode`,同时支持 Cursor/Windsurf |
| GitHub Action | ❌ | ✅ `codex exec` 包装 |
| Codex Web | ❌ | ✅ `chatgpt.com/codex` |
| Codex Cloud Tasks | ❌ | ✅ `cloud-tasks/` + `cloud-tasks-client/` + `cloud-tasks-mock-client/` + `cloud-config/` |
| Codex Mobile | ❌ | ✅ iOS + Android via ChatGPT app |
| IDE context integration | ❌ | ✅ `tui/src/ide_context.rs` 从 VSCode 接收 context |
| 安装方法 | `npm run tauri dev`(开发) / 打包后安装 | curl 脚本 / PowerShell / `npm i -g @openai/codex` / `brew install --cask codex` / DotSlash / 从源码 |
| 系统要求 | macOS / Windows / Linux (Tauri 2) | macOS 12+ / Ubuntu 20.04+ / Debian 10+ / Windows 11 (WSL2 or native) / Git 2.23+ / 4GB RAM min |
| Release 频率 | 周级 (workspace `0.1.9`) | 887 个 releases(2024-2026 两年) |
| Binary 分发 | Tauri 打包 (`.dmg`/`.exe`/`.AppImage`) | npm `optionalDependencies` 按 platform 拉对应 tarball |

---

## 10. 治理 / 工程 / 质量

| 维度 | OpenDev | Codex |
|---|---|---|
| Constitution | ✅ 14 冻结设计原则 | (无显式 constitution, AGENTS.md 散布) |
| ADR 数量 | 7 份 (001-007) + 1 份补充 (0005) | (无显式 ADR 目录) |
| 架构文档 | 6 份 (crate-layering / data-flow / frontend / security-model / desktop-communication / adr-0005) | 多份 AGENTS.md + 每 crate README |
| Audit 报告 | ✅ 504 行,29 issues,工程评分 68-78/100,top 10 P0 | (无公开) |
| Remediation plan | ✅ 702 行,34 GitHub issues,12 PRs,90 天路线图 | (无公开) |
| Project status (中文 LLM 评估) | ✅ 966 行 | (无) |
| Roadmap | ✅ v0.1.9 → v0.2 → v0.3 → v0.5 → v1.0 | (无显式公开 roadmap) |
| CI | 5 jobs, fmt+deny blocking, clippy+test+audit report-only | 多 workflow, 严格 |
| Build system | cargo only | cargo + Bazel + Just + Nix + pnpm |
| Lint 工具 | cargo fmt + clippy + cargo-deny + rustsec/audit-check | cargo fmt + clippy + argument-comment-lint(自研) + wiremock + insta + divan |
| 测试数量 | 3,183 unit + 12 integration | 大量(未明确) |
| Proptest | v0.1.9 移除 (声明未用) | (无) |
| 依赖策略 | deny.toml (17 ignored advisories) + cargo-deny 0.19 + `.cargo/audit.toml` | (无显式 deny.toml) |
| License allowlist | MIT/Apache-2.0/BSD-2/BSD-3/0BSD/ISC/Unicode-3.0/MPL-2.0/CDLA-Permissive-2.0/BSL-1.0/Zlib/CC0-1.0 | Apache-2.0 (单 license) |
| 文档丰富度 | 高(中英混合) | 极高(`AGENTS.md` 散布,每 crate README,developers.openai.com) |
| 社区规模 | (本地) | 94.1k stars, 13.9k forks, 480+ contributors, 5k+ open issues |

---

## 11. 已识别的技术债 / 待办

### 11.1 OpenDev (来自 AUDIT_REPORT / REMEDIATION_PLAN)

**P0 (v0.1.9 已修):**
- ✅ SSRF fix (`WebFetchTool::is_private_url` + fail-closed)
- ✅ Bedrock SigV4 signing
- ✅ CI/CD pipeline
- ✅ cargo-deny + audit config

**P1 (in progress):**
- HTML converter regex `LazyLock` (done)
- `bash::prepare_command` regex `LazyLock` (done)
- Mutex poison recovery (done, 100+ sites)
- `spawn_blocking` for blocking I/O
- Replace unbounded channels with bounded
- `// SAFETY:` comments on all `unsafe` (done)

**P2/P3 (deferred):**
- `std::sync::RwLock` → `tokio::sync::RwLock` (team_manager / team_task_list)
- `opendev-tools-impl → opendev-agents` layer violation (extract shared types to `opendev-models`)
- `opendev-runtime` split (90 文件 kitchen sink)
- `opendev-cli` God Object split (15 internal deps)
- `opendev-sandbox` stubs (100% placeholders, Linux-only)
- `SqliteSessionStore::block_on` hack
- `opendev-memory` 和 `opendev-history` share SQLite pool (currently independent)

**Other:**
- `dirs 5` (in `opendev-http`) vs `dirs 6` (everywhere else) — version inconsistency
- `proptest` removed but `Cargo.toml` cleanup not yet complete
- `ts-rs` not feature-gated in `opendev-models` (adds compile time)
- `reqwest` in `opendev-config` violates "pure config" principle
- 5 production `unsafe` blocks (all have `// SAFETY:` in v0.1.9)
- `server.rs` 临时桥接 (待删除)
- 事件命名混用(`mcp:status_changed` vs `mcp_status_update` vs `mcp.server.connected`)
- `CredentialStore` 未使用 OS keyring (TODO: zeroize)

### 11.2 Codex (从仓库推断)

(无公开 audit,但根据 887 个 release + 5k+ open issues 可推断持续维护中)

- frozen `protocol` + v1/v2 双轨(向后兼容策略)
- `core` 增长中(AGENTS.md 明确说 "resist adding code to codex-core",鼓励往 `tools/` 提取)
- 大量 `Bazel + cargo + just + nix` 多构建系统维护成本
- `core` 同时被 TUI / app-server / mcp-server / exec 共享,核心改动影响面大

---

## 12. 关键差异 Insights

### 12.1 设计哲学差异

**OpenDev = "广度优先的本地工程平台"**
- 把 agent 当作 **开发平台核心**,围绕它建 UI 矩阵 (TUI/CLI/REPL/Web/桌面/Telegram)
- 重视**工具广度**:40+ 工具覆盖 LSP/符号/AST/MCP/Plan/Team/Mailbox/Todo/Schedule 等
- 重视**多通道**:Telegram bot 是 first-class
- 架构风格:**严格分层** + 严格 Cargo workspace,composition root 单点,事件命名 `domain.object.action`
- 工程治理:**constitution + ADR + audit + remediation** 完整闭环
- 缺点:核心 `opendev-runtime` 是 kitchen sink,沙箱是 stub,事件协议有遗留命名混用

**Codex = "深度优先的协议化商用产品"**
- 把 agent 当作 **OpenAI 商业产品的核心能力**,通过 `app-server` JSON-RPC 协议驱动 N 个独立产品(CLI/TUI/App/VSCode/Web/Mobile)
- 重视**协议稳定性**:`protocol/` (内部) + `app-server-protocol/` (外部) v1/v2 双轨,W3C TraceContext,`#[experimental(...)]` 标记
- 重视**商业生态集成**:Connectors、Apps、Realtime voice、Codex Cloud、Attestation TEE、Remote Control (手机驱动)
- 重视**安全深度**:Guardian 自动化审批 + Starlark execpolicy + 4 类沙箱 (Seatbelt/Landlock/bwrap/Windows) + Split fs policy + Network policy
- 重视**开发者体验**:TS SDK + Python SDK + CLI 6+ 子命令 + 完整 AGENTS.md 散布
- 缺点:`core` 增长压力大(AGENTS.md 警告),Bazel + cargo + just + nix 多构建系统维护成本,Tauri 桌面由 `cli/src/desktop_app/` 触发但完整 UI 不在仓库

### 12.2 共有的"硬骨头"

两个项目都在解决同一组核心问题,只是路径不同:

| 问题 | OpenDev 解 | Codex 解 |
|---|---|---|
| LLM 上下文管理 | `opendev-context` 7 个子模块 (compaction/picker/env/retrieval/subdir/validated/worktree) | `core/src/{context,context_manager,compact*,agents_md*,session_prefix}.rs` + `<context_window>` XML 包装器 + `ContextualUserFragment` trait |
| Agent 协议 | `FrontendEvent` 23 变体 + `AgentEventCallback` 10 events | `Op` 25 变体 + `EventMsg` 50+ 变体 (含 Realtime/Guardian/Hook) |
| 工具沙箱化 | `BaseTool` async trait + `ToolRegistry` middleware + ToolSearch 延迟加载 | `ToolSpec` + `ConfiguredToolSpec` + `LoadableToolSpec` + `ResponsesApiNamespace` + MCP tool search |
| 会话持久化 | Event-sourced (10 SessionEvent + Tombstone + undo) | JSONL rollout + SQLite state |
| 审批策略 | `ApprovalRulesManager` pattern 匹配 + `AskUserTool` + `PlanApprovalRequest` | `AskForApproval` enum (UnlessTrusted/OnRequest/Granular/Never) + `Guardian` 自动化 + `request_permissions` tool |
| 多 agent | `SubagentManager` (6 内置) + `TeamManager` + `Mailbox` + `TeamTaskList` | `codex_delegate` + `core/src/agent/` + `agent-graph-store`(无 mailbox) |
| LSP 集成 | `opendev-tools-lsp` + `opendev-tools-symbol` (独立 crate) | 无 (Codex 不做 LSP,Codex 走 MCP/GPT 风格) |
| 沙箱 | 100% stub (P3) | 4 类跨平台 (macOS Seatbelt / Linux Landlock+bwrap / Windows elevated+unelevated) + Starlark 策略 |
| Memory | Short+long, FTS5, decay, WriteGate | `ext/memories/` + `memories/{read,write}/` + `MemoryCitation` |
| 桌面应用 | Tauri 2 自带 (深集成) | Codex App (仓库外) + `cli/src/desktop_app/` 触发 |

### 12.3 互不可替代的能力

**OpenDev 独有 (Codex 缺失):**
- Telegram bot (`opendev-channels`)
- Web 模式 (Axum + Argon2 + HMAC + cookie) (`opendev-web`)
- LSP 集成 + 符号级编辑 (5 个独立 tool)
- AST-grep 工具
- Codebase entity extraction retrieval(`CodebaseIndexer` + `EntityExtractor` + FTS 索引)
- Shadow git snapshot per-step undo (`SnapshotManager`)
- Event-sourced session (10 SessionEvent 变体 + Tombstone + undo)
- 6 变体 Todo 工具 + unified TodoTool
- Team / Mailbox 多 agent 协作子系统
- Worktree manager (adjective-noun 命名)
- Plan mode 5 阶段工作流 (显式 phase)
- Memory consolidation 后台 worker
- Sound / Matrix Rain 装饰
- Web screenshot / Browser automation / Open browser
- "Surface Ladder" 设计系统 (DESIGN.md 418 行规范)
- constitution + audit + remediation 完整治理闭环
- 9 themes (前端)

**Codex 独有 (OpenDev 缺失):**
- App-server JSON-RPC v1 + v2 公共协议(80+ 种 RPC 方法)
- VSCode / Cursor / Windsurf 扩展
- iOS / Android (Codex Mobile via ChatGPT app)
- Codex Web / Codex Cloud Tasks
- ChatGPT Connectors / Apps 集成
- Realtime WebRTC 语音 (`realtime-webrtc/` + `RealtimeVoice` 枚举)
- Image generation tool
- View image tool (本地图片 → 模型上下文)
- Code-mode (V8 JS 沙箱执行)
- Guardian 自动化审批 (风险等级 + 单次豁免)
- Starlark execpolicy DSL
- 完整 4 类跨平台沙箱 (Seatbelt/Landlock/bwrap/Windows)
- Split filesystem policy + Network policy + Network proxy + WPAD
- Per-thread goal (set/get/clear)
- Review mode (`Op::Review` + `EnteredReviewMode`/`ExitedReviewMode`)
- TS SDK `@openai/codex-sdk` + Python SDK
- Sentry 错误上报
- Analytics / telemetry / OpenAI compliance
- Attestation (TEE 证明)
- W3C TraceContext 协议级 trace
- `agent-graph-store` (有向图 agent 关系)
- `external-agent-migration` (从 Claude Code 导入配置)
- `responses_request_properties_match` (增量 WebSocket cache 友好)
- `bwrap` vendored + arg0 trick
- `apply_patch` 独立 crate + streaming parser
- `codex_doctor` 诊断命令
- Remote control (手机/Chrome 驱动 Codex)
- `dynamic_tools` (host 注入)
- `update_plan` tool (plan 模式作为 tool)
- `request_permissions` tool (运行时请求额外权限)
- `RealtimeOutputModality::{Text,Audio}`
- `Personality` 配置 + migration
- `service_tier_resolution`
- `RealtimeVoice` (Alloy/Arbor/.../Verse)
- `responses-api-proxy` (Node 代理)

### 12.4 互相借鉴建议

**OpenDev 可以从 Codex 学:**
1. **抽出 `app-server-protocol` 风格的稳定 wire protocol** — 当前 `FrontendEvent` 23 变体散落在前端 `WSMessage` + 后端 `FrontendEvent`,且命名混用
2. **完善 `opendev-sandbox`** — 4 类跨平台沙箱是必须的,Starlark DSL 写策略比纯 Rust `pattern match` 更易维护
3. **App-server 思路** — 当前 4 个 UI 共享 `MainAgent` 的紧耦合,可以演化为 `core` + `app-server` 的协议驱动,允许独立进程 / 独立 client
4. **Guardian 自动化审批** — 比纯 `ApprovalRulesManager` 强大
5. **OS keyring 集成** — 替换 `CredentialStore` 的 JSON 0600
6. **Sentry 错误上报** + **OTLP 全栈**
7. **Realtime 语音** — 如果产品要扩展
8. **AGENTS.md 发现** + **Memories** — 完善 `SubdirInstructionTracker` 流程
9. **`update_plan` tool 模式** — 把 plan 模式从 5 阶段硬编码改为 tool 调用
10. **image generation + view_image** — 多模态能力补全
11. **TypeScript SDK / Python SDK** — 允许第三方集成
12. **Bazel 多构建系统** — 当编译时间成为瓶颈时考虑

**Codex 可以从 OpenDev 学:**
1. **Telegram bot 多通道** — 如果想扩展到 Discord/Slack
2. **LSP 集成** — Codex 当前无 LSP,这是 symbol-level editing 的关键
3. **AST-grep 工具** — 强于纯 grep
4. **Symbol 工具** (find/references/rename/replace_body) — 精准编辑
5. **Shadow git snapshot** — 当前 Codex 没有 per-step undo
6. **Event-sourced session** — 当前 JSONL only,undo 不友好
7. **Memory consolidation 后台 worker** — 主动整理长期记忆
8. **Tauri 桌面应用的 React + Zustand 模板** — Codex App 不在仓库,但 OpenDev 经验可借鉴
9. **constitution + ADR + audit + remediation 完整治理** — 文档化决策过程
10. **9 主题 + Surface Ladder 设计** — 设计系统文档化
11. **Event naming convention (`domain.object.action`)** — 严格的命名约定
12. **Composition root 单点 + 严格分层** — 避免 `core` 膨胀

---

## 13. 一句话对比表

| 维度 | OpenDev Desktop | OpenAI Codex |
|---|---|---|
| **本质** | 本地工程平台,核心 agent + 5 端 UI + Telegram | OpenAI 商业 agent 产品线,核心 + 协议驱动 N 端 |
| **形态** | Tauri 桌面优先 | CLI 优先 + 桌面 App 旁路 |
| **架构** | 严格 4 层 + Cargo workspace | `core` + `app-server` + `tui/exec/cli` 多面 |
| **工具广度** | ⭐⭐⭐⭐⭐ (40+) | ⭐⭐⭐ (~10 核心 + MCP) |
| **协议稳定性** | ⭐⭐ (事件混用) | ⭐⭐⭐⭐⭐ (v1 frozen + v2 active + W3C trace) |
| **沙箱** | ⭐ (100% stub) | ⭐⭐⭐⭐⭐ (4 平台 + Starlark) |
| **LSP / 符号** | ⭐⭐⭐⭐⭐ | ⭐ (无) |
| **多通道** | ⭐⭐⭐⭐ (Telegram + Web) | ⭐⭐ (VSCode + Cursor + iOS + Android) |
| **多 agent** | ⭐⭐⭐⭐ (SubagentManager + Team + Mailbox) | ⭐⭐ (codex_delegate + graph store) |
| **多模态** | ⭐⭐ (VLM + screenshot + browser) | ⭐⭐⭐⭐⭐ (Realtime voice + image gen + view) |
| **SDK / 集成** | ⭐ (内嵌 core) | ⭐⭐⭐⭐⭐ (TS SDK + Python SDK + 扩展) |
| **工程治理** | ⭐⭐⭐⭐⭐ (constitution + ADR + audit + remediation) | ⭐⭐⭐ (AGENTS.md + 每 crate README) |
| **设计文档** | ⭐⭐⭐⭐⭐ (DESIGN.md 418 行) | ⭐⭐ (无显式 design system) |
| **商业成熟度** | ⭐ (pre-product) | ⭐⭐⭐⭐⭐ (商用 + 5k+ issues 维护) |
| **协议化程度** | ⭐⭐ (FrontendEvent 23 变体) | ⭐⭐⭐⭐⭐ (Op 25 + EventMsg 50+) |

---

## 14. 总结

**OpenDev Desktop** 是一个**工程深度极强**的本地 agent 平台:
- 23 个 crate 严格分层,composition root 单点
- 80+ 用户功能,覆盖 LSP/符号/AST/MCP/Memory/Plan/Team/Mailbox/Todo/Schedule/Channels 全套
- 工程治理闭环完善(constitution + 7 ADR + 504 行 audit + 702 行 remediation)
- 缺点:沙箱是 stub,事件协议有遗留命名混用,核心 `opendev-runtime` 是 kitchen sink,无 SDK/无云端

**OpenAI Codex** 是一个**商业化深度极强**的协议化 agent 产品:
- 75+ crate,核心 `Codex` + `app-server` JSON-RPC 协议(v1 frozen + v2 active)
- 50+ 用户功能,覆盖 Realtime voice / Code-mode V8 / Guardian / Starlark sandbox / Codex Cloud / Connectors / Apps / TEE attestation
- 通过 `app-server` 驱动 CLI / TUI / VSCode / Codex App / Codex Web / Codex Mobile / iOS / Android 全产品线
- 缺点:`core` 增长压力大(AGENTS.md 警告),多构建系统维护成本,UI 完整度在仓库外

两者在 **agent 核心 + 工具系统 + 上下文管理 + 会话持久化 + 审批策略** 上解决同一组问题,但**实现路径**截然不同:
- OpenDev 走"广度 + 严格分层"路线,**适合本地工程化使用**
- Codex 走"深度 + 协议驱动"路线,**适合商业产品集成**

**互补而非竞争**。OpenDev 适合:开发者本地多通道使用,IDE 集成,自托管;Codex 适合:商业产品集成,云端 + 桌面 + 移动多端,OpenAI 生态深度集成。

---

## 附录 A — 来源

- OpenDev Desktop recon: `~/.local/share/opencode/tool-output/tool_f0bad7e1d001BB5RQpZDWSwzBh` (本会话产出)
- OpenAI Codex recon: `~/.local/share/opencode/tool-output/tool_f0ba9df0e001Halm292BG03jtz` (本会话产出)
- OpenDev Desktop 源码: `/Users/van/projects/opendev-desktop`
- OpenAI Codex 仓库: https://github.com/openai/codex
  - 根 README: https://raw.githubusercontent.com/openai/codex/main/README.md
  - AGENTS.md: https://raw.githubusercontent.com/openai/codex/main/AGENTS.md
  - codex-rs README: https://raw.githubusercontent.com/openai/codex/main/codex-rs/README.md
  - core README: https://raw.githubusercontent.com/openai/codex/main/codex-rs/core/README.md
  - protocol README: https://raw.githubusercontent.com/openai/codex/main/codex-rs/protocol/README.md
  - app-server README: https://raw.githubusercontent.com/openai/codex/main/codex-rs/app-server/README.md
  - tools README: https://raw.githubusercontent.com/openai/codex/main/codex-rs/tools/README.md
  - 文档站: https://developers.openai.com/codex/cli
  - Releases: https://github.com/openai/codex/releases

## 附录 B — 文档交叉引用

- OpenDev Desktop 自有文档:
  - `ARCHITECTURE.md` (151 行,4 层架构概览)
  - `DESIGN.md` (418 行,Surface Ladder 设计系统)
  - `docs/constitution.md` (225 行,14 冻结原则)
  - `docs/architecture/crate-layering.md`
  - `docs/architecture/data-flow.md`
  - `docs/architecture/frontend.md`
  - `docs/architecture/security-model.md`
  - `docs/architecture/desktop-communication.md`
  - `docs/architecture/adr-0005-desktop-native-communication-architecture.md`
  - `docs/AUDIT_REPORT.md` (504 行)
  - `docs/REMEDIATION_PLAN.md` (702 行)
  - `docs/PROJECT_STATUS.md` (966 行)
  - `docs/roadmap.md` (v0.1.9 → v1.0)
  - 7 份 ADR (001-007)
  - 5 份 engineering 标准

— 完 —
