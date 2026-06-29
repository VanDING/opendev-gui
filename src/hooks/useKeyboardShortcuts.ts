import { useEffect, useCallback } from 'react';

/**
 * Pre-defined keyboard shortcuts for OpenDev.
 * Supported shortcuts:
 * - Ctrl+K: Open command palette
 * - Ctrl+L: Clear chat
 * - Ctrl+Shift+C: Copy last response
 * - Ctrl+Shift+E: Export session
 * - Ctrl+.: Interrupt current task
 * - Ctrl+B: Toggle sidebar
 * - Ctrl+R: Open session status
 * - Ctrl+Shift+M: Cycle mode (Normal/Plan)
 */
export interface ShortcutDef {
  key: string;
  ctrl?: boolean;
  shift?: boolean;
  meta?: boolean;
  alt?: boolean;
  description: string;
  handler: () => void;
  /** Whether to prevent default browser behavior */
  preventDefault?: boolean;
  /** Enable/disable this shortcut */
  enabled?: boolean;
}

export type ShortcutMap = Record<string, ShortcutDef>;

const DEFAULT_SHORTCUTS: ShortcutDef[] = [
  { key: 'k', ctrl: true, description: 'Open command palette', handler: () => {}, preventDefault: true },
  { key: 'l', ctrl: true, description: 'Clear chat', handler: () => {}, preventDefault: true },
  { key: 'c', ctrl: true, shift: true, description: 'Copy last response', handler: () => {}, preventDefault: true },
  { key: 'e', ctrl: true, shift: true, description: 'Export session', handler: () => {}, preventDefault: true },
  { key: '.', ctrl: true, description: 'Interrupt current task', handler: () => {}, preventDefault: true },
  { key: 'b', ctrl: true, description: 'Toggle sidebar', handler: () => {}, preventDefault: true },
  { key: 'r', ctrl: true, description: 'Open session status', handler: () => {}, preventDefault: true },
  { key: 'm', ctrl: true, shift: true, description: 'Cycle mode', handler: () => {}, preventDefault: true },
];

/**
 * Hook to register and manage global keyboard shortcuts.
 *
 * @param shortcuts - Array of shortcut definitions. Merge with defaults.
 * @param deps - Dependency array for re-registration.
 */
export function useKeyboardShortcuts(
  shortcuts: ShortcutDef[],
  deps: React.DependencyList = [],
) {
  const merged = [...DEFAULT_SHORTCUTS, ...shortcuts];

  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    for (const sc of merged) {
      if (sc.enabled === false) continue;
      const ctrlOrMeta = sc.ctrl || sc.meta;
      const matchesCtrl = ctrlOrMeta ? (e.ctrlKey || e.metaKey) : !e.ctrlKey && !e.metaKey;
      const matchesShift = sc.shift ? e.shiftKey : !e.shiftKey;
      const matchesAlt = sc.alt ? e.altKey : !e.altKey;
      const matchesKey = e.key.toLowerCase() === sc.key.toLowerCase();

      if (matchesCtrl && matchesShift && matchesAlt && matchesKey) {
        if (sc.preventDefault) e.preventDefault();
        sc.handler();
        return;
      }
    }
  }, [merged]);

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [handleKeyDown, ...deps]);
}

/**
 * Build a human-readable label for a shortcut.
 */
export function shortcutLabel(sc: ShortcutDef): string {
  const parts: string[] = [];
  if (sc.ctrl || sc.meta) parts.push('Ctrl');
  if (sc.shift) parts.push('Shift');
  if (sc.alt) parts.push('Alt');
  parts.push(sc.key.toUpperCase());
  return parts.join('+');
}
