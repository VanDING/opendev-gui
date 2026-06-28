/**
 * Transport — 前端唯一知道平台的地方。
 *
 * 实现者：
 *   ├─ TauriTransport      (Desktop — invoke/event/channel)
 *   ├─ HttpTransport       (Web — fetch/sse)
 *   ├─ DirectTransport     (CLI embed — 直接调用)
 *   └─ MockTransport       (Testing)
 */

/** Stream controller for long-lived data streams. */
export interface StreamController<T> {
  onData(handler: (data: T) => void): () => void;
  close(): void;
}

/** Transport 接口定义 */
export interface Transport {
  /** 同步 Request/Response */
  invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;

  /** 全局事件监听 → 返回 unsubscribe */
  onEvent<T>(event: string, handler: (payload: T) => void): Promise<() => void>;

  /** 长生命周期数据流 → 返回 StreamController */
  openStream<T>(): StreamController<T>;
}

/** V1 protocol method names — matches the Rust Method enum. */
export type ProtocolMethod =
  // Session (5)
  | 'session/list' | 'session/start' | 'session/resume' | 'session/delete' | 'session/turns'
  // Turn (3)
  | 'turn/start' | 'turn/interrupt' | 'turn/steer'
  // Tool (3)
  | 'tool/list' | 'tool/search' | 'tool/approve'
  // Approval (2)
  | 'approval/list' | 'approval/respond'
  // MCP (7)
  | 'mcp/server/list' | 'mcp/server/get' | 'mcp/server/create'
  | 'mcp/server/update' | 'mcp/server/delete' | 'mcp/server/connect' | 'mcp/server/disconnect'
  // Skill (2)
  | 'skill/list' | 'skill/pin'
  // Config (5)
  | 'config/get' | 'config/update' | 'config/mode/set' | 'config/autonomy/set' | 'config/model/verify'
  // FS (3)
  | 'fs/browse' | 'fs/verify-path' | 'fs/list-workspace'
  // Workspace (2)
  | 'workspace/list' | 'workspace/get';

/**
 * V1 transport — extends base Transport with protocol-typed methods.
 * During migration, use invoke() for legacy Tauri commands.
 * After v0.3.0, switch to protocolCall() with typed methods.
 */
export interface V1Transport extends Transport {
  /** Call a v1 protocol method with typed request/response. */
  protocolCall<TParams, TResult>(
    method: ProtocolMethod,
    params: TParams
  ): Promise<TResult>;
}
