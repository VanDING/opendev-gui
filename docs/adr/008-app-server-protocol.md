# ADR-008: App-Server Protocol v1

## Status

Accepted 2026-06-28

## Context

OpenDev Desktop has grown 5 client surfaces (Tauri desktop, CLI/TUI, Web, Telegram, Workspace)
that all communicate with the same agent runtime. Currently there are 3 parallel protocol
surfaces doing the same job:

1. **`FrontendEvent`** (opendev-models) — 17 variant tagged union with ts-rs exports, but
   never actually serialized over the wire.
2. **`WSMessage`** (src/types/index.ts) — 32 string literal union, the actual running
   protocol shape consumed by 25+ frontend handlers.
3. **`WsMessageType`** (opendev-web) — 40 variant enum with serde(rename) to strings, used
   by the WebSocket web layer but not the Tauri binary.

Additionally, event naming uses 4 inconsistent conventions:
- `mcp_servers_updated` (snake_case)
- `mcp:servers_updated` (colon)
- `mcp.server.connected` (dot)
- `mcp_status_update` (underscore alias, dead code)

There is no `protocol_version` field, no version negotiation, and the Tauri bridge
(`src-tauri/src/server.rs`) is an explicitly temporary shim that must eventually be removed.

Without a unified protocol, each new client surface requires ad-hoc string matching;
adding new methods or events risks breaking existing clients silently; and there is
no path for protocol evolution.

## Decision

Create a new `opendev-protocol` crate as the single source of truth for the app-server
wire protocol. The protocol follows these design principles:

### Naming Convention

- **Methods:** `<domain>/<verb>` (30 methods) — e.g., `session/start`, `turn/interrupt`, `mcp/server/list`
- **Events:** `<noun>/<past-tense>` (18 events) — e.g., `message/chunked`, `tool/started`, `approval/required`
- **Fields:** snake_case on wire, camelCase in TypeScript (via ts-rs)

### Wire Format

JSON over a JSON-RPC 2.0-like envelope with fixed field order:

```json
{
  "v": {"major": 1, "minor": 0, "patch": 0},
  "id": "<uuid-v7>",
  "src": "<participant-id>",
  "dst": "<participant-id>",
  "kind": "request",
  "method": "session/start",
  "params": {...}
}
```

### Version Strategy

- **V1 (frozen):** v0.2.0 GA freeze. Bug fixes only; no new methods/fields.
- **V2 (active):** v0.3.0 start. New methods/fields; V1 clients remain compatible.
- **Negotiation:** Client sends `protocol_version` on connect; server responds with
  `min_supported` + `max_supported`. Client selects compatible version.

### Transport Layer

5 client types each implement the `Transport` trait:
- **Tauri:** `TauriTransport` (tauri::invoke + Channel<T>)
- **TUI:** `TuiInProcessTransport` (tokio mpsc, in-process)
- **Web:** `WebSocketTransport` (axum WS, JSONL frames)
- **Telegram:** deferred to V2
- **Workspace:** `UnixSocketTransport` (macOS/Linux) / `NamedPipeTransport` (Windows)

### Migration Strategy

**Dual-emit period (v0.2.0 → v0.3.0):**

| Version | Server behavior | Frontend migration |
|---------|----------------|-------------------|
| v0.2.0 | Dual-emit: legacy names + v1 names | 25+ handlers continue working |
| v0.2.x | Dual-emit continues | Frontend handlers migrate incrementally (1-2 per PR) |
| v0.3.0 | Stop dual-emit; v1 names only | All handlers migrated |
| v0.4.0 | Remove server.rs shim | Old WSMessage.type aliases removed |

TS event names are centralized in `src/api/eventNames.ts` with an ESLint rule
requiring the first argument of `eventBridge.on` to use an imported constant.

### TypeScript Bindings

All Rust types derive `ts_rs::TS` with `#[ts(export)]`. Generated `.ts` files
live in `crates/opendev-protocol/bindings/` and form the `@opendev/protocol-types`
package for consumption by all frontend clients.

## Alternatives

- **MessagePack/CBOR/bincode:** More compact, but not human-readable for debugging,
  not browser-friendly without JS library.
- **Protobuf/gRPC:** Strong schemas but heavy tooling; overkill for local IPC.
- **Continue with stringly-typed events:** Zero implementation cost, but continues
  the naming chaos and prevents protocol evolution.
- **GraphQL over WebSocket:** Complex query language; not justified for a fixed set
  of 30 methods + 18 events.

## Consequences

- Single source of truth for all wire types (Rust + TS via ts-rs).
- Typed methods and events — compiler catches name mismatches.
- Version negotiation enables safe protocol evolution.
- Dual-emit period requires both legacy and v1 event emissions (doubles Tauri IPC
  traffic temporarily).
- Frontend handlers need incremental migration (25+ sites in chat.ts).
- Server.rs shim survives until v0.4.0 (2 minor releases after protocol v1).
- All 5 client surfaces must implement the `Transport` trait (4 in scope now,
  Telegram deferred to v2).

## References

- Design: `docs/architecture/infrastructure-foundation-design.md` (§3)
- Recon: protocol recon report (tool_f0bd5566e001)
- Crate: `crates/opendev-protocol/`
- Naming: `docs/architecture/protocol-naming.md`
- Data flow: `docs/architecture/data-flow.md`
- Existing events: `crates/opendev-models/src/frontend_event.rs`
- Legacy bridge: `src-tauri/src/server.rs`
- Frontend handlers: `src/stores/chat.ts`
