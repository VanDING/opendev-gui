import { useStatusStore } from '../../stores/status';
import { useChatStore } from '../../stores/chat';

function formatTokens(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}k`;
  return String(n);
}

function contextColor(pct: number): string {
  if (pct >= 90) return 'text-intent-danger-muted';
  if (pct >= 70) return 'text-intent-warning-muted';
  return 'text-content-secondary';
}

export function StatusBar() {
  const data = useStatusStore((s) => s.data);
  const status = useChatStore((s) => s.status);
  const currentSessionId = useChatStore((s) => s.currentSessionId);
  const runningSessions = useChatStore((s) => s.runningSessions);

  if (!currentSessionId) return null;

  const runningCount = runningSessions.size;

  const model = data.model || status?.model || '—';
  const branch = data.gitBranch || status?.git_branch || null;
  const autonomy = data.autonomyLevel || status?.autonomy_level || 'Semi-Auto';
  const totalTokens = data.inputTokens + data.outputTokens;
  const cost = data.sessionCostUsd || status?.session_cost || 0;
  const contextPct = data.contextUsagePct || status?.context_usage_pct || 0;

  return (
    <div className="flex items-center gap-4 px-4 py-1.5 bg-surface-elevated border-t border-border-default/30 text-xs font-mono text-content-tertiary select-none shrink-0">
      {/* Model */}
      <span className="text-content-secondary font-medium truncate max-w-[180px]" title={model}>
        {model}
      </span>

      {/* Autonomy */}
      <span className={`px-1.5 py-0.5 rounded text-[10px] font-semibold uppercase tracking-wide ${
        autonomy === 'Auto' ? 'bg-intent-success-muted text-intent-success-fg' :
        autonomy === 'Semi-Auto' ? 'bg-intent-warning-muted text-intent-warning-fg' :
        'bg-surface-3 text-content-tertiary'
      }`}>
        {autonomy}
      </span>

      {/* Git branch */}
      {branch && (
        <span className="text-content-secondary truncate max-w-[120px]" title={branch}>
          <span className="text-content-tertiary mr-1">⎇</span>{branch}
        </span>
      )}

      <div className="flex-1" />

      {/* Tokens */}
      <span className="text-content-secondary">
        {formatTokens(totalTokens)}/{formatTokens(data.maxTokens)}
      </span>

      {/* Context usage */}
      <span className={contextColor(contextPct)}>
        {contextPct.toFixed(0)}%
      </span>

      {/* MCP */}
      {data.mcpTotal > 0 && (
        <span className={data.mcpConnected === data.mcpTotal ? 'text-intent-success-muted' : 'text-intent-warning-muted'}>
          MCP {data.mcpConnected}/{data.mcpTotal}
        </span>
      )}

      {/* Cost */}
      {cost > 0 && (
        <span className="text-content-secondary">${cost.toFixed(2)}</span>
      )}

      {/* Running sessions (background tasks) */}
      {runningCount > 0 && (
        <span className="text-accent-primary-muted flex items-center gap-1">
          <span className="inline-block w-2 h-2 bg-accent-primary-muted rounded-full animate-pulse" />
          {runningCount} running
        </span>
      )}

      {/* File changes */}
      {data.fileChanges && data.fileChanges.files > 0 && (
        <span className="text-content-secondary">
          {data.fileChanges.files} files
          <span className="text-intent-success-muted ml-1">+{data.fileChanges.additions}</span>
          <span className="text-intent-danger-muted ml-1">-{data.fileChanges.deletions}</span>
        </span>
      )}
    </div>
  );
}
