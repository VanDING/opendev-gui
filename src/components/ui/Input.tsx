import React, { forwardRef } from 'react';

function cn(...classes: Array<string | false | null | undefined>) {
  return classes.filter(Boolean).join(' ');
}

interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  fullWidth?: boolean;
}

export const Input = forwardRef<HTMLInputElement, InputProps>(function Input(
  { fullWidth, className, ...props },
  ref,
) {
  return (
    <input
      ref={ref}
      className={cn(
        'bg-surface-input text-content-primary border border-border-default rounded-md px-3 py-1.5 text-sm',
        'focus:border-accent-primary focus:ring-2 focus:ring-accent-primary/20 focus:outline-hidden',
        'disabled:opacity-50 disabled:cursor-not-allowed',
        'placeholder:text-content-tertiary',
        fullWidth ? 'w-full' : '',
        className,
      )}
      {...props}
    />
  );
});
