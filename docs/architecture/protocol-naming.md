# Protocol Naming Convention

> Reference for all v1 protocol method and event names.
> Frozen at v0.2.0 GA. New names go into v2 protocol.

## Methods: `<domain>/<verb>`

Methods are client→server RPC calls expecting a response.

| Domain | Method | Wire name | Tauri command (existing) |
|--------|--------|-----------|--------------------------|
| Session (5) | List sessions | `session/list` | `list_sessions` |
| | Create session | `session/start` | `create_session` |
| | Resume session | `session/resume` | `resume_session` |
| | Delete session | `session/delete` | `delete_session` |
| | List turns | `session/turns` | (new) |
| Turn (3) | Start turn | `turn/start` | `send_chat_query` |
| | Interrupt turn | `turn/interrupt` | `interrupt_chat` |
| | Steer turn | `turn/steer` | (new, Codex-origin) |
| Tool (3) | List tools | `tool/list` | (new via protocol) |
| | Search tools | `tool/search` | (new) |
| | Approve tool | `tool/approve` | `approve_tool` |
| Approval (2) | List approvals | `approval/list` | (new) |
| | Respond to approval | `approval/respond` | `approve_tool` |
| MCP Server (7) | List servers | `mcp/server/list` | `list_mcp_servers` |
| | Get server | `mcp/server/get` | `get_mcp_server` |
| | Create server | `mcp/server/create` | `create_mcp_server` |
| | Update server | `mcp/server/update` | `update_mcp_server` |
| | Delete server | `mcp/server/delete` | `delete_mcp_server` |
| | Connect server | `mcp/server/connect` | `connect_mcp_server` |
| | Disconnect server | `mcp/server/disconnect` | `disconnect_mcp_server` |
| Skill (2) | List skills | `skill/list` | `list_skills` |
| | Toggle pin | `skill/pin` | `toggle_skill_pin` |
| Config (5) | Get config | `config/get` | `get_app_config` |
| | Update config | `config/update` | `update_app_config` |
| | Set mode | `config/mode/set` | `set_operation_mode` |
| | Set autonomy | `config/autonomy/set` | `set_autonomy_level` |
| | Verify model | `config/model/verify` | `verify_model` |
| FS (3) | Browse directory | `fs/browse` | `browse_directory` |
| | Verify path | `fs/verify-path` | `verify_path` |
| | List workspace files | `fs/list-workspace` | `list_workspace_files` |
| Workspace (2) | List workspaces | `workspace/list` | (new) |
| | Get workspace | `workspace/get` | (new) |

## Events: `<noun>/<past-tense>`

Events are server→client notifications. No ack expected.

| Category | Event | Wire name | Legacy name | Trigger |
|----------|-------|-----------|-------------|---------|
| Message | Started | `message/started` | `message_start` | Assistant message begins |
| | Chunked | `message/chunked` | `message_chunk` | Text streamed chunk |
| | Completed | `message/completed` | `message_complete` | Message finished |
| Thinking | Chunked | `thinking/chunked` | `thinking_block` | Reasoning block |
| Tool | Started | `tool/started` | `tool_call` | Tool execution begins |
| | Completed | `tool/completed` | `tool_result` | Tool execution ends |
| Subagent | Spawned | `subagent/spawned` | `subagent_start` | Subagent created |
| | Completed | `subagent/completed` | `subagent_complete` | Subagent finished |
| Nested | Tool started | `nested/tool/started` | `nested_tool_call` | Nested tool call |
| | Tool completed | `nested/tool/completed` | `nested_tool_result` | Nested tool result |
| Status | Updated | `status/updated` | `status_update` | Token/cost update |
| Progress | Updated | `progress/updated` | `progress` | Progress change |
| Approval | Required | `approval/required` | `approval_required` | Tool needs approval |
| Ask | Required | `ask/required` | `ask_user_required` | User question needed |
| Plan | Required | `plan/required` | `plan_approval_required` | Plan needs approval |
| Session | Activity | `session/activity` | `session_activity` | Session state change |
| MCP | Server connected | `mcp/server/connected` | `mcp_servers_updated` etc. | MCP server state |
| Error | Raised | `error/raised` | `error` | Error event |

## Wire Field Conventions

| Convention | Example | Rationale |
|------------|---------|-----------|
| snake_case (wire) | `session_id`, `created_at` | Rust native, serde default |
| camelCase (TS) | `sessionId`, `createdAt` | TS native, ts-rs auto-converts |
| UUID v7 (string) | `"0193f6b4-1234-7abc-..."` | Time-ordered, sortable |
| Unix seconds (i64) | `1719590400` | Language-agnostic, no timezone ambiguity |

## Migration: Legacy → V1 Name Mapping

Implemented in `src-tauri/src/server.rs::legacy_event_name_to_v1()`.

The 4 legacy naming conventions (snake_case, colon, dot, underscore alias)
all map to a single v1 event name. See the function for the complete mapping table.

## Version Policy

- **V1 methods/events:** NEVER add new entries after v0.2.0 GA freeze.
- **V1 field additions:** Backward-compatible only (new optional fields allowed).
- **V2 methods/events:** New namespace — e.g., `realtime/voice/start`.
- **Experimental:** Marked with `#[experimental("description")]` in Rust;
  on wire, carry `"experimental": true` flag. Unstable, may change without notice.
