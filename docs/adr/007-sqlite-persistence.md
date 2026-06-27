# ADR-007: SQLite for Persistence

## Status

Accepted 2026-06-24

## Context

The project needs local persistence for sessions, memory, and cost tracking.
The desktop app runs locally — no server deployment. Requirements:
- No external database server — the app must work offline.
- Concurrent reads (UI reads session list while agent writes events).
- Full-text search for memory recall.
- Schema migrations for forward compatibility.

## Decision

SQLite is used as the embedded database engine. It serves three persistence subsystems:
- **Session store** (`opendev-history`): session CRUD, message storage, cost records.
- **Memory system** (`opendev-memory`): long/short-term memory with FTS5 full-text search.
- **Cost tracking** (`opendev-history`): per-session cost records.

Configuration:
- WAL mode for concurrent read/write.
- `rusqlite` with `bundled-full` feature (no system SQLite dependency).
- `rusqlite_migration` for schema versioning.
- FTS5 for memory search.

## Alternatives

- **PostgreSQL** — overkill for a desktop app; requires server installation.
- **Redis** — in-memory, not persistent by default; would need snapshotting.
- **sled (embedded database)** — embedded but less mature ecosystem than SQLite.
- **Plain JSON files** — used for event store (JSONL for append-only events), but
  query performance degrades without indexing.

## Consequences

- Zero-deployment database — SQLite is bundled with the app binary.
- WAL mode allows concurrent reads without writer blocking.
- FTS5 enables efficient memory recall queries.
- Schema migrations must be backward-compatible (add columns, don't remove).
- SQLite write concurrency is inherently serialized (one writer at a time) — acceptable
  for a local desktop app.

## References

- ADR-002: Event-Sourced Session History
- `crates/opendev-history/src/sqlite_store.rs`
- `crates/opendev-memory/src/repo.rs`
- `crates/opendev-memory/src/migration.rs`
- Root `Cargo.toml` — `rusqlite`, `rusqlite_migration` workspace dependencies
