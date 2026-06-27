import type { Transport } from './Transport';

export function createSkillRepository(transport: Transport) {
  return {
    listSkills: () => transport.invoke<any[]>('list_skills'),
    togglePin: (name: string) =>
      transport.invoke<{ status: string; pinned?: boolean }>(
        'toggle_skill_pin',
        { name },
      ),
  };
}
