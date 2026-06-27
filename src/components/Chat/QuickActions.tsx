import { Search, Code2, FileText, Box } from 'lucide-react';

interface QuickAction {
  id: string;
  command: string;
  description: string;
  icon: React.ComponentType<{ className?: string }>;
  colorClass: string;
  borderClass: string;
}

const ACTIONS: QuickAction[] = [
  {
    id: 'audit',
    command: '/audit',
    description: 'Audit code for issues\nand improvements',
    icon: Search,
    colorClass: 'text-intent-success',
    borderClass: 'group-hover:border-intent-success-muted',
  },
  {
    id: 'review',
    command: '/review',
    description: 'Review PR or code\nchanges',
    icon: Code2,
    colorClass: 'text-intent-info',
    borderClass: 'group-hover:border-intent-info-muted',
  },
  {
    id: 'explain',
    command: '/explain',
    description: 'Explain code, error,\nor architecture',
    icon: FileText,
    colorClass: 'text-intent-purple',
    borderClass: 'group-hover:border-intent-purple-muted',
  },
  {
    id: 'build',
    command: '/build',
    description: 'Generate code, tests,\nor documentation',
    icon: Box,
    colorClass: 'text-accent-magenta',
    borderClass: 'group-hover:border-accent-magenta-muted',
  },
];

interface QuickActionsProps {
  onSelect?: (command: string) => void;
  onBrowseAll?: () => void;
}

export function QuickActions({ onSelect, onBrowseAll }: QuickActionsProps) {
  return (
    <div className="w-full">
      <div className="flex items-center gap-2 mb-3">
        <span className="text-[10px] font-mono font-semibold uppercase tracking-[0.25em] text-content-tertiary">
          Quick Actions
        </span>
        <span className="flex-1 h-px bg-border-subtle" />
      </div>

      <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
        {ACTIONS.map(({ id, command, description, icon: Icon, colorClass, borderClass }) => (
          <button
            key={id}
            type="button"
            onClick={() => onSelect?.(command)}
            className={[
              'group text-left px-4 py-4 rounded-md',
              'bg-surface-elevated border border-border-subtle',
              'transition-colors duration-150',
              'hover:bg-surface-2',
              borderClass,
            ].join(' ')}
          >
            <div className="flex items-center gap-2 mb-2">
              <Icon className={`w-4 h-4 ${colorClass}`} />
              <span className="font-mono text-sm font-medium text-content-primary">
                {command}
              </span>
            </div>
            <p className="text-xs text-content-tertiary whitespace-pre-line leading-relaxed font-mono">
              {description}
            </p>
          </button>
        ))}
      </div>

      <button
        type="button"
        onClick={onBrowseAll}
        className="mt-4 inline-flex items-center gap-1.5 text-xs font-mono text-accent-primary hover:text-accent-primary-hover transition-colors"
      >
        Browse all commands
        <span aria-hidden>→</span>
      </button>
    </div>
  );
}
