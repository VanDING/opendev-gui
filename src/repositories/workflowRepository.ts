import type { Transport } from './Transport';

export function createWorkflowRepository(transport: Transport) {
  return {
    approveTool: (
      approval_id: string,
      approved: boolean,
      auto_approve?: boolean,
    ) =>
      transport.invoke<any>('approve_tool', {
        req: { approval_id, approved, auto_approve },
      }),
    respondToAsk: (
      request_id: string,
      answers: Record<string, unknown> | null,
      cancelled?: boolean,
    ) =>
      transport.invoke<any>('respond_to_ask', {
        req: { request_id, answers, cancelled },
      }),
    respondToPlan: (
      request_id: string,
      action: string,
      feedback?: string,
    ) =>
      transport.invoke<any>('respond_to_plan', {
        req: { request_id, action, feedback },
      }),
  };
}
