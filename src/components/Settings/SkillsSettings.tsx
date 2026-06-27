import { useState, useEffect } from 'react';
import { Button } from '../ui/Button';
import { skillRepository } from '../../repositories';

interface Skill {
  name: string;
  description: string;
  namespace: string;
  source: string;
  pinned: boolean;
  status: string;
  usage_count: number;
  tags: string[];
}

export function SkillsSettings() {
  const [skills, setSkills] = useState<Skill[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadSkills();
  }, []);

  async function loadSkills() {
    setLoading(true);
    setError(null);
    try {
      const data = await skillRepository.listSkills();
      setSkills(data);
    } catch (e: any) {
      setError(e.message);
    } finally {
      setLoading(false);
    }
  }

  async function handleTogglePin(name: string) {
    try {
      const result = await skillRepository.togglePin(name);
      setSkills(prev => prev.map(s => s.name === name ? { ...s, pinned: result.pinned ?? !s.pinned } : s));
    } catch (e: any) {
      setError(e.message);
    }
  }

  if (loading) {
    return (
      <div className="space-y-4">
        <h3 className="text-sm font-semibold text-content-primary mb-1">Skills</h3>
        <p className="text-xs text-content-tertiary mb-5">
          Manage available skills for the AI agent.
        </p>
        {[1, 2, 3].map(i => (
          <div key={i} className="animate-pulse bg-surface-2 rounded-lg h-24" />
        ))}
      </div>
    );
  }

  if (error) {
    return (
      <div>
        <h3 className="text-sm font-semibold text-content-primary mb-1">Skills</h3>
        <div className="bg-intent-danger-muted text-intent-danger text-sm p-3 rounded-lg mb-3">
          {error}
        </div>
        <Button onClick={loadSkills} variant="secondary" size="sm">Retry</Button>
      </div>
    );
  }

  if (skills.length === 0) {
    return (
      <div>
        <h3 className="text-sm font-semibold text-content-primary mb-1">Skills</h3>
        <p className="text-xs text-content-tertiary mb-5">
          Manage available skills for the AI agent.
        </p>
        <p className="text-sm text-content-tertiary">No skills found.</p>
      </div>
    );
  }

  return (
    <div>
      <h3 className="text-sm font-semibold text-content-primary mb-1">Skills</h3>
      <p className="text-xs text-content-tertiary mb-5">
        Manage available skills for the AI agent.
      </p>

      <div className="space-y-3">
        {skills.map(skill => (
          <div
            key={skill.name}
            className="border border-border-default rounded-lg bg-surface-elevated p-4"
          >
            <div className="flex items-start justify-between gap-3">
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 mb-1">
                  <span className="text-sm font-semibold text-content-primary truncate">
                    {skill.name}
                  </span>
                  <span className={`text-xs px-1.5 py-0.5 rounded-full ${
                    skill.source === 'builtin'
                      ? 'bg-accent-primary-muted text-accent-primary'
                      : 'bg-surface-2 text-content-tertiary'
                  }`}>
                    {skill.source}
                  </span>
                  {skill.status !== 'Active' && (
                    <span className="text-xs text-content-tertiary">{skill.status}</span>
                  )}
                </div>
                <p className="text-xs text-content-secondary line-clamp-2 mb-2">
                  {skill.description}
                </p>
                {skill.tags.length > 0 && (
                  <div className="flex flex-wrap gap-1">
                    {skill.tags.map(tag => (
                      <span key={tag} className="text-[10px] px-1.5 py-0.5 bg-surface-2 text-content-tertiary rounded-full">
                        {tag}
                      </span>
                    ))}
                  </div>
                )}
              </div>

              <div className="flex flex-col items-end gap-2 flex-shrink-0">
                <button
                  onClick={() => handleTogglePin(skill.name)}
                  className={`px-3 py-1 text-xs font-medium rounded-md border transition-colors ${
                    skill.pinned
                      ? 'bg-accent-primary text-content-inverse border-accent-primary'
                      : 'bg-surface-2 text-content-secondary border-border-default hover:border-accent-primary hover:text-accent-primary'
                  }`}
                >
                  {skill.pinned ? 'Pinned' : 'Pin'}
                </button>
                <span className="text-[10px] text-content-tertiary">
                  used {skill.usage_count} times
                </span>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
