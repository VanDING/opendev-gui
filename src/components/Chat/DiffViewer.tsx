interface DiffViewerProps {
  diff: string;
}

interface DiffLine {
  type: 'add' | 'remove' | 'context' | 'header';
  content: string;
}

function parseDiff(raw: string): DiffLine[] {
  const lines = raw.split('\n');
  const result: DiffLine[] = [];

  for (const line of lines) {
    if (line.startsWith('+++') || line.startsWith('---') || line.startsWith('@@')) {
      result.push({ type: 'header', content: line });
    } else if (line.startsWith('+')) {
      result.push({ type: 'add', content: line });
    } else if (line.startsWith('-')) {
      result.push({ type: 'remove', content: line });
    } else {
      result.push({ type: 'context', content: line });
    }
  }

  return result;
}

/**
 * Renders a unified diff with colored add/remove lines.
 */
export function DiffViewer({ diff }: DiffViewerProps) {
  if (!diff) return null;

  const lines = parseDiff(diff);

  return (
    <div className="font-mono text-sm leading-6 rounded border border-border-default/15 bg-surface-primary overflow-x-auto">
      {lines.map((line, i) => {
        let className = 'px-3 py-0 whitespace-pre ';
        switch (line.type) {
          case 'add':
            className += 'bg-diff-added-bg text-diff-added-text';
            break;
          case 'remove':
            className += 'bg-diff-removed-bg text-diff-removed-text';
            break;
          case 'header':
            className += 'bg-diff-header-bg text-diff-header-text';
            break;
          default:
            className += 'text-content-secondary';
        }

        return (
          <div key={i} className={className}>
            {line.content}
          </div>
        );
      })}
    </div>
  );
}
