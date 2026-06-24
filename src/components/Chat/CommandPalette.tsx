import { useEffect, useRef, useState, useCallback, useMemo } from 'react';
import { useChatStore } from '../../stores/chat';
import { Modal } from '../ui/Modal';

interface Command {
  id: string;
  label: string;
  description: string;
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

  // ⚡ Bolt Performance Optimization:
  // Wrapped array filtering in useMemo and precomputed a case-insensitive RegExp
  // This avoids redundant string allocations and prevents an O(N) repetitive
  // toLowerCase operations during every render cycle (especially on keystrokes in the search box).
  const filtered = useMemo(() => {
    const commands: Command[] = [
      { id: 'clear', label: '/clear', description: 'Clear chat history', action: () => { clearChat(); onClose(); } },
      { id: 'mode', label: '/mode', description: 'Toggle between Normal and Plan mode', action: () => { toggleMode(); onClose(); } },
      { id: 'status', label: '/status', description: 'Show session status dialog', action: () => { onOpenStatus(); onClose(); } },
      { id: 'interrupt', label: '/interrupt', description: 'Interrupt the current task', action: () => { sendInterrupt(); onClose(); } },
      { id: 'autonomy', label: 'Cycle Autonomy', description: 'Cycle: Manual → Semi-Auto → Auto', action: () => { cycleAutonomy(); onClose(); } },
      { id: 'thinking', label: 'Cycle Thinking', description: 'Cycle: Off → Low → Medium → High', action: () => { cycleThinkingLevel(); onClose(); } },
      { id: 'sidebar', label: 'Toggle Sidebar', description: 'Show/hide the sessions sidebar', action: () => { toggleSidebar(); onClose(); } },
    ];

    if (!query) return commands;
    const queryRegex = new RegExp(query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'i');
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

  return (
    <Modal isOpen={isOpen} onClose={onClose} size="sm">
      
        <input
          ref={inputRef}
          type="text"
          value={query}
          onChange={e => setQuery(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Type a command..."
          className="w-full px-4 py-3 text-sm bg-transparent border-b border-border-default/20 text-content-primary placeholder-content-tertiary outline-none"
        />
        <div className="max-h-64 overflow-y-auto py-1">
          {filtered.length === 0 ? (
            <div className="px-4 py-3 text-sm text-content-tertiary">No matching commands</div>
          ) : (
            filtered.map((cmd, i) => (
              <button
                key={cmd.id}
                onClick={cmd.action}
                className={`w-full text-left px-4 py-2.5 flex items-center gap-3 text-sm transition-colors ${
                  i === selectedIndex ? 'bg-accent-primary/10 text-content-primary' : 'text-content-secondary hover:bg-surface-2'
                }`}
              >
                <span className="font-mono font-medium text-accent-primary min-w-[120px]">{cmd.label}</span>
                <span className="text-content-secondary">{cmd.description}</span>
              </button>
            ))
          )}
        </div>
        <div className="px-4 py-2 border-t border-border-default/20 text-xs text-content-tertiary flex gap-3">
          <span><kbd className="px-1 py-0.5 bg-surface-2 rounded text-content-secondary">↑↓</kbd> navigate</span>
          <span><kbd className="px-1 py-0.5 bg-surface-2 rounded text-content-secondary">Enter</kbd> select</span>
          <span><kbd className="px-1 py-0.5 bg-surface-2 rounded text-content-secondary">Esc</kbd> close</span>
        </div>
    </Modal>
  );
}
