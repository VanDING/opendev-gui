# Coding Standards

## Rust Style

### Formatting

Enforced by `rustfmt` with the project's `rustfmt.toml`:

```toml
edition = "2024"
max_width = 100
use_small_heuristics = "Max"
```

Run: `cargo fmt --all` before every commit.

### Linting

`cargo clippy` is run in CI. The current policy:
- ~40 pre-existing warnings (config, channels, tools-core, history, http, mcp).
- New code should not introduce new clippy warnings.
- Use `#[allow(...)]` only with a comment explaining why.

### Naming

| Element | Convention | Example |
|---|---|---|
| Crates | `opendev-{name}` | `opendev-history`, `opendev-tools-core` |
| Types | PascalCase | `SessionEvent`, `BaseTool` |
| Functions | snake_case | `load_session`, `execute_tool` |
| Variables | snake_case | `session_id`, `tool_registry` |
| Constants | SCREAMING_SNAKE_CASE | `MAX_RETRIES` |
| Type parameters | short PascalCase | `T`, `E`, `Provider` |
| Test functions | descriptive snake_case | `test_empty_session_load` |

### Imports

- Group: `std` → external crates → internal crates.
- Single line per crate.
- No `use crate::*` or `use super::*` except in test modules.

```rust
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use opendev_models::Session;
```

## Error Handling

- Return `Result<T, Error>` for all fallible operations.
- Use `thiserror` for crate-specific error enums.
- Use `anyhow` only in composition roots and binary entry points.
- Panic only for programming errors (index out of bounds, unwrap on `None` that
  should never happen — and only after careful consideration).
- Recover from Mutex poisoning with `unwrap_or_else(|e| e.into_inner())`.

See `error-handling.md` for detailed patterns.

## Async

- Use `tokio` async runtime (full features).
- File I/O and CPU-heavy operations: `tokio::task::spawn_blocking`.
- Mutex: `tokio::sync::Mutex` for async-held locks, `std::sync::Mutex` for
  short synchronous critical sections.
- Prefer `futures::join_all` over manual `tokio::spawn` + `JoinHandle` collection.

## Documentation

- All public API items must have doc comments (`///`).
- Use `# Examples` in doc comments for non-trivial functions.
- Include `# Panics` sections when the function can panic.
- Include `# Errors` sections when returning `Result`.
- Use `// SAFETY:` comments on every `unsafe` block (required by v0.1.9 hardening).

## Dependencies

- `openssl-sys` is banned (use `rustls`).
- No git dependencies.
- Prefer few, well-known crates over many niche ones.
- New dependencies must be reviewed and justified.
