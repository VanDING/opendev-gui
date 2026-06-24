import React, { useEffect, useState } from 'react';
import { useSubagentStore, formatToolVerb, formatToolArg, type SubagentState, type ActiveToolCall } from '../../stores/subagents';

function formatElapsed(ms: number): string {
  const secs = Math.floor(ms / 1000);
  if (secs < 60) return `${secs}s`;
  const mins = Math.floor(secs / 60);
  const remSecs = secs % 60;
  return `${mins}m${remSecs}s`;
}

function formatTokens(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}k`;
  return String(n);
}


// ⚡ Bolt Performance Optimization:
// Isolate the high-frequency state update (1000ms interval) into a dedicated leaf component.
// This prevents `SubagentNode` and `ActiveToolRow` from re-rendering their entire subtrees every second.
const ElapsedTimeDisplay = React.memo(function ElapsedTimeDisplay({ startedAt, finished }: { startedAt: number; finished?: boolean }) {
  const [elapsed, setElapsed] = useState(() => Date.now() - startedAt);

  useEffect(() => {
    if (finished) return;
    const interval = setInterval(() => {
      setElapsed(Date.now() - startedAt);
    }, 1000);
    return () => clearInterval(interval);
  }, [startedAt, finished]);

  return <>{formatElapsed(elapsed)}</>;
});

const SPINNER_FRAMES = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

function Spinner({ className }: { className?: string }) {
  const [frame, setFrame] = useState(0);

  useEffect(() => {
    const interval = setInterval(() => {
      setFrame((f) => (f + 1) % SPINNER_FRAMES.length);
    }, 80);
    return () => clearInterval(interval);
  }, []);

  return <span className={className}>{SPINNER_FRAMES[frame]}</span>;
}

function ActiveToolRow({ tool, isLast }: { tool: ActiveToolCall; isLast: boolean }) {
  const verb = formatToolVerb(tool.toolName);
  const arg = formatToolArg(tool.toolName, tool.args);
  const connector = isLast ? '└─' : '├─';

  return (
    <div className="flex items-center gap-1.5 text-sm font-mono text-content-secondary leading-6 pl-8">
      <span className="text-content-tertiary">{connector}</span>
      <Spinner className="text-accent-primary-muted" />
      <span className="text-content-secondary">{verb}</span>
      {arg && <span className="text-content-tertiary truncate max-w-[300px]">{arg}</span>}
      <span className="text-content-tertiary ml-auto shrink-0">(<ElapsedTimeDisplay startedAt={tool.startedAt} />)</span>
    </div>
  );
}

function CompletedToolRow({ toolName, success, isLast }: { toolName: string; success: boolean; isLast: boolean }) {
  const connector = isLast ? '└─' : '├─';
  const icon = success ? '✓' : '✗';
  const color = success ? 'text-intent-success-muted' : 'text-intent-danger-muted';

  return (
    <div className="flex items-center gap-1.5 text-sm font-mono text-content-tertiary leading-6 pl-8">
      <span className="text-content-tertiary">{connector}</span>
      <span className={color}>{icon}</span>
      <span>{formatToolVerb(toolName)}</span>
    </div>
  );
}

function SubagentNode({ sa }: { sa: SubagentState }) {
  // Status indicator
  const statusEl = sa.finished ? (
    sa.success ? (
      <span className="text-intent-success-muted font-bold">✓</span>
    ) : (
      <span className="text-intent-danger-muted font-bold">✗</span>
    )
  ) : (
    <Spinner className="text-accent-primary-muted" />
  );

  // Stats string
  const tokenStr = sa.tokenCount > 0 ? ` · ${formatTokens(sa.tokenCount)} tokens` : '';
  const statsPrefix = `(${sa.toolCallCount} tool uses${tokenStr} · `;
  const statsSuffix = `)`;

  // Expanded/collapsed state
  const [expanded, setExpanded] = useState(false);

  // Display name
  const displayName = sa.name.split(/[-_]/).map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ');
  const taskPreview = sa.description.length > 37 ? sa.description.slice(0, 34) + '...' : sa.description;

  // Active tools
  const activeToolEntries = Array.from(sa.activeTools.values());
  // Show last 3 completed
  const completedVisible = sa.completedTools.slice(-3);
  const hiddenCount = Math.max(0, sa.toolCallCount - activeToolEntries.length - completedVisible.length);

  return (
    <div className="mb-1">
      {/* Header line */}
      <div
        className="flex items-center gap-1.5 text-sm font-mono leading-6 cursor-pointer select-none"
        onClick={() => setExpanded(e => !e)}
      >
        <span className="text-content-tertiary pl-2">├─</span>
        {statusEl}
        <span className="text-intent-info-muted font-semibold">{displayName}</span>
        <span className="text-content-tertiary truncate">: {taskPreview}</span>
        <span className="text-content-tertiary shrink-0 text-xs ml-1">{statsPrefix}<ElapsedTimeDisplay startedAt={sa.startedAt} finished={sa.finished} />{statsSuffix}</span>
        <span className="text-content-tertiary shrink-0 ml-1 w-4 text-center text-[10px]">{expanded ? '▲' : '▼'}</span>
      </div>

      {expanded && (
        <>
          {/* Active tool calls */}
          {activeToolEntries.map((tool, i) => (
            <ActiveToolRow
              key={tool.toolId}
              tool={tool}
              isLast={i === activeToolEntries.length - 1 && completedVisible.length === 0}
            />
          ))}

          {/* Last 3 completed tools */}
          {completedVisible.map((tool, i) => (
            <CompletedToolRow
              key={`completed-${i}`}
              toolName={tool.toolName}
              success={tool.success}
              isLast={i === completedVisible.length - 1}
            />
          ))}

          {/* Hidden count */}
          {hiddenCount > 0 && !sa.finished && (
            <div className="text-xs font-mono text-content-tertiary italic pl-10 leading-6">
              +{hiddenCount} more tool uses
            </div>
          )}

          {/* Shallow warning */}
          {sa.shallowWarning && (
            <div className="text-xs font-mono text-intent-warning-muted pl-10 leading-6">
              {sa.shallowWarning}
            </div>
          )}

          {/* Completion summary (persistent after finish) */}
          {sa.finished && (
            <div className="text-xs font-mono text-content-tertiary pl-10 leading-6">
              Done ({sa.toolCallCount} tool uses{tokenStr} · <ElapsedTimeDisplay startedAt={sa.startedAt} finished={sa.finished} />)
            </div>
          )}
        </>
      )}
    </div>
  );
}

interface SubagentTreeProps {
  embed?: boolean;
}

export function SubagentTree({ embed }: SubagentTreeProps) {
  const subagents = useSubagentStore((s) => s.subagents);
  const order = useSubagentStore((s) => s.order);

  if (order.length === 0) return null;

  // Only show if at least one subagent is not finished, or recently finished
  const activeSubagents = order.map(id => subagents.get(id)).filter(Boolean) as SubagentState[];
  if (activeSubagents.length === 0) return null;

  if (embed) {
    return (
      <>
        {activeSubagents.map((sa) => (
          <SubagentNode key={sa.subagentId} sa={sa} />
        ))}
      </>
    );
  }

  return (
    <div className="border-t border-border-default/30 bg-surface-elevated/30 py-2 px-2 shrink-0">
      <div className="text-xs font-mono text-content-secondary font-semibold uppercase tracking-wide px-2 pb-1">
        Subagents
      </div>
      {activeSubagents.map((sa) => (
        <SubagentNode key={sa.subagentId} sa={sa} />
      ))}
    </div>
  );
}
