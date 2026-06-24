import { Link } from 'react-router-dom';
import { ChevronRight } from 'lucide-react';

interface BreadcrumbItem {
  label: string;
  path?: string;
}

interface BreadcrumbProps {
  items: BreadcrumbItem[];
}

export function Breadcrumb({ items }: BreadcrumbProps) {
  return (
    <nav className="h-10 bg-surface-elevated border-b border-border-default">
      <div className="h-full max-w-[1400px] mx-auto px-6 flex items-center">
        <ol className="flex items-center gap-2 text-sm">
          {items.map((item, index) => (
            <li key={index} className="flex items-center gap-2">
              {index > 0 && (
                <ChevronRight className="w-3 h-3 text-content-tertiary" />
              )}
              {item.path ? (
                <Link
                  to={item.path}
                  className="text-content-secondary hover:text-content-primary transition-colors"
                >
                  {item.label}
                </Link>
              ) : (
                <span className="text-content-primary font-medium">{item.label}</span>
              )}
            </li>
          ))}
        </ol>
      </div>
    </nav>
  );
}