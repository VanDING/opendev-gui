import { useState, useEffect } from 'react';
import { useChatStore } from '../../stores/chat';
import { MarkdownContent } from './MarkdownContent';
import { Modal } from '../ui/Modal';

export function PlanApprovalDialog() {
  const pendingPlanApproval = useChatStore(state => {
    const sid = state.currentSessionId;
    return sid ? state.sessionStates[sid]?.pendingPlanApproval ?? null : null;
  });
  const respondToPlanApproval = useChatStore(state => state.respondToPlanApproval);

  const [selectedIndex, setSelectedIndex] = useState(0);
  const [showFeedback, setShowFeedback] = useState(false);
  const [feedback, setFeedback] = useState('');

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
    }
  }, [pendingPlanApproval]);

  // Keyboard shortcuts
  useEffect(() => {
    if (!pendingPlanApproval) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      // Number keys to select options
      const num = parseInt(e.key);
      if (!isNaN(num) && num >= 1 && num <= options.length) {
        e.preventDefault();
        setSelectedIndex(num - 1);
        setShowFeedback(false);
      }

      // Enter to confirm
      if (e.key === 'Enter' && !e.shiftKey) {
        if (showFeedback && document.activeElement?.tagName === 'TEXTAREA') {
          return; // Let textarea handle Enter
        }
        e.preventDefault();
        handleConfirm();
      }

      // Escape to revise
      if (e.key === 'Escape') {
        e.preventDefault();
        handleRevise();
      }

      // Arrow keys
      if (e.key === 'ArrowUp') {
        e.preventDefault();
        setSelectedIndex(prev => (prev - 1 + options.length) % options.length);
        setShowFeedback(false);
      }
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        setSelectedIndex(prev => (prev + 1) % options.length);
        setShowFeedback(false);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [pendingPlanApproval, selectedIndex, showFeedback, feedback]);

  if (!pendingPlanApproval) return null;

  const handleConfirm = () => {
    const action = options[selectedIndex].action;
    if (action === 'modify') {
      if (!showFeedback) {
        setShowFeedback(true);
        return;
      }
      respondToPlanApproval(pendingPlanApproval.request_id, action, feedback);
    } else {
      respondToPlanApproval(pendingPlanApproval.request_id, action);
    }
  };

  const handleRevise = () => {
    setSelectedIndex(2); // Revise plan
    setShowFeedback(true);
  };

  return (
    <Modal isOpen={!!pendingPlanApproval} onClose={handleRevise} title="Plan Ready for Review" size="lg">
      {/* Plan content - scrollable */}
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
                setShowFeedback(false);
                if (i !== 2) {
                  respondToPlanApproval(pendingPlanApproval.request_id, opt.action);
                } else {
                  setShowFeedback(true);
                }
              }}
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
              onClick={() => respondToPlanApproval(pendingPlanApproval.request_id, 'modify', feedback)}
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
