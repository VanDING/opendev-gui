function cn(...classes: Array<string | false | null | undefined>) {
  return classes.filter(Boolean).join(' ');
}

export interface SegmentOption<T extends string> {
  label: string;
  value: T;
}

interface SegmentedControlProps<T extends string> {
  options: SegmentOption<T>[];
  value: T;
  onChange: (value: T) => void;
  className?: string;
}

export function SegmentedControl<T extends string>({ options, value, onChange, className }: SegmentedControlProps<T>) {
  return (
    <div className={cn('inline-flex items-center gap-1 p-1 bg-surface-2 rounded-lg border border-border-default', className)}>
      {options.map(opt => {
        const active = opt.value === value;
        return (
          <button
            key={opt.value}
            onClick={() => onChange(opt.value)}
            className={cn(
              'px-2.5 py-1 text-xs font-medium rounded-md transition-colors',
              active ? 'bg-surface-primary text-content-primary shadow-sm border border-border-default' : 'text-content-secondary hover:text-content-primary'
            )}
          >
            {opt.label}
          </button>
        );
      })}
    </div>
  );
}
