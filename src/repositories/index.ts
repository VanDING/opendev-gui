import { tauriTransport } from './TauriTransport';
import { createConfigRepository } from './configRepository';
import { createSessionRepository } from './sessionRepository';
import { createChatRepository } from './chatRepository';
import { createWorkflowRepository } from './workflowRepository';
import { createMCPServerRepository } from './mcpRepository';
import { createSkillRepository } from './skillRepository';
import { createFileRepository } from './fileRepository';

// Create production repositories using TauriTransport
// These are the single instances used by the app
export const configRepository = createConfigRepository(tauriTransport);
export const sessionRepository = createSessionRepository(tauriTransport);
export const chatRepository = createChatRepository(tauriTransport);
export const workflowRepository = createWorkflowRepository(tauriTransport);
export const mcpRepository = createMCPServerRepository(tauriTransport);
export const skillRepository = createSkillRepository(tauriTransport);
export const fileRepository = createFileRepository(tauriTransport);

export { tauriTransport } from './TauriTransport';
export type { Transport, StreamController } from './Transport';
