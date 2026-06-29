import { useState, useEffect, useCallback } from 'react';
import { useChatStore } from '../../stores/chat';
import { MarkdownContent } from './MarkdownContent';
import { Modal } from '../ui/Modal';

/** Archive a plan to ~/.opendev/plans/ for later reference. */
function archivePlan(requestId: string, planContent: string, decision: string): void {
  try {
    const fs = require('fs');
    const path = require('path');
    const home = process.env.HOME || process.env.USERPROFILE || '';
    const plansDir = path.join(home, '.opendev', 'plans');
    if (!fs.existsSync(plansDir)) {
      fs.mkdirSync(plansDir, { recursive: true });
    }
    const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
    const safeId = requestId.replace(/[^a-zA-Z0-9_-]/g, '_').slice(0, 20);
    const filename = `plan-${timestamp}-${safeId}.md`;
    const content = `# Plan (${decision})\n\nArchived: ${new Date().toISOString()}\nRequest ID: ${requestId}\n\n---\n\n${planContent}`;
    fs.writeFileSync(path.join(plansDir, filename), content, 'utf-8');
  } catch {
    // Silently fail — archive is best-effort
  }
}

export function PlanApprovalDialog() {
  const pendingPlanApproval = useChatStore(state => {
    const sid = state.currentSessionId;
    return sid ? state.sessionStates[sid]?.pendingPlanApproval ?? null : null;
  });
  const respondToPlanApproval = useChatStore(state => state.respondToPlanApproval);

  const [selectedIndex, setSelectedIndex] = useState(0);
  const [showFeedback, setShowFeedback] = useState(false);
  const [feedback, setFeedback] = useState('');
  const [autoAcceptEdits, setAutoAcceptEdits] = useState(false);
  const [showDiff, setShowDiff] = useState(false);
  const [focusIndex, setFocusIndex] = useState<number>(0);

  const options = [
    {
      label: 'Start implementation',
      description: 'Auto-approve file edits during implementation.',
      action: 'approve_auto',
    },
    {
      label: 'Start implementation (review edits)',
      description: 'Review each file edit before it\'s applied.',
      action: 'approve',
    },
    {
      label: 'Revise plan',
      description: 'Stay in plan mode and provide feedback.',
      action: 'modify',
    },
  ];

  // Reset state when new request comes in
  useEffect(() => {
    if (pendingPlanApproval) {
      setSelectedIndex(0);
      setShowFeedback(false);
      setFeedback('');
      setAutoAcceptEdits(false);
      setShowDiff(false);
      setFocusIndex(0);
    }
  }, [pendingPlanApproval]);

  const handleConfirm = useCallback(() => {
    const action = options[selectedIndex].action;
    if (action === 'modify') {
      if (!showFeedback) {
        setShowFeedback(true);
        return;
      }
      if (pendingPlanApproval) {
        archivePlan(pendingPlanApproval.request_id, pendingPlanApproval.plan_content, 'modified');
        respondToPlanApproval(pendingPlanApproval.request_id, action, feedback);
      }
    } else {
      if (pendingPlanApproval) {
        const effectiveAction = autoAcceptEdits ? 'approve_auto' : action;
        archivePlan(pendingPlanApproval.request_id, pendingPlanApproval.plan_content, effectiveAction);
        respondToPlanApproval(pendingPlanApproval.request_id, effectiveAction);
      }
    }
  }, [selectedIndex, showFeedback, feedback, autoAcceptEdits, pendingPlanApproval, respondToPlanApproval]);

  const handleRevise = useCallback(() => {
    setSelectedIndex(2);
    setShowFeedback(true);
  }, []);

  // Keyboard shortcuts — Tab between options, Enter to confirm
  useEffect(() => {
    if (!pendingPlanApproval) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      // Tab key: cycle through options
      if (e.key === 'Tab' && !e.shiftKey) {
        e.preventDefault();
        setFocusIndex(prev => (prev + 1) % (options.length + 2)); // +2 for auto-accept toggle + submit
      }
      if (e.key === 'Tab' && e.shiftKey) {
        e.preventDefault();
        setFocusIndex(prev => (prev - 1 + options.length + 2) % (options.length + 2));
      }

      // Number keys to select options
      const num = parseInt(e.key);
      if (!isNaN(num) && num >= 1 && num <= options.length) {
        e.preventDefault();
        setSelectedIndex(num - 1);
        setFocusIndex(num - 1);
        setShowFeedback(false);
      }

      // Enter to confirm
      if (e.key === 'Enter' && !e.shiftKey) {
        if (showFeedback && document.activeElement?.tagName === 'TEXTAREA') {
          return;
        }
        e.preventDefault();
        handleConfirm();
      }

      // Escape to revise
      if (e.key === 'Escape') {
        e.preventDefault();
        handleRevise();
      }

      // Arrow keys for option navigation
      if (e.key === 'ArrowUp') {
        e.preventDefault();
        const newIdx = (selectedIndex - 1 + options.length) % options.length;
        setSelectedIndex(newIdx);
        setFocusIndex(newIdx);
        setShowFeedback(false);
      }
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        const newIdx = (selectedIndex + 1) % options.length;
        setSelectedIndex(newIdx);
        setFocusIndex(newIdx);
        setShowFeedback(false);
      }

      // Space to toggle auto-accept
      if (e.key === ' ' && focusIndex === options.length) {
        e.preventDefault();
        setAutoAcceptEdits(prev => !prev);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [pendingPlanApproval, selectedIndex, showFeedback, feedback, focusIndex, handleConfirm, handleRevise]);

  if (!pendingPlanApproval) return null;

  return (
    <Modal isOpen={!!pendingPlanApproval} onClose={handleRevise} title="Plan Ready for Review" size="lg">
      {/* Plan diff toggle */}
      {showDiff && (
        <div className="border-b border-border-default/15 px-6 py-3 bg-surface-2/50">
          <div className="text-xs font-semibold text-content-tertiary uppercase mb-2">Plan Diff (previous → current)</div>
          <div className="font-mono text-xs text-content-secondary bg-surface-primary rounded p-2 overflow-x-auto max-h-32 overflow-y-auto">
            <span className="text-diff-added-text">+ Added new implementation steps</span>
            <br />
            <span className="text-diff-removed-text">- Removed outdated approach</span>
            <br />
            <span className="text-diff-header-text">@@ -10,7 +10,9 @@ ...</span>
          </div>
        </div>
      )}

      {/* Plan content — scrollable */}
      <div className="overflow-y-auto px-6 py-4 max-h-[50vh]">
        <div className="prose prose-sm prose-invert max-w-none text-content-secondary">
          <MarkdownContent content={pendingPlanApproval.plan_content} />
        </div>
      </div>

      {/* Options */}
      <div className="border-t border-border-default/15 px-6 py-4 space-y-2">
        {options.map((opt, i) => {
          const isSelected = i === selectedIndex;
          return (
            <button
              key={i}
              onClick={() => {
                setSelectedIndex(i);
                setFocusIndex(i);
                setShowFeedback(false);
                if (i !== 2) {
                  const effectiveAction = autoAcceptEdits ? 'approve_auto' : opt.action;
                  if (pendingPlanApproval) {
                    archivePlan(pendingPlanApproval.request_id, pendingPlanApproval.plan_content, effectiveAction);
                    respondToPlanApproval(pendingPlanApproval.request_id, effectiveAction);
                  }
                } else {
                  setShowFeedback(true);
                }
              }}
              tabIndex={focusIndex === i ? 0 : -1}
              className={`w-full px-4 py-3 text-sm text-left rounded-lg border-2 transition-all flex items-center gap-3 ${
                isSelected
                  ? 'border-accent-secondary/50 bg-accent-secondary-muted/50'
                  : 'border-border-default/15 hover:border-accent-secondary/30 hover:bg-surface-elevated'
              }`}
            >
              <span className="text-xs font-mono text-content-tertiary bg-surface-2 px-1.5 py-0.5 rounded">
                {i + 1}
              </span>
              <div className="flex-1">
                <span className={`font-medium ${isSelected ? 'text-content-primary' : 'text-content-primary'}`}>
                  {opt.label}
                </span>
                <span className="text-content-tertiary text-xs ml-2">{opt.description}</span>
              </div>
            </button>
          );
        })}

        {/* Auto-accept edits toggle */}
        <label
          tabIndex={focusIndex === options.length ? 0 : -1}
          className="flex items-center gap-3 px-4 py-2 text-sm cursor-pointer rounded-lg hover:bg-surface-elevated transition-colors"
          onKeyDown={(e) => {
            if (e.key === 'Enter' || e.key === ' ') {
              e.preventDefault();
              setAutoAcceptEdits(prev => !prev);
            }
          }}
        >
          <div
            className={`w-5 h-5 rounded border-2 flex items-center justify-center flex-shrink-0 transition-colors ${
              autoAcceptEdits
                ? 'border-accent-secondary bg-accent-secondary'
                : 'border-border-default/30'
            }`}
          >
            {autoAcceptEdits && (
              <svg className="w-3.5 h-3.5 text-content-inverse" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={3}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
              </svg>
            )}
          </div>
          <div>
            <span className="font-medium text-content-primary">Auto-accept file edits</span>
            <span className="text-content-tertiary text-xs ml-2">
              Skip future edit confirmations for this session
            </span>
          </div>
        </label>

        {/* Plan diff toggle */}
        <button
          onClick={() => setShowDiff(!showDiff)}
          className="w-full text-xs text-accent-secondary hover:text-accent-secondary/80 transition-colors text-center py-1"
        >
          {showDiff ? 'Hide plan diff' : 'Show plan diff'}
        </button>

        {/* Feedback textarea for revise */}
        {showFeedback && (
          <div className="mt-3 space-y-2">
            <textarea
              value={feedback}
              onChange={e => setFeedback(e.target.value)}
              placeholder="What changes would you like? (optional)"
              className="w-full px-4 py-2.5 border border-border-default/20 rounded-lg text-sm text-content-primary bg-surface-primary focus:outline-none focus:ring-2 focus:ring-accent-secondary focus:border-accent-secondary placeholder-content-tertiary resize-none"
              rows={3}
              autoFocus
            />
            <button
              onClick={() => {
                if (pendingPlanApproval) {
                  archivePlan(pendingPlanApproval.request_id, pendingPlanApproval.plan_content, 'modified');
                  respondToPlanApproval(pendingPlanApproval.request_id, 'modify', feedback);
                }
              }}
              className="px-4 py-2 text-sm font-medium text-content-inverse bg-accent-secondary rounded-lg hover:bg-accent-secondary/90 transition-colors"
            >
              Submit Feedback
            </button>
          </div>
        )}
      </div>

      {/* Keyboard hints */}
      <div className="text-center pb-4">
        <p className="text-xs text-content-tertiary">
          Press{' '}
          <kbd className="px-1 py-0.5 bg-surface-2 border border-border-default/20 rounded text-xs font-mono">Tab</kbd>{' '}
          to navigate,{' '}
          <kbd className="px-1 py-0.5 bg-surface-2 border border-border-default/20 rounded text-xs font-mono">1</kbd>-
          <kbd className="px-1 py-0.5 bg-surface-2 border border-border-default/20 rounded text-xs font-mono">3</kbd>{' '}
          to select,{' '}
          <kbd className="px-1 py-0.5 bg-surface-2 border border-border-default/20 rounded text-xs font-mono">Enter</kbd>{' '}
          to confirm,{' '}
          <kbd className="px-1 py-0.5 bg-surface-2 border border-border-default/20 rounded text-xs font-mono">Esc</kbd>{' '}
          to revise
        </p>
      </div>
    </Modal>
  );
}
