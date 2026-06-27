/**
 * EventBridge — 统一事件入口。
 *
 * 替代 wsClient.on()，使用 Tauri IPC listen() 接收事件。
 * 提供与 wsClient 相同的接口，使 stores 无需修改处理函数。
 */

import { tauriTransport } from '../repositories/TauriTransport';
import type { WSMessage } from '../types';

type EventHandler = (message: WSMessage) => void;

class EventBridge {
  private handlers: Map<string, Set<EventHandler>> = new Map();
  private unlisteners: Map<string, () => void> = new Map();

  /**
   * 订阅事件，与 wsClient.on() 接口兼容。
   * handler 接收 { type, data } 格式的消息。
   * 返回 unsubscribe 函数。
   */
  async on(eventType: string, handler: EventHandler): Promise<() => void> {
    // Register handler
    if (!this.handlers.has(eventType)) {
      this.handlers.set(eventType, new Set());
    }
    this.handlers.get(eventType)!.add(handler);

    // Subscribe to Tauri event once per type
    if (!this.unlisteners.has(eventType)) {
      const unlisten = await tauriTransport.onEvent<any>(eventType, (rawPayload) => {
        const msg: WSMessage = { type: eventType as any, data: rawPayload };
        const handlers = this.handlers.get(eventType);
        if (handlers) {
          handlers.forEach(h => h(msg));
        }
      });
      this.unlisteners.set(eventType, unlisten);
    }

    // Return unsubscribe function
    return () => {
      const handlers = this.handlers.get(eventType);
      if (handlers) {
        handlers.delete(handler);
        if (handlers.size === 0) {
          // Clean up Tauri listener when no more handlers
          const unlisten = this.unlisteners.get(eventType);
          if (unlisten) {
            unlisten();
            this.unlisteners.delete(eventType);
          }
        }
      }
    };
  }
}

export const eventBridge = new EventBridge();
