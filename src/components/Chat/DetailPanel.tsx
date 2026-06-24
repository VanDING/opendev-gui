import { X } from 'lucide-react';
import { useSubagentStore } from '../../stores/subagents';
import { useTodoStore } from '../../stores/todo';
import { TodoPanel } from './TodoPanel';
import { SubagentTree } from './SubagentTree';

interface DetailPanelProps {
  collapsed: boolean;
  onToggle: () => void;
}

export function DetailPanel({ collapsed, onToggle }: DetailPanelProps) {
  const hasAgents = useSubagentStore(s => s.order.length > 0);
  const hasTodos = useTodoStore(s => s.items.length > 0);

  if (!hasAgents && !hasTodos) return null;
  if (collapsed) return null;

  return (
    <div className="w-80 border-l border-border-default bg-surface-elevated flex flex-col shrink-0">
      {/* Header */}
      <div className="flex items-center px-3 py-2 border-b border-border-default">
        <span className="text-xs font-mono font-semibold uppercase tracking-wide text-content-secondary">Details</span>
        <div className="flex-1" />
        <button onClick={onToggle} className="text-content-tertiary hover:text-content-secondary">
          <X className="w-4 h-4" />
        </button>
      </div>

      {/* Todo section: 30% */}
      <div className="overflow-y-auto" style={{ flex: '3 1 0' }}>
        <TodoPanel embed />
      </div>

      {/* Divider */}
      <div className="border-t border-border-default" />

      {/* Agent section: 70% */}
      <div className="overflow-y-auto" style={{ flex: '7 1 0' }}>
        <SubagentTree embed />
      </div>
    </div>
  );
}
