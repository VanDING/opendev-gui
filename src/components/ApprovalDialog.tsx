import { useEffect } from 'react';
import { useChatStore } from '../stores/chat';
import { Modal } from './ui/Modal';

export function ApprovalDialog() {
  const pendingApproval = useChatStore(state => {
    const sid = state.currentSessionId;
    return sid ? state.sessionStates[sid]?.pendingApproval ?? null : null;
  });
  const respondToApproval = useChatStore(state => state.respondToApproval);

  // Define handlers first (before hooks)
  const handleApprove = () => {
    if (pendingApproval) {
      respondToApproval(pendingApproval.id, true, false);
    }
  };

  const handleApproveAll = () => {
    if (pendingApproval) {
      respondToApproval(pendingApproval.id, true, true);
    }
  };

  const handleDeny = () => {
    if (pendingApproval) {
      respondToApproval(pendingApproval.id, false, false);
    }
  };

  // Debug log - MUST be before early return
  useEffect(() => {
    if (pendingApproval) {
      console.log('[ApprovalDialog] Showing approval dialog:', pendingApproval);
    } else {
      console.log('[ApprovalDialog] No pending approval');
    }
  }, [pendingApproval]);

  // Keyboard shortcuts - MUST be before early return
  useEffect(() => {
    if (!pendingApproval) return;

    const handleKeyPress = (e: KeyboardEvent) => {
      if (e.key === '1') {
        handleApprove();
      } else if (e.key === '2') {
        handleApproveAll();
      } else if (e.key === '3') {
        handleDeny();
      }
    };

    window.addEventListener('keydown', handleKeyPress);
    return () => window.removeEventListener('keydown', handleKeyPress);
  }, [pendingApproval]);

  // Early return AFTER all hooks
  if (!pendingApproval) {
    return null;
  }

  return (
    <Modal isOpen={!!pendingApproval} onClose={handleDeny} title="Approval Required" size="md">
      <div className="px-6 py-5 space-y-4 overflow-y-auto max-h-[60vh]">
        {/* Tool Name */}
        <div>
          <div className="text-xs font-medium text-content-tertiary uppercase tracking-wider mb-1">
            Tool
          </div>
          <div className="flex items-center gap-2">
            <div className="w-2 h-2 rounded-full bg-accent-primary" />
            <code className="text-sm font-mono bg-surface-elevated px-2 py-1 rounded border border-border-default/15">
              {pendingApproval.tool_name}
            </code>
          </div>
        </div>

        {/* Description */}
        <div>
          <div className="text-xs font-medium text-content-tertiary uppercase tracking-wider mb-1">
            Description
          </div>
          <p className="text-sm text-content-primary leading-relaxed">
            {pendingApproval.description}
          </p>
        </div>

        {/* Preview */}
        {pendingApproval.preview && (
          <div>
            <div className="text-xs font-medium text-content-tertiary uppercase tracking-wider mb-2">
              Preview
            </div>
            <div className="bg-surface-elevated rounded-lg border border-border-default/15 p-4 max-h-48 overflow-y-auto">
              <pre className="text-xs text-content-primary font-mono whitespace-pre-wrap">
                {pendingApproval.preview}
              </pre>
            </div>
          </div>
        )}

        {/* Arguments */}
        {Object.keys(pendingApproval.arguments).length > 0 && (
          <div>
            <div className="text-xs font-medium text-content-tertiary uppercase tracking-wider mb-2">
              Arguments
            </div>
            <div className="bg-surface-elevated rounded-lg border border-border-default/15 p-4 max-h-64 overflow-y-auto">
              <pre className="text-xs text-content-primary font-mono whitespace-pre-wrap">
                {JSON.stringify(pendingApproval.arguments, null, 2)}
              </pre>
            </div>
          </div>
        )}

        {/* Warning */}
        <div className="bg-intent-warning/10 border border-intent-warning/20 rounded-lg p-4">
          <div className="flex gap-3">
            <svg className="w-5 h-5 text-intent-warning flex-shrink-0 mt-0.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
            <div>
              <p className="text-sm font-medium text-content-primary">Review carefully before approving</p>
              <p className="text-xs text-content-secondary mt-1">
                This operation will be executed with your current permissions. Make sure you understand what it will do.
              </p>
            </div>
          </div>
        </div>
      </div>

      {/* Footer - 3 Options */}
      <div className="border-t border-border-default/15 px-6 py-4 bg-surface-elevated">
        <div className="space-y-2">
          {/* Option 1: Yes, run this command */}
          <button
            onClick={handleApprove}
            className="w-full px-4 py-3 text-sm font-medium text-left bg-surface-primary border-2 border-intent-success/30 rounded-lg hover:bg-intent-success/5 hover:border-intent-success/50 transition-all flex items-center gap-3 group"
          >
            <div className="w-6 h-6 rounded-full bg-intent-success/10 group-hover:bg-intent-success/20 flex items-center justify-center text-intent-success font-semibold text-xs">
              1
            </div>
            <span className="text-content-primary">Yes, run this command</span>
          </button>

          {/* Option 2: Yes, and auto-approve */}
          <button
            onClick={handleApproveAll}
            className="w-full px-4 py-3 text-sm font-medium text-left bg-surface-primary border-2 border-accent-secondary/30 rounded-lg hover:bg-accent-secondary/5 hover:border-accent-secondary/50 transition-all flex items-center gap-3 group"
          >
            <div className="w-6 h-6 rounded-full bg-accent-secondary/10 group-hover:bg-accent-secondary/20 flex items-center justify-center text-accent-secondary font-semibold text-xs">
              2
            </div>
            <div className="flex-1">
              <div className="text-content-primary">Yes, and auto-approve all <span className="font-semibold text-accent-secondary">{pendingApproval.tool_name}</span> commands</div>
              <div className="text-xs text-content-tertiary mt-0.5">Future similar commands will run automatically</div>
            </div>
          </button>

          {/* Option 3: No, cancel */}
          <button
            onClick={handleDeny}
            className="w-full px-4 py-3 text-sm font-medium text-left bg-surface-primary border-2 border-intent-danger/30 rounded-lg hover:bg-intent-danger/5 hover:border-intent-danger/50 transition-all flex items-center gap-3 group"
          >
            <div className="w-6 h-6 rounded-full bg-intent-danger/10 group-hover:bg-intent-danger/20 flex items-center justify-center text-intent-danger font-semibold text-xs">
              3
            </div>
            <span className="text-content-primary">No, cancel and provide feedback</span>
          </button>
        </div>

        {/* Keyboard shortcuts hint */}
        <div className="mt-4 pt-3 border-t border-border-default/15 text-center">
          <p className="text-xs text-content-tertiary">
            Press <kbd className="px-1.5 py-0.5 bg-surface-2 border border-border-default/20 rounded text-xs font-mono">1</kbd>, <kbd className="px-1.5 py-0.5 bg-surface-2 border border-border-default/20 rounded text-xs font-mono">2</kbd>, or <kbd className="px-1.5 py-0.5 bg-surface-2 border border-border-default/20 rounded text-xs font-mono">3</kbd> to select
          </p>
        </div>
      </div>
    </Modal>
  );
}
