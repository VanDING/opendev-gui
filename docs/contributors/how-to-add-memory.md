# How to Add a New Memory Backend

This guide describes how to add a new memory persistence backend
(e.g., Redis, PostgreSQL) to the memory system.

## Overview

The memory system in `opendev-memory` follows the same architectural pattern as
tools and providers: **trait-based abstraction**. The `MemoryProvider` trait
defines the storage interface, and the `MemoryFacade` provides the unified API
that the agent uses.

## Architecture

```
MemoryFacade (unified API used by agent)
    │
    ├── ShortTermMemory (in-memory, per-session)
    ├── WriteGate (noise filter)
    ├── CascadeBuffer (batch writer)
    └── MemoryProvider (trait — pluggable backend)
              │
              ├── SqliteMemoryProvider (current default)
              └── YourNewMemoryProvider (new)
```

## Steps

### 1. Implement MemoryProvider

Create a new file in `crates/opendev-memory/src/`:

```rust
use async_trait::async_trait;
use crate::types::{MemoryEntry, RecallOptions, MemoryProvider, MemoryResult};

pub struct YourMemoryProvider {
    // Your backend connection/config
}

#[async_trait]
impl MemoryProvider for YourMemoryProvider {
    async fn store(&self, entry: MemoryEntry) -> MemoryResult<()> {
        // Store entry in your backend
    }

    async fn recall(&self, query: &str, opts: RecallOptions) -> MemoryResult<Vec<MemoryEntry>> {
        // Search and return relevant entries
    }

    async fn list(&self, project: &str, category: Option<&str>) -> MemoryResult<Vec<MemoryEntry>> {
        // List entries for a project
    }

    async fn delete(&self, id: &str) -> MemoryResult<()> {
        // Delete by ID
    }

    async fn flush(&self) -> MemoryResult<()> {
        // Flush any pending writes
    }

    async fn clear(&self) -> MemoryResult<()> {
        // Clear all entries
    }
}
```

### 2. Wire into MemoryFacade

The `MemoryFacade` accepts any `Box<dyn MemoryProvider>`. Add a constructor or
configuration path that creates your provider:

```rust
// In MemoryFacade or the provider initialization:
let provider: Box<dyn MemoryProvider> = match config.backend {
    MemoryBackend::Sqlite => Box::new(SqliteMemoryProvider::new(path)),
    MemoryBackend::YourBackend => Box::new(YourMemoryProvider::new(config)),
};
let facade = MemoryFacade::new(provider, config);
```

### 3. Consider the trait's semantics

- `store()` — may return immediately (async write) or block until persisted.
- `recall()` — should support FTS-like search; if your backend doesn't support it,
  implement a fallback using SQL `LIKE` or in-memory filtering.
- `flush()` — important for backends that buffer writes.
- All methods are async — don't block the async runtime.

### 4. Test

```bash
cargo test -p opendev-memory
cargo check --workspace
```

## Current Implementation

The default backend is `SqliteMemoryProvider` using SQLite with FTS5 full-text search.
See `crates/opendev-memory/src/provider.rs` and `crates/opendev-memory/src/repo.rs`
for reference.
