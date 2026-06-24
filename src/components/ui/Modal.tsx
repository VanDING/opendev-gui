import * as Dialog from '@radix-ui/react-dialog';

interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
  title?: string;
  size?: 'sm' | 'md' | 'lg';
  className?: string;
  children: React.ReactNode;
}

const SIZE_MAP = {
  sm: 'max-w-sm',
  md: 'max-w-lg',
  lg: 'max-w-2xl',
};

export function Modal({ isOpen, onClose, title, size = 'md', className, children }: ModalProps) {
  return (
    <Dialog.Root open={isOpen} onOpenChange={(open) => !open && onClose()}>
      <Dialog.Portal>
        <Dialog.Overlay
          className={[
            'fixed inset-0 z-overlay bg-surface-overlay',
            'data-[state=open]:animate-modal-overlay-in',
            'data-[state=closed]:animate-modal-overlay-out',
          ].join(' ')}
        />
        <Dialog.Content
          className={[
            'fixed left-1/2 top-1/2 z-modal -translate-x-1/2 -translate-y-1/2',
            'w-[calc(100vw-2rem)]',
            SIZE_MAP[size],
            'bg-surface-elevated border border-border-default rounded-xl shadow-popover',
            'data-[state=open]:animate-modal-content-in',
            'data-[state=closed]:animate-modal-content-out',
            className,
          ].filter(Boolean).join(' ')}
        >
          {title && (
            <Dialog.Title className="text-lg font-semibold text-content-primary px-6 pt-5 pb-0">
              {title}
            </Dialog.Title>
          )}
          {children}
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
