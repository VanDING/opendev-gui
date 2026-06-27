/**
 * TauriTransport — Desktop 平台的 Tauri IPC 实现。
 *
 * 使用 Tauri 的 invoke() 和 listen() API 实现 Transport 接口。
 */

import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import { listen as tauriListen } from '@tauri-apps/api/event';
import type { Transport, StreamController } from './Transport';

class TauriTransportImpl implements Transport {
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
