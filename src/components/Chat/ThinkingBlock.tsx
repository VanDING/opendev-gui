import { useState, useRef, useEffect } from 'react';

interface ThinkingBlockProps {
  content: string;
  level?: string;
  isActive?: boolean;
}

const LEVEL_COLORS: Record<string, string> = {
  Low: 'bg-intent-success-muted text-intent-success-fg',
  Medium: 'bg-intent-info-muted text-intent-info',
  High: 'bg-intent-purple-muted text-intent-purple-fg',
};

export function ThinkingBlock({ content, level, isActive }: ThinkingBlockProps) {
  const [isExpanded, setIsExpanded] = useState(false);
  const contentRef = useRef<HTMLDivElement>(null);
  const [contentHeight, setContentHeight] = useState(0);

  const isCritique = content.startsWith('[Critique]');
  const accentColor = isCritique ? 'border-l-intent-warning/70' : 'border-l-intent-purple/70';
  const levelBadge = level && LEVEL_COLORS[level] ? level : null;

  // ⚡ Bolt Performance Optimization:
  // Avoid synchronous layout reflows from scrollHeight measurements
  // when the component is collapsed. By checking `isExpanded` before measuring,
  // we prevent unnecessary layout recalculations and subsequent state updates
  // during streaming updates to the `content` prop while the block is hidden.
  useEffect(() => {
    if (isExpanded && contentRef.current) {
      setContentHeight(contentRef.current.scrollHeight);
    }
  }, [isExpanded, content]);

  return (
    <div className="animate-slide-up">
      <div className={`border-l-2 ${accentColor} rounded-r-lg overflow-hidden bg-surface-elevated/50`}>
        {/* Header */}
        <button
          onClick={() => setIsExpanded(!isExpanded)}
          className={`w-full px-3 py-2 flex items-center gap-2 text-left hover:bg-surface-2/50 transition-colors cursor-pointer ${isActive ? 'thinking-shimmer' : ''}`}
        >
          {/* Brain icon */}
          <svg
            className={`w-3.5 h-3.5 flex-shrink-0 transition-colors ${isCritique ? 'text-intent-warning-muted/70' : 'text-intent-purple-muted/70'}`}
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            strokeWidth={1.5}
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              d="M9.75 3.104v5.714a2.25 2.25 0 0 1-.659 1.591L5 14.5M9.75 3.104c-.251.023-.501.05-.75.082m.75-.082a24.301 24.301 0 0 1 4.5 0m0 0v5.714a2.25 2.25 0 0 0 .659 1.591L19 14.5m-4.75-11.396c.251.023.501.05.75.082M12 3c2.5 0 5 .5 7 1.5M12 3c-2.5 0-5 .5-7 1.5m14 0v3m-14-3v3"
            />
          </svg>

          <span className="text-xs font-medium text-content-tertiary uppercase tracking-wide">
            {isCritique ? 'Critique' : 'Thinking'}
          </span>

          {/* Level badge */}
          {levelBadge && (
            <span className={`text-[10px] font-medium px-1.5 py-0.5 rounded-full ${LEVEL_COLORS[levelBadge]}`}>
              {levelBadge}
            </span>
          )}

          {/* Chevron */}
          <svg
            className={`w-3 h-3 text-content-tertiary transition-transform duration-200 flex-shrink-0 ml-auto ${
              isExpanded ? 'rotate-90' : ''
            }`}
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
          </svg>
        </button>

        {/* Content — always rendered, clipped when collapsed */}
        <div className="relative">
          <div
            className="overflow-hidden transition-all duration-300 ease-in-out"
            style={{
              maxHeight: isExpanded ? `${contentHeight + 24}px` : '48px',
            }}
          >
            <div ref={contentRef} className="px-3 pb-3">
              <pre className="text-xs text-content-secondary whitespace-pre-wrap font-mono leading-relaxed">
                {content}
              </pre>
            </div>
          </div>
          {/* Gradient fade when collapsed */}
          <div
            className={`absolute bottom-0 left-0 right-0 h-8 bg-gradient-to-t from-surface-elevated/80 to-transparent pointer-events-none transition-opacity duration-300 ${
              isExpanded ? 'opacity-0' : 'opacity-100'
            }`}
          />
        </div>
      </div>
    </div>
  );
}
