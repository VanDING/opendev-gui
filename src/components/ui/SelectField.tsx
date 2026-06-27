import { useState, useRef, useEffect } from 'react';
import { ChevronDown } from 'lucide-react';

interface SelectOption {
  value: string;
  label: string;
  description?: string;
}

interface SelectFieldProps {
  options: SelectOption[];
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  disabled?: boolean;
  className?: string;
}

export function SelectField({
  options,
  value,
  onChange,
  placeholder = 'Select...',
  disabled = false,
  className = '',
}: SelectFieldProps) {
  const [open, setOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  const selected = options.find(o => o.value === value);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    if (open) {
      document.addEventListener('mousedown', handleClickOutside);
    }
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [open]);

  return (
    <div ref={containerRef} className={`relative ${className}`}>
      <button
        type="button"
        onClick={() => !disabled && setOpen(!open)}
        disabled={disabled}
        className={`w-full px-3 py-2 text-sm text-left border border-border-emphasis rounded-lg focus:outline-none focus:ring-2 focus:ring-accent-primary focus:border-transparent bg-surface-primary flex items-center justify-between gap-2 ${disabled ? 'opacity-50 cursor-not-allowed' : 'hover:border-border-default cursor-pointer'}`}
      >
        <span className={selected ? 'text-content-primary' : 'text-content-tertiary'}>
          {selected ? selected.label : placeholder}
        </span>
        <ChevronDown className={`w-4 h-4 text-content-tertiary flex-shrink-0 transition-transform ${open ? 'rotate-180' : ''}`} />
      </button>

      {open && (
        <div className="absolute z-dropdown top-full left-0 right-0 mt-1 bg-surface-primary border border-border-default rounded-lg shadow-popover overflow-hidden animate-scale-in max-h-60 overflow-y-auto">
          {options.map(option => (
            <button
              key={option.value}
              type="button"
              onClick={() => {
                onChange(option.value);
                setOpen(false);
              }}
              className={`w-full px-3 py-2 text-left text-sm hover:bg-surface-2 flex flex-col ${
                option.value === value ? 'bg-surface-2' : ''
              }`}
            >
              <span className="text-content-primary">{option.label}</span>
              {option.description && (
                <span className="text-xs text-content-tertiary mt-0.5">{option.description}</span>
              )}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
