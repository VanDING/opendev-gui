# Cabinet v3 → OpenDev 融合实施计划

> 基于 `opendev-desktop`（OpenDev fork + Tauri GUI），融入 Cabinet v3 的核心优势模块。
> 颗粒度级别：按文件/函数拆分，含具体代码路径、依赖预检、验证步骤。

---

## 前置条件

### 当前状态验证

| 检查项 | 状态 | 备注 |
|--------|------|------|
| `cargo check --workspace` | ✅ | `opendev-desktop` 根目录 |
| `cargo test --workspace --lib` | ✅ 1,067/1,068 passed | 1 个 OAuth 网络测试失败（非阻塞） |
| `cargo clippy --workspace -- -D warnings` | 待验证 | 可能有新 Rust 1.96 lint |
| `npm run build` | ✅ 1.09s | 前端构建通过 |
| `git status` | ⚠️ | 当前在工作目录，需要确认 clean |

### 已确认的依赖缺口

| 依赖 | OpenDev 现状 | 影响 |
|------|-------------|------|
| `rusqlite` | **无一已有** | 阻塞 memory 持久化、session SQLite、cost tracking |
| `rusqlite_migration` | **无** | 需要简单 migration 框架 |
| `opentelemetry` 相关 | **无** | 阻塞 Otel 移植 |
| `perfetto-writer` | **无** | 阻塞 Perfetto tracing |

### 已确认的现有基础设施

| 组件 | 路径 | 状态 |
|------|------|------|
| `SqliteSessionStore` | `crates/opendev-history/src/sqlite_store.rs` | 217 行已 scaffolded，全部方法返回 `Err("not yet implemented")` |
| `init_tracing()` | `crates/opendev-cli/src/helpers.rs` | 68 行简单 `fmt()` + `EnvFilter` |
| `SemanticMemoryCollector` | `crates/opendev-agents/src/react_loop/loop_state.rs:86` | 当前基于文件系统，是 memory 集成点 |
| `SkillLoader` | `crates/opendev-agents/src/skills/loader.rs` | 当前纯内存，是 skill 增强目标 |
| `InvokeSkillTool` | `crates/opendev-tools-impl/src/invoke_skill/mod.rs` | 工具注册点 |

---

## 阶段 0：基础设施 — rusqlite + SQLite 后端（3 天）

### 任务 0.1：添加 rusqlite 到 workspace 依赖

**文件**: `/Users/van/projects/opendev-desktop/Cargo.toml`

添加 workspace dependency:
```toml
rusqlite = { version = "0.40", features = ["bundled-full"] }
rusqlite_migration = "2.6"
sha2 = "0.10"
hex = "0.4"
```

### 任务 0.2：实现 SqliteSessionStore（解除 stubs）

**文件**: `crates/opendev-history/src/sqlite_store.rs`

当前状态：217 行 struct + SQL 常量 + 方法签名，全部返回 `Err("not yet implemented")`。

需要实现的方法（按顺序）：
1. `SqliteSessionStore::open(path) -> Result<Self>` — 打开/创建 SQLite 连接，WAL 模式，执行 migration
2. `SqliteSessionStore::save_session(session) -> Result<()>` — INSERT OR REPLACE
3. `SqliteSessionStore::load_session(id) -> Result<Option<Session>>` — SELECT
4. `SqliteSessionStore::list_sessions(filter) -> Result<Vec<Session>>` — SELECT with WHERE
5. `SqliteSessionStore::delete_session(id) -> Result<()>` — DELETE
6. `SqliteSessionStore::save_messages(id, messages) -> Result<()>` — batch INSERT
7. `SqliteSessionStore::load_messages(id) -> Result<Vec<ChatMessage>>` — SELECT
8. `SqliteSessionStore::search_sessions(query) -> Result<Vec<(String, Vec<usize>)>>` — LIKE search

**验证**:
```bash
cargo test -p opendev-history --lib
# 预期：所有 SQLite 测试通过
```

### 任务 0.3：在 CI/开发流程中加入 rusqlite

**文件**: `crates/opendev-history/Cargo.toml`

```toml
[dependencies]
rusqlite = { workspace = true }
rusqlite_migration = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
```

---

## 阶段 1：Memory 系统（5-7 天）

### 目标

用 Cabinet v3 的 5 层 memory pipeline 替换 OpenDev 的文件系统 markdown memory。

### 架构：新增 crate `opendev-memory`

```
crates/opendev-memory/
├── Cargo.toml
├── src/
│   ├── lib.rs               ← 公开 re-export
│   ├── types.rs             ← MemoryEntry, MemoryCategory, MemorySource, WriteGateTier, RecallOptions, MemoryProvider trait
│   ├── write_gate.rs        ← 噪音分类（77 行，零依赖，直接复制）
│   ├── cascade.rs           ← 批处理缓冲（124 行，直接复制）
│   ├── decay.rs             ← 衰减评分（124 行，直接复制）
│   ├── short_term.rs        ← 会话内短期记忆（111 行，直接复制）
│   ├── provider.rs          ← SQLite 持久化提供者（85 行，适配 rusqlite）
│   ├── repo.rs              ← MemoryRepo（SQLite + FTS5，449 行，适配）
│   ├── facade.rs            ← MemoryFacade 统一 API（966 行，简化版：无 Sideagent、无 SecretScanner）
│   ├── error.rs             ← 错误类型（39 行，简化版）
│   ├── config.rs            ← MemoryConfig（简单的 Rust struct）
│   ├── migration.rs         ← SQLite migration（FTS5 表创建）
│   └── tests/
│       ├── write_gate_tests.rs
│       ├── cascade_tests.rs
│       ├── decay_tests.rs
│       ├── short_term_tests.rs
│       └── facade_tests.rs
```

### 任务 1.1：创建 crate + types

**子任务**:
1. `mkdir -p crates/opendev-memory/src/tests`
2. 创建 `Cargo.toml`，依赖：`rusqlite`, `serde`, `chrono`, `tokio`, `async-trait`, `uuid`, `parking_lot`, `thiserror`, `tracing`
3. 创建 `types.rs`：从 Cabinet v3 复制以下类型，去掉 `cabinet-types` 依赖
   - `MemoryEntry` — struct
   - `MemoryCategory` — enum
   - `MemorySource` — enum  
   - `WriteGateTier` — enum
   - `DateTime` — `type DateTime = chrono::DateTime<chrono::Utc>`
   - `RecallOptions` — struct
   - `MemoryProvider` — async trait: `store()`, `recall()`, `list()`, `delete()`, `flush()`, `clear()`
   - `VerifiedMemory` — struct（为 Sideagent 预留，先 stub）
4. 验证: `cargo check -p opendev-memory`

### 任务 1.2：复制零依赖模块

从 `/Users/van/projects/cabinet-v3/crates/memory/src/` 复制：
1. `write_gate.rs` → `crates/opendev-memory/src/write_gate.rs`（77 行，纯字符串匹配）
2. `cascade.rs` → `crates/opendev-memory/src/cascade.rs`（124 行）
3. `decay.rs` → `crates/opendev-memory/src/decay.rs`（124 行）
4. `short_term.rs` → `crates/opendev-memory/src/short_term.rs`（111 行）

操作：直接文件复制，修改 `crate::` import 路径。

**验证**: `cargo check -p opendev-memory`

### 任务 1.3：实现 SQLite 持久化

1. 创建 `migration.rs`：FTS5 表 DDL
   ```sql
   CREATE TABLE IF NOT EXISTS long_term_memory (
       id TEXT PRIMARY KEY,
       content TEXT NOT NULL,
       category TEXT NOT NULL,
       confidence REAL NOT NULL DEFAULT 1.0,
       source TEXT NOT NULL,
       project_path TEXT NOT NULL,
       importance REAL NOT NULL DEFAULT 0.5,
       access_count INTEGER NOT NULL DEFAULT 0,
       last_accessed_at TEXT,
       expires_at TEXT,
       created_at TEXT NOT NULL,
       updated_at TEXT NOT NULL
   );
   CREATE VIRTUAL TABLE IF NOT EXISTS memory_fts USING fts5(content, category, project_path);
   ```

2. 创建 `repo.rs`：从 Cabinet v3 `storage/src/repos/memory.rs` 复制 `MemoryRepo`（449 行）
   - 替换 `Database`（`ReentrantMutex<rusqlite::Connection>`）为直接的 `rusqlite::Connection`
   - 去掉 `cabinet-storage::Database` 依赖

3. 创建 `provider.rs`：实现 `MemoryProvider` trait
   - `SqliteMemoryProvider` struct: `conn: Arc<Mutex<rusqlite::Connection>>`
   - 实现所有 trait 方法

4. 创建 `config.rs`：简单的 `MemoryConfig` struct（不依赖 `cabinet-base`）
   ```rust
   pub struct MemoryConfig {
       pub confidence_threshold: f64,      // default 0.7
       pub long_term_max_entries: usize,   // default 10000
       pub cascade_batch_size: usize,      // default 50
       pub cascade_flush_interval_ms: u64, // default 5000
       pub decay_half_life_days: i64,      // default 30
       pub decay_new_protection_days: i64, // default 7
       pub sideagent_enabled: bool,        // default false (skip for MVP)
   }
   impl Default for MemoryConfig { ... }
   ```

**验证**: `cargo check -p opendev-memory`

### 任务 1.4：实现 MemoryFacade（简化版）

从 Cabinet v3 `memory/src/facade.rs` 复制（966 行），做以下简化：

1. 去掉 `Sideagent` 模块（243 行）— 用 `Option<()>` 替代，`sideagent_enabled: false` 时跳过
2. 去掉 `SecretScanner` — OpenDev 有自己的 credential 处理
3. 去掉 `cabinet-base::config::MemoryConfig` — 使用自己的 `config.rs`
4. 保留：
   - `save()` — 入口：ShortTermMemory → WriteGate → Cascade/LTM
   - `recall_within_budget()` — token-budgeted 检索
   - `list()`, `delete()`, `curate()` — 管理操作
   - `push_turn()` — 短期记忆中记录每轮
   - `auto_nudge()` — 提示 agent 保存记忆
   - 后台 flush worker（tokio::spawn）

**验证**: `cargo check -p opendev-memory`

### 任务 1.5：编写测试

从 Cabinet v3 复制测试，适配新的模块路径：
- `write_gate_tests.rs`
- `cascade_tests.rs`
- `decay_tests.rs`
- `short_term_tests.rs`
- `facade_tests.rs`

```bash
cargo test -p opendev-memory --lib
# 预期：全部通过
```

### 任务 1.6：集成到 OpenDev Agent Loop

**文件**: `crates/opendev-agents/src/react_loop/loop_state.rs`

**改动**: 替换 `SemanticMemoryCollector` + `SessionMemoryCollector`

当前代码（第 80-88 行）：
```rust
Box::new(SemanticMemoryCollector::new(15)),
Box::new(SessionMemoryCollector::new()),
```

替换为新的 `CabinetMemoryCollector`：

**新建文件**: `crates/opendev-agents/src/attachments/cabinet_memory_collector.rs`

```rust
use opendev_memory::MemoryFacade;

pub struct CabinetMemoryCollector {
    facade: Arc<MemoryFacade>,
    max_items: usize,
}

impl ContextCollector for CabinetMemoryCollector {
    fn name(&self) -> &'static str { "cabinet_memory" }
    
    async fn collect(&self, ctx: &TurnContext<'_>) -> Option<Attachment> {
        let query = ctx.last_user_message()?;
        let project = ctx.working_dir.to_string_lossy();
        
        // Recall with token budget (e.g., 500 tokens)
        let entries = self.facade.recall_within_budget(
            &query, &project, 500
        ).await.ok()?;
        
        if entries.is_empty() { return None; }
        
        let content = format_memory_section(&entries);
        Some(Attachment::new("Memory", content))
    }
    
    // Auto-nudge after turn completion
    fn did_fire(&self, turn: usize) {
        if let Some(suggestion) = self.facade.auto_nudge(turn) {
            // Queue suggestion as system message
        }
    }
}
```

**文件**: `crates/opendev-agents/Cargo.toml` — 添加依赖：
```toml
opendev-memory = { path = "../opendev-memory" }
```

**文件**: `crates/opendev-agents/src/react_loop/loop_state.rs:80-88`

替换 collectors 注册：
```rust
// 旧: Box::new(SemanticMemoryCollector::new(15)),
// 旧: Box::new(SessionMemoryCollector::new()),
// 新:
Box::new(CabinetMemoryCollector::new(facade.clone(), 15)),
```

### 任务 1.7：在 src-tauri/server.rs 中初始化 MemoryFacade

**文件**: `src-tauri/src/server.rs`

1. 创建 memory DB 路径
2. 打开 MemoryFacade
3. 注入到 agent builder

```rust
// 在 build_server() 中添加：
let memory_db_path = paths.data_dir().join("memory.db");
let memory = MemoryFacade::open(&memory_db_path, MemoryConfig::default())
    .map_err(|e| format!("Failed to init memory: {}", e))?;
```

### 阶段 1 验证清单

```bash
# 单元测试
cargo test -p opendev-memory --lib

# 编译
cargo check --workspace

# 全量测试回归
cargo test --workspace --lib
# 预期：1,067+ passed（新增 memory 测试，原有全部通过）

# Clippy
cargo clippy --workspace -- -D warnings
```

---

## 阶段 2：Skill 系统增强（4-5 天）

### 目标

在 OpenDev 现有 SkillLoader 上添加 Cabinet v3 的 curation、conditional visibility、pinning、token-budgeted injection、usage tracking。

### 架构：增强现有代码，不新建 crate

```
opendev-agents/src/skills/
├── metadata.rs    ← 扩展 SkillMetadata（+6 字段）
├── loader.rs      ← 增强 SkillLoader（+5 方法）
├── curator.rs     ← 新增：生命周期管理
├── visibility.rs  ← 新增：条件可见性检查
├── budget.rs      ← 新增：token-budgeted 注入
├── discovery.rs   ← 保留（不加 Cabinet 特性）
├── parsing.rs     ← 保留（不加 Cabinet 特性）
└── builtins.rs    ← 保留（不加 Cabinet 特性）
```

### 任务 2.1：扩展 SkillMetadata

**文件**: `crates/opendev-agents/src/skills/metadata.rs`

添加字段：
```rust
pub struct SkillMetadata {
    // --- 现有字段（保留）---
    pub name: String,
    pub description: String,
    pub namespace: Option<String>,
    pub source: SkillSource,
    pub model: Option<String>,
    pub agent: Option<String>,
    pub path: PathBuf,
    
    // --- 新增 Cabinet 字段 ---
    pub pinned: bool,                    // false by default
    pub status: SkillStatus,             // Active, Stale, Archived, Superseded
    pub requires_tools: Option<Vec<String>>,  // hidden unless all tools available
    pub fallback_for_tools: Option<Vec<String>>, // hidden when any listed tool available
    pub allowed_tools: Option<Vec<String>>,    // tools this skill can use
    pub usage_count: u64,                // invocation counter
    pub last_used: Option<DateTime>,     // last invocation time
    pub tags: Vec<String>,               // discovery tags
}
```

新增 enum：
```rust
pub enum SkillStatus {
    Active,
    Stale,
    Archived,
    Superseded,
}
```

**验证**: `cargo check -p opendev-agents`

### 任务 2.2：实现 Curator

**新建文件**: `crates/opendev-agents/src/skills/curator.rs`

从 Cabinet v3 `skill/src/curator.rs` 移植规则（~90 行）：

```rust
pub struct Curator;

impl Curator {
    /// Apply lifecycle rules to all loaded skills
    pub fn curate(skills: &mut HashMap<String, LoadedSkill>) -> Vec<String> {
        let now = chrono::Utc::now();
        let mut log = Vec::new();
        
        for (name, skill) in skills.iter_mut() {
            if skill.metadata.pinned { continue; } // pinned skills exempt
            
            match skill.metadata.status {
                SkillStatus::Active => {
                    // Active + 30 days unused → Stale
                    if let Some(last) = skill.metadata.last_used {
                        if now - last > Duration::days(30) {
                            skill.metadata.status = SkillStatus::Stale;
                            log.push(format!("{name}: Active → Stale"));
                        }
                    }
                }
                SkillStatus::Stale => {
                    // Stale + recently used → reactivate
                    if let Some(last) = skill.metadata.last_used {
                        if now - last < Duration::days(7) {
                            skill.metadata.status = SkillStatus::Active;
                            log.push(format!("{name}: Stale → Active (reactivated)"));
                            continue;
                        }
                    }
                    // Stale + 90 days unused → Archived
                    if let Some(last) = skill.metadata.last_used {
                        if now - last > Duration::days(90) {
                            skill.metadata.status = SkillStatus::Archived;
                            log.push(format!("{name}: Stale → Archived"));
                        }
                    }
                }
                _ => {}
            }
        }
        log
    }
}
```

**验证**: `cargo check -p opendev-agents`

### 任务 2.3：实现条件可见性检查

**新建文件**: `crates/opendev-agents/src/skills/visibility.rs`

```rust
/// Returns true if the skill should be visible given the currently available tools.
pub fn is_visible(skill: &LoadedSkill, available_tools: &HashSet<String>) -> bool {
    // Archived/superseded skills are hidden
    if skill.metadata.status == SkillStatus::Archived 
    || skill.metadata.status == SkillStatus::Superseded {
        return false;
    }
    
    // requires_tools: hidden unless ALL required tools are available
    if let Some(required) = &skill.metadata.requires_tools {
        if !required.iter().all(|t| available_tools.contains(t.as_str())) {
            return false;
        }
    }
    
    // fallback_for_tools: hidden when ANY listed tool IS available
    if let Some(fallbacks) = &skill.metadata.fallback_for_tools {
        if fallbacks.iter().any(|t| available_tools.contains(t.as_str())) {
            return false;
        }
    }
    
    true
}
```

### 任务 2.4：实现 token-budgeted 注入

**新建文件**: `crates/opendev-agents/src/skills/budget.rs`

从 Cabinet v3 `skill/src/registry.rs` 移植 `list_active_for_injection()` 逻辑：

```rust
/// Returns skills for prompt injection, capped by token budget.
/// Sorts by: pinned first, then usage_count descending.
pub fn skills_within_budget(
    skills: &HashMap<String, LoadedSkill>,
    available_tools: &HashSet<String>,
    token_budget: usize,
    max_count: usize,
) -> Vec<&LoadedSkill> {
    let mut visible: Vec<&LoadedSkill> = skills.values()
        .filter(|s| is_visible(s, available_tools))
        .collect();
    
    // Sort: pinned first, then by usage_count descending
    visible.sort_by(|a, b| {
        b.metadata.pinned.cmp(&a.metadata.pinned)       // pinned = priority
            .then(b.metadata.usage_count.cmp(&a.usage_count))  // most used
    });
    
    // Cap by token budget: ~30 chars per skill name+desc → ~30 tokens each
    let cap = max_count.min(token_budget / 30).min(20);  // hard cap at 20
    visible.truncate(cap);
    
    visible
}
```

### 任务 2.5：增强 SkillLoader

**文件**: `crates/opendev-agents/src/skills/loader.rs`

添加方法：

```rust
impl SkillLoader {
    /// Record that a skill was invoked.
    pub fn record_usage(&mut self, name: &str) {
        if let Some(skill) = self.skills.get_mut(name) {
            skill.metadata.usage_count += 1;
            skill.metadata.last_used = Some(chrono::Utc::now());
            if skill.metadata.status == SkillStatus::Stale {
                skill.metadata.status = SkillStatus::Active;
            }
        }
    }
    
    /// Pin/unpin a skill to prevent override and curation.
    pub fn set_pinned(&mut self, name: &str, pinned: bool) -> Result<()> {
        if let Some(skill) = self.skills.get_mut(name) {
            skill.metadata.pinned = pinned;
            Ok(())
        } else {
            Err(SkillError::NotFound(name.to_string()))
        }
    }
    
    /// Build skills index with token-budget awareness.
    pub fn build_skills_index_with_budget(
        &self,
        available_tools: &HashSet<String>,
        token_budget: usize,
    ) -> String {
        let visible = skills_within_budget(&self.skills, available_tools, token_budget, 20);
        self.format_skills_list(&visible)
    }
    
    /// Run curation pass (call periodically, e.g., on session start).
    pub fn curate(&mut self) -> Vec<String> {
        Curator::curate(&mut self.skills)
    }
}
```

### 任务 2.6：改造 InvokeSkillTool

**文件**: `crates/opendev-tools-impl/src/invoke_skill/mod.rs`

在成功调用技能后添加：
```rust
// After successful skill invocation:
if let Some(ref loader) = self.loader {
    let mut loader = loader.lock().unwrap();
    loader.record_usage(&skill_name);
}
```

### 任务 2.7：添加前端管理接口（可选，1 天）

**文件**: `crates/opendev-web/src/` — 添加 REST 端点：
- `GET /api/skills` — 列出所有技能（含 status, usage_count）
- `POST /api/skills/{name}/pin` — 固定/取消固定
- `POST /api/skills/curate` — 手动触发 curation

### 阶段 2 验证清单

```bash
# 现有测试回归
cargo test -p opendev-agents --lib

# 新增功能测试（手动编写）
# - curator: test Active→Stale→Archived transition
# - visibility: test requires_tools / fallback_for_tools gating
# - budget: test token cap and sort order

# 编译
cargo check --workspace

# 集成回归
cargo test --workspace --lib
```

---

## 阶段 3：Otel/Perfetto 可观测性（2-3 天）

### 目标

添加生产级可观测性：OTLP 导出、Perfetto tracing、结构化日志。

### 架构：新增 crate `opendev-observability`

```
crates/opendev-observability/
├── Cargo.toml
├── src/
│   ├── lib.rs           ← re-export
│   ├── config.rs        ← ObservabilityConfig（120 行，从 cabinet-otel 移植）
│   ├── guard.rs         ← ObservabilityGuard（387 行，从 cabinet-otel 移植）
│   └── error.rs         ← 错误类型（52 行，简化）
```

### 任务 3.1：创建 crate

**子任务**:
1. `mkdir -p crates/opendev-observability/src`
2. 创建 `Cargo.toml`，依赖：
   ```toml
   [dependencies]
   tracing = { workspace = true }
   tracing-subscriber = { workspace = true, features = ["env-filter", "json"] }
   opentelemetry = "0.32"
   opentelemetry_sdk = { version = "0.32", features = ["rt-tokio"] }
   opentelemetry-otlp = "0.32"
   tracing-opentelemetry = "0.33"
   tracing-perfetto-writer = "0.3"
   serde = { workspace = true }
   opendev-config = { path = "../opendev-config" }
   ```

### 任务 3.2：从 cabinet-otel 移植代码

从 `/Users/van/projects/cabinet-v3/crates/otel/src/` 复制：
1. `config.rs` → 适配 `opendev_config::Paths`（替换 `cabinet_base::paths`）
2. `guard.rs` → 替换 `cabinet_types::SessionId` → `String`
3. `error.rs` → 去掉 `CabinetError` trait impl

关键适配：
```rust
// 旧: cabinet_base::paths::CabinetPaths
// 新: opendev_config::Paths

pub fn init(config: ObservabilityConfig, paths: &Paths) -> Result<Self> {
    let log_dir = paths.data_dir().join("logs");
    // ...
}
```

### 任务 3.3：替换 init_tracing()

**文件**: `crates/opendev-cli/src/helpers.rs`

```rust
// 旧: init_tracing() → 简单的 fmt() + EnvFilter

// 新:
use opendev_observability::{ObservabilityGuard, ObservabilityConfig};

pub fn init_tracing(paths: &Paths) -> ObservabilityGuard {
    let config = ObservabilityConfig {
        level: std::env::var("OPENDEV_LOG").unwrap_or_else(|_| "info".into()),
        otlp_endpoint: std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok(),
        perfetto_enabled: std::env::var("OPENDEV_PERFETTO").is_ok(),
        ..Default::default()
    };
    ObservabilityGuard::init(config, paths)
        .expect("Failed to initialize observability")
}
```

**文件**: `src-tauri/src/server.rs`

注入 `ObservabilityGuard` 到 app state，窗口关闭时调用 `guard.shutdown()`。

### 阶段 3 验证清单

```bash
cargo check --workspace
cargo test -p opendev-observability --lib
# 设置 OPENDEV_LOG=debug 验证日志输出
# 设置 OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317 验证 OTLP
```

---

## 阶段 4：Session Cost Tracking（3-4 天）

### 目标

在 OpenDev session 中添加 per-event 成本追踪。

### 架构：在 opendev-history 中添加成本模块

```
crates/opendev-history/src/
├── cost/
│   ├── mod.rs           ← 模块声明
│   ├── tracker.rs       ← CostTracker（内存累加器，~150 行）
│   ├── repo.rs          ← CostRepo（SQLite，~150 行）
│   └── types.rs         ← PricingConfig, CostRecord
├── sqlite_store.rs      ← 扩展：cost_records 表 migration
```

### 任务 4.1：定义成本类型

**新建文件**: `crates/opendev-history/src/cost/types.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    pub prompt_price_per_1k: f64,       // USD per 1K prompt tokens
    pub completion_price_per_1k: f64,   // USD per 1K completion tokens
    pub cache_read_price_per_1k: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostRecord {
    pub session_id: String,
    pub model: String,
    pub provider: String,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub cache_read_tokens: Option<u64>,
    pub cache_write_tokens: Option<u64>,
    pub thinking_tokens: Option<u64>,
    pub cost_usd: f64,
    pub created_at: String,
}
```

### 任务 4.2：实现 CostTracker

**新建文件**: `crates/opendev-history/src/cost/tracker.rs`

从 Cabinet v3 `gateway/src/cost.rs` 移植（217 行），适配：
- 去掉 `cabinet-gateway-types` 依赖，使用自有 types
- `record_usage()` 累加 prompt/completion/cache tokens
- `total_cost()` 返回 USD
- `total_tokens()` 返回 split 计数

### 任务 4.3：扩展 SqliteSessionStore schema

**文件**: `crates/opendev-history/src/sqlite_store.rs`

添加 migration:
```sql
CREATE TABLE IF NOT EXISTS cost_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,
    model TEXT NOT NULL,
    provider TEXT NOT NULL,
    prompt_tokens INTEGER NOT NULL DEFAULT 0,
    completion_tokens INTEGER NOT NULL DEFAULT 0,
    cache_read_tokens INTEGER,
    cache_write_tokens INTEGER,
    thinking_tokens INTEGER,
    cost_usd REAL NOT NULL DEFAULT 0.0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (session_id) REFERENCES sessions(id)
);
CREATE INDEX IF NOT EXISTS idx_cost_records_session ON cost_records(session_id);
```

### 任务 4.4：集成到 LLM 调用路径

**文件**: `crates/opendev-agents/src/agent_client.rs`（或调用 LLM 的地方）

在每次 LLM 响应后记录成本：
```rust
// After getting LLM response:
if let Some(usage) = response.usage {
    cost_tracker.record(
        &session_id,
        &model_name,
        &provider_name,
        usage.prompt_tokens,
        usage.completion_tokens,
        usage.cache_read_tokens,
        usage.thinking_tokens,
    ).await;
}
```

### 任务 4.5：WebSocket 推送成本信息

**文件**: `crates/opendev-web/src/protocol.rs`

在 `WsMessageType` 枚举中添加：
```rust
SessionCost { session_id: String, cost_usd: f64, total_tokens: u64 },
```

### 阶段 4 验证清单

```bash
cargo test -p opendev-history --lib
cargo check --workspace
# 手动：运行一个 session，验证 cost_records 表写入
```

---

## 阶段 5：Workflow Engine（5-7 天）

### 目标

从 Cabinet v3 提取 workflow engine（pipeline/barrier/loop），作为 OpenDev 的独立编排层。

### 架构：新增 crate + AgentSpawner trait

```
crates/opendev-workflow/
├── Cargo.toml
├── src/
│   ├── lib.rs           ← re-export
│   ├── types.rs         ← WorkflowDefinition, WorkflowPhase, LoopConfig, LoopState（164 行，零依赖）
│   ├── barrier.rs       ← 屏障模式（74 行，需 AgentSpawner trait）
│   ├── pipeline.rs      ← 流水线模式（125 行，需 AgentSpawner trait）
│   ├── loops.rs         ← 循环模式（61 行，纯状态机）
│   ├── spawner.rs       ← AgentSpawner trait 定义
│   └── tests/
│       ├── types_tests.rs
│       ├── loops_tests.rs
│       └── integration_tests.rs
```

### 任务 5.1：提取纯数据 types

从 `/Users/van/projects/cabinet-v3/crates/agent/src/workflow/` 复制：
1. `mod.rs` → `types.rs`（WorkflowDefinition, WorkflowPhase, LoopConfig）
2. `loops.rs` → `loops.rs`（LoopState, loop_until_count, loop_until_budget, loop_until_dry）
3. `persistence.rs` → 合并到 `types.rs`

这些是纯数据 + 纯逻辑，零外部依赖。

### 任务 5.2：定义 AgentSpawner trait

**新建文件**: `crates/opendev-workflow/src/spawner.rs`

```rust
use async_trait::async_trait;

#[async_trait]
pub trait AgentSpawner: Send + Sync {
    /// Spawn multiple sub-agents with the same template and different items.
    async fn spawn_subagents(
        &self,
        agent_type: &str,
        items: &[String],
        prompt_template: &str,
    ) -> Result<Vec<AgentResult>, WorkflowError>;

    /// Spawn a single sub-agent with a complete prompt.
    async fn spawn_single_subagent(
        &self,
        agent_type: &str,
        prompt: &str,
    ) -> Result<AgentResult, WorkflowError>;
}

#[derive(Debug, Clone)]
pub struct AgentResult {
    pub agent_name: String,
    pub output: String,
    pub success: bool,
}
```

### 任务 5.3：移植执行模式

从 Cabinet v3 移植 `barrier.rs`、`pipeline.rs`，将 `ToolDispatcher` 调用替换为 `Box<dyn AgentSpawner>` trait 方法。

**barrier.rs 关键改动**：
```rust
// 旧: dispatcher.spawn_subagents(agent_type, &all_items, prompt)
// 新: spawner.spawn_subagents(agent_type, &all_items, prompt).await

// 旧: dispatcher.spawn_single_subagent(agent_type, &prompt)
// 新: spawner.spawn_single_subagent(agent_type, &prompt).await
```

### 任务 5.4：在 OpenDev 中实现 AgentSpawner

**新建文件**: `crates/opendev-agents/src/workflow_spawner.rs`

```rust
use opendev_workflow::{AgentSpawner, AgentResult, WorkflowError};

pub struct OpenDevAgentSpawner {
    agent_factory: Arc<AgentFactory>,
    config: AppConfig,
}

#[async_trait]
impl AgentSpawner for OpenDevAgentSpawner {
    async fn spawn_subagents(
        &self,
        agent_type: &str,
        items: &[String],
        prompt_template: &str,
    ) -> Result<Vec<AgentResult>, WorkflowError> {
        // Use OpenDev's existing subagent spawning
        let handles: Vec<_> = items.iter().map(|item| {
            let prompt = prompt_template.replace("{item}", item);
            let agent = self.agent_factory.create_agent(agent_type)?;
            tokio::spawn(async move { agent.run_sync(&prompt).await })
        }).collect();
        
        let results = futures::future::join_all(handles).await;
        // Map to AgentResult...
        Ok(results)
    }
    
    async fn spawn_single_subagent(
        &self,
        agent_type: &str,
        prompt: &str,
    ) -> Result<AgentResult, WorkflowError> {
        let agent = self.agent_factory.create_agent(agent_type)?;
        let output = agent.run_sync(prompt).await?;
        Ok(AgentResult { agent_name: agent_type.to_string(), output, success: true })
    }
}
```

### 任务 5.5：注册为工具

**文件**: OpenDev tool registration point

将 workflow engine 注册为工具 `"workflow"`：
```rust
registry.register(WorkflowTool::new(workflow_engine, spawner));
```

### 阶段 5 验证清单

```bash
cargo test -p opendev-workflow --lib
cargo check --workspace
cargo test --workspace --lib
```

---

## 阶段 6：集成与收尾（3-5 天）

### 任务 6.1：src-tauri/server.rs 完整初始化

**文件**: `src-tauri/src/server.rs`

```rust
use opendev_memory::{MemoryFacade, MemoryConfig};
use opendev_observability::ObservabilityGuard;

pub struct AppHandle {
    server_handle: ServerHandle,
    observability_guard: ObservabilityGuard,
    memory_facade: Arc<MemoryFacade>,
}

pub fn build_app(working_dir: &Path) -> Result<AppHandle, String> {
    let paths = Paths::new(Some(working_dir.to_path_buf()));
    
    // 1. Observability (first — capture all subsequent logs)
    let obs_guard = init_tracing(&paths);
    
    // 2. Config
    let config = ConfigLoader::load(/* ... */)?;
    
    // 3. Session + Cost tracker
    let session_manager = SessionManager::new(/* ... */)?;
    let cost_tracker = Arc::new(CostTracker::new());
    
    // 4. Memory
    let memory_db = paths.data_dir().join("memory.db");
    let memory = Arc::new(MemoryFacade::open(&memory_db, MemoryConfig::default())?);
    
    // 5. Skill loader (with enhanced features)
    let skill_loader = SkillLoader::new(/* ... */);
    
    // 6. Agent factory (inject all new subsystems)
    let agent_factory = AgentFactory::new(
        config, registry, skill_loader,
        memory.clone(), cost_tracker.clone(),
    );
    
    // 7. Workflow engine
    let workflow_spawner = OpenDevAgentSpawner::new(agent_factory.clone());
    let workflow_engine = WorkflowEngine::new(Arc::new(workflow_spawner));
    
    // 8. Axum server
    let state = AppState::new(/* ... */);
    let router = build_app(state, None);
    
    // ... start server
}
```

### 任务 6.2：Cargo.toml workspace 更新

**文件**: `/Users/van/projects/opendev-desktop/Cargo.toml`

```toml
[workspace]
members = [
    "src-tauri",
    "crates/opendev-models",
    # ... existing 20 crates ...
    "crates/opendev-memory",          # 新增
    "crates/opendev-observability",   # 新增
    "crates/opendev-workflow",        # 新增
]

[workspace.dependencies]
# ... existing ...
rusqlite = "0.40"                     # 新增（带 bundled-full features）
rusqlite_migration = "2.6"           # 新增
sha2 = "0.10"                         # 新增
hex = "0.4"                           # 新增
opentelemetry = "0.32"               # 新增
opentelemetry_sdk = { version = "0.32", features = ["rt-tokio"] }
opentelemetry-otlp = "0.32"         # 新增
tracing-opentelemetry = "0.33"      # 新增
tracing-perfetto-writer = "0.3"     # 新增
```

### 任务 6.3：前端 Settings UI 扩展（可选）

**文件**: `src/components/Settings/`

1. `MemorySettings.tsx` — memory 配置面板（置信度阈值、最大条目数、衰减期）
2. `SkillManagement.tsx` — 技能管理面板（状态、固定、curation 触发）
3. `CostDisplay.tsx` — 成本显示组件（集成到 StatusBar）

### 任务 6.4：最终验证

```bash
# 1. 全部编译
cargo check --workspace --all-targets
cargo build --release

# 2. Lint
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check

# 3. 全部测试
cargo test --workspace --lib
cargo test --workspace --test '*'

# 4. 前端
npm run build

# 5. 桌面应用
npm run tauri build    # 生产构建
npm run tauri dev      # 开发模式验证
```

---

## 时间线总览

```
Week 1-2:  Phase 0 (rusqlite) + Phase 1 (memory)
Week 3:    Phase 2 (skill enhancement)
Week 4:    Phase 3 (otel/perfetto)
Week 5:    Phase 4 (cost tracking)
Week 6-7:  Phase 5 (workflow engine)
Week 8:    Phase 6 (integration + testing)
─────────────────────────────────────────
Total:     8 weeks
```

## 依赖关系图

```
Phase 0 (rusqlite/SQLite) ──────────────────────────────────────────┐
  │                                                                  │
  ├── Phase 1 (memory) ←── needs rusqlite ─────────────────────────┤
  │                                                                  │
  ├── Phase 2 (skill) ←── NO rusqlite needed (pure logic) ─────────┤
  │     └── 可与 Phase 1 并行                                        │
  │                                                                  │
  ├── Phase 3 (otel) ←── NO rusqlite needed (纯 wrapper) ──────────┤
  │     └── 可与 Phase 1-2 并行                                      │
  │                                                                  │
  ├── Phase 4 (cost) ←── needs rusqlite + SqliteSessionStore ──────┤
  │     └── 需要 Phase 0 完成                                        │
  │                                                                  │
  └── Phase 5 (workflow) ←── needs AgentSpawner trait (新 trait) ──┤
        └── 需要理解 OpenDev 的 subagent API，但无其他依赖            │

Phase 6 (integration) ←── needs all phases complete
```

## 风险矩阵

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|---------|
| `rusqlite` 编译跨平台问题 | 低 | 高 | 使用 `bundled-full` feature，预编译 SQLite |
| Memory facade 集成导致 agent 崩溃 | 中 | 高 | 渐进集成：先只注入 collector，不替换现有 |
| Skill metadata 扩展破坏现有前端 | 中 | 中 | 保持向后兼容的 JSON 序列化 |
| Workflow engine 与 OpenDev subagent API 不匹配 | 高 | 中 | 先定义 AgentSpawner trait，验证适配 |
| Tauri 跨平台打包问题 | 中 | 中 | 只专注 macOS MVP，Linux/Windows 后置 |
| OpenDev 上游更新导致冲突 | 高 | 低 | 我们的改动在独立 crates 中，冲突面小 |
