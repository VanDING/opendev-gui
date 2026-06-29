import React, { useMemo } from 'react';
import { ansiToHtml } from '../../utils/ansiToHtml';

interface AnsiOutputProps {
  text: string;
  className?: string;
}

/**
 * Renders text with ANSI escape codes as styled HTML.
 *
 * Uses `dangerouslySetInnerHTML` with sanitized output from `ansiToHtml`.
 * The input is parsed for SGR escape sequences and converted to inline styles.
 * All other ANSI sequences (cursor movement, screen clearing) are filtered out.
 */
export const AnsiOutput: React.FC<AnsiOutputProps> = React.memo(({ text, className }) => {
  const html = useMemo(() => ansiToHtml(text), [text]);

  return (
    <span
      className={className}
      dangerouslySetInnerHTML={{ __html: html }}
      style={{ whiteSpace: 'pre-wrap', fontFamily: 'monospace' }}
    />
  );
});

AnsiOutput.displayName = 'AnsiOutput';
