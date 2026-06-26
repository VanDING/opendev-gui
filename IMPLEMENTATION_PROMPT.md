## 任务背景
你正在 /Users/van/projects/opendev-desktop 项目中工作。这是一个 Tauri + React 桌面应用，底层是 OpenDev（开源 CLI AI 编程 agent，Rust，edition 2024，21 个 crate）。

## 项目现状
- `cargo check --workspace` ✅ 通过
- `cargo test --workspace --lib` ✅ 1,067/1,068 passed（1 个 OAuth 网络测试失败，忽略）
- `npm run build` ✅ 通过（React 前端，86 个 TSX 文件）
- 前端是完整的桌面 GUI：聊天、会话管理、模型设置、MCP 管理、审批、子 agent 可视化
- 后端是 OpenDev v0.1.8 全部 20 个 crate

## 要做的事（按阶段顺序，严格逐个实施）
我们将 Cabinet v3 的核心优势模块移植到 OpenDev 中。参考源码在 /Users/van/projects/cabinet-v3/。

详细计划见 /Users/van/projects/opendev-desktop/IMPLEMENTATION_PLAN.md，必须严格参照。

## 实施规则（必须遵守）

### 规则 1：阶段锁定
每轮对话只能实施一个阶段的一个任务。完成并验证后，方可进入下一个任务。不得提前实施后续阶段。

### 规则 2：验证门禁
每个任务完成后，必须运行且全部通过：
```bash
cargo check --workspace 2>&1 | tail -5  # 必须无 error
```
如有测试：
```bash
cargo test -p <crate-name> --lib 2>&1 | tail -5  # 必须全部 passed
```

### 规则 3：禁止跳过
不得跳过任何子任务。每个子任务必须完成并验证后才能继续。

### 规则 4：禁止偏离
严格参照 IMPLEMENTATION_PLAN.md 中的代码路径、文件结构。如有不确定，先查阅参考源码（/Users/van/projects/cabinet-v3/）和 OpenDev 源码。

### 规则 5：最小侵入
- 新增 crate 放在 crates/ 下
- 修改现有文件时，只改必要的部分
- 不改动前端 src/，除非明确要求
- 不改动 src-tauri/ 的 Tauri 配置，除非明确要求
- 不改动 OpenDev 核心 agent loop 的架构

---

## 阶段 0：基础设施（优先完成）

### 任务 0.1：添加 rusqlite 到 workspace
编辑 /Users/van/projects/opendev-desktop/Cargo.toml，在 [workspace.dependencies] 中添加：
- rusqlite = "0.40" (带 bundled-full features 在具体 crate 中启用)
- rusqlite_migration = "2.6"
- sha2 = "0.10"
- hex = "0.4"

### 任务 0.2：解除 SqliteSessionStore 的 stubs
文件: crates/opendev-history/src/sqlite_store.rs
当前状态：217 行，所有方法返回 Err("not yet implemented")
要求：逐个实现方法，每个方法实现后编译验证，确保所有方法都有实际逻辑（不是 stub）
需要实现的方法：open, save_session, load_session, list_sessions, delete_session, save_messages, load_messages, search_sessions

验证：cargo test -p opendev-history --lib

### 任务 0.3：crates/opendev-history/Cargo.toml 添加依赖
添加 rusqlite (bundled-full), rusqlite_migration, serde, serde_json

---

## 阶段 1：Memory 系统

### 任务 1.1：创建 opendev-memory crate
- mkdir -p crates/opendev-memory/src/tests
- 创建 Cargo.toml（依赖 rusqlite, serde, chrono, tokio, async-trait, uuid, parking_lot, thiserror, tracing）
- 创建 src/lib.rs（模块声明）
- 创建 src/types.rs：从 /Users/van/projects/cabinet-v3/crates/types/src/memory.rs 移植 MemoryEntry 等类型（去掉 cabinet-types 依赖，改为自有类型）
- 创建 src/error.rs：简化错误类型
- 创建 src/config.rs：MemoryConfig struct with Default
验证：cargo check -p opendev-memory（此时有未使用警告，正常）

### 任务 1.2：移植零依赖模块
从 /Users/van/projects/cabinet-v3/crates/memory/src/ 复制：
- write_gate.rs → crates/opendev-memory/src/write_gate.rs
- cascade.rs → crates/opendev-memory/src/cascade.rs
- decay.rs → crates/opendev-memory/src/decay.rs
- short_term.rs → crates/opendev-memory/src/short_term.rs
修改 crate:: import 路径以匹配新 crate 结构。
验证：cargo check -p opendev-memory

### 任务 1.3：实现 SQLite 持久化
- 创建 src/migration.rs：FTS5 DDL
- 创建 src/repo.rs：从 /Users/van/projects/cabinet-v3/crates/storage/src/repos/memory.rs 移植 MemoryRepo（去掉 cabinet-storage::Database 依赖，直接用 rusqlite::Connection）
- 创建 src/provider.rs：SqliteMemoryProvider 实现 MemoryProvider trait
验证：cargo check -p opendev-memory

### 任务 1.4：移植 MemoryFacade
从 /Users/van/projects/cabinet-v3/crates/memory/src/facade.rs 移植（966 行），简化：
- 去掉 Sideagent（用 Option<()> 替代，配置为 disabled）
- 去掉 SecretScanner
- 保留：save, recall_within_budget, list, delete, curate, push_turn, auto_nudge, 后台 flush worker
验证：cargo check -p opendev-memory

### 任务 1.5：编写测试并验证
从 Cabinet v3 移植测试，运行 cargo test -p opendev-memory --lib

### 任务 1.6：集成到 Agent Loop
- 新建 crates/opendev-agents/src/attachments/cabinet_memory_collector.rs（实现 ContextCollector trait，调用 MemoryFacade）
- 编辑 crates/opendev-agents/src/react_loop/loop_state.rs，替换 SemanticMemoryCollector + SessionMemoryCollector
- 编辑 crates/opendev-agents/Cargo.toml 添加 opendev-memory 依赖
验证：cargo check --workspace

---

## 阶段 2：Skill 系统增强

### 任务 2.1：扩展 SkillMetadata
文件：crates/opendev-agents/src/skills/metadata.rs
添加字段：pinned, status (Active/Stale/Archived/Superseded), requires_tools, fallback_for_tools, allowed_tools, usage_count, last_used, tags
参考：/Users/van/projects/cabinet-v3/crates/types/src/skill.rs
验证：cargo check -p opendev-agents

### 任务 2.2：实现 Curator
新建文件：crates/opendev-agents/src/skills/curator.rs
实现 Curator::curate() — Active→Stale(30d)→Archived(90d)，pinned 技能豁免，使用后重新激活
参考：/Users/van/projects/cabinet-v3/crates/skill/src/curator.rs
验证：cargo check -p opendev-agents

### 任务 2.3：实现条件可见性
新建文件：crates/opendev-agents/src/skills/visibility.rs
实现 is_visible(skill, available_tools) — 检查 requires_tools / fallback_for_tools / status
验证：cargo check -p opendev-agents

### 任务 2.4：实现 token-budgeted 注入
新建文件：crates/opendev-agents/src/skills/budget.rs
实现 skills_within_budget() — 按 pinned + usage_count 排序，token cap
验证：cargo check -p opendev-agents

### 任务 2.5：增强 SkillLoader
文件：crates/opendev-agents/src/skills/loader.rs
添加方法：record_usage(), set_pinned(), build_skills_index_with_budget(), curate()
验证：cargo check -p opendev-agents

### 任务 2.6：改造 InvokeSkillTool
文件：crates/opendev-tools-impl/src/invoke_skill/mod.rs
在成功调用技能后调用 record_usage()
验证：cargo check -p opendev-tools-impl

### 任务 2.7：测试
运行 cargo test -p opendev-agents --lib（回归）
编写新测试：curator, visibility, budget

---

## 阶段 3：Otel/Perfetto 可观测性

### 任务 3.1：创建 opendev-observability crate
- mkdir -p crates/opendev-observability/src
- 创建 Cargo.toml（依赖 tracing, tracing-subscriber, opentelemetry, opentelemetry_sdk, opentelemetry-otlp, tracing-opentelemetry, tracing-perfetto-writer）
- 创建 src/lib.rs, src/config.rs, src/guard.rs, src/error.rs
参考：/Users/van/projects/cabinet-v3/crates/otel/src/
关键：替换 cabinet_base::paths → opendev_config::Paths
验证：cargo check -p opendev-observability

### 任务 3.2：替换 init_tracing()
文件：crates/opendev-cli/src/helpers.rs
替换简单的 fmt() 为 opendev_observability::ObservabilityGuard::init()
验证：cargo check -p opendev-cli
验证：cargo check --workspace

---

## 阶段 4：成本追踪

### 任务 4.1：定义成本类型
新建 crates/opendev-history/src/cost/ 目录，创建 mod.rs, types.rs
定义 ModelPricing, CostRecord

### 任务 4.2：实现 CostTracker
新建 crates/opendev-history/src/cost/tracker.rs
从 /Users/van/projects/cabinet-v3/crates/gateway/src/cost.rs 移植
验证：cargo check -p opendev-history

### 任务 4.3：扩展 SQLite schema
文件：crates/opendev-history/src/sqlite_store.rs
添加 cost_records 表 migration 和相关方法
验证：cargo check -p opendev-history

### 任务 4.4：集成到 LLM 调用
找到 LLM 调用后记录 usage 的位置，添加 cost_tracker.record() 调用

### 任务 4.5：测试
运行 cargo test -p opendev-history --lib

---

## 阶段 5：Workflow Engine

### 任务 5.1：创建 opendev-workflow crate
- mkdir -p crates/opendev-workflow/src/tests
- 创建 Cargo.toml, src/lib.rs, src/types.rs, src/loops.rs, src/spawner.rs
- 从 /Users/van/projects/cabinet-v3/crates/agent/src/workflow/ 移植纯数据结构和循环逻辑
验证：cargo check -p opendev-workflow

### 任务 5.2：移植执行模式
创建 src/barrier.rs, src/pipeline.rs
将 ToolDispatcher 调用替换为 Box<dyn AgentSpawner> trait 方法
验证：cargo check -p opendev-workflow

### 任务 5.3：在 OpenDev 中实现 AgentSpawner
新建 crates/opendev-agents/src/workflow_spawner.rs
实现 AgentSpawner trait，调用 OpenDev 的 subagent 创建 API
验证：cargo check -p opendev-agents

### 任务 5.4：注册为工具
在 OpenDev tool registration point 注册 workflow 工具
验证：cargo check --workspace

---

## 阶段 6：集成收尾

### 任务 6.1：更新 src-tauri/src/server.rs
完整初始化所有新子系统（ObservabilityGuard, MemoryFacade, CostTracker, WorkflowEngine）

### 任务 6.2：更新 workspace Cargo.toml
添加新 crate 到 members 列表
添加 workspace 级依赖声明

### 任务 6.3：最终验证
```bash
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
cargo test --workspace --lib
npm run build
```

---

## 禁止事项
1. 不要修改 OpenDev 的 6 阶段 ReAct loop、5 角色模型路由、Planner subagent 设计
2. 不要引入 WASM plugin runtime（wasmtime）—— Cabinet 的 WASM 插件全部是 stubs，无实用价值
3. 不要删除 Cabinet 参考仓库（/Users/van/projects/cabinet-v3/）
4. 不要修改前端 src/ 除非明确要求（前端已可正常工作）
5. 每个任务完成后必须验证编译通过，不允许在编译失败时继续下一个任务

## 遇到困难时的做法
1. 先查阅参考代码（/Users/van/projects/cabinet-v3/）的实现
2. 再查阅 OpenDev 当前代码（/Users/van/projects/opendev-desktop/crates/）的上下文
3. 如果接口不匹配，优先选择最小适配——不要引入不必要的抽象层
4. 如果某个 Cabinet 特性过于复杂或依赖太深，标记为 "deferred"，继续下一个任务

## 开始实施
请从阶段 0 任务 0.1 开始。每完成一个子任务，汇报完成状态并验证，然后自动继续下一个子任务。确保每个子任务都有编译验证。
