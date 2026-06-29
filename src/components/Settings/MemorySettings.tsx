import { useState } from 'react';

interface MemoryEntry {
  id: string;
  content: string;
  category: string;
  confidence: number;
  created_at: string;
}

/**
 * Memory settings panel — view/manage memories, configure retention.
 */
export function MemorySettings() {
  const [memories] = useState<MemoryEntry[]>([
    { id: '1', content: 'User prefers Python for data analysis tasks', category: 'UserPreference', confidence: 0.9, created_at: '2024-01-15' },
    { id: '2', content: 'Project uses FastAPI for the backend API', category: 'ProjectFact', confidence: 0.8, created_at: '2024-01-14' },
    { id: '3', content: 'Decision: Use SQLAlchemy for ORM', category: 'Decision', confidence: 0.7, created_at: '2024-01-13' },
  ]);
  const [maxMemories, setMaxMemories] = useState(500);
  const [ttlDays, setTtlDays] = useState(90);
  const [selectedMemory, setSelectedMemory] = useState<string | null>(null);

  const deleteMemory = (id: string) => {
    // In real implementation, call memory API
    console.log('Delete memory:', id);
  };

  return (
    <div className="space-y-4">
      <div>
        <h3 className="text-sm font-semibold text-content-primary mb-1">Memory Retention</h3>
        <div className="space-y-2">
          <label className="flex items-center gap-2 text-sm text-content-secondary">
            <span className="w-32">Max memories:</span>
            <input
              type="number"
              value={maxMemories}
              onChange={e => setMaxMemories(parseInt(e.target.value) || 500)}
              min={50}
              max={10000}
              className="w-24 px-2 py-1 text-sm bg-surface-primary border border-border-default/20 rounded"
            />
          </label>
          <label className="flex items-center gap-2 text-sm text-content-secondary">
            <span className="w-32">TTL (days):</span>
            <input
              type="number"
              value={ttlDays}
              onChange={e => setTtlDays(parseInt(e.target.value) || 90)}
              min={1}
              max={365}
              className="w-24 px-2 py-1 text-sm bg-surface-primary border border-border-default/20 rounded"
            />
          </label>
        </div>
      </div>

      <div>
        <h3 className="text-sm font-semibold text-content-primary mb-1">Stored Memories ({memories.length})</h3>
        <div className="space-y-1 max-h-64 overflow-y-auto">
          {memories.map(m => (
            <div
              key={m.id}
              className={`px-3 py-2 rounded-lg cursor-pointer transition-colors ${
                selectedMemory === m.id ? 'bg-accent-secondary-muted/20 border border-accent-secondary/30' : 'bg-surface-2 hover:bg-surface-elevated'
              }`}
              onClick={() => setSelectedMemory(selectedMemory === m.id ? null : m.id)}
            >
              <div className="flex items-center justify-between">
                <span className={`text-[10px] font-mono px-1.5 py-0.5 rounded ${
                  m.category === 'UserPreference' ? 'bg-accent-primary-muted/20 text-accent-primary-muted' :
                  m.category === 'ProjectFact' ? 'bg-intent-success-muted/20 text-intent-success-muted' :
                  'bg-intent-warning-muted/20 text-intent-warning-muted'
                }`}>
                  {m.category}
                </span>
                <div className="flex items-center gap-2">
                  <span className="text-xs text-content-tertiary">{m.created_at}</span>
                  <span className="text-[10px] text-content-tertiary">{Math.round(m.confidence * 100)}%</span>
                </div>
              </div>
              <p className="text-sm text-content-secondary mt-1 line-clamp-2">{m.content}</p>
              {selectedMemory === m.id && (
                <div className="mt-2 flex gap-2">
                  <button
                    onClick={(e) => { e.stopPropagation(); deleteMemory(m.id); }}
                    className="text-xs text-intent-danger-muted hover:underline"
                  >
                    Delete
                  </button>
                </div>
              )}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
