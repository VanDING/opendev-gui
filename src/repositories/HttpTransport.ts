/**
 * HttpTransport — v1 protocol over WebSocket + HTTP for Web client.
 * 
 * Implements the Transport + V1Transport interfaces for browser-based clients.
 * Uses native WebSocket API + fetch() for HTTP fallback.
 */
import type { Transport, V1Transport, ProtocolMethod, StreamController } from './Transport';

export class HttpTransport implements Transport, V1Transport {
  private ws: WebSocket | null = null;
  private url: string;
  private eventHandlers: Map<string, Set<(payload: any) => void>> = new Map();
  private pendingRequests: Map<string, {
    resolve: (value: any) => void;
    reject: (error: any) => void;
  }> = new Map();

  constructor(url: string = 'ws://localhost:3000/ws') {
    this.url = url;
  }

  async connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      try {
        this.ws = new WebSocket(this.url);
        this.ws.onopen = () => {
          console.log('[HttpTransport] WebSocket connected to', this.url);
          resolve();
        };
        this.ws.onmessage = (event) => {
          try {
            const frame = JSON.parse(event.data);
            this.handleFrame(frame);
          } catch (e) {
            console.error('[HttpTransport] Failed to parse frame:', e);
          }
        };
        this.ws.onclose = () => {
          console.log('[HttpTransport] WebSocket closed');
          this.ws = null;
        };
        this.ws.onerror = (err) => {
          console.error('[HttpTransport] WebSocket error:', err);
          reject(err);
        };
      } catch (e) {
        reject(e);
      }
    });
  }

  disconnect(): void {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  // ── Transport interface ──

  async invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
    // HTTP fallback for non-streaming requests
    const resp = await fetch(`${this.url.replace('/ws', '/api')}/${command}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(args ?? {}),
    });
    if (!resp.ok) {
      throw new Error(`HTTP ${resp.status}: ${resp.statusText}`);
    }
    return resp.json();
  }

  async onEvent<T>(event: string, handler: (payload: T) => void): Promise<() => void> {
    if (!this.eventHandlers.has(event)) {
      this.eventHandlers.set(event, new Set());
    }
    this.eventHandlers.get(event)!.add(handler);

    return () => {
      const handlers = this.eventHandlers.get(event);
      if (handlers) {
        handlers.delete(handler);
        if (handlers.size === 0) {
          this.eventHandlers.delete(event);
        }
      }
    };
  }

  openStream<T>(): StreamController<T> {
    const handlers = new Set<(data: T) => void>();
    return {
      onData: (handler: (data: T) => void) => {
        handlers.add(handler);
        return () => handlers.delete(handler);
      },
      close: () => handlers.clear(),
    };
  }

  // ── V1Transport interface ──

  async protocolCall<TParams, TResult>(
    method: ProtocolMethod,
    params: TParams
  ): Promise<TResult> {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
      throw new Error('WebSocket not connected');
    }
    return new Promise((resolve, reject) => {
      const id = crypto.randomUUID();
      this.pendingRequests.set(id, { resolve, reject });

      const frame = {
        kind: 'request',
        v: { major: 1, minor: 0, patch: 0 },
        id,
        src: 'web-client',
        dst: '',
        method,
        params,
      };

      this.ws!.send(JSON.stringify(frame));

      // Timeout after 30s
      setTimeout(() => {
        if (this.pendingRequests.has(id)) {
          this.pendingRequests.delete(id);
          reject(new Error(`Request ${method} timed out`));
        }
      }, 30_000);
    });
  }

  // ── Internal frame handling ──

  private handleFrame(frame: any): void {
    switch (frame.kind) {
      case 'response': {
        const pending = this.pendingRequests.get(frame.id);
        if (pending) {
          this.pendingRequests.delete(frame.id);
          pending.resolve(frame.result);
        }
        break;
      }
      case 'notification': {
        const handlers = this.eventHandlers.get(frame.event);
        if (handlers) {
          handlers.forEach(h => h(frame.data));
        }
        break;
      }
      case 'error': {
        const pending = frame.id ? this.pendingRequests.get(frame.id) : undefined;
        if (pending) {
          this.pendingRequests.delete(frame.id!);
          pending.reject(new Error(frame.message));
        }
        break;
      }
    }
  }
}
