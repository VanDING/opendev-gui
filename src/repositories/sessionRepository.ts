import type { Transport } from './Transport';

export function createSessionRepository(transport: Transport) {
  return {
    listSessions: () => transport.invoke<any[]>('list_sessions'),
    createSession: (working_directory?: string) =>
      transport.invoke<{ id: string; status: string }>('create_session', {
        req: { working_directory },
      }),
    getSession: (id: string) => transport.invoke<any>('get_session', { id }),
    deleteSession: (id: string) =>
      transport.invoke<void>('delete_session', { id }),
    resumeSession: (id: string) =>
      transport.invoke<string>('resume_session', { id }),
    getSessionMessages: (id: string) =>
      transport.invoke<any[]>('get_session_messages', { id }),
    getSessionModel: (id: string) =>
      transport.invoke<any>('get_session_model', { id }),
    updateSessionModel: (id: string, model?: string, provider?: string) =>
      transport.invoke<void>('update_session_model', {
        id,
        req: { model, provider },
      }),
    clearSessionModel: (id: string) =>
      transport.invoke<void>('clear_session_model', { id }),
  };
}
