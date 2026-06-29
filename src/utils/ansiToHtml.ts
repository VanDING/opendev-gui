/**
 * Converts ANSI escape code sequences (colors, bold, dim, italic, underline)
 * to styled HTML span elements with inline CSS.
 *
 * Supports:
 * - SGR colors: 8/16 standard, 256-color palette, truecolor (24-bit)
 * - Text styles: bold, dim, italic, underline, strikethrough
 * - Cursor movement and screen clearing (filtered out)
 */

interface AnsiState {
  bold: boolean;
  dim: boolean;
  italic: boolean;
  underline: boolean;
  strikethrough: boolean;
  fg: string | null;
  bg: string | null;
}

const EMPTY_STATE: AnsiState = {
  bold: false, dim: false, italic: false,
  underline: false, strikethrough: false,
  fg: null, bg: null,
};

/** Standard 16-color ANSI palette */
const ANSI_COLORS: Record<number, string> = {
  0: '#000000', 1: '#AA0000', 2: '#00AA00', 3: '#AA5500',
  4: '#0000AA', 5: '#AA00AA', 6: '#00AAAA', 7: '#AAAAAA',
  8: '#555555', 9: '#FF5555', 10: '#55FF55', 11: '#FFFF55',
  12: '#5555FF', 13: '#FF55FF', 14: '#55FFFF', 15: '#FFFFFF',
};

function get256Color(code: number): string {
  if (code < 16) return ANSI_COLORS[code] || '#000000';
  if (code < 232) {
    const idx = code - 16;
    const r = Math.round((idx / 36) % 6 * 51);
    const g = Math.round((idx / 6) % 6 * 51);
    const b = Math.round(idx % 6 * 51);
    return `rgb(${r},${g},${b})`;
  }
  const gray = Math.round((code - 232) * 10.2);
  return `rgb(${gray},${gray},${gray})`;
}

function parseSgrParams(params: number[]): Partial<AnsiState> {
  const state: Partial<AnsiState> = {};
  let i = 0;
  while (i < params.length) {
    const code = params[i];
    switch (code) {
      case 0: return { ...EMPTY_STATE };
      case 1: state.bold = true; break;
      case 2: state.dim = true; break;
      case 3: state.italic = true; break;
      case 4: state.underline = true; break;
      case 9: state.strikethrough = true; break;
      case 22: state.bold = false; state.dim = false; break;
      case 23: state.italic = false; break;
      case 24: state.underline = false; break;
      case 29: state.strikethrough = false; break;
      case 30: case 31: case 32: case 33:
      case 34: case 35: case 36: case 37:
        state.fg = ANSI_COLORS[code - 30]; break;
      case 38: { // 256/truecolor foreground
        const extType = params[i + 1];
        if (extType === 5) { state.fg = get256Color(params[i + 2]); i += 2; }
        else if (extType === 2) {
          state.fg = `rgb(${params[i + 2]},${params[i + 3]},${params[i + 4]})`;
          i += 4;
        }
        break;
      }
      case 39: state.fg = null; break;
      case 40: case 41: case 42: case 43:
      case 44: case 45: case 46: case 47:
        state.bg = ANSI_COLORS[code - 40]; break;
      case 48: { // 256/truecolor background
        const extType = params[i + 1];
        if (extType === 5) { state.bg = get256Color(params[i + 2]); i += 2; }
        else if (extType === 2) {
          state.bg = `rgb(${params[i + 2]},${params[i + 3]},${params[i + 4]})`;
          i += 4;
        }
        break;
      }
      case 49: state.bg = null; break;
      default: break;
    }
    i++;
  }
  return state;
}

function stateToStyle(s: AnsiState): React.CSSProperties {
  const style: React.CSSProperties = {};
  if (s.bold) style.fontWeight = 'bold';
  if (s.dim) style.opacity = 0.7;
  if (s.italic) style.fontStyle = 'italic';
  if (s.underline) style.textDecoration = 'underline';
  if (s.strikethrough) style.textDecoration = 'line-through';
  if (s.fg) style.color = s.fg;
  if (s.bg) style.backgroundColor = s.bg;
  return style;
}

/** ANSI escape sequence regex */
const ANSI_RE = /(\x1b\[([\d;]*)m|\x1b\[[\d;]*[ABCDHKJ]|\x1b\][^\x1b]*\x1b\\|\x1b[^\[].)/g;

/** Parse ANSI params from a semicolon-separated string */
function parseParams(paramStr: string): number[] {
  if (!paramStr) return [0];
  return paramStr.split(';').filter(Boolean).map(Number);
}

/**
 * Convert ANSI-escaped text to an array of { text, style } segments.
 */
export function parseAnsi(text: string): Array<{ text: string; style: React.CSSProperties }> {
  const segments: Array<{ text: string; style: React.CSSProperties }> = [];
  let lastIndex = 0;
  let state: AnsiState = { ...EMPTY_STATE };

  text.replace(ANSI_RE, (match, _full, paramStr, offset) => {
    // Push text before the escape
    if (offset > lastIndex) {
      segments.push({ text: text.slice(lastIndex, offset), style: stateToStyle(state) });
    }
    lastIndex = offset + match.length;

    if (paramStr !== undefined) {
      // SGR sequence
      const params = parseParams(paramStr);
      const update = parseSgrParams(params);
      state = { ...state, ...update };
    }
    // Other escape sequences (cursor movement, screen clearing) are filtered out
    return '';
  });

  // Push remaining text
  if (lastIndex < text.length) {
    segments.push({ text: text.slice(lastIndex), style: stateToStyle(state) });
  }

  return segments;
}

/**
 * Convert ANSI-escaped text to an HTML string with inline styles.
 */
export function ansiToHtml(text: string): string {
  const segments = parseAnsi(text);
  return segments.map(seg => {
    const keys = Object.keys(seg.style);
    if (keys.length === 0) return escapeHtml(seg.text);
    const styleStr = keys
      .map(k => `${k.replace(/[A-Z]/g, c => '-' + c.toLowerCase())}: ${(seg.style as any)[k]}`)
      .join('; ');
    return `<span style="${styleStr}">${escapeHtml(seg.text)}</span>`;
  }).join('');
}

function escapeHtml(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}
