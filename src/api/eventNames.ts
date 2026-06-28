/**
 * Centralized v1 protocol event name constants.
 * 
 * Use these with eventBridge.on() instead of magic strings.
 * ESLint rule enforces first argument of eventBridge.on must be imported from here.
 * 
 * See: docs/architecture/protocol-naming.md
 * See: ADR-008 (App-Server Protocol v1)
 * 
 * Migration: During the dual-emit period (v0.2.0 → v0.3.0), both legacy names
 * and these v1 names are emitted. Frontend handlers migrate incrementally.
 * Legacy aliases are removed in v0.4.0.
 */

// ── Message Lifecycle ──
export const MESSAGE_STARTED = 'message/started' as const;
export const MESSAGE_CHUNKED = 'message/chunked' as const;
export const MESSAGE_COMPLETED = 'message/completed' as const;

// ── Thinking ──
export const THINKING_CHUNKED = 'thinking/chunked' as const;

// ── Tool Events ──
export const TOOL_STARTED = 'tool/started' as const;
export const TOOL_COMPLETED = 'tool/completed' as const;

// ── Subagent Events ──
export const SUBAGENT_SPAWNED = 'subagent/spawned' as const;
export const SUBAGENT_COMPLETED = 'subagent/completed' as const;
export const NESTED_TOOL_STARTED = 'nested/tool/started' as const;
export const NESTED_TOOL_COMPLETED = 'nested/tool/completed' as const;

// ── Status & Progress ──
export const STATUS_UPDATED = 'status/updated' as const;
export const PROGRESS_UPDATED = 'progress/updated' as const;

// ── Approval / Ask / Plan ──
export const APPROVAL_REQUIRED = 'approval/required' as const;
export const ASK_REQUIRED = 'ask/required' as const;
export const PLAN_REQUIRED = 'plan/required' as const;

// ── Session ──
export const SESSION_ACTIVITY = 'session/activity' as const;

// ── MCP ──
export const MCP_SERVER_CONNECTED = 'mcp/server/connected' as const;

// ── Error ──
export const ERROR_RAISED = 'error/raised' as const;

// ── Type exports ──
/** Union type of all v1 protocol event names. */
export type ProtocolEventName =
  | typeof MESSAGE_STARTED
  | typeof MESSAGE_CHUNKED
  | typeof MESSAGE_COMPLETED
  | typeof THINKING_CHUNKED
  | typeof TOOL_STARTED
  | typeof TOOL_COMPLETED
  | typeof SUBAGENT_SPAWNED
  | typeof SUBAGENT_COMPLETED
  | typeof NESTED_TOOL_STARTED
  | typeof NESTED_TOOL_COMPLETED
  | typeof STATUS_UPDATED
  | typeof PROGRESS_UPDATED
  | typeof APPROVAL_REQUIRED
  | typeof ASK_REQUIRED
  | typeof PLAN_REQUIRED
  | typeof SESSION_ACTIVITY
  | typeof MCP_SERVER_CONNECTED
  | typeof ERROR_RAISED;

/** All v1 protocol event names as an array (for iteration). */
export const ALL_V1_EVENT_NAMES: readonly ProtocolEventName[] = [
  MESSAGE_STARTED, MESSAGE_CHUNKED, MESSAGE_COMPLETED,
  THINKING_CHUNKED,
  TOOL_STARTED, TOOL_COMPLETED,
  SUBAGENT_SPAWNED, SUBAGENT_COMPLETED,
  NESTED_TOOL_STARTED, NESTED_TOOL_COMPLETED,
  STATUS_UPDATED, PROGRESS_UPDATED,
  APPROVAL_REQUIRED, ASK_REQUIRED, PLAN_REQUIRED,
  SESSION_ACTIVITY,
  MCP_SERVER_CONNECTED,
  ERROR_RAISED,
];

/**
 * Maps legacy event names to their v1 equivalents.
 * Used during the dual-emit migration period.
 */
export const LEGACY_TO_V1: Record<string, ProtocolEventName> = {
  'message_start': MESSAGE_STARTED,
  'message_chunk': MESSAGE_CHUNKED,
  'message_complete': MESSAGE_COMPLETED,
  'thinking_block': THINKING_CHUNKED,
  'tool_call': TOOL_STARTED,
  'tool_result': TOOL_COMPLETED,
  'subagent_start': SUBAGENT_SPAWNED,
  'subagent_complete': SUBAGENT_COMPLETED,
  'nested_tool_call': NESTED_TOOL_STARTED,
  'nested_tool_result': NESTED_TOOL_COMPLETED,
  'status_update': STATUS_UPDATED,
  'progress': PROGRESS_UPDATED,
  'approval_required': APPROVAL_REQUIRED,
  'approval_resolved': APPROVAL_REQUIRED,
  'ask_user_required': ASK_REQUIRED,
  'ask_user_resolved': ASK_REQUIRED,
  'plan_approval_required': PLAN_REQUIRED,
  'plan_approval_resolved': PLAN_REQUIRED,
  'session_activity': SESSION_ACTIVITY,
  'mcp_servers_updated': MCP_SERVER_CONNECTED,
  'mcp_servers_update': MCP_SERVER_CONNECTED,
  'mcp:servers_updated': MCP_SERVER_CONNECTED,
  'mcp:status_changed': MCP_SERVER_CONNECTED,
  'mcp.server.connected': MCP_SERVER_CONNECTED,
  'mcp.server.disconnected': MCP_SERVER_CONNECTED,
  'mcp_status_update': MCP_SERVER_CONNECTED,
  'error': ERROR_RAISED,
};
