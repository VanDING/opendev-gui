# Error Handling Patterns

## Error Type Hierarchy

### Crate-Level Errors

Each crate defines its own error type using `thiserror`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum HistoryError {
    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

### Composition Root Errors

Binary entry points (`opendev-cli`, `src-tauri`) use `anyhow::Error` for error
aggregation since they combine errors from many crates.

### Tool Execution Errors

Tools return `ToolResult<T>` defined in `opendev-tools-core`:

```rust
pub type ToolResult<T> = Result<T, ToolError>;
```

## Mutex Poison Recovery

The codebase uses a consistent pattern for Mutex poison recovery:

```rust
let guard = mutex.lock().unwrap_or_else(|e| e.into_inner());
```

This recovers the lock even if another thread panicked while holding it. The data
may be in an inconsistent state, but the system continues operating.

## Panic Policy

| Situation | Panic? | Alternative |
|---|---|---|
| Index out of bounds | Yes | Bounds check first |
| Unreachable state (should never happen) | Yes, with `unreachable!()` | Only after exhaustively proving correctness |
| Config file corrupt at startup | No | Fail fast with `expect()` → clean error message |
| Network failure | No | Return `Err` |
| User input invalid | No | Return validation error |
| Database migration fails | No | Fail fast with clear error |

## spawn_blocking for I/O

File I/O, regex compilation, and CPU-heavy work use `spawn_blocking`:

```rust
let result = tokio::task::spawn_blocking(move || {
    std::fs::read_to_string(path)
}).await.unwrap_or_else(|e| ...)?;
```

## Error Propagation

Errors are propagated upward using `?` operator. At the boundary (API response,
WebSocket message), errors are converted to user-facing error messages.

```rust
async fn handle_request(req: Request) -> Response {
    let result = do_something().await;
    match result {
        Ok(data) => Response::success(data),
        Err(e) => Response::error(format!("Operation failed: {e}")),
    }
}
```
