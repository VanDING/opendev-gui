# opendev-desktop — 项目搭建执行提示词

> 将完整提示词发送到新的对话中，让 AI 执行项目搭建。

---

## 你的任务

严格按照 `/Users/van/projects/opendev-desktop/SETUP_PLAN.md` 执行，创建完整的 opendev-desktop 桌面应用项目。

## 执行铁律

1. **逐步骤执行，不跳步**：计划有 11 个步骤（A→H），必须按顺序执行。
2. **每步自检**：目录创建用 `ls` 验证，文件复制用 `wc -l` 或 `find` 验证数量。
3. **`cargo check` 是最终验证**：所有 crate 编译通过才算成功。
4. **Cargo.toml path 修正是最容易出错的地方**：20 个 crate 的内部依赖路径必须批量修正，不能手动改。

## 关键陷阱

### 陷阱 1：crate 内部路径

opendev 原项目的 crate 相互依赖写的是：

```toml
opendev-web = { version = "0.1.6", path = "../opendev-web" }
```

搬到 `opendev-desktop/crates/` 后，workspace root 变深了一层，路径需要改为：

```toml
opendev-web = { version = "0.1.6", path = "../../crates/opendev-web" }
```

**批量修正命令**（在 opendev-desktop 根目录执行）：

```bash
find crates -name 'Cargo.toml' -exec sed -i '' 's|path = "\.\./opendev-|path = "../../crates/opendev-|g' {} +
```

这个 sed 只匹配 `path = "../opendev-xxx"` 模式，不会误伤其他 path。执行后 `grep -r 'path = "\.\./\.\./crates' crates/` 确认数量正确（约 18-25 条）。

### 陷阱 2：Cargo workspace 根

`opendev-desktop/Cargo.toml` 的 `[workspace.dependencies]` 中包含 20 个 internal crates 的声明。从 `opendev/Cargo.toml` 复制时，**不需要改 path**——workspace 根只需要声明 `opendev-web = { version = "0.1.6", path = "crates/opendev-web" }`（因为 workspace root 直接包含 crates/ 目录）。

### 陷阱 3：Tauri 版本兼容性

`@tauri-apps/cli` 需要和 Rust 侧的 `tauri` crate 版本匹配。先执行 `npm view @tauri-apps/cli version` 和 `cargo search tauri` 确认两边版本都满足 v2.x。

### 陷阱 4：frontend 无 proxy 配置

桌面版不需要 Vite proxy（`/api` 和 `/ws` 代理）。前端通过注入的 `window.__OPENDEV_PORT__` 直接连接本地 server。

### 陷阱 5：opendev-web 的依赖

`opendev-web` 依赖于 `opendev-agents`、`opendev-history` 等。确认在修复 crate path 后，`cargo check -p opendev-web` 能通过。如果失败，逐层 `cargo check -p opendev-xxx` 找到缺失的 path 修正。

## 执行步骤

**A.** 创建目录骨架（`mkdir -p`）

**B.** 搬迁源码：
- B1. 前端：`cp -r ~/projects/opendev-gui/src ~/projects/opendev-desktop/`
- B2. 后端：`cp -r ~/projects/opendev/crates ~/projects/opendev-desktop/`

**C.** 创建前端配置文件（package.json / vite.config.ts / tsconfig.json / index.html）：

- `package.json`：复制 opendev-gui/package.json，加 `@tauri-apps/cli` 和 `"tauri": "tauri"` script
- `vite.config.ts`：去掉 proxy 配置，`build.outDir` 改为 `dist`
- 其余文件直接复制

**D.** 前端 API 层适配：
- `src/api/websocket.ts`：端口从 `(window as any).__OPENDEV_PORT__` 读取
- `src/api/client.ts`：同理

**E.** Cargo workspace：
- 创建 `Cargo.toml`（根），声明 21 个 members
- 从 `opendev/Cargo.toml` 复制 `[workspace.dependencies]`
- 执行 crate path 批量修正

**F.** Tauri 配置：
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`
- `src-tauri/capabilities/default.json`

**G.** Rust 代码：
- `src-tauri/src/server.rs`
- `src-tauri/src/main.rs`
- `src-tauri/build.rs`（如果 Tauri 需要）

**H.** 编译验证：
- `cargo check --workspace`
- `npm install && npm run build`
- `npm run tauri dev`（如果可以）

## 验证命令

```bash
# 步骤 A 后
find ~/projects/opendev-desktop -type d | sort

# 步骤 B 后
find ~/projects/opendev-desktop/src -type f | wc -l      # 前端文件数
find ~/projects/opendev-desktop/crates -maxdepth 2 -name Cargo.toml | wc -l  # 应为 20+

# 步骤 C 后
ls ~/projects/opendev-desktop/package.json

# 步骤 E 后
grep -c 'path = "../../crates/' crates/*/Cargo.toml 2>/dev/null || find crates -name Cargo.toml -exec grep -l '../../crates' {} \; | wc -l

# 步骤 H 后
cargo check --workspace 2>&1 | tail -5
npm run build 2>&1 | tail -5
```

## 禁止

- ❌ 跳过 crate path 批量修正
- ❌ 手动改 20 个 crate 的 Cargo.toml
- ❌ 忘记创建 `src-tauri/build.rs`（Tauri 需要）
- ❌ 在 `vite.config.ts` 中保留 proxy 配置
- ❌ 猜测 `tauri.conf.json` 的字段——对照 Tauri 官方文档
- ❌ 在 `npm install` 失败时说"先跳过"
