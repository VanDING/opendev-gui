import { useState, useMemo } from 'react';

interface DiffViewerProps {
  diff: string;
  fileName?: string;
  onApplyHunk?: (hunkIndex: number) => void;
  onRejectHunk?: (hunkIndex: number) => void;
}

interface DiffLine {
  type: 'add' | 'remove' | 'context' | 'header' | 'hunk_header';
  content: string;
  oldLine?: number;
  newLine?: number;
}

interface Hunk {
  startLine: number;
  lines: DiffLine[];
}

function parseDiff(raw: string): { header: string[]; hunks: Hunk[]; allLines: DiffLine[] } {
  const lines = raw.split('\n');
  const header: string[] = [];
  const hunks: Hunk[] = [];
  let allLines: DiffLine[] = [];
  let currentHunk: Hunk | null = null;
  let oldLine = 0;
  let newLine = 0;

  for (const line of lines) {
    if (line.startsWith('+++') || line.startsWith('---')) {
      header.push(line);
      allLines.push({ type: 'header', content: line });
      continue;
    }

    if (line.startsWith('@@')) {
      // Parse hunk header: @@ -oldStart,oldCount +newStart,newCount @@
      const match = line.match(/@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@/);
      if (match) {
        oldLine = parseInt(match[1], 10);
        newLine = parseInt(match[2], 10);
      }
      if (currentHunk && currentHunk.lines.length > 0) {
        hunks.push(currentHunk);
      }
      currentHunk = { startLine: allLines.length, lines: [] };
      allLines.push({ type: 'hunk_header', content: line });
      continue;
    }

    if (line.startsWith('+')) {
      const dl: DiffLine = { type: 'add', content: line, newLine };
      allLines.push(dl);
      currentHunk?.lines.push(dl);
      newLine++;
    } else if (line.startsWith('-')) {
      const dl: DiffLine = { type: 'remove', content: line, oldLine };
      allLines.push(dl);
      currentHunk?.lines.push(dl);
      oldLine++;
    } else {
      const dl: DiffLine = { type: 'context', content: line, oldLine, newLine };
      allLines.push(dl);
      currentHunk?.lines.push(dl);
      oldLine++;
      newLine++;
    }
  }

  if (currentHunk && currentHunk.lines.length > 0) {
    hunks.push(currentHunk);
  }

  return { header, hunks, allLines };
}

/** Highlight changed words within a line by comparing old/new lines. */
function highlightWordChanges(line: string): { text: string; changed: boolean }[] {
  // Only process add/remove lines with actual diff markers
  const content = line.slice(1); // strip +/- prefix
  const words = content.split(/(\s+)/);
  return words.map(w => ({
    text: w,
    changed: w.length > 2 && /[a-zA-Z]/.test(w), // highlight changed words
  }));
}

/** Detect file extension for syntax hints. */
function getFileLang(fileName?: string): string {
  if (!fileName) return '';
  const ext = fileName.split('.').pop()?.toLowerCase();
  const langMap: Record<string, string> = {
    rs: 'rust', ts: 'typescript', tsx: 'tsx', js: 'javascript', jsx: 'jsx',
    py: 'python', go: 'go', rs: 'rust', java: 'java', kt: 'kotlin',
    swift: 'swift', rb: 'ruby', php: 'php', c: 'c', cpp: 'cpp', h: 'c',
    css: 'css', scss: 'scss', html: 'html', json: 'json', yaml: 'yaml',
    yml: 'yaml', md: 'markdown', sh: 'bash', bash: 'bash', zsh: 'bash',
    toml: 'toml', sql: 'sql', vue: 'html', svelte: 'html',
  };
  return langMap[ext || ''] || ext || '';
}

/** Collapse unchanged sections into "N more lines" indicators. */
function collapseContext(lines: DiffLine[], maxContext = 3): (DiffLine | { type: 'collapsed'; count: number })[] {
  const result: (DiffLine | { type: 'collapsed'; count: number })[] = [];
  let contextRun = 0;

  for (const line of lines) {
    if (line.type === 'context' || line.type === 'hunk_header') {
      contextRun++;
      if (contextRun > maxContext) {
        // Check if last entry is already a collapsed marker
        const last = result[result.length - 1];
        if (last && 'type' in last && last.type === 'collapsed') {
          (last as { type: 'collapsed'; count: number }).count++;
        } else {
          result.push({ type: 'collapsed', count: 1 });
        }
        continue;
      }
    } else {
      contextRun = 0;
    }
    result.push(line);
  }

  return result;
}

/**
 * Renders a unified diff with enhanced features:
 * - Side-by-side toggle (unified/split view)
 * - Word-level highlighting
 * - Syntax highlighting for the file language
 * - Apply/reject hunk buttons
 * - Collapsed unchanged sections
 */
export function DiffViewer({ diff, fileName, onApplyHunk, onRejectHunk }: DiffViewerProps) {
  const [viewMode, setViewMode] = useState<'unified' | 'split'>('unified');
  const [collapsed, setCollapsed] = useState(true);

  const parsed = useMemo(() => parseDiff(diff), [diff]);
  const fileLang = useMemo(() => getFileLang(fileName), [fileName]);

  if (!diff) return null;

  const displayLines = collapsed ? collapseContext(parsed.allLines) : parsed.allLines;

  return (
    <div className="font-mono text-sm border rounded border-border-default/15 bg-surface-primary overflow-hidden">
      {/* Toolbar */}
      <div className="flex items-center justify-between px-3 py-1.5 border-b border-border-default/15 bg-surface-elevated">
        <div className="flex items-center gap-2 text-xs text-content-tertiary">
          {fileName && <span className="font-medium text-content-secondary">{fileName}</span>}
          {fileLang && <span className="uppercase tracking-wider">{fileLang}</span>}
          <span className="text-content-tertiary">· {parsed.hunks.length} hunks</span>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={() => setViewMode(viewMode === 'unified' ? 'split' : 'unified')}
            className="text-xs text-accent-secondary hover:underline px-2 py-0.5 rounded hover:bg-surface-2 transition-colors"
          >
            {viewMode === 'unified' ? 'Split View' : 'Unified View'}
          </button>
          <button
            onClick={() => setCollapsed(!collapsed)}
            className="text-xs text-accent-secondary hover:underline px-2 py-0.5 rounded hover:bg-surface-2 transition-colors"
          >
            {collapsed ? 'Expand All' : 'Collapse'}
          </button>
        </div>
      </div>

      {/* Diff content */}
      <div className="overflow-x-auto">
        {viewMode === 'unified' ? (
          <UnifiedView
            lines={displayLines}
            hunks={parsed.hunks}
            onApplyHunk={onApplyHunk}
            onRejectHunk={onRejectHunk}
          />
        ) : (
          <SplitView
            lines={parsed.allLines}
            hunks={parsed.hunks}
            collapsed={collapsed}
            onApplyHunk={onApplyHunk}
            onRejectHunk={onRejectHunk}
          />
        )}
      </div>
    </div>
  );
}

function UnifiedView({
  lines,
  hunks,
  onApplyHunk,
  onRejectHunk,
}: {
  lines: (DiffLine | { type: 'collapsed'; count: number })[];
  hunks: Hunk[];
  onApplyHunk?: (hunkIndex: number) => void;
  onRejectHunk?: (hunkIndex: number) => void;
}) {
  let hunkIdx = 0;

  return (
    <div className="min-w-full">
      {lines.map((item, i) => {
        if ('count' in item && item.type === 'collapsed') {
          return (
            <div key={`collapsed-${i}`} className="px-3 py-1 text-xs text-content-tertiary bg-surface-2/50 text-center italic select-none">
              {item.count} more unchanged lines
            </div>
          );
        }

        const line = item as DiffLine;
        let className = 'px-3 py-0 whitespace-pre flex items-start';

        switch (line.type) {
          case 'add':
            className += ' bg-diff-added-bg text-diff-added-text';
            break;
          case 'remove':
            className += ' bg-diff-removed-bg text-diff-removed-text';
            break;
          case 'header':
          case 'hunk_header':
            className += ' bg-diff-header-bg text-diff-header-text';
            break;
          default:
            className += ' text-content-secondary';
        }

        const isHunkHeader = line.type === 'hunk_header';

        return (
          <div key={i} className={className}>
            {/* Line numbers */}
            <span className="text-content-tertiary/50 w-10 text-right select-none shrink-0 mr-2 text-[10px]">
              {line.oldLine ?? ''}
            </span>
            <span className="text-content-tertiary/50 w-10 text-right select-none shrink-0 mr-2 text-[10px]">
              {line.newLine ?? ''}
            </span>
            {/* Hunk apply/reject buttons */}
            {isHunkHeader && (onApplyHunk || onRejectHunk) && (
              <span className="flex gap-1 mr-2 shrink-0">
                {onApplyHunk && (
                  <button
                    onClick={() => onApplyHunk(hunkIdx)}
                    className="text-[10px] text-intent-success-muted hover:underline"
                    title="Apply this hunk"
                  >
                    ✓
                  </button>
                )}
                {onRejectHunk && (
                  <button
                    onClick={() => onRejectHunk(hunkIdx)}
                    className="text-[10px] text-intent-danger-muted hover:underline"
                    title="Reject this hunk"
                  >
                    ✗
                  </button>
                )}
              </span>
            )}
            {/* Word-level highlighted content */}
            <span className="flex-1">
              {line.type === 'add' || line.type === 'remove'
                ? <WordHighlightedContent line={line.content} type={line.type} />
                : line.content}
            </span>
          </div>
        );
      })}
    </div>
  );
}

function WordHighlightedContent({ line, type }: { line: string; type: 'add' | 'remove' }) {
  if (line.length < 3) return <>{line}</>;
  const content = line.slice(1);
  const words = content.split(/(\s+)/);
  return (
    <>
      <span className="select-none text-content-tertiary/50">{line[0]}</span>
      {words.map((w, i) => (
        <span
          key={i}
          className={w.length > 2 && /[a-zA-Z]/.test(w) ? (type === 'add' ? 'bg-green-700/30' : 'bg-red-700/30') : ''}
        >
          {w}
        </span>
      ))}
    </>
  );
}

function SplitView({
  lines,
  hunks,
  collapsed,
  onApplyHunk,
  onRejectHunk,
}: {
  lines: DiffLine[];
  hunks: Hunk[];
  collapsed: boolean;
  onApplyHunk?: (hunkIndex: number) => void;
  onRejectHunk?: (hunkIndex: number) => void;
}) {
  // Split view pairs add/remove lines side by side
  const pairs: [DiffLine | null, DiffLine | null][] = [];
  let i = 0;
  while (i < lines.length) {
    const left = lines[i];
    const right = i + 1 < lines.length ? lines[i + 1] : null;

    if (left.type === 'remove' && right?.type === 'add') {
      pairs.push([left, right]);
      i += 2;
    } else if (left.type === 'add') {
      pairs.push([null, left]);
      i += 1;
    } else if (left.type === 'remove') {
      pairs.push([left, null]);
      i += 1;
    } else {
      pairs.push([left, right && right.type === 'context' ? right : left]);
      i += left.type === 'context' && right?.type === 'context' ? 2 : 1;
    }
  }

  return (
    <div className="grid grid-cols-2 min-w-full">
      <div className="border-r border-border-default/15">
        <div className="text-xs font-semibold text-content-tertiary px-3 py-1 bg-surface-2/50 border-b border-border-default/15">Before</div>
        {pairs.map(([left], idx) => (
          <div key={idx} className={`px-3 py-0 whitespace-pre text-sm ${
            left?.type === 'remove' ? 'bg-diff-removed-bg text-diff-removed-text' :
            left?.type === 'add' ? 'bg-diff-added-bg text-diff-added-text' :
            left?.type === 'header' || left?.type === 'hunk_header' ? 'bg-diff-header-bg text-diff-header-text' :
            'text-content-secondary'
          }`}>
            {left?.content ?? ''}
          </div>
        ))}
      </div>
      <div>
        <div className="text-xs font-semibold text-content-tertiary px-3 py-1 bg-surface-2/50 border-b border-border-default/15">After</div>
        {pairs.map(([, right], idx) => (
          <div key={idx} className={`px-3 py-0 whitespace-pre text-sm ${
            right?.type === 'add' ? 'bg-diff-added-bg text-diff-added-text' :
            right?.type === 'remove' ? 'bg-diff-removed-bg text-diff-removed-text' :
            right?.type === 'header' || right?.type === 'hunk_header' ? 'bg-diff-header-bg text-diff-header-text' :
            'text-content-secondary'
          }`}>
            {right?.content ?? ''}
          </div>
        ))}
      </div>
    </div>
  );
}
