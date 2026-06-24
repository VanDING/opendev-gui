import { useState } from 'react';
import { useFileChangesStore } from '../../stores/fileChanges';
import { useChatStore } from '../../stores/chat';
import { Modal } from '../ui/Modal';
import { Button } from '../ui/Button';

export function FileChangesButton() {
  const [isModalOpen, setIsModalOpen] = useState(false);
  const { changes, summary, loadFileChanges, isLoading } = useFileChangesStore();
  const { currentSessionId } = useChatStore(state => state);

  const handleClick = () => {
    if (currentSessionId && !isModalOpen) {
      loadFileChanges(currentSessionId);
    }
    setIsModalOpen(!isModalOpen);
  };

  const hasChanges = changes && changes.length > 0;
  const changeCount = changes.length;

  return (
    <>
      <button
        onClick={handleClick}
        className="flex items-center gap-2 px-3 py-2 text-xs text-content-secondary bg-surface-elevated border border-border-default rounded-lg hover:bg-surface-2 hover:text-content-primary transition-colors"
        title={hasChanges ? `${changeCount} file changes` : 'View file changes'}
      >
        <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
          <path strokeLinecap="round" strokeLinejoin="round" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
        </svg>
        <span className="font-medium">File Changes</span>
        {hasChanges && (
          <span className="px-1.5 py-0.5 bg-accent-primary-muted text-accent-primary rounded-md font-medium">
            {changeCount}
          </span>
        )}
      </button>

      {isModalOpen && (
        <FileChangesModal
          onClose={() => setIsModalOpen(false)}
          changes={changes}
          summary={summary}
          isLoading={isLoading}
        />
      )}
    </>
  );
}

interface FileChangesModalProps {
  onClose: () => void;
  changes: any[];
  summary: any;
  isLoading: boolean;
}

function FileChangesModal({ onClose, changes, summary, isLoading }: FileChangesModalProps) {
  return (
    <Modal isOpen={true} onClose={onClose} title="File Changes" size="lg">
      {/* Summary */}
      {summary && (
        <div className="p-4 bg-surface-elevated border-b border-border-default">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-center">
            <div className="bg-surface-primary p-3 rounded-lg border border-border-default">
              <div className="text-xl font-bold text-accent-primary">{summary.total}</div>
              <div className="text-xs text-content-secondary">Total Changes</div>
            </div>
            <div className="bg-surface-primary p-3 rounded-lg border border-border-default">
              <div className="text-xl font-bold text-intent-success">+{summary.total_lines_added}</div>
              <div className="text-xs text-content-secondary">Lines Added</div>
            </div>
            <div className="bg-surface-primary p-3 rounded-lg border border-border-default">
              <div className="text-xl font-bold text-intent-danger">-{summary.total_lines_removed}</div>
              <div className="text-xs text-content-secondary">Lines Removed</div>
            </div>
            <div className="bg-surface-primary p-3 rounded-lg border border-border-default">
              <div className="text-xl font-bold text-intent-purple">{summary.net_lines}</div>
              <div className="text-xs text-content-secondary">Net Change</div>
            </div>
          </div>
        </div>
      )}

      {/* Changes List */}
      <div className="flex-1 overflow-y-auto p-4">
        {isLoading ? (
          <div className="flex items-center justify-center py-8">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-accent-primary"></div>
          </div>
        ) : changes && changes.length > 0 ? (
          <div className="space-y-2">
            {changes.map((change, index) => (
              <FileChangeItem key={index} change={change} />
            ))}
          </div>
        ) : (
          <div className="text-center py-8 text-content-tertiary">
            <svg className="w-12 h-12 mx-auto mb-4 text-content-tertiary" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
            <div className="text-lg font-medium">No file changes yet</div>
            <div className="text-sm">Start making changes to see them here</div>
          </div>
        )}
      </div>

      {/* Footer */}
      <div className="p-4 border-t border-border-default bg-surface-elevated">
        <div className="flex justify-between items-center">
          <div className="text-xs text-content-tertiary">
            {changes.length} change{changes.length !== 1 ? 's' : ''} tracked
          </div>
          <Button variant="primary" onClick={onClose}>
            Close
          </Button>
        </div>
      </div>
    </Modal>
  );
}

interface FileChangeItemProps {
  change: any;
}

function FileChangeItem({ change }: FileChangeItemProps) {
  const getTimeAgo = (timestamp: string) => {
    const now = new Date();
    const changeTime = new Date(timestamp);
    const diffInMinutes = Math.floor((now.getTime() - changeTime.getTime()) / (1000 * 60));

    if (diffInMinutes < 1) return 'just now';
    if (diffInMinutes < 60) return `${diffInMinutes}m ago`;
    if (diffInMinutes < 1440) return `${Math.floor(diffInMinutes / 60)}h ago`;
    return `${Math.floor(diffInMinutes / 1440)}d ago`;
  };

  const getStatusColor = (color: string) => {
    switch (color) {
      case 'green': return 'text-intent-success bg-green-50 border-green-200';
      case 'blue': return 'text-accent-primary bg-accent-primary-muted border-accent-primary-muted';
      case 'red': return 'text-intent-danger bg-intent-danger-muted border-intent-danger-muted';
      case 'orange': return 'text-intent-warning bg-intent-warning-muted border-intent-warning';
      default: return 'text-content-secondary bg-surface-elevated border-border-default';
    }
  };

  const statusClasses = getStatusColor(change.color);

  return (
    <div className="bg-surface-primary border border-border-default rounded-lg p-3 hover:bg-surface-elevated transition-colors">
      <div className="flex items-center gap-3">
        <span className="text-lg">{change.icon}</span>

        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <span className="font-medium text-content-primary truncate">
              {change.file_path.split('/').pop()}
            </span>
            <span className={`px-2 py-1 rounded text-xs font-medium border ${statusClasses}`}>
              {change.type}
            </span>
          </div>

          <div className="text-xs text-content-tertiary mb-1">
            {change.file_path}
          </div>

          {change.summary && (
            <div className="text-xs text-content-secondary font-mono mb-1">
              {change.summary}
            </div>
          )}

          {change.description && (
            <div className="text-xs text-content-tertiary">
              {change.description}
            </div>
          )}
        </div>

        <div className="text-right">
          <div className="text-xs text-content-tertiary">
            {getTimeAgo(change.timestamp)}
          </div>
          {(change.lines_added > 0 || change.lines_removed > 0) && (
            <div className="text-xs font-medium mt-1">
              <span className="text-intent-success">+{change.lines_added}</span>
              <span className="text-intent-danger ml-1">-{change.lines_removed}</span>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}