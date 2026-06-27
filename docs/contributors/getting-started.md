# Getting Started as a Contributor

## Prerequisites

- **Rust 1.94+** — install via `rustup` (the project uses Edition 2024).
- **Node.js 20+** — for the frontend build.
- **Cargo tools** — `cargo fmt`, `cargo clippy` components (install with
  `rustup component add rustfmt clippy`).
- **An LLM API key** — if you want to run the agent (OpenAI, Anthropic, etc.).

## First Build

```bash
# Clone the repository
git clone https://github.com/opendev-to/opendev
cd opendev-desktop

# Install frontend dependencies
npm install

# Build the entire workspace (may take a while the first time)
cargo build --workspace

# Run tests
cargo test --workspace --lib
```

## Development Workflows

### Desktop App

```bash
npm run tauri dev
```

This starts the Vite dev server + Tauri development window.

### CLI Mode

```bash
# Run a single query
cargo run -p opendev-cli -- chat "Hello"

# Start interactive REPL
cargo run -p opendev-cli -- repl

# Start terminal UI
cargo run -p opendev-cli -- tui

# Start web server
cargo run -p opendev-cli -- serve
```

## Pre-commit Checklist

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features  # Fix any new warnings
cargo test --workspace --lib                           # All tests pass
cargo deny check                                       # License/dep policy
```

## Understanding the Codebase

Start with these documents in order:

1. `docs/architecture/README.md` — architecture overview.
2. `docs/constitution.md` — design principles (read this before writing code).
3. `ARCHITECTURE.md` — high-level architecture at project root.
4. `docs/contributors/project-layout.md` — where things live.
5. `docs/contributors/architecture-tour.md` — key types and entry points.

## First Contribution Ideas

- Fix a clippy warning (there are ~40 pre-existing ones).
- Add a test for an untested edge case.
- Improve error messages in any crate.
- Help implement stubs in `opendev-sandbox`.
- Improve documentation.
