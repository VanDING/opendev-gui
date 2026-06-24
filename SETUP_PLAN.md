# opendev-desktop — 项目搭建计划

> **目标**：创建独立桌面应用项目，合并前端源码 + OpenDev 所有 crate + Tauri，单 workspace 构建。

---

## 1. 项目结构

```
opendev-desktop/
├── package.json              # React 前端依赖 + Tauri CLI
├── vite.config.ts            # Vite React 配置
├── tsconfig.json             # TypeScript 配置
├── tsconfig.node.json
├── index.html                # SPA 入口
├── .gitignore
│
├── src/                      # React 前端（从 opendev-gui/src 搬来）
│   ├── main.tsx
│   ├── App.tsx
│   ├── api/
│   ├── stores/
│   ├── components/
│   ├── contexts/
│   ├── pages/
│   ├── hooks/
│   ├── types/
│   ├── utils/
│   ├── constants/
│   └── index.css
│
├── public/                   # 静态资源
│   ├── icon_blue.png
│   └── vite.svg
│
├── dist/                     # npm run build 产物（gitignore）
│
├── src-tauri/                # Tauri Rust 壳
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── default.json
│   ├── icons/
│   └── src/
│       ├── main.rs           # Tauri 入口 + axum 启动
│       └── server.rs         # axum server 封装
│
├── crates/                   # OpenDev 全部 20 个 crate（从 opendev/crates 搬来）
│   ├── opendev-models/
│   ├── opendev-config/
│   ├── opendev-http/
│   ├── opendev-context/
│   ├── opendev-history/
│   ├── opendev-tools-core/
│   ├── opendev-tools-impl/
│   ├── opendev-tools-lsp/
│   ├── opendev-tools-symbol/
│   ├── opendev-agents/
│   ├── opendev-web/
│   ├── opendev-mcp/
│   ├── opendev-channels/
│   ├── opendev-tui/
│   ├── opendev-repl/
│   ├── opendev-cli/
│   ├── opendev-runtime/
│   ├── opendev-hooks/
│   ├── opendev-plugins/
│   └── opendev-sandbox/
│
└── Cargo.toml                # Workspace 根：members = ["src-tauri", "crates/*"]
```

---

## 2. 步骤 A：创建项目骨架

```bash
mkdir -p ~/projects/opendev-desktop
mkdir -p ~/projects/opendev-desktop/{src-tauri/src,src-tauri/capabilities,src-tauri/icons}
mkdir -p ~/projects/opendev-desktop/{src,public}
```

---

## 3. 步骤 B：搬迁源码

### B1. 前端：从 opendev-gui 搬

```bash
cp -r ~/projects/opendev-gui/src/* ~/projects/opendev-desktop/src/
cp ~/projects/opendev-gui/public/* ~/projects/opendev-desktop/public/
cp ~/projects/opendev-gui/index.html ~/projects/opendev-desktop/
```

### B2. 后端：从 opendev 搬 20 个 crate

```bash
cp -r ~/projects/opendev/crates/* ~/projects/opendev-desktop/crates/
```

> 如果 opendev workspace 的 Cargo.toml 根有 `[workspace.dependencies]` 等配置，一并参考。

---

## 4. 步骤 C：前端配置文件

### 4.1 package.json

```json
{
  "name": "opendev-desktop",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc --noEmit && vite build",
    "preview": "vite preview",
    "lint": "eslint .",
    "tauri": "tauri"
  },
  "dependencies": {
    "@heroicons/react": "^2.2.0",
    "@xyflow/react": "^12.10.1",
    "lucide-react": "^1.21.0",
    "react": "^19.2.7",
    "react-dom": "^19.2.7",
    "react-router-dom": "^7.18.0",
    "zustand": "^5.0.14"
  },
  "devDependencies": {
    "@eslint/js": "^9.0.0",
    "@radix-ui/react-dialog": "^1.0.0",
    "@tailwindcss/vite": "^4.3.1",
    "@tauri-apps/cli": "^2.0.0",
    "@types/react": "^19.2.17",
    "@types/react-dom": "^19.2.3",
    "@vitejs/plugin-react": "^6.0.3",
    "eslint": "^9.0.0",
    "eslint-plugin-react-hooks": "^7.0.0",
    "eslint-plugin-react-refresh": "^0.4.0",
    "markdown-it": "^14.0.0",
    "shiki": "^3.0.0",
    "sonner": "^2.0.0",
    "tailwindcss": "^4.3.1",
    "typescript": "^6.0.3",
    "typescript-eslint": "^8.0.0",
    "vite": "^8.1.0"
  }
}
```

> 对照 opendev-gui/package.json。加 `@tauri-apps/cli` 和 `@tauri-apps/api`。

### 4.2 vite.config.ts

```ts
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  plugins: [react(), tailwindcss()],
  server: {
    port: 5173,
    strictPort: false,
  },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
  },
})
```

去掉旧 proxy 配置（`/api` 和 `/ws`）——桌面版不需要代理到后端，连的是自己启动的本地 server。

### 4.3 tsconfig.json、tsconfig.node.json、index.html

与 opendev-gui 一致，无需改动。

---

## 5. 步骤 D：前端 API 层适配

### 5.1 api/websocket.ts 改动

```diff
- const wsUrl = isDev
-   ? 'ws://localhost:8080/ws'
-   : `${protocol}//${window.location.host}/ws`;
+ const getPort = () => (window as any).__OPENDEV_PORT__ || 8080;
+ const port = getPort();
+ const wsUrl = isDev
+   ? `ws://localhost:${port}/ws`
+   : `${protocol}//${window.location.host}/ws`;
```

### 5.2 api/client.ts 改动

```diff
- const API_BASE = '/api';
+ const getPort = () => (window as any).__OPENDEV_PORT__ || 8080;
+ const API_BASE = `http://localhost:${(window as any).__OPENDEV_PORT__ || 8080}/api`;
```

### 5.3 main.tsx 改动

```diff
+ import { Toaster } from 'sonner';
+ import { ThemeProvider } from './contexts/ThemeContext';
+ import { ErrorBoundary } from './components/ErrorBoundary';

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <ErrorBoundary>
      <ThemeProvider>
        <Toaster ... />
        <App />
      </ThemeProvider>
    </ErrorBoundary>
  </React.StrictMode>,
);
```

> 确认 `ThemeProvider`、`ErrorBoundary` 已在 src 中；确认 Toaster 配置从 opendev-gui/src/main.tsx 移植。

---

## 6. 步骤 E：Cargo Workspace

### 6.1 Cargo.toml（项目根）

```toml
[workspace]
resolver = "2"
members = [
    "src-tauri",
    "crates/opendev-models",
    "crates/opendev-config",
    "crates/opendev-http",
    "crates/opendev-context",
    "crates/opendev-history",
    "crates/opendev-tools-core",
    "crates/opendev-tools-impl",
    "crates/opendev-tools-lsp",
    "crates/opendev-tools-symbol",
    "crates/opendev-agents",
    "crates/opendev-web",
    "crates/opendev-mcp",
    "crates/opendev-channels",
    "crates/opendev-tui",
    "crates/opendev-repl",
    "crates/opendev-cli",
    "crates/opendev-runtime",
    "crates/opendev-hooks",
    "crates/opendev-plugins",
    "crates/opendev-sandbox",
]

[workspace.package]
version = "0.1.8"
edition = "2024"
license = "MIT"
repository = "https://github.com/opendev-to/opendev"
rust-version = "1.94"

[profile.release]
strip = true
lto = "thin"

[workspace.dependencies]
# 从 opendev/Cargo.toml 复制全部 [workspace.dependencies] 内容
# 包括 serde, serde_json, tokio, axum, tracing, 所有 internal crates 等
```

> **关键**：crates 内的 `Cargo.toml` 中 `opendev-xxx = { version = "0.1.6", path = "..." }` 需要把 path 修正为 `"../crates/opendev-xxx"`（因为 workspace root 变了一层）。

### 6.2 批量修正 crate 内部 path

20 个 crate 的 `Cargo.toml` 中，`path = "crates/opendev-xxx"` 需要改为 `path = "../crates/opendev-xxx"`。可用 sed 批量处理：

```bash
find crates -name 'Cargo.toml' -exec sed -i '' 's|path = "../|path = "../../crates/|g' {} +
```

---

## 7. 步骤 F：Tauri 配置

### 7.1 src-tauri/Cargo.toml

```toml
[package]
name = "opendev-desktop"
version = "0.1.0"
edition = "2024"

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
axum = "0.8"

# 引用 workspace 内部的 crate
opendev-web = { path = "../crates/opendev-web" }
opendev-config = { path = "../crates/opendev-config" }
opendev-history = { path = "../crates/opendev-history" }
opendev-models = { path = "../crates/opendev-models" }
opendev-runtime = { path = "../crates/opendev-runtime" }

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]
```

### 7.2 src-tauri/tauri.conf.json

```json5
{
  "$schema": "https://raw.githubusercontent.com/tauri-apps/tauri/dev/crates/tauri-cli/schema.json",
  "productName": "OpenDev Desktop",
  "version": "0.1.0",
  "identifier": "com.opendev.desktop",
  "build": {
    "devUrl": "http://localhost:5173",
    "frontendDist": "../dist",
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build"
  },
  "app": {
    "withGlobalTauri": false,
    "windows": [
      {
        "title": "OpenDev Desktop",
        "width": 1200,
        "height": 800,
        "minWidth": 900,
        "minHeight": 600
      }
    ],
    "security": {
      "csp": null
    }
  },
  "plugins": {}
}
```

### 7.3 src-tauri/capabilities/default.json

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Default capability",
  "windows": ["main"],
  "permissions": [
    "core:default"
  ]
}
```

---

## 8. 步骤 G：Rust 代码

### 8.1 src-tauri/src/server.rs

```rust
use std::net::TcpListener;
use opendev_config::ConfigLoader;
use opendev_history::SessionManager;

pub struct ServerHandle {
    pub port: u16,
    pub shutdown_tx: tokio::sync::oneshot::Sender<()>,
}

pub fn build_server(working_dir: &std::path::Path) -> Result<ServerHandle, String> {
    // Load config
    let paths = opendev_config::Paths::new();
    let config = ConfigLoader::new(&paths)
        .load(None, working_dir)
        .map_err(|e| format!("Failed to load config: {}", e))?;

    // Session manager
    let session_manager = SessionManager::new(&paths, working_dir.to_path_buf())
        .map_err(|e| format!("Failed to init session manager: {}", e))?;

    // Build axum app
    let app_state = opendev_web::build_app_state(config, session_manager, working_dir);
    let router = opendev_web::build_app(app_state);

    // Bind random port
    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|e| format!("Failed to bind: {}", e))?;
    let port = listener.local_addr().map(|a| a.port())
        .map_err(|e| format!("Failed to get port: {}", e))?;

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    // Spawn axum server
    tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            })
            .await
            .ok();
    });

    Ok(ServerHandle { port, shutdown_tx })
}
```

### 8.2 src-tauri/src/main.rs

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod server;

use std::sync::Mutex;

struct AppState {
    server: Mutex<Option<server::ServerHandle>>,
}

fn main() {
    tauri::Builder::default()
        .manage(AppState { server: Mutex::new(None) })
        .setup(|app| {
            let working_dir = std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."));

            match server::build_server(&working_dir) {
                Ok(handle) => {
                    let port = handle.port;

                    // Inject port into all webview windows
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.eval(&format!(
                            "window.__OPENDEV_PORT__ = {}",
                            port
                        ));
                    }

                    // Store handle for graceful shutdown
                    let state = app.state::<AppState>();
                    *state.server.lock().unwrap() = Some(handle);

                    println!("OpenDev server started on http://127.0.0.1:{}", port);
                }
                Err(e) => {
                    eprintln!("Failed to start OpenDev server: {}", e);
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.eval(&format!(
                            "document.body.innerHTML = '<h1>Server Error</h1><pre>{}</pre>'",
                            e.replace('\'', "\\'")
                        ));
                    }
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                let app = window.app_handle();
                let state = app.state::<AppState>();
                if let Some(handle) = state.server.lock().unwrap().take() {
                    let _ = handle.shutdown_tx.send(());
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running OpenDev Desktop");
}
```

---

## 9. 步骤 H：编译与验证

### 9.1 首次编译

```bash
cd ~/projects/opendev-desktop

# 确认所有 crate 编译通过
cargo check --workspace

# 前端构建
npm install
npm run build

# Tauri 开发模式（打开桌面窗口）
npm run tauri dev
```

### 9.2 生产构建

```bash
npm run tauri build
```

产物在 `src-tauri/target/release/bundle/`：
- macOS: `.dmg` + `.app`
- Linux: `.deb` + `.AppImage`
- Windows: `.msi` + `.exe`

---

## 10. 开发工作流

| 场景 | 命令 |
|------|------|
| 纯前端开发 | `npm run dev` → localhost:5173（浏览器，不需要后端） |
| 前端 + 后端 | 终端 1: `npm run dev`，终端 2: `cargo run -p opendev-cli -- run ui` |
| 桌面开发 | `npm run tauri dev`（Tauri 窗口 + Vite HMR + 内嵌后端） |

---

## 11. 验证清单

- [ ] `cargo check --workspace` 通过
- [ ] `npm run build` 通过
- [ ] `npm run tauri dev` 启动桌面窗口
- [ ] 窗口中前端正确渲染
- [ ] WebSocket 连接到内嵌的 axum server（随机端口）
- [ ] REST API 正常响应
- [ ] 窗口关闭时 axum server 优雅退出
- [ ] 生产构建 `npm run tauri build` 产出 `.app`/`.dmg`
