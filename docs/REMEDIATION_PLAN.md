# OpenDev Desktop — 工程整改计划（Engineering Remediation Plan）

> **项目**：OpenDev Desktop v0.1.8  
> **基于审计报告**：AUDIT_REPORT.md（2026-06-26）  
> **整改范围**：29 个问题 · 34 个 GitHub Issues · 12 个 PRs · 90 天路线图  
> **预计总工时**：18-24 人天（含验证）

---

## 执行摘要

**项目当前状态**：v0.1.8，早期开发阶段。架构分层意识清晰，适配器模式应用得当，测试覆盖充分（3,183 测试）。主要风险集中在：(1) 缺少 CI/CD 自动化导致质量退化无守卫，(2) SSRF 安全漏洞 + Bedrock 适配器功能缺失，(3) async Rust 惯用法违规增加生产环境不稳定风险。

**关键发现**：大部分问题修复成本低（Easy/Trivial），但 Sandbox 桩代码、tools→agents 依赖倒置、CLI God Object 等架构问题如需彻底重构则成本高且当前 ROI 低，建议推迟到 v0.3+。

**整改策略**：P0 立即修复安全漏洞 + 建立 CI 防线；P1 一个月内提升运行时稳定性 + 性能优化；P2/P3 架构重构视资源逐步推进，避免为了"架构漂亮"进行无价值重构。

---

## 第一阶段：问题重新分类

### Security（安全）

| ID | 问题 | CWE | 严重度 | 置信度 |
|----|------|-----|--------|--------|
| SEC-01 | SSRF：WebFetch 无私有 IP 过滤 | CWE-918 | HIGH | 高 |
| SEC-02 | 5 个 production unsafe 块中 3 个缺 `// SAFETY:` 注释 | RustSec | MEDIUM | 高 |
| SEC-03 | Default HMAC key `"change-me-in-production"` | CWE-312 | LOW | 高 |
| SEC-04 | Session Cookie 缺 `Secure` flag | CWE-614 | LOW | 中 |
| SEC-05 | LIKE 通配符未转义 | CWE-89 变体 | LOW-MEDIUM | 高 |
| SEC-06 | API keys 明文存 `CredentialStore.cache`，无 zeroize | CWE-312 | LOW | 中 |
| SEC-07 | 33 个测试 unsafe 块（`env::set_var`）线程不安全 | 测试 | LOW | 高 |

### Reliability（稳定性）

| ID | 问题 | 严重度 | 置信度 |
|----|------|--------|--------|
| REL-01 | Mutex 中毒 → 进程崩溃（25+ 处 `.expect("poisoned")`） | MEDIUM | 高 |
| REL-02 | `std::sync::RwLock` 在 async 上下文中 | MEDIUM | 高 |
| REL-03 | `block_on` hack in `sqlite_store.rs` | MEDIUM | 高 |
| REL-04 | Bedrock 适配器 SigV4 签名缺失 | HIGH | 高 |
| REL-05 | Sandbox crate 100% 桩代码 | MEDIUM | 高 |
| REL-06 | 无界通道（unbounded channels）无背压 | LOW-MEDIUM | 高 |
| REL-07 | `adapted_client.rs:114` — `as_object_mut().unwrap()` on 格式错误 JSON | LOW | 高 |

### Performance（性能）

| ID | 问题 | 严重度 | 置信度 |
|----|------|--------|--------|
| PERF-01 | HTML 转换器每次调用重编译 ~17 个正则 | HIGH | 高 |
| PERF-02 | 阻塞 I/O 无 `spawn_blocking` | HIGH | 高 |
| PERF-03 | `prepare_command` 每次 bash 调用重编译正则 | MEDIUM | 高 |
| PERF-04 | `tool_dispatch.rs` — `args_map` 被克隆 3+ 次 | LOW | 中 |

### Architecture（架构）

| ID | 问题 | 严重度 | 置信度 |
|----|------|--------|--------|
| ARC-01 | `opendev-tools-impl` → `opendev-agents` 依赖倒置 | MEDIUM | 高 |
| ARC-02 | CLI God Object（15 个内部 crate 依赖） | LOW-MEDIUM | 高 |
| ARC-03 | `opendev-runtime` 杂货袋（83 文件） | LOW-MEDIUM | 高 |
| ARC-04 | `opendev-memory` 和 `opendev-history` 各自独立 SQLite | LOW | 高 |
| ARC-05 | `ProviderAdapter` `#[async_trait]` on 纯同步 trait | LOW | 高 |
| ARC-06 | `MainAgent::run()` 14 参数 | LOW | 高 |
| ARC-07 | `opendev-config` 依赖 `reqwest`+`tokio` | LOW | 高 |
| ARC-08 | `opendev-models` 依赖 `ts-rs` | LOW | 高 |

### Engineering（工程化）

| ID | 问题 | 严重度 | 置信度 |
|----|------|--------|--------|
| ENG-01 | 无 CI/CD | HIGH | 高 |
| ENG-02 | 无 `README.md`、无架构文档、无贡献指南 | MEDIUM | 高 |
| ENG-03 | `dirs` v5 和 v6 版本不一致 | LOW | 高 |
| ENG-04 | `proptest` 已声明依赖但未使用 | LOW | 高 |
| ENG-05 | 无 `rustfmt.toml` / `clippy.toml` | LOW | 高 |
| ENG-06 | 无 `cargo audit` / `cargo deny` 集成 | LOW-MEDIUM | 高 |

---

## 第二阶段：问题真实性评估

| ID | AI 可信度 | 验证成本 | 需人工验证？ | 验证步骤 |
|----|----------|---------|-------------|---------|
| SEC-01 (SSRF) | **高** | 低 | 是 | 发送 `curl http://127.0.0.1:8080` 到 WebFetch，确认是否被拦截 |
| SEC-02 (unsafe 缺注释) | **高** | 低 | 否 | 直接读取 bash 三个文件和 remote_claim.rs 即可确认 |
| SEC-03 (默认 HMAC key) | **高** | 低 | 否 | 已确认 |
| SEC-04 (Cookie Secure) | **中** | 低 | 是 | 审查 `routes/auth.rs` cookie 构建逻辑，确认生产部署配置 |
| SEC-05 (LIKE 注入) | **高** | 低 | 是 | 测试搜索 `%` 和 `_` 字符，确认是否匹配所有记录 |
| SEC-06 (API keys 明文) | **中** | 低 | 否 | 搜索 `CredentialStore` 结构体字段类型确认 |
| SEC-07 (测试 env 线程不安全) | **高** | 低 | 是 | 并行运行 `cargo test` 多次，观察是否有偶发失败 |
| REL-01 (Mutex 中毒崩溃) | **高** | 低 | 否 | 已确认 |
| REL-02 (std RwLock in async) | **高** | 低 | 是 | 在所有 lock 调用点确认是否跨越 `.await` |
| REL-03 (block_on hack) | **高** | 中 | 是 | 追踪调用链确认是否有嵌套 tokio context 场景 |
| REL-04 (Bedrock SigV4) | **高** | 低 | 是 | 对 AWS Bedrock 发送真实请求，确认返回 403 |
| REL-05 (Sandbox 桩代码) | **高** | 低 | 否 | 已确认 |
| REL-06 (无界通道) | **高** | 低 | 否 | 已确认 |
| REL-07 (unwrap on JSON) | **高** | 低 | 是 | 构造格式错误 JSON 输入测试 |
| PERF-01 (HTML 正则重编译) | **高** | 低 | 否 | 已确认 |
| PERF-02 (无 spawn_blocking) | **高** | 低 | 是 | 在关键 I/O 路径添加 tracing 测量阻塞时长 |
| PERF-03 (prepare_command 正则) | **高** | 低 | 否 | 已确认 |
| PERF-04 (args_map 克隆) | **中** | 低 | 是 | 用 `heaptrack` 或 `cargo-instruments` 测量 |
| ARC-01 (tools→agents) | **高** | 低 | 否 | 已确认 |
| ARC-02 (CLI God Object) | **高** | 低 | 否 | 已确认 |
| ARC-03 (runtime 杂货袋) | **高** | 低 | 否 | 已确认 |
| ARC-04 (双 SQLite) | **高** | 中 | 是 | 确认两个数据库 schema 是否有重叠/冲突 |
| ARC-05~08 | **高** | 低 | 否 | 已确认 |
| ENG-01~06 | **高** | 低 | 否 | 已确认 |

**AI 误报风险**：本次审计中所有 29 个发现均来自代码直接读取，可信度为中-高。**无需标记为误报**的项目。需人工验证的 12 项均为运行期行为确认（如 SSRF 边界测试、Bedrock 真实请求、并行测试竞态观察），非代码解读争议。

---

## 第三阶段：优先级重排

评分维度：**Impact**（1-5）× **Likelihood**（1-5）× **Effort**（反转：1=难/5=易）

| ID | 问题 | Impact | Likelihood | Effort (易=5) | 综合 |
|----|------|--------|-----------|---------------|------|
| **ENG-01** | 无 CI/CD | 5 | 5 | 3 | **P0** |
| **SEC-01** | SSRF WebFetch | 5 | 4 | 4 | **P0** |
| **REL-04** | Bedrock SigV4 缺失 | 4 | 5 | 3 | **P0** |
| **ENG-06** | 无 cargo audit/deny | 4 | 5 | 4 | **P0** |
| **PERF-01** | HTML 正则重编译 | 2 | 5 | 5 | **P1** |
| **PERF-02** | 无 spawn_blocking | 3 | 4 | 2 | **P1** |
| **REL-01** | Mutex 中毒崩溃 | 3 | 3 | 4 | **P1** |
| **REL-06** | 无界通道 | 2 | 3 | 5 | **P1** |
| **PERF-03** | prepare_command 正则 | 1 | 5 | 5 | **P1** |
| **SEC-02** | Unsafe 缺注释 | 1 | 3 | 5 | **P1** |
| **REL-02** | std RwLock in async | 3 | 2 | 3 | **P2** |
| **SEC-07** | 测试 env unsafe | 1 | 4 | 4 | **P2** |
| **SEC-04** | Cookie Secure flag | 2 | 2 | 5 | **P2** |
| **SEC-05** | LIKE 通配符 | 2 | 2 | 4 | **P2** |
| **SEC-03** | 默认 HMAC key | 1 | 2 | 5 | **P2** |
| **SEC-06** | API keys zeroize | 1 | 2 | 3 | **P2** |
| **REL-03** | block_on hack | 2 | 2 | 2 | **P3** |
| **REL-05** | Sandbox 桩代码 | 2 | 1 | 2 | **P3** |
| **ENG-02** | 无文档 | 2 | 3 | 2 | **P3** |
| **ARC-01** | tools→agents 依赖倒置 | 2 | 2 | 1 | **P3** |
| **ARC-03** | runtime 拆分 | 1 | 2 | 1 | **P3** |
| **ARC-04** | 双 SQLite 统一 | 1 | 1 | 1 | **P3** |
| **ARC-02** | CLI God Object | 1 | 2 | 1 | **P3** |
| **ARC-05** | async_trait on sync trait | 1 | 2 | 5 | **P3** |
| **ARC-06** | 14 参数方法 | 1 | 2 | 4 | **P3** |
| **ARC-07** | config 依赖 reqwest | 1 | 1 | 1 | **P3** |
| **ARC-08** | ts-rs in models | 1 | 1 | 5 | **P3** |
| **ENG-03** | dirs 版本不一致 | 1 | 2 | 5 | **P3** |
| **ENG-04** | proptest 未使用 | 1 | 1 | 5 | **P3** |
| **REL-07** | unwrap on JSON | 2 | 2 | 5 | **P2** |
| **PERF-04** | args_map 克隆 | 1 | 2 | 3 | **P3** |

### 最终优先级

```
P0 (本周必须完成) — 4 items
P1 (本月必须完成) — 7 items
P2 (季度内完成) — 6 items
P3 (视资源决定) — 13 items
```

---

## 第四阶段：Issue Backlog

---

### P0-1: 建立 CI/CD Pipeline 🚀

**标题**：Set up CI/CD with clippy, test, cargo-audit, cargo-deny

**背景**：项目当前无任何自动化质量检查。892 个 .rs 文件的代码库在无 clippy、无测试自动化、无依赖审计的情况下发展，质量退化不可避免。每次 PR 应有自动化检查确保代码质量不低于当前基线。

**技术方案**：
1. 创建 `.github/workflows/ci.yml`
2. 添加 `cargo fmt --check` job
3. 添加 `cargo clippy --workspace -- -D warnings` job
4. 添加 `cargo test --workspace` job
5. 添加 `cargo audit` job
6. 添加 `cargo deny check` job
7. 创建 `deny.toml`（许可证 + 重复依赖检查）

**验收标准**：
- PR 必须通过所有 CI checks 才能 merge
- `cargo clippy` 零 warning（初始阶段可设为 warning 不阻塞，逐步清零）
- `cargo test --workspace` 全部通过
- `cargo audit` 无 critical/high 漏洞
- CI 在 15 分钟内完成

**工作量**：**S**（2-4 小时）  
**风险**：现有 clippy warning 可能较多。策略：第一步添加 CI 配置但不设置 `-D warnings`，创建独立 Issue 逐步修复所有 clippy warning 后改为强制。

---

### P0-2: 修复 WebFetch SSRF 漏洞 🔒

**标题**：Fix SSRF vulnerability — add private IP filtering to WebFetchTool

**背景**：`crates/opendev-tools-impl/src/web_fetch/mod.rs` 仅检查 URL 是否以 `http://` 或 `https://` 开头，未阻止访问内网地址。LLM 可通过精心构造的 prompt 扫描内网服务（127.0.0.1、10.x、192.168.x、169.254.x 等）。

**技术方案**：
```rust
fn is_blocked_url(host: &str) -> bool {
    if let Ok(ip) = host.parse::<IpAddr>() {
        return ip.is_loopback() || ip.is_private() 
            || ip.is_link_local() || ip.is_unspecified()
            || ip.is_multicast();
    }
    if let Ok(addrs) = format!("{host}:0").to_socket_addrs() {
        return addrs.any(|a| {
            let ip = a.ip();
            ip.is_loopback() || ip.is_private() || ip.is_link_local()
        });
    }
    true // fail closed
}
```
- 修改位置：`web_fetch/mod.rs` 中 URL 验证逻辑附近
- 添加 7+ 个单元测试覆盖边界

**验收标准**：
- `http://127.0.0.1:8080` → 被拒绝
- `http://localhost:3000` → 被拒绝
- `http://10.0.0.1/api` → 被拒绝
- `http://192.168.1.1/admin` → 被拒绝
- `http://[::1]:8080` → 被拒绝
- `http://169.254.169.254/latest/meta-data` → 被拒绝（AWS IMDS）
- `https://example.com` → 正常通过
- 单元测试全部通过

**工作量**：**XS**（1-2 小时）  
**风险**：DNS 解析引入延迟（~10-50ms）。可先做 IP 字面量检查（零成本），DNS 解析放第二步。

---

### P0-3: 实现 Bedrock SigV4 签名 🔒

**标题**：Implement missing SigV4 signing in Bedrock adapter

**背景**：`crates/opendev-http/src/adapters/bedrock/mod.rs` 中 5 个 TODO 标记 SigV4 签名完全未实现。Authorization header 中包含字面字符串 `"AWS4-HMAC-SHA256 Credential=TODO/...`。当前 Bedrock 适配器对 AWS 的任何请求都会返回 403。

**技术方案**：
- Workspace 已有 `sha2` + `hmac` 依赖（已在 `opendev-web` 中使用）
- 实现标准 SigV4 签名或引入 `aws-sigv4` crate
- 修改 `bedrock/mod.rs:85-153`，替换 TODO 占位符

**验收标准**：
- Bedrock 请求能成功获得 200 响应（需真实 AWS 凭证测试）
- 单元测试：签名输出与 AWS 示例向量匹配
- Authorization header 不再包含 `"TODO"` 字符串

**工作量**：**S**（4-8 小时）  
**风险**：SigV4 实现细节多，容易出错。建议直接引入 `aws-sigv4` crate 降低维护负担。

---

### P0-4: 添加 cargo-deny 依赖审计

**标题**：Add cargo-deny for license compliance and duplicate detection

**背景**：965 个 crate（含传递依赖），无许可证合规检查。`dirs` v5/v6 并存等问题无法自动发现。

**技术方案**：
1. 添加 `deny.toml` 配置文件
2. 配置许可证白名单（MIT、Apache-2.0 等）
3. 启用 duplicate dependency 检测
4. 集成到 CI pipeline（P0-1）

**验收标准**：
- `cargo deny check` 通过
- CI 自动运行 deny check

**工作量**：**XS**（1 小时）  
**风险**：可能有依赖使用 GPL/ApGL 等 copyleft 许可证需要评估。

---

### P1-1: HTML 转换器正则缓存优化 ⚡

**标题**：Cache regex compilation in html_to_markdown converter

**背景**：`crates/opendev-tools-impl/src/web_fetch/html_converter.rs` 每次 `html_to_markdown()` 调用重编译 ~17 个正则表达式。WebFetch 是高频工具，每次调用可节省数百 ms CPU。

**技术方案**：将所有 `Regex::new()` 移到 `static LazyLock<Regex>` 中。

**验收标准**：
- 现有 HTML→Markdown 测试全部通过
- 正则仅编译一次

**工作量**：**XS**（1-2 小时）  
**风险**：极低。纯局部重构。

---

### P1-2: 引入 spawn_blocking 避免阻塞 async 线程 ⚡

**标题**：Wrap blocking I/O in spawn_blocking to prevent async runtime starvation

**背景**：文件读写、SQLite 操作、bash 进程管理等阻塞 I/O 直接在 tokio 工作线程上执行。在并发场景下可能导致其他任务饥饿。

**技术方案**：影响文件：
- `opendev-tools-impl/src/file_read/mod.rs`
- `opendev-tools-impl/src/file_write.rs`
- `opendev-tools-impl/src/file_edit.rs`
- `opendev-tools-impl/src/bash/foreground.rs`
- `opendev-tools-impl/src/bash/background.rs`

```rust
// Before: std::fs::read(&path)
// After:
let bytes = tokio::task::spawn_blocking(move || {
    std::fs::read(&path)
}).await.map_err(|join_err| ...)??
  .map_err(|io_err| ...)?;
```

**验收标准**：
- 所有文件操作在 `spawn_blocking` 内执行
- 现有测试全部通过

**工作量**：**M**（4-8 小时，涉及 5+ 文件）  
**风险**：错误处理链变复杂（需处理 `JoinError` + `io::Error`）。建议先创建 helper 函数统一模式。

---

### P1-3: Mutex 中毒恢复 🔧

**标题**：Replace .lock().unwrap() with poison recovery pattern

**背景**：52 处 `.lock().unwrap()` 调用在 Mutex 中毒时直接 panic。Mutex 中毒是可恢复的。

**技术方案**：全局替换模式：`.lock().unwrap()` → `.lock().unwrap_or_else(|e| e.into_inner())`

**验收标准**：
- 所有生产路径的 `.lock().unwrap()` 替换为中毒恢复
- 测试中可保留 `unwrap()`

**工作量**：**XS-S**（1-3 小时）  
**风险**：极低。纯机械替换。

---

### P1-4: 无界通道替换为有界通道 🔧

**标题**：Replace unbounded channels with bounded channels

**背景**：`tui_runner/mod.rs:224` 和 `telegram/polling.rs:212` 使用无界通道。

**技术方案**：
```rust
let (user_tx, user_rx) = mpsc::channel::<String>(128);    // tui_runner
let (chunk_tx, chunk_rx) = mpsc::channel::<String>(1024);  // polling
```

**验收标准**：
- 通道有界
- 满通道时行为正确（不 panic）

**工作量**：**XS**（1 小时）  
**风险**：需确认生产-消费模式在独立 task 中，避免死锁。

---

### P1-5: prepare_command 正则缓存 ⚡

**标题**：Cache regex compilation in prepare_command

**背景**：`bash/helpers.rs:151` 每次 bash 调用重编译 `^(python3?)\s+` 正则。

**技术方案**：使用 `static LazyLock<Regex>`。

**工作量**：**XS**（10 分钟）  
**风险**：零风险。

---

### P1-6: 补充 unsafe 块 SAFETY 注释 📝

**标题**：Add SAFETY comments to production unsafe blocks

**背景**：3 个 production unsafe 块缺少 `// SAFETY:` 注释。

**技术方案**：在 `bash/helpers.rs`、`bash/foreground.rs`、`bash/background.rs` 添加文档注释。

**工作量**：**XS**（30 分钟）  
**风险**：零风险。

---

### P1-7: Session Cookie Secure flag 🔒

**标题**：Set Secure flag on session cookie in production

**背景**：`crates/opendev-web/src/routes/auth.rs` 构建 session cookie 时未设置 `Secure` flag。

**技术方案**：
```rust
if !cfg!(debug_assertions) {
    cookie = cookie.secure(true);
}
```

**工作量**：**XS**（30 分钟）  
**风险**：需确认部署环境使用 HTTPS。

---

### P2-1: std::sync::RwLock → tokio::sync::RwLock 🔧

**标题**：Replace std::sync::RwLock with tokio::sync::RwLock in async contexts

**背景**：`task_manager`、`team_manager`、`team_task_list` 使用 `std::sync::RwLock`。

**技术方案**：替换为 `tokio::sync::RwLock`，所有 `read()`/`write()` 改为 `.await`。

**工作量**：**M**（4-8 小时）  
**风险**：中等。函数签名变更可能触发连锁修改。

---

### P2-2: 迁移测试 env::set_var 到 temp_env 🧪

**标题**：Replace unsafe env::set_var in tests with temp_env crate

**技术方案**：33 处机械替换 `unsafe { std::env::set_var() }` → `temp_env::with_var()`

**工作量**：**S**（2-3 小时）  
**风险**：极低。

---

### P2-3: LIKE 通配符转义 🔒

**标题**：Escape LIKE wildcards in SQLite search queries

**技术方案**：添加 `escape_like()` 函数转义 `%` 和 `_`。

**工作量**：**XS**（30 分钟）  
**风险**：零风险。

---

### P2-4: 默认 HMAC key 强化 🔒

**标题**：Replace hardcoded default HMAC key with startup check

**技术方案**：生产构建中，如果 `OPENDEV_SECRET_KEY` 未设置，启动时 panic 或自动生成随机 key。

**工作量**：**XS**（1 小时）  
**风险**：极低。

---

### P3 项目（视资源决定）

| ID | 标题 | 工作量 | 建议 |
|----|------|--------|------|
| REL-03 | 消除 sqlite_store block_on hack | L (1-2天) | 推迟 — 当前可工作 |
| REL-05 | Sandbox 桩代码处理 | M | 推迟 — 取决于产品计划 |
| PERF-04 | args_map 克隆优化 | S | 可选 |
| ENG-02 | README + 架构文档 | M (1-2天) | 建议做 |
| ARC-01~08 | 架构重构 | L-XL (2-5天每个) | 推迟到 v0.3+ |

---

## 第五阶段：GitHub Project 组织

### Epic 1: Security Hardening (7 issues · 1-2 人天)

| Story ID | 标题 | 优先级 | 工时 |
|----------|------|--------|------|
| SEC-01 | Fix SSRF in WebFetchTool | P0 | XS |
| SEC-02 | Add SAFETY comments to unsafe blocks | P1 | XS |
| SEC-03 | Escape LIKE wildcards in search | P2 | XS |
| SEC-04 | Set Secure flag on session cookie | P1 | XS |
| SEC-05 | Strengthen default HMAC key | P2 | XS |
| SEC-06 | Add zeroize for API keys | P2 | S |
| SEC-07 | Fix test env::set_var thread safety | P2 | S |

### Epic 2: Async Runtime Stabilization (7 issues · 3-5 人天)

| Story ID | 标题 | 优先级 | 工时 |
|----------|------|--------|------|
| ASYNC-01 | Wrap blocking I/O in spawn_blocking | P1 | M |
| ASYNC-02 | Fix Mutex poison → crash | P1 | XS |
| ASYNC-03 | Replace unbounded channels | P1 | XS |
| ASYNC-04 | std RwLock → tokio RwLock | P2 | M |
| ASYNC-05 | Fix unwrap on malformed JSON | P2 | XS |
| ASYNC-06 | Remove block_on hack | P3 (推迟) | L |
| ASYNC-07 | Implement Bedrock SigV4 | P0 | S |

### Epic 3: Engineering Excellence (8 issues · 2-4 人天)

| Story ID | 标题 | 优先级 | 工时 |
|----------|------|--------|------|
| ENG-01 | Set up CI/CD pipeline | P0 | S |
| ENG-02 | Add cargo-deny | P0 | XS |
| ENG-03 | Add cargo-audit to CI | P0 | XS |
| ENG-04 | Fix dirs version inconsistency | P3 | XS |
| ENG-05 | Remove or use proptest dep | P3 | XS |
| ENG-06 | Add rustfmt.toml + clippy.toml | P3 | XS |
| ENG-07 | Write README.md | P3 | S |
| ENG-08 | Write architecture docs | P3 (推迟) | M |

### Epic 4: Performance Optimization (3 issues · <1 人天)

| Story ID | 标题 | 优先级 | 工时 |
|----------|------|--------|------|
| PERF-01 | Cache HTML converter regexes | P1 | XS |
| PERF-02 | Cache prepare_command regex | P1 | XS |
| PERF-03 | Reduce args_map clones | P3 (推迟) | S |

### Epic 5: Architecture & Tech Debt (9 issues · 全部推迟)

| Story ID | 标题 | 优先级 | 工时 |
|----------|------|--------|------|
| ARC-01 | Extract shared types (tools→agents dep fix) | P3 | L |
| ARC-02 | Split CLI God Object | P3 | L |
| ARC-03 | Split opendev-runtime | P3 | XL |
| ARC-04 | Unify SQLite pools | P3 | L |
| ARC-05 | Remove async_trait from ProviderAdapter | P3 | XS |
| ARC-06 | Introduce ReactLoopContext | P3 | S |
| ARC-07 | Feature-flag ts-rs in models | P3 | XS |
| ARC-08 | Feature-flag reqwest in config | P3 | S |
| ARC-09 | Remove/feature-gate opendev-sandbox | P3 | S |

### 全部统计

| Epic | Issues | Active | 预计工时 | 关键路径 |
|------|--------|--------|---------|---------|
| Security Hardening | 7 | 6 | 1-2 人天 | 无 |
| Async Runtime Stabilization | 7 | 6 | 3-5 人天 | spawn_blocking → RwLock |
| Engineering Excellence | 8 | 5 | 2-4 人天 | CI → audit/deny |
| Performance Optimization | 3 | 2 | <1 人天 | 无 |
| Architecture & Tech Debt | 9 | 1 | 5-10 人天 | 全部推迟 |
| **总计** | **34** | **20** | **6-12 人天** | |

---

## 第六阶段：PR 拆分计划

### Epic 1: Security Hardening (4 PRs)

| PR | 标题 | 文件数 | 依赖 |
|----|------|--------|------|
| PR-SEC-1 | Fix SSRF in WebFetchTool + add tests | 1-2 | 无 |
| PR-SEC-2 | Add SAFETY comments to all unsafe blocks | 3 | 无 |
| PR-SEC-3 | Escape LIKE wildcards + strengthen HMAC key | 2 | 无 |
| PR-SEC-4 | Cookie Secure flag + zeroize + test env safety | 分散 | 无 |

### Epic 2: Async Runtime (4 PRs)

| PR | 标题 | 文件数 | 依赖 |
|----|------|--------|------|
| PR-ASYNC-1 | Implement Bedrock SigV4 signing | 1-2 | 无 |
| PR-ASYNC-2 | Mutex poison recovery (global replacement) | ~15 | 无 |
| PR-ASYNC-3 | Wrap blocking I/O in spawn_blocking | 5-7 | 无 |
| PR-ASYNC-4 | Replace unbounded channels + std RwLock → tokio RwLock | 5-6 | PR-ASYNC-3 |

### Epic 3: Engineering Excellence (3 PRs)

| PR | 标题 | 文件数 | 依赖 |
|----|------|--------|------|
| PR-ENG-1 | Set up CI/CD pipeline (GitHub Actions) | 2-3 (new) | 无 |
| PR-ENG-2 | Add cargo-deny + cargo-audit config | 1-2 (new) | PR-ENG-1 |
| PR-ENG-3 | Write README.md | 1 (new) | 无 |

### Epic 4: Performance (1 PR)

| PR | 标题 | 文件数 | 依赖 |
|----|------|--------|------|
| PR-PERF-1 | Cache regexes: HTML converter + prepare_command | 2 | 无 |

### 总计：12 PRs

所有 PR 相互独立（除 PR-ENG-2 → PR-ENG-1），可按任何顺序提交。每个 PR 独立可 Review、独立可 Merge、独立可 Rollback。

---

## 第七阶段：90 天整改路线图

### Week 1-2（第 1-14 天）：安全加固 + CI 建立

**目标**：消除安全漏洞 + 建立自动化质量防线

| 周 | 任务 | 负责人 | 验收标准 |
|----|------|--------|---------|
| W1 | PR-ENG-1: CI/CD pipeline | Platform | CI 配置可运行（初始不阻塞 merge） |
| W1 | PR-SEC-1: WebFetch SSRF fix | Backend | 私有 IP 被拒绝 |
| W1 | PR-ASYNC-1: Bedrock SigV4 | Backend | AWS 请求成功 |
| W2 | PR-ENG-2: cargo-deny + audit | Platform | deny check 通过 |
| W2 | PR-SEC-2: SAFETY comments | Backend | 代码审查通过 |
| W2 | PR-PERF-1: Regex caching | Backend | 性能测试无退化 |

---

### Week 3-4（第 15-30 天）：运行时稳定性提升

**目标**：消除 async 运行时反模式，提升高并发场景可靠性

| 周 | 任务 | 负责人 | 验收标准 |
|----|------|--------|---------|
| W3 | PR-ASYNC-2: Mutex poison recovery | Backend | 所有生产路径无 unwrap 崩溃 |
| W3 | PR-ASYNC-3: spawn_blocking I/O | Backend | 手动压测确认无运行时饥饿 |
| W4 | PR-ASYNC-4: Unbounded channels + RwLock | Backend | 功能测试 + 死锁检测 |
| W4 | PR-SEC-3~4: LIKE escape + Cookie + HMAC | Backend | 手动验证 |

---

### Week 5-8（第 31-60 天）：工程化完善

**目标**：文档完善 + 技术债务清理

| 周 | 任务 | 负责人 | 验收标准 |
|----|------|--------|---------|
| W5-6 | PR-ENG-3: README + architecture docs | Tech Lead | 文档评审通过 |
| W5-6 | clippy warnings 批量修复 | Backend | CI 零 warning |
| W7-8 | Sandbox crate 决策（移除/feature-gate） | Architect + Product | Sandbox 不再占据编译时间 |
| W7-8 | proptest 实际使用（WebFetch/Bash property tests） | Backend | 至少 5 个 proptest 测试 |

---

### Week 9-12（第 61-90 天）：架构规划

**目标**：架构重构决策 + 长期方向

| 周 | 任务 | 负责人 | 验收标准 |
|----|------|--------|---------|
| W9-10 | 架构评估：tools→agents 解耦可行性 | Architect | Go/No-Go 决策文档 |
| W9-10 | 架构评估：runtime 拆分方案 | Architect | 方案设计文档 |
| W11-12 | `ProviderAdapter` async_trait 移除 + `ReactLoopContext` 引入 | Backend | 编译 + 测试通过 |
| W11-12 | 技术债务梳理：更新 Issue Backlog | Tech Lead | 更新的 backlog |

---

## 第八阶段：架构重构决策

| 架构问题 | 是否值得重构 | 重构收益 | 重构成本 | 推荐时间点 | 推迟？ |
|---------|------------|---------|---------|-----------|-------|
| tools→agents 依赖倒置 | **否** | 工具层独立复用 | 高（2-5 人天） | v0.3+ | ✅ 推迟 — 无复用需求 |
| CLI God Object | **否** | 编译时间减少 | 中（1-3 人天） | v0.4+ | ✅ 推迟 — 编译时间可接受 |
| runtime 拆分 | **否** | 模块边界清晰 | 高（3-7 人天） | v0.5+ | ✅ 推迟 — 文件多但逻辑内聚 |
| 双 SQLite 统一 | **视情况** | 共享连接池 | 中（2-4 人天） | v0.3+ | ⚠️ 有条件推迟 — 出现冲突前 |
| ProviderAdapter async_trait | **可随时做** | 消除堆分配 | 极低 | 任何 sprint | ❌ 不推迟 — 10 分钟小任务 |

> **原则**：避免为了"架构漂亮"而进行无价值重构。所有架构重构在当前阶段（v0.1.8）的 ROI 不划算，推迟到功能更稳定后。

---

## 第九阶段：ROI 分析

| 项目 | 收益 | 成本 | ROI | 决策 |
|------|------|------|-----|------|
| **CI/CD** | 防止质量退化 | 2-4h | **极高** | ✅ 立即做 |
| **cargo-deny/audit** | 防止供应链攻击 | 1h | **极高** | ✅ 立即做 |
| **SSRF fix** | 防止内网扫描 | 1-2h | **极高** | ✅ 立即做 |
| **Bedrock SigV4** | 解锁 Bedrock 用户 | 4-8h | **极高** | ✅ 立即做 |
| **Mutex poison recovery** | 防止 50+ crash sites | 1-3h | **高** | ✅ 近期做 |
| **HTML regex cache** | 每次 fetch 省 ~100ms | 1-2h | **高** | ✅ 近期做 |
| **spawn_blocking** | 防止运行时饥饿 | 4-8h | **高** | ✅ 近期做 |
| **Unsafe comments** | Rust 合规 | 30m | **高** | ✅ 近期做 |
| **Unbounded channels** | 防止 OOM | 1h | **中** | ✅ 近期做 |
| **Prepare command regex** | 每次 bash 省微秒 | 10m | **中** | ✅ 近期做 |
| **Cookie Secure flag** | 部署 HTTPS 无影响 | 30m | **低** | ⚠️ 可做 |
| **LIKE escape** | 轻微信息泄漏 | 30m | **低** | ⚠️ 可延后 |
| **HMAC key** | 仅影响未配置部署 | 1h | **低** | ⚠️ 可延后 |
| **RwLock → tokio** | 消除未来风险 | 4-8h | **中** | ⚠️ 可延后 |
| **Test env safety** | 并行测试稳定性 | 2-3h | **中** | ⚠️ 可延后 |
| **README + docs** | 开源社区必须 | 1-3天 | **中** | ⚠️ 按时做 |
| **tools→agents 解耦** | 无复用需求 | 2-5天 | **极低** | ❌ 推迟 |
| **runtime 拆分** | 边界模糊不阻塞 | 3-7天 | **极低** | ❌ 推迟 |
| **CLI God Object** | 编译时间可接受 | 1-3天 | **极低** | ❌ 推迟 |
| **双 SQLite 统一** | 无功能冲突 | 2-4天 | **低** | ❌ 推迟 |
| **proptest** | 当前未使用 | 10m | — | ✅ 移除未使用依赖 |

### 建议暂不处理的问题

| 问题 | 原因 |
|------|------|
| `opendev-tools-impl → agents` 依赖倒置 | 无复用需求，重构成本高，收益不明确 |
| `opendev-runtime` 拆分 | 文件虽多但逻辑内聚，拆分管理成本 > 收益 |
| CLI God Object 拆分 | 编译时间当前可接受，过早拆分增加维护负担 |
| `opendev-memory` + `opendev-history` SQLite 统一 | 两个数据库各司其职，无冲突 |
| `ts-rs` in models feature-gating | 当前构建时间可接受 |
| 14 参数 `MainAgent::run()` | 非阻塞性问题 |
| `ProviderAdapter` async_trait 移除 | 可以做但收益极微 |

---

## 附录：风险矩阵

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|---------|
| SSRF 被 LLM prompt 利用 | 中 | 高 | P0 立即修复 |
| CI 缺失导致质量退化 | 高 | 高 | P0 立即建立 |
| Bedrock 用户流失 | 高 | 中 | P0 立即修复 |
| async 运行时饥饿 | 低 | 中 | P1 spawn_blocking |
| Mutex 中毒崩溃 | 低 | 中 | P1 poison recovery |
| 架构重构引入回归 | — | — | 推迟所有架构重构到 v0.3+ |

---

**整改计划完成。** 建议立即启动 P0 的四项工作，两周内完成安全加固和 CI 建立。所有架构重构推迟到 v0.3+ 以避免在当前快速迭代阶段引入不必要的风险。
