import { useEffect, useState, useRef } from 'react';
import { PanelLeft, Command, Palette } from 'lucide-react';
import { useChatStore } from '../../stores/chat';
import { configRepository } from '../../repositories';
import { useTheme, ALL_THEMES } from '../../contexts/ThemeContext';
import type { Theme } from '../../contexts/ThemeContext';
import { Button } from '../ui/Button';

const MODE_STYLES = {
  normal: 'bg-surface-muted/40 text-content-secondary border-border-emphasis hover:bg-surface-muted/60',
  plan: 'bg-accent-magenta-muted text-accent-magenta border-accent-magenta/50 hover:bg-accent-magenta-muted/80',
} as const;

const AUTONOMY_STYLES = {
  'Manual': 'bg-surface-muted/40 text-content-secondary border-border-emphasis hover:bg-surface-muted/60',
  'Semi-Auto': 'bg-accent-magenta-muted text-accent-magenta border-accent-magenta/50 hover:bg-accent-magenta-muted/80',
  'Auto': 'bg-intent-success/10 text-intent-success border-intent-success/20 hover:bg-intent-success/15',
} as const;

const THINKING_STYLES: Record<string, string> = {
  'Off':           'bg-surface-2 text-content-tertiary border-border-emphasis hover:bg-surface-3',
  'Low':           'bg-intent-info-muted text-intent-info border-intent-info-muted hover:bg-cyan-500/15',
  'Medium':        'bg-intent-success/10 text-intent-success border-intent-success/20 hover:bg-intent-success/15',
  'High':          'bg-intent-warning-muted text-intent-warning border-intent-warning-muted hover:bg-intent-warning/15',
} as const;

function formatCost(cost: number): string {
  return cost < 0.01 ? `$${cost.toFixed(4)}` : `$${cost.toFixed(2)}`;
}

function getContextColor(pct: number): string {
  const remaining = 100 - pct;
  if (remaining < 25) return 'bg-intent-danger-muted/10 text-intent-danger border-intent-danger-muted';
  if (remaining < 50) return 'bg-intent-warning-muted text-intent-warning border-intent-warning-muted';
  return 'bg-intent-success-muted/10 text-intent-success border-emerald-500/20';
}

interface TopBarProps {
  onOpenCommandPalette?: () => void;
  detailCollapsed?: boolean;
  onToggleDetail?: () => void;
}

const THEME_LABELS: Record<Theme, string> = {
  'cyberpunk':     'Cyberpunk',
  'dark-default':  'Dark',
  'light-default': 'Light',
  'warm':          'Warm',
  'geek':          'Geek',
  'sumi-e':        'Sumi-e',
  'synthwave':     'Synthwave',
  'techno':        'Techno',
  'brutalism':     'Brutalism',
};

export function TopBar({ onOpenCommandPalette, detailCollapsed, onToggleDetail }: TopBarProps) {
  const { theme, setTheme } = useTheme();
  const [themeOpen, setThemeOpen] = useState(false);
  const themeRef = useRef<HTMLDivElement>(null);
  const status = useChatStore(state => state.status);
  const isConnected = useChatStore(state => state.isConnected);
  const thinkingLevel = useChatStore(state => state.thinkingLevel);
  const sidebarCollapsed = useChatStore(state => state.sidebarCollapsed);
  const toggleMode = useChatStore(state => state.toggleMode);
  const cycleAutonomy = useChatStore(state => state.cycleAutonomy);
  const cycleThinkingLevel = useChatStore(state => state.cycleThinkingLevel);
  const toggleSidebar = useChatStore(state => state.toggleSidebar);

  // Load initial config on mount
  useEffect(() => {
    const loadStatus = async () => {
      try {
        const configData = await configRepository.getConfig();
        useChatStore.setState({
          thinkingLevel: configData.thinking_level || 'Medium',
        });
        useChatStore.getState().setStatus({
          mode: configData.mode || 'normal',
          autonomy_level: configData.autonomy_level || 'Manual',
          thinking_level: configData.thinking_level || 'Medium',
          model: configData.model,
          model_provider: configData.model_provider,
          working_dir: configData.working_dir || '',
          git_branch: configData.git_branch,
        });
      } catch (_) { /* ignore */ }
    };
    loadStatus();
  }, []);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.shiftKey && e.key === 'T') {
        e.preventDefault();
        cycleThinkingLevel();
      }
      if (e.ctrlKey && e.shiftKey && e.key === 'A') {
        e.preventDefault();
        cycleAutonomy();
      }
      if ((e.ctrlKey || e.metaKey) && e.key === 'b') {
        e.preventDefault();
        toggleSidebar();
      }
      if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
        e.preventDefault();
        onOpenCommandPalette?.();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [cycleThinkingLevel, cycleAutonomy, toggleSidebar, onOpenCommandPalette]);

  // Click outside to close theme dropdown
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (themeRef.current && !themeRef.current.contains(e.target as Node)) {
        setThemeOpen(false);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const getProjectName = (path: string) => {
    if (!path) return '';
    const parts = path.replace(/\/$/, '').split('/');
    return parts[parts.length - 1] || path;
  };

  const pillBase = 'inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full border text-xs font-medium cursor-pointer transition-colors select-none hover-scale-pill';

  return (
    <header className="h-12 flex-shrink-0 sticky top-0 z-40 flex items-center gap-3 px-4 bg-surface-primary border-b border-border-default">
      {/* ── Left: Sidebar toggle + Brand ── */}
      <div className="flex items-center gap-3 flex-shrink-0">
        <button
          onClick={toggleSidebar}
          className="w-8 h-8 rounded-md flex items-center justify-center hover:bg-surface-3/50 transition-colors hover-lift"
          title={sidebarCollapsed ? 'Expand sidebar (Ctrl/Cmd+B)' : 'Collapse sidebar (Ctrl/Cmd+B)'}
        >
          <PanelLeft className="w-5 h-5 text-content-secondary" />
        </button>

        {/* Logo */}
        <img src="/icon_blue.png" alt="OpenDev" className="w-7 h-7 rounded-lg shadow-sm flex-shrink-0" />

        <div className="flex items-baseline gap-1.5">
          <span className="font-display text-sm font-black tracking-[0.15em] text-content-primary">OPENDEV</span>
          <span className="text-[10px] uppercase tracking-wider text-content-tertiary hidden sm:inline">AI Assistant</span>
        </div>
      </div>

      {/* ── Spacer ── */}
      <div className="flex-1" />

      {/* ── Center-Right: Status Pills ── */}
      {status && (
        <div className="flex items-center gap-2 flex-shrink-0">
          {/* Cost pill — only shown when agent has run */}
          {status.session_cost != null && status.session_cost > 0 && (
            <span
              className={`${pillBase} cursor-default bg-surface-2 text-content-secondary border-border-default/30`}
              title={`Session cost: ${formatCost(status.session_cost)}`}
            >
              {formatCost(status.session_cost)}
            </span>
          )}

          {/* Context usage pill — only shown when available */}
          {status.context_usage_pct != null && (
            <span
              className={`${pillBase} cursor-default ${getContextColor(status.context_usage_pct)}`}
              title={`Context window: ${Math.round(status.context_usage_pct)}% used, ${Math.round(100 - status.context_usage_pct)}% remaining`}
            >
              Ctx: {Math.round(status.context_usage_pct)}%
            </span>
          )}

          {/* Mode pill */}
          <button
            onClick={toggleMode}
            className={`${pillBase} ${MODE_STYLES[status.mode]}`}
            title="Normal: full tool access · Plan: read-only exploration. Click to toggle"
          >
            {status.mode === 'plan' && (
              <svg className="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
              </svg>
            )}
            Mode: {status.mode === 'normal' ? 'Normal' : 'Plan'}
          </button>

          {/* Autonomy pill */}
          <button
            onClick={cycleAutonomy}
            className={`${pillBase} ${AUTONOMY_STYLES[status.autonomy_level]}`}
            title="Manual: approve each tool · Semi-Auto: auto-approve safe tools · Auto: approve all. Click to cycle (Ctrl+Shift+A)"
          >
            Approval: {status.autonomy_level}
          </button>

          {/* Thinking pill */}
          <button
            onClick={cycleThinkingLevel}
            className={`${pillBase} ${THINKING_STYLES[thinkingLevel] || THINKING_STYLES['Medium']}`}
            title="Controls how much the AI reasons before responding. Click to cycle (Ctrl+Shift+T)"
          >
            Think: {thinkingLevel}
          </button>

          {/* Command palette button */}
          <Button
            variant="ghost"
            onClick={onOpenCommandPalette}
            title="Command palette (Ctrl/Cmd+K)"
          >
            <Command className="w-3 h-3" />
          </Button>

          {/* Details button (shown when DetailPanel collapsed) */}
          {detailCollapsed && onToggleDetail && (
            <Button
              variant="ghost"
              onClick={onToggleDetail}
              title="Show details panel"
            >
              DETAILS
            </Button>
          )}

          {/* Connection pill */}
          <span className={`${pillBase} cursor-default ${
            isConnected
              ? 'bg-intent-success-muted text-intent-success border-intent-success-muted'
              : 'bg-surface-2 text-content-tertiary border-border-default'
          }`}>
            <span className={`w-2 h-2 rounded-full ${isConnected ? 'bg-intent-success-muted' : 'bg-surface-muted'}`} />
            {isConnected ? 'Connected' : 'Offline'}
          </span>
        </div>
      )}

      {/* ── Theme selector ── */}
      <div className="relative" ref={themeRef}>
        <Button
          variant="ghost"
          onClick={() => setThemeOpen(!themeOpen)}
          title="Switch theme"
        >
          <Palette className="w-4 h-4" />
        </Button>

        {themeOpen && (
          <div className="absolute right-0 top-full mt-1 w-40 bg-surface-primary border border-border-default rounded-xl shadow-popover z-50 py-1 animate-fade-in">
            {ALL_THEMES.map(t => (
              <button
                key={t}
                onClick={() => { setTheme(t); setThemeOpen(false); }}
                className={`w-full px-3 py-2 text-left text-xs transition-colors ${
                  t === theme
                    ? 'bg-accent-primary-muted text-accent-primary font-semibold'
                    : 'text-content-secondary hover:bg-surface-2 hover:text-content-primary'
                }`}
              >
                {THEME_LABELS[t]}
              </button>
            ))}
          </div>
        )}
      </div>

      {/* ── Far-Right: Project / Model ── */}
      {status && (
        <div className="flex items-center gap-2 text-xs text-content-tertiary flex-shrink-0 ml-1 hidden md:flex">
          {status.working_dir && (
            <span className="truncate max-w-[160px]" title={status.working_dir}>
              {getProjectName(status.working_dir)}
              {status.git_branch && (
                <span className="text-content-tertiary">
                  <span className="text-content-tertiary"> / </span>{status.git_branch}
                </span>
              )}
            </span>
          )}

          {status.working_dir && status.model && (
            <span className="text-content-tertiary">|</span>
          )}

          {status.model && (
            <span className="font-mono text-content-tertiary truncate max-w-[140px]" title={`${status.model_provider}/${status.model}`}>
              {status.model}
            </span>
          )}
        </div>
      )}
    </header>
  );
}
