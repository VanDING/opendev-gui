import React, { useEffect, useState, useCallback } from 'react';
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

function formatCost(cost?: number): string {
  if (cost == null) return '';
  if (cost < 0.01) return `$${cost.toFixed(4)}`;
  return `$${cost.toFixed(2)}`;
}

// ⚡ Bolt Performance Optimization:
// Isolate the high-frequency state update (1000ms interval) into a dedicated leaf component.
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

/** Status color class mapping. */
function statusColor(sa: SubagentState): string {
  if (!sa.finished) return 'border-accent-primary-muted'; // running = blue
  if (sa.success) return 'border-intent-success-muted'; // success = green
  return 'border-intent-danger-muted'; // error = red
}

function statusBadge(sa: SubagentState): { text: string; color: string } {
  if (!sa.finished) return { text: 'Running', color: 'text-accent-primary-muted' };
  if (sa.success) return { text: 'Success', color: 'text-intent-success-muted' };
  return { text: 'Failed', color: 'text-intent-danger-muted' };
}

function ActiveToolRow({ tool, isLast }: { tool: ActiveToolCall; isLast: boolean }) {
  const verb = formatToolVerb(tool.toolName);
  const arg = formatToolArg(tool.toolName, tool.args);
  const connector = isLast ? '└─' : '├─';

  // Tool output preview (first 200 chars)
  const [showOutput, setShowOutput] = useState(false);
  const outputPreview = tool.result
    ? (typeof tool.result === 'string' ? tool.result : JSON.stringify(tool.result)).slice(0, 200)
    : '';

  return (
    <div className="flex items-center gap-1.5 text-sm font-mono text-content-secondary leading-6 pl-8">
      <span className="text-content-tertiary">{connector}</span>
      <Spinner className="text-accent-primary-muted" />
      <span className="text-content-secondary">{verb}</span>
      {arg && <span className="text-content-tertiary truncate max-w-[300px]">{arg}</span>}
      <span className="text-content-tertiary ml-auto shrink-0">
        (<ElapsedTimeDisplay startedAt={tool.startedAt} />)
      </span>
      {outputPreview && (
        <button
          onClick={() => setShowOutput(!showOutput)}
          className="text-[10px] text-accent-secondary hover:underline shrink-0 ml-1"
        >
          {showOutput ? 'hide' : 'preview'}
        </button>
      )}
      {showOutput && outputPreview && (
        <div className="absolute left-0 right-0 top-full z-10 mt-1 bg-surface-elevated border border-border-default rounded p-2 text-xs text-content-secondary max-h-24 overflow-y-auto shadow-lg">
          {outputPreview}
        </div>
      )}
    </div>
  );
}

function CompletedToolRow({ toolName, success, isLast, toolOutput }: { toolName: string; success: boolean; isLast: boolean; toolOutput?: string }) {
  const connector = isLast ? '└─' : '├─';
  const icon = success ? '✓' : '✗';
  const color = success ? 'text-intent-success-muted' : 'text-intent-danger-muted';
  const [showOutput, setShowOutput] = useState(false);
  const outputPreview = toolOutput ? toolOutput.slice(0, 200) : '';

  return (
    <div className="relative flex items-center gap-1.5 text-sm font-mono text-content-tertiary leading-6 pl-8">
      <span className="text-content-tertiary">{connector}</span>
      <span className={color}>{icon}</span>
      <span>{formatToolVerb(toolName)}</span>
      {outputPreview && (
        <button
          onClick={() => setShowOutput(!showOutput)}
          className="text-[10px] text-accent-secondary hover:underline ml-1"
        >
          {showOutput ? 'hide' : 'preview'}
        </button>
      )}
      {showOutput && outputPreview && (
        <div className="absolute left-0 right-0 top-full z-10 mt-1 bg-surface-elevated border border-border-default rounded p-2 text-xs text-content-secondary max-h-24 overflow-y-auto shadow-lg">
          {outputPreview}
        </div>
      )}
    </div>
  );
}

function SubagentNode({ sa, onStop }: { sa: SubagentState; onStop?: (id: string) => void }) {
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

  // Status badge
  const badge = statusBadge(sa);

  // Stats string with cost
  const tokenStr = sa.tokenCount > 0 ? ` · ${formatTokens(sa.tokenCount)} tokens` : '';
  const costStr = sa.cost != null ? ` · ${formatCost(sa.cost)}` : '';
  const statsPrefix = `(${sa.toolCallCount} tool uses${tokenStr}${costStr} · `;
  const statsSuffix = `)`;

  // Expanded/collapsed state
  const [expanded, setExpanded] = useState(false);

  // Display name
  const displayName = sa.name.split(/[-_]/).map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ');
  const taskPreview = sa.description.length > 37 ? sa.description.slice(0, 34) + '...' : sa.description;

  // Active tools
  const activeToolEntries = Array.from(sa.activeTools.values());
  // Show last 5 completed
  const completedVisible = sa.completedTools.slice(-5);
  const hiddenCount = Math.max(0, sa.toolCallCount - activeToolEntries.length - completedVisible.length);

  const hasPulsingBorder = !sa.finished;

  return (
    <div className={`mb-1 rounded-lg ${hasPulsingBorder ? 'animate-pulse-border' : ''}`} style={{ borderLeft: `3px solid ${statusColor(sa)}` }}>
      {/* Header line */}
      <div
        className="flex items-center gap-1.5 text-sm font-mono leading-6 cursor-pointer select-none pl-2"
        onClick={() => setExpanded(e => !e)}
      >
        <span className="text-content-tertiary">├─</span>
        {statusEl}
        <span className="text-accent-primary-muted font-semibold">{displayName}</span>
        <span className="text-xs font-semibold uppercase tracking-wider">{badge.text}</span>
        <span className="text-content-tertiary truncate">: {taskPreview}</span>
        <span className="text-content-tertiary shrink-0 text-xs ml-1">{statsPrefix}<ElapsedTimeDisplay startedAt={sa.startedAt} finished={sa.finished} />{statsSuffix}</span>
        <span className="text-content-tertiary shrink-0 ml-1 w-4 text-center text-[10px]">{expanded ? '▲' : '▼'}</span>
      </div>

      {expanded && (
        <div className="relative">
          {/* Active tool calls */}
          {activeToolEntries.map((tool, i) => (
            <ActiveToolRow
              key={tool.toolId}
              tool={tool}
              isLast={i === activeToolEntries.length - 1 && completedVisible.length === 0}
            />
          ))}

          {/* Last 5 completed tools */}
          {completedVisible.map((tool, i) => (
            <CompletedToolRow
              key={`completed-${i}`}
              toolName={tool.toolName}
              success={tool.success}
              isLast={i === completedVisible.length - 1}
              toolOutput={tool.output}
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

          {/* Cost display */}
          {sa.cost != null && (
            <div className="text-xs font-mono text-content-tertiary pl-10 leading-5">
              Cost: {formatCost(sa.cost)}
            </div>
          )}

          {/* Stop button for running agents */}
          {!sa.finished && onStop && (
            <button
              onClick={(e) => { e.stopPropagation(); onStop(sa.subagentId); }}
              className="ml-10 mt-1 px-2 py-0.5 text-xs font-medium text-intent-danger border border-intent-danger/30 rounded hover:bg-intent-danger-muted/10 transition-colors"
            >
              Stop Subagent
            </button>
          )}

          {/* Completion summary (persistent after finish) */}
          {sa.finished && (
            <div className="text-xs font-mono text-content-tertiary pl-10 leading-6">
              Done ({sa.toolCallCount} tool uses{tokenStr}{costStr} · <ElapsedTimeDisplay startedAt={sa.startedAt} finished={sa.finished} />)
            </div>
          )}
        </div>
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
  const stopSubagent = useSubagentStore((s) => s.stopSubagent);

  const handleStop = useCallback((id: string) => {
    stopSubagent(id);
  }, [stopSubagent]);

  if (order.length === 0) return null;

  // Only show if at least one subagent is not finished, or recently finished
  const activeSubagents = order.map(id => subagents.get(id)).filter(Boolean) as SubagentState[];
  if (activeSubagents.length === 0) return null;

  if (embed) {
    return (
      <>
        {activeSubagents.map((sa) => (
          <SubagentNode key={sa.subagentId} sa={sa} onStop={handleStop} />
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
        <SubagentNode key={sa.subagentId} sa={sa} onStop={handleStop} />
      ))}
    </div>
  );
}
