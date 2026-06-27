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
