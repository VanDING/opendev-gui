import { useState, useEffect, useMemo } from 'react';
import { ChevronRight, Folder, Search } from 'lucide-react';
import { apiClient } from '../../api/client';
import { useChatStore } from '../../stores/chat';
import { Modal } from '../ui/Modal';
import { Button } from '../ui/Button';
import { Input } from '../ui/Input';

interface NewSessionModalProps {
  isOpen: boolean;
  onClose: () => void;
}

interface DirEntry {
  name: string;
  path: string;
}

const getBreadcrumbs = (absPath: string) => {
  const parts = absPath.split('/').filter(Boolean);
  return parts.map((part, i) => ({
    label: part,
    path: '/' + parts.slice(0, i + 1).join('/'),
  }));
};

export function NewSessionModal({ isOpen, onClose }: NewSessionModalProps) {
  const [currentPath, setCurrentPath] = useState('');
  const [parentPath, setParentPath] = useState<string | null>(null);
  const [directories, setDirectories] = useState<DirEntry[]>([]);
  const [manualPath, setManualPath] = useState('');
  const [showHidden, setShowHidden] = useState(false);
  const [isLoadingDirs, setIsLoadingDirs] = useState(false);
  const [browseError, setBrowseError] = useState<string | null>(null);
  const [filterText, setFilterText] = useState('');
  const [isCreating, setIsCreating] = useState(false);
  const [createError, setCreateError] = useState<string | null>(null);
  const loadSession = useChatStore(state => state.loadSession);
  const bumpSessionList = useChatStore(state => state.bumpSessionList);

  const fetchDirectory = async (path: string, hidden?: boolean) => {
    setIsLoadingDirs(true);
    setBrowseError(null);
    try {
      const result = await apiClient.browseDirectory(path, hidden ?? showHidden);
      setCurrentPath(result.current_path);
      setParentPath(result.parent_path);
      setDirectories(result.directories);
      setManualPath(result.current_path);
      setFilterText('');
      if (result.error) {
        setBrowseError(result.error);
      }
    } catch (err) {
      setBrowseError('Failed to browse directory');
    } finally {
      setIsLoadingDirs(false);
    }
  };

  // Load home directory on open
  useEffect(() => {
    if (isOpen) {
      fetchDirectory('');
    }
  }, [isOpen]);

  // Refetch when showHidden toggles
  useEffect(() => {
    if (isOpen && currentPath) {
      fetchDirectory(currentPath, showHidden);
    }
  }, [showHidden]);

  const handleManualGo = () => {
    if (manualPath.trim()) {
      fetchDirectory(manualPath.trim());
    }
  };

  const handleSelect = async () => {
    if (!currentPath) return;
    setIsCreating(true);
    setCreateError(null);
    try {
      const result = await apiClient.createSession(currentPath);
      bumpSessionList();
      const sessionId = result.id;
      if (sessionId) {
        await loadSession(sessionId);
      }
      onClose();
    } catch (err) {
      console.error('Failed to create session:', err);
      setCreateError(
        err instanceof Error ? err.message : 'Failed to create session. Please try again.'
      );
    } finally {
      setIsCreating(false);
    }
  };

  // ⚡ Bolt: Hoist array filtering out of the JSX IIFE and memoize it.
  // This prevents O(N) repetitive string operations (toLowerCase) on every keystroke
  // when the user types in the filter input.
  const filteredDirs = useMemo(() => {
    if (!filterText) return directories;
    const queryRegex = new RegExp(filterText.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'i');
    return directories.filter(d => queryRegex.test(d.name));
  }, [directories, filterText]);

  const breadcrumbs = currentPath ? getBreadcrumbs(currentPath) : [];

  return (
    <Modal isOpen={isOpen} onClose={onClose} title="Select Workspace" size="md">
      <div className="flex flex-col max-h-[75vh]">

        {/* Path input bar */}
        <div className="px-5 pt-4 pb-2">
          <div className="flex gap-2">
            <Input
              type="text"
              value={manualPath}
              onChange={(e) => setManualPath(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') handleManualGo();
              }}
              placeholder="/path/to/directory"
              className="font-mono"
              fullWidth
            />
            <Button variant="secondary" onClick={handleManualGo} disabled={!manualPath.trim()}>
              Go
            </Button>
          </div>
        </div>

        {/* Breadcrumb bar */}
        <div className="px-5 py-2 flex items-center gap-1 flex-wrap">
          <button
            onClick={() => fetchDirectory('/')}
            className="text-xs font-medium text-intent-warning hover:text-intent-warning hover:underline"
          >
            /
          </button>
          {breadcrumbs.map((crumb, i) => (
            <span key={crumb.path} className="flex items-center gap-1">
              <ChevronRight className="w-3 h-3 text-content-tertiary" />
              <button
                onClick={() => fetchDirectory(crumb.path)}
                className={`text-xs font-medium ${
                  i === breadcrumbs.length - 1
                    ? 'text-content-primary'
                    : 'text-intent-warning hover:text-intent-warning hover:underline'
                }`}
              >
                {crumb.label}
              </button>
            </span>
          ))}
          <label className="ml-auto flex items-center gap-1.5 text-xs text-content-tertiary cursor-pointer select-none">
            <input
              type="checkbox"
              checked={showHidden}
              onChange={(e) => setShowHidden(e.target.checked)}
              className="rounded border-border-emphasis text-intent-warning focus:ring-intent-warning"
            />
            Show hidden
          </label>
        </div>

        {/* Directory listing */}
        <div className="flex-1 overflow-y-auto border-t border-border-subtle min-h-0">
          {isLoadingDirs ? (
            <div className="flex items-center justify-center py-12">
              <div className="w-6 h-6 border-2 border-border-default border-t-intent-warning rounded-full animate-spin" />
            </div>
          ) : browseError ? (
            <div className="px-5 py-8 text-center">
              <p className="text-sm text-intent-danger">{browseError}</p>
            </div>
          ) : (
            <div className="py-1">
              {/* Filter input */}
              {directories.length > 0 && (
                <div className="px-5 py-2">
                  <div className="relative">
                    <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-content-tertiary" />
                    <Input
                      type="text"
                      value={filterText}
                      onChange={(e) => setFilterText(e.target.value)}
                      placeholder="Filter folders..."
                      className="pl-8"
                      fullWidth
                    />
                  </div>
                </div>
              )}

              {/* Parent directory row */}
              {parentPath && (
                <button
                  onClick={() => fetchDirectory(parentPath)}
                  className="w-full px-5 py-2.5 flex items-center gap-3 hover:bg-intent-warning-muted text-left"
                >
                  <svg className="w-4 h-4 text-content-tertiary flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 10l7-7m0 0l7 7m-7-7v18" />
                  </svg>
                  <span className="text-sm text-content-secondary">..</span>
                </button>
              )}

              {/* Directory rows */}
              {(() => {
                if (directories.length === 0 && !parentPath) {
                  return (
                    <div className="px-5 py-8 text-center">
                      <p className="text-sm text-content-tertiary">No subdirectories</p>
                    </div>
                  );
                }
                if (directories.length === 0 && parentPath) {
                  return (
                    <div className="px-5 py-6 text-center">
                      <p className="text-sm text-content-tertiary">No subdirectories</p>
                    </div>
                  );
                }

                if (filteredDirs.length === 0) {
                  return (
                    <div className="px-5 py-6 text-center">
                      <p className="text-sm text-content-tertiary">No matching folders</p>
                    </div>
                  );
                }
                return filteredDirs.map((dir) => (
                  <button
                    key={dir.path}
                    onClick={() => fetchDirectory(dir.path)}
                    className="w-full px-5 py-2.5 flex items-center gap-3 hover:bg-intent-warning-muted text-left"
                  >
                    <Folder className="w-4 h-4 text-intent-warning flex-shrink-0" />
                    <span className="text-sm text-content-primary truncate">{dir.name}</span>
                  </button>
                ));
              })()}
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="px-5 py-4 border-t border-border-default">
          {createError && (
            <p className="text-sm text-intent-danger mb-3">{createError}</p>
          )}
          <div className="flex items-center gap-3">
          <div className="flex-1 min-w-0">
            <p className="text-xs font-mono text-content-tertiary truncate" title={currentPath}>
              {currentPath}
            </p>
          </div>
          <Button variant="secondary" onClick={onClose}>
            Cancel
          </Button>
          <Button variant="primary" onClick={handleSelect} disabled={!currentPath || isCreating} loading={isCreating}>
            {isCreating ? 'Creating...' : 'Select This Directory'}
          </Button>
          </div>
        </div>
      </div>
    </Modal>
  );
}
