/**
 * TauriTransport — Desktop 平台的 Tauri IPC 实现。
 *
 * 使用 Tauri 的 invoke() 和 listen() API 实现 Transport 接口。
 */

import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import { listen as tauriListen } from '@tauri-apps/api/event';
import type { Transport, V1Transport, ProtocolMethod, StreamController } from './Transport';

class TauriTransportImpl implements Transport, V1Transport {
  async invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
    return tauriInvoke<T>(command, args);
  }

  async onEvent<T>(event: string, handler: (payload: T) => void): Promise<() => void> {
    const unlisten = await tauriListen<T>(event, (eventPayload) => {
      handler(eventPayload.payload);
    });
    return unlisten;
  }

  openStream<T>(): StreamController<T> {
    return new TauriStreamController<T>();
  }

  /**
   * V1 protocol method call.
   * Maps protocol method names to Tauri command names.
   * Legacy invoke() path is kept as fallback for the migration period.
   */
  async protocolCall<TParams, TResult>(
    method: ProtocolMethod,
    params: TParams
  ): Promise<TResult> {
    // Map v1 method to existing Tauri command name
    const command = this.mapMethodToCommand(method);
    return tauriInvoke<TResult>(command, { req: params } as Record<string, unknown>);
  }

  /**
   * Maps v1 protocol method names to existing Tauri command names.
   * This is temporary — after v0.3.0, new Tauri commands will use v1 names directly.
   */
  private mapMethodToCommand(method: ProtocolMethod): string {
    const mapping: Record<ProtocolMethod, string> = {
      'session/list': 'list_sessions',
      'session/start': 'create_session',
      'session/resume': 'resume_session',
      'session/delete': 'delete_session',
      'session/turns': 'get_session_messages', // fallback
      'turn/start': 'send_chat_query',
      'turn/interrupt': 'interrupt_chat',
      'turn/steer': 'send_chat_query', // fallback
      'tool/list': 'list_skills', // fallback until new command
      'tool/search': 'list_skills', // fallback
      'tool/approve': 'approve_tool',
      'approval/list': 'get_session_messages', // fallback
      'approval/respond': 'approve_tool',
      'mcp/server/list': 'list_mcp_servers',
      'mcp/server/get': 'get_mcp_server',
      'mcp/server/create': 'create_mcp_server',
      'mcp/server/update': 'update_mcp_server',
      'mcp/server/delete': 'delete_mcp_server',
      'mcp/server/connect': 'connect_mcp_server',
      'mcp/server/disconnect': 'disconnect_mcp_server',
      'skill/list': 'list_skills',
      'skill/pin': 'toggle_skill_pin',
      'config/get': 'get_app_config',
      'config/update': 'update_app_config',
      'config/mode/set': 'set_operation_mode',
      'config/autonomy/set': 'set_autonomy_level',
      'config/model/verify': 'verify_model',
      'fs/browse': 'browse_directory',
      'fs/verify-path': 'verify_path',
      'fs/list-workspace': 'list_workspace_files',
      'workspace/list': 'list_sessions', // fallback
      'workspace/get': 'get_session',   // fallback
    };
    return mapping[method] ?? method.replace(/\//g, '_');
  }
}

class TauriStreamController<T> implements StreamController<T> {
  private handlers: Set<(data: T) => void> = new Set();
  private _closed = false;

  onData(handler: (data: T) => void): () => void {
    this.handlers.add(handler);
    return () => {
      this.handlers.delete(handler);
    };
  }

  close(): void {
    this._closed = true;
    this.handlers.clear();
  }

  /** Called by the backend to push data. */
  push(data: T): void {
    if (this._closed) return;
    for (const handler of this.handlers) {
      handler(data);
    }
  }

  get closed(): boolean {
    return this._closed;
  }
}

export const tauriTransport = new TauriTransportImpl();
export type { TauriTransportImpl };
