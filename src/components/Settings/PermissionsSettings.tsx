import { useState } from 'react';

interface PermissionRule {
  id: string;
  pattern: string;
  action: 'allow' | 'deny' | 'prompt';
  priority: number;
}

/**
 * Permissions settings panel — view/edit allow/deny rules with pattern builder UI.
 */
export function PermissionsSettings() {
  const [rules, setRules] = useState<PermissionRule[]>([
    { id: '1', pattern: 'bash:ls *', action: 'allow', priority: 100 },
    { id: '2', pattern: 'bash:rm -rf *', action: 'deny', priority: 100 },
    { id: '3', pattern: 'edit:*', action: 'prompt', priority: 50 },
  ]);
  const [newPattern, setNewPattern] = useState('');
  const [newAction, setNewAction] = useState<'allow' | 'deny' | 'prompt'>('prompt');

  const addRule = () => {
    if (!newPattern.trim()) return;
    setRules(prev => [...prev, {
      id: crypto.randomUUID?.() || `${Date.now()}`,
      pattern: newPattern.trim(),
      action: newAction,
      priority: 50,
    }]);
    setNewPattern('');
  };

  const removeRule = (id: string) => {
    setRules(prev => prev.filter(r => r.id !== id));
  };

  return (
    <div className="space-y-4">
      <div>
        <h3 className="text-sm font-semibold text-content-primary mb-1">Permission Rules</h3>
        <p className="text-xs text-content-tertiary">
          Define patterns for auto-allow, auto-deny, or prompt on tool invocations.
        </p>
      </div>

      {/* Existing rules */}
      <div className="space-y-1 max-h-60 overflow-y-auto">
        {rules.map(rule => (
          <div key={rule.id} className="flex items-center gap-2 px-3 py-2 bg-surface-2 rounded-lg text-sm">
            <span className={`text-xs font-mono font-bold px-1.5 py-0.5 rounded ${
              rule.action === 'allow' ? 'bg-intent-success-muted/20 text-intent-success-muted' :
              rule.action === 'deny' ? 'bg-intent-danger-muted/20 text-intent-danger-muted' :
              'bg-intent-warning-muted/20 text-intent-warning-muted'
            }`}>
              {rule.action}
            </span>
            <code className="flex-1 text-content-secondary font-mono text-xs">{rule.pattern}</code>
            <span className="text-[10px] text-content-tertiary">p{rule.priority}</span>
            <button
              onClick={() => removeRule(rule.id)}
              className="text-intent-danger-muted hover:text-intent-danger text-xs"
            >
              ✕
            </button>
          </div>
        ))}
      </div>

      {/* Add rule form */}
      <div className="flex items-center gap-2">
        <input
          type="text"
          value={newPattern}
          onChange={e => setNewPattern(e.target.value)}
          placeholder="e.g. bash:git *"
          className="flex-1 px-3 py-1.5 text-sm bg-surface-primary border border-border-default/20 rounded-lg text-content-primary placeholder-content-tertiary focus:outline-none focus:ring-2 focus:ring-accent-secondary"
          onKeyDown={e => e.key === 'Enter' && addRule()}
        />
        <select
          value={newAction}
          onChange={e => setNewAction(e.target.value as any)}
          className="px-2 py-1.5 text-sm bg-surface-primary border border-border-default/20 rounded-lg text-content-secondary"
        >
          <option value="allow">Allow</option>
          <option value="deny">Deny</option>
          <option value="prompt">Prompt</option>
        </select>
        <button
          onClick={addRule}
          className="px-3 py-1.5 text-sm font-medium text-content-inverse bg-accent-secondary rounded-lg hover:bg-accent-secondary/90 transition-colors"
        >
          Add
        </button>
      </div>

      <div className="text-xs text-content-tertiary space-y-1">
        <p>Pattern format: <code className="text-accent-secondary">tool_name:pattern</code></p>
        <p>Examples: <code className="text-accent-secondary">bash:ls *</code>, <code className="text-accent-secondary">edit:src/**</code></p>
      </div>
    </div>
  );
}
