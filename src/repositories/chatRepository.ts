import type { Transport, StreamController } from './Transport';

export function createChatRepository(transport: Transport) {
  return {
    sendQuery: (message: string, session_id?: string) =>
      transport.invoke<{ status: string; session_id?: string }>(
        'send_chat_query',
        { req: { message, session_id } },
      ),
    interrupt: () =>
      transport.invoke<{ status: string }>('interrupt_chat'),
    clearChat: (workspace?: string) =>
      transport.invoke<{ status: string; session_id?: string }>(
        'clear_chat',
        { workspace },
      ),
    getMessages: () => transport.invoke<any[]>('get_chat_messages'),
    openStream: <T>() => transport.openStream<T>(),
  };
}
