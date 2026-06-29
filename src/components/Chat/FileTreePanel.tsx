import { useState, useMemo, useCallback } from 'react';

interface FileTreeEntry {
  name: string;
  path: string;
  type: 'file' | 'directory';
  children?: FileTreeEntry[];
  gitStatus?: 'M' | 'A' | 'D' | 'U' | ' ';
}

interface FileTreePanelProps {
  /** The file tree data to display. */
  tree: FileTreeEntry[];
  /** Called when a file is selected. */
  onFileSelect?: (path: string) => void;
  /** Called when a file is dragged for context. */
  onFileDrag?: (path: string) => void;
  /** Current working directory for display. */
  rootPath?: string;
}

function GitStatusBadge({ status }: { status?: string }) {
  if (!status || status === ' ') return null;
  const colors: Record<string, string> = {
    M: 'text-intent-warning-muted',  // modified
    A: 'text-intent-success-muted',  // staged
    D: 'text-intent-danger-muted',   // deleted
    U: 'text-accent-primary-muted',   // untracked
  };
  return (
    <span className={`text-[10px] font-mono font-bold ml-1 ${colors[status] || 'text-content-tertiary'}`}>
      {status}
    </span>
  );
}

/** Return a file-type-specific emoji icon based on the filename extension. */
function getFileIcon(filename: string): string {
  const ext = filename.split('.').pop()?.toLowerCase() || '';
  switch (ext) {
    case 'ts': case 'tsx': return '📘';
    case 'js': case 'jsx': return '📒';
    case 'rs': return '🦀';
    case 'py': return '🐍';
    case 'json': return '📋';
    case 'md': case 'mdx': return '📝';
    case 'css': case 'scss': case 'sass': case 'less': return '🎨';
    case 'png': case 'jpg': case 'jpeg': case 'gif': case 'svg': case 'webp': return '🖼️';
    case 'toml': case 'yaml': case 'yml': case 'env': case 'ini': case 'cfg': return '⚙️';
    case 'html': case 'htm': return '🌐';
    case 'sh': case 'bash': case 'zsh': case 'fish': return '💻';
    case 'lock': return '🔒';
    case 'gitignore': return '🙈';
    default: return '📄';
  }
}

function FileIcon({ type, name }: { type: 'file' | 'directory'; name: string }) {
  if (type === 'directory') {
    return <span className="shrink-0">📁</span>;
  }
  return <span className="shrink-0">{getFileIcon(name)}</span>;
}

interface TreeNodeProps {
  entry: FileTreeEntry;
  depth: number;
  onFileSelect?: (path: string) => void;
  onFileDrag?: (path: string) => void;
}

function TreeNode({ entry, depth, onFileSelect, onFileDrag }: TreeNodeProps) {
  const [expanded, setExpanded] = useState(depth < 1); // Auto-expand first level
  const hasChildren = entry.type === 'directory' && entry.children && entry.children.length > 0;

  const handleClick = useCallback(() => {
    if (entry.type === 'directory') {
      setExpanded(e => !e);
    } else {
      onFileSelect?.(entry.path);
    }
  }, [entry, onFileSelect]);

  const handleDragStart = useCallback((e: React.DragEvent) => {
    e.dataTransfer.setData('text/plain', entry.path);
    e.dataTransfer.effectAllowed = 'copy';
    onFileDrag?.(entry.path);
  }, [entry.path, onFileDrag]);

  return (
    <div>
      <div
        className={`flex items-center gap-1.5 py-0.5 px-2 cursor-pointer rounded text-sm hover:bg-surface-2/50 transition-colors select-none ${
          entry.type === 'directory' ? 'text-content-secondary' : 'text-content-secondary'
        }`}
        style={{ paddingLeft: `${8 + depth * 16}px` }}
        onClick={handleClick}
        draggable={entry.type === 'file'}
        onDragStart={entry.type === 'file' ? handleDragStart : undefined}
      >
        {hasChildren && (
          <span className="text-[10px] text-content-tertiary w-3 shrink-0">
            {expanded ? '▼' : '▶'}
          </span>
        )}
        {!hasChildren && <span className="w-3 shrink-0" />}
        <FileIcon type={entry.type} name={entry.name} />
        <span className="truncate flex-1">{entry.name}</span>
        <GitStatusBadge status={entry.gitStatus} />
      </div>
      {hasChildren && expanded && entry.children && (
        <div>
          {entry.children.map(child => (
            <TreeNode
              key={child.path}
              entry={child}
              depth={depth + 1}
              onFileSelect={onFileSelect}
              onFileDrag={onFileDrag}
            />
          ))}
        </div>
      )}
    </div>
  );
}

/**
 * Recursive directory tree view with file type icons, git status,
 * click-to-open, drag-to-context, and search/filter.
 */
export function FileTreePanel({ tree, onFileSelect, onFileDrag, rootPath }: FileTreePanelProps) {
  const [searchQuery, setSearchQuery] = useState('');

  const filteredTree = useMemo(() => {
    if (!searchQuery.trim()) return tree;

    const filterTree = (entries: FileTreeEntry[]): FileTreeEntry[] => {
      return entries
        .map(entry => {
          if (entry.type === 'directory' && entry.children) {
            const filteredChildren = filterTree(entry.children);
            if (filteredChildren.length > 0) {
              return { ...entry, children: filteredChildren };
            }
          }
          if (entry.name.toLowerCase().includes(searchQuery.toLowerCase())) {
            return entry;
          }
          return null;
        })
        .filter((e): e is FileTreeEntry => e !== null);
    };

    return filterTree(tree);
  }, [tree, searchQuery]);

  return (
    <div className="flex flex-col h-full">
      {/* Search input */}
      <div className="px-2 py-2 border-b border-border-default/15">
        <input
          type="text"
          value={searchQuery}
          onChange={e => setSearchQuery(e.target.value)}
          placeholder="Search files..."
          className="w-full px-3 py-1.5 text-sm bg-surface-primary border border-border-default/20 rounded-lg text-content-primary placeholder-content-tertiary focus:outline-none focus:ring-2 focus:ring-accent-secondary focus:border-accent-secondary"
        />
      </div>

      {/* Root path indicator */}
      {rootPath && (
        <div className="px-3 py-1 text-[10px] text-content-tertiary font-mono truncate border-b border-border-default/10 bg-surface-2/30">
          {rootPath}
        </div>
      )}

      {/* Tree */}
      <div className="flex-1 overflow-y-auto py-1">
        {filteredTree.length === 0 ? (
          <div className="text-xs text-content-tertiary text-center py-8">
            {searchQuery ? 'No files match your search' : 'No files found'}
          </div>
        ) : (
          filteredTree.map(entry => (
            <TreeNode
              key={entry.path}
              entry={entry}
              depth={0}
              onFileSelect={onFileSelect}
              onFileDrag={onFileDrag}
            />
          ))
        )}
      </div>

      {/* Status bar */}
      <div className="border-t border-border-default/10 px-3 py-1 text-[10px] text-content-tertiary flex items-center gap-2">
        <span>{tree.length} items</span>
        {searchQuery && (
          <span>· {filteredTree.length} matched</span>
        )}
      </div>
    </div>
  );
}
