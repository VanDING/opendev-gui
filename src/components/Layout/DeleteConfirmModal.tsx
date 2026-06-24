import { useState } from 'react';
import { Modal } from '../ui/Modal';
import { Button } from '../ui/Button';

interface DeleteConfirmModalProps {
  isOpen: boolean;
  workspacePath: string;
  onConfirm: () => void;
  onCancel: () => void;
}

export function DeleteConfirmModal({ isOpen, workspacePath, onConfirm, onCancel }: DeleteConfirmModalProps) {
  const [isDeleting, setIsDeleting] = useState(false);

  const handleConfirm = async () => {
    setIsDeleting(true);
    await onConfirm();
    setIsDeleting(false);
  };

  return (
    <Modal isOpen={isOpen} onClose={onCancel} title="" size="sm">
      <div className="p-6">
        {/* Header */}
        <div className="flex items-center gap-3 mb-4">
          <div className="w-12 h-12 rounded-full bg-intent-danger-muted flex items-center justify-center">
            <svg className="w-6 h-6 text-intent-danger" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
          </div>
          <div>
            <h2 className="text-xl font-bold text-content-primary">Delete Workspace</h2>
            <p className="text-sm text-content-tertiary">This action cannot be undone</p>
          </div>
        </div>

        {/* Content */}
        <div className="mb-6">
          <p className="text-sm text-content-secondary mb-3">
            Are you sure you want to delete this workspace and all its sessions?
          </p>
          <div className="px-4 py-3 bg-surface-elevated border border-border-default rounded-lg">
            <p className="text-xs font-semibold text-content-secondary mb-1">Workspace:</p>
            <p className="text-sm text-content-primary font-mono break-all">{workspacePath}</p>
          </div>
          <div className="mt-3 flex items-start gap-2 text-xs text-intent-danger">
            <svg className="w-4 h-4 flex-shrink-0 mt-0.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
            <p>All chat history and sessions will be permanently deleted.</p>
          </div>
        </div>

        {/* Footer */}
        <div className="flex gap-3">
          <Button variant="secondary" onClick={onCancel} disabled={isDeleting} className="flex-1">
            Cancel
          </Button>
          <Button variant="destructive" onClick={handleConfirm} disabled={isDeleting} loading={isDeleting} className="flex-1">
            {isDeleting ? 'Deleting...' : 'Delete Workspace'}
          </Button>
        </div>
      </div>
    </Modal>
  );
}
