import React from 'react';
import { HaloSpinner } from './HaloSpinner';

type ButtonVariant = 'primary' | 'secondary' | 'destructive' | 'ghost' | 'outline';
type ButtonSize = 'xs' | 'sm' | 'md';

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  loading?: boolean;
  fullWidth?: boolean;
}

const VARIANT_CLASSES: Record<ButtonVariant, string> = {
  primary:
    'bg-accent-primary text-content-inverse hover:bg-accent-primary-hover',
  secondary:
    'bg-surface-2 text-content-primary border border-border-default hover:bg-surface-3',
  destructive:
    'bg-intent-danger text-content-inverse hover:bg-intent-danger-hover',
  ghost:
    'bg-transparent text-content-secondary hover:bg-surface-2',
  outline:
    'bg-transparent text-accent-primary border border-accent-primary hover:bg-accent-primary-muted',
};

const SIZE_CLASSES: Record<ButtonSize, string> = {
  xs: 'px-2 py-1 text-xs rounded-sm',
  sm: 'px-3 py-1.5 text-sm rounded-md',
  md: 'px-4 py-2 text-base rounded-md',
};

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  (
    {
      className = '',
      variant = 'primary',
      size = 'sm',
      loading = false,
      fullWidth = false,
      disabled,
      children,
      ...props
    },
    ref,
  ) => {
    return (
      <button
        ref={ref}
        disabled={disabled || loading}
        className={[
          'inline-flex items-center justify-center font-medium transition-colors',
          'focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-accent-primary/50 focus-visible:ring-offset-2 focus-visible:ring-offset-surface-primary',
          'disabled:opacity-50 disabled:cursor-not-allowed',
          fullWidth ? 'w-full' : '',
          VARIANT_CLASSES[variant],
          SIZE_CLASSES[size],
          className,
        ]
          .filter(Boolean)
          .join(' ')}
        {...props}
      >
        {loading && (
          <span className="mr-2 inline-flex items-center">
            <HaloSpinner />
          </span>
        )}
        {children}
      </button>
    );
  },
);

Button.displayName = 'Button';

export { Button };
