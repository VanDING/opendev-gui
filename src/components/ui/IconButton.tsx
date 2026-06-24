import React from 'react';

function cn(...classes: Array<string | false | null | undefined>) {
  return classes.filter(Boolean).join(' ');
}

interface IconButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  size?: 'sm' | 'md';
  variant?: 'subtle' | 'ghost';
}

export function IconButton({
  className,
  size = 'sm',
  variant = 'subtle',
  children,
  ...props
}: IconButtonProps) {
  const sizes = {
    sm: 'p-2',
    md: 'p-2.5',
  }[size];

  const variants = {
    subtle: 'text-content-secondary hover:text-content-primary bg-surface-primary hover:bg-surface-2 border border-border-default rounded-md',
    ghost: 'text-content-secondary hover:text-content-primary hover:bg-surface-2 rounded-md',
  }[variant];

  return (
    <button className={cn('transition-colors', sizes, variants, className)} {...props}>
      {children}
    </button>
  );
}
