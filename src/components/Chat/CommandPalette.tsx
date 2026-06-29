import { useEffect, useRef, useState, useCallback, useMemo } from 'react';
import { useChatStore } from '../../stores/chat';
import { Modal } from '../ui/Modal';

interface Command {
  id: string;
  label: string;
  description: string;
  shortcut?: string;
  action: () => void;
}

interface CommandPaletteProps {
  isOpen: boolean;
  onClose: () => void;
  onOpenStatus: () => void;
}

export function CommandPalette({ isOpen, onClose, onOpenStatus }: CommandPaletteProps) {
  const [query, setQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  const toggleMode = useChatStore(state => state.toggleMode);
  const cycleAutonomy = useChatStore(state => state.cycleAutonomy);
  const cycleThinkingLevel = useChatStore(state => state.cycleThinkingLevel);
  const clearChat = useChatStore(state => state.clearChat);
  const sendInterrupt = useChatStore(state => state.sendInterrupt);
  const toggleSidebar = useChatStore(state => state.toggleSidebar);

  const filtered = useMemo(() => {
    const commands: Command[] = [
      // ── Session ──
      { id: 'clear', label: '/clear', description: 'Clear chat history', shortcut: 'Ctrl+L', action: () => { clearChat(); onClose(); } },
      { id: 'compact', label: '/compact', description: 'Compact session context to reduce token usage', action: () => { onClose(); } },
      { id: 'status', label: '/status', description: 'Show session status and usage', shortcut: 'Ctrl+R', action: () => { onOpenStatus(); onClose(); } },
      { id: 'cost', label: '/cost', description: 'Show cost breakdown for this session', action: () => { onClose(); } },
      { id: 'diff', label: '/diff', description: 'Show git diff of current changes', action: () => { onClose(); } },
      { id: 'rename', label: '/rename', description: 'Rename the current session', action: () => { onClose(); } },
      { id: 'export', label: '/export', description: 'Export session as markdown', shortcut: 'Ctrl+Shift+E', action: () => { onClose(); } },

      // ── Actions ──
      { id: 'mode', label: '/mode', description: 'Toggle between Normal and Plan mode', action: () => { toggleMode(); onClose(); } },
      { id: 'interrupt', label: '/interrupt', description: 'Interrupt the current task', shortcut: 'Ctrl+.', action: () => { sendInterrupt(); onClose(); } },
      { id: 'review', label: '/review', description: 'Review current changes for issues', action: () => { onClose(); } },
      { id: 'commit', label: '/commit', description: 'Stage and commit changes', action: () => { onClose(); } },
      { id: 'init', label: '/init', description: 'Initialize or re-initialize the project', action: () => { onClose(); } },
      { id: 'doctor', label: '/doctor', description: 'Run diagnostics on the project', action: () => { onClose(); } },

      // ── Settings ──
      { id: 'autonomy', label: 'Cycle Autonomy', description: 'Cycle: Manual → Semi-Auto → Auto', action: () => { cycleAutonomy(); onClose(); } },
      { id: 'thinking', label: 'Cycle Thinking', description: 'Cycle: Off → Low → Medium → High', action: () => { cycleThinkingLevel(); onClose(); } },
      { id: 'sidebar', label: 'Toggle Sidebar', description: 'Show/hide the sessions sidebar', action: () => { toggleSidebar(); onClose(); } },

      // ── Navigation ──
      { id: 'resume', label: '/resume', description: 'Resume a previous session', action: () => { onClose(); } },
      { id: 'feedback', label: '/feedback', description: 'Submit feedback about OpenDev', action: () => { onClose(); } },
      { id: 'share', label: '/share', description: 'Share current session as snapshot', action: () => { onClose(); } },
    ];

    if (!query) return commands;
    const escaped = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    const queryRegex = new RegExp(escaped, 'i');
    return commands.filter(c =>
      queryRegex.test(c.label) ||
      queryRegex.test(c.description)
    );
  }, [query, clearChat, onClose, toggleMode, onOpenStatus, sendInterrupt, cycleAutonomy, cycleThinkingLevel, toggleSidebar]);

  useEffect(() => {
    if (isOpen) {
      setQuery('');
      setSelectedIndex(0);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [isOpen]);

  useEffect(() => {
    setSelectedIndex(0);
  }, [query]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      setSelectedIndex(prev => Math.min(prev + 1, filtered.length - 1));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setSelectedIndex(prev => Math.max(prev - 1, 0));
    } else if (e.key === 'Enter' && filtered.length > 0) {
      e.preventDefault();
      filtered[selectedIndex]?.action();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      onClose();
    }
  }, [filtered, selectedIndex, onClose]);

  if (!isOpen) return null;

  return (
    <Modal onClose={onClose} title="Commands">
      <div style={{ padding: '8px' }}>
        <input
          ref={inputRef}
          type="text"
          value={query}
          onChange={e => setQuery(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Search commands..."
          style={{
            width: '100%',
            padding: '8px 12px',
            borderRadius: '6px',
            border: '1px solid var(--border)',
            background: 'var(--bg-secondary)',
            color: 'var(--text)',
            fontSize: '14px',
            outline: 'none',
            boxSizing: 'border-box',
          }}
        />
        <div style={{ marginTop: '8px', maxHeight: '300px', overflowY: 'auto' }}>
          {filtered.map((cmd, i) => (
            <div
              key={cmd.id}
              onClick={() => cmd.action()}
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                padding: '8px 12px',
                cursor: 'pointer',
                borderRadius: '4px',
                background: i === selectedIndex ? 'var(--bg-active, rgba(255,255,255,0.1))' : 'transparent',
              }}
            >
              <div>
                <span style={{ fontWeight: 500 }}>{cmd.label}</span>
                <span style={{ marginLeft: '8px', fontSize: '12px', opacity: 0.7 }}>{cmd.description}</span>
              </div>
              {cmd.shortcut && (
                <kbd style={{
                  fontSize: '11px',
                  padding: '2px 6px',
                  borderRadius: '3px',
                  border: '1px solid var(--border)',
                  opacity: 0.6,
                }}>
                  {cmd.shortcut}
                </kbd>
              )}
            </div>
          ))}
          {filtered.length === 0 && (
            <div style={{ padding: '12px', opacity: 0.5, textAlign: 'center' }}>No commands found</div>
          )}
        </div>
      </div>
    </Modal>
  );
}
