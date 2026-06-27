# ADR-002: Event-Sourced Session History

## Status

Accepted 2026-06-24

## Context

The agent needs to persist session history — user messages, tool calls, LLM responses,
and system events. The persistence system must support:
- Append-only writes for performance
- Full replay of any session
- Incremental cost tracking across a session
- Snapshotting for long sessions
- Search across sessions

Two approaches were available: traditional row-based storage (each message a row) and
event sourcing (each event appended to a stream).

## Decision

Session history uses event sourcing. The `SessionEvent` enum in `opendev-models` defines
the event types. The event store (`opendev-history`) persists events as JSONL files
(append-only) with a SQLite index for query and search.

The architecture is:
- `EventStore` — appends `SessionEvent` to JSONL, flushes to SQLite index
- `SqliteSessionStore` — SQLite-backed session CRUD (currently being implemented)
- `Projector` — projects event streams into queryable views
- `Snapshot` — periodic snapshots for compaction of long sessions

## Alternatives

- **Row-per-message SQLite** — simpler but loses the event ordering and makes replay
  harder. Cost tracking would need separate tables.
- **Full SQLite with serialized blobs** — loses queryability of individual events.
- **External database (PostgreSQL)** — overkill for a local desktop app; adds deployment
  complexity.

## Consequences

- Session replay is free — replay the event stream to reconstruct state.
- Cost tracking is incremental — each LLM call event carries usage data.
- JSONL files enable debugging by reading raw event files.
- Event schema evolution requires careful migration (new fields are optional, variants
  are added, never removed).
- Snapshotting is needed for sessions with >1000 events to avoid replay cost.

## References

- ADR-007: SQLite for Persistence
- `crates/opendev-history/src/event_store.rs`
- `crates/opendev-history/src/sqlite_store.rs`
- `crates/opendev-models/src/session.rs` — `SessionEvent` enum
