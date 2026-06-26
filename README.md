# OpenDev Desktop

AI-powered coding agent desktop application. OpenDev uses LLM-driven ReAct loops to
generate code, edit files, run shell commands, search the web, and manage complex
multi-step development workflows — all through a local desktop UI or terminal.

## Features

- **Multi-provider LLM support** — OpenAI, Anthropic, Google Gemini, AWS Bedrock, Groq, Mistral, Ollama
- **Multiple interfaces** — Tauri desktop app, terminal UI (TUI), CLI, and web server
- **Agent orchestration** — Main agent with subagent spawning for parallel task execution
- **File editing** — Read, write, edit, and preview diffs with concurrent file locking
- **Shell execution** — Bash command execution with process group management
- **Web tools** — URL fetching, web search, HTML-to-markdown conversion
- **MCP integration** — Model Context Protocol client for external tool servers
- **Memory system** — Long-term and short-term memory with FTS5 SQLite persistence
- **Plugin marketplace** — Extensible plugin system
- **Telegram channel** — Remote interaction via Telegram bot
- **Session management** — Event-sourced session persistence, snapshots, and cost tracking

## Quick Start

### Prerequisites

- Rust 1.94+
- Node.js 20+
- An LLM API key (OpenAI, Anthropic, etc.)

### Build

```bash
# Install dependencies
npm install

# Build the desktop app
npm run tauri build

# Or build just the CLI
cargo build --release -p opendev-cli
```

### Run

```bash
# Tauri desktop app
npm run tauri dev

# Terminal UI
cargo run --release -p opendev-cli -- tui

# CLI mode (single query)
cargo run --release -p opendev-cli -- chat "Explain this codebase"

# Web server
cargo run --release -p opendev-cli -- serve
```

### Configuration

OpenDev stores configuration in `~/.config/opendev/`. On first run, the interactive
setup wizard will guide you through configuring API providers and model preferences.

Key configuration:
```json
{
  "model_provider": "openai",
  "model_name": "gpt-4o",
  "api_key": "sk-..."
}
```

Environment variables can override settings:
- `OPENAI_API_KEY`
- `ANTHROPIC_API_KEY`
- `GOOGLE_API_KEY`
- `OPENDEV_SECRET_KEY` (server mode)

## Project Structure

```
src-tauri/          Tauri desktop shell
crates/
  opendev-models    Core data types and domain model
  opendev-config    Configuration, model registry, paths
  opendev-http      HTTP client, LLM provider adapters
  opendev-agents    Agent orchestration, ReAct loop, subagents
  opendev-tools-*   Tool framework (core traits, implementations, LSP, symbols)
  opendev-history   Session persistence (JSONL + SQLite)
  opendev-context   Context management and compaction
  opendev-runtime   Approval, permissions, event bus, secrets detection
  opendev-cli       CLI entry point
  opendev-tui       Terminal UI (ratatui)
  opendev-web       Web server (axum + WebSocket)
  opendev-mcp       MCP protocol client
  opendev-channels  Telegram message channel
  opendev-memory    Long/short-term memory
  opendev-plugins   Plugin manager and marketplace
  opendev-hooks     Lifecycle hook system
  opendev-workflow  Workflow engine
src/                TypeScript/React frontend
```

## Development

```bash
# Format code
cargo fmt --all

# Run linter
cargo clippy --workspace --all-targets

# Run tests
cargo test --workspace

# Check dependencies (local DB may need --no-fetch on slow networks)
cargo audit --no-fetch
cargo deny check
```

## License

MIT
