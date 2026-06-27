import { useEffect, useRef } from 'react';
import { SPINNER_FRAMES } from '../../constants/spinner';

interface RainColumn {
  y: number;
  speed: number;
  trailLen: number;
  charOffset: number;
  hueOffset: number;
}

const LETTERS = '  O  P  E  N  D  E  V  ';
const LOGO_WIDTH = LETTERS.length; // 23
const LETTER_ROW = 1; // middle row of the 3-row logo
const FRAME_ROW_TOP = 0;
const FRAME_ROW_BOTTOM = 2;
const LOGO_ROWS = 3;
const SUBTITLE = 'AI OPERATING ENVIRONMENT';

interface Hsl { h: number; s: number; l: number; }

function hexToHsl(hex: string): Hsl {
  let h = hex.replace('#', '').trim();
  if (h.length === 3) h = h.split('').map((c) => c + c).join('');
  const r = parseInt(h.slice(0, 2), 16) / 255;
  const g = parseInt(h.slice(2, 4), 16) / 255;
  const b = parseInt(h.slice(4, 6), 16) / 255;
  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const l = (max + min) / 2;
  let hue = 0;
  let sat = 0;
  if (max !== min) {
    const d = max - min;
    sat = l > 0.5 ? d / (2 - max - min) : d / (max + min);
    if (max === r) hue = (g - b) / d + (g < b ? 6 : 0);
    else if (max === g) hue = (b - r) / d + 2;
    else hue = (r - g) / d + 4;
    hue *= 60;
  }
  return { h: isNaN(hue) ? 0 : hue, s: sat * 100, l: l * 100 };
}

function lcg(seed: { value: number }): number {
  seed.value = (seed.value * 1664525 + 1013904223) >>> 0;
  return seed.value / 0xffffffff;
}

const SAFE_FALLBACK_CH = '\u00B7';
const SANS_STACK = '"Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", "SF Pro", system-ui, sans-serif';
const MONO_STACK = '"JetBrains Mono", "Fira Code", "Cascadia Code", "SF Mono", Menlo, Monaco, Consolas, ui-monospace, monospace';

interface MatrixRainProps {
  /** Tuning multiplier on top of the theme's --rain-opacity. Defaults to 1.0. */
  opacity?: number;
  cellSize?: number;
}

interface ThemeTokens {
  accent: string;
  accentHsl: Hsl;
  surfaceElevated: string;
  subtitleHsl: Hsl;
  rainOpacity: number;
}

export function MatrixRain({ opacity = 1, cellSize = 14 }: MatrixRainProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const stateRef = useRef<{
    cols: RainColumn[];
    width: number;
    height: number;
    seed: { value: number };
    raf: number;
    live: boolean;
    frame: number;
  } | null>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const root = document.documentElement;
    const theme: ThemeTokens = {
      accent: '#00F0FF',
      accentHsl: { h: 187, s: 100, l: 50 },
      surfaceElevated: '#12121F',
      subtitleHsl: { h: 187, s: 80, l: 70 },
      rainOpacity: 0.55,
    };

    const refreshTheme = () => {
      const cs = getComputedStyle(root);
      const accent = cs.getPropertyValue('--color-accent-primary').trim();
      const surface = cs.getPropertyValue('--color-surface-elevated').trim();
      const rainOp = cs.getPropertyValue('--rain-opacity').trim();
      if (accent) {
        theme.accent = accent;
        theme.accentHsl = hexToHsl(accent);
      }
      if (surface) theme.surfaceElevated = surface;
      const parsed = parseFloat(rainOp);
      if (!Number.isNaN(parsed)) theme.rainOpacity = parsed;
      // Subtitle uses accent color, not gray secondary
      theme.subtitleHsl = { ...theme.accentHsl, s: Math.min(100, theme.accentHsl.s), l: 70 };
    };

    refreshTheme();
    const themeObserver = new MutationObserver(refreshTheme);
    themeObserver.observe(root, { attributes: true, attributeFilter: ['data-theme'] });

    const seed = { value: (Date.now() & 0xffffffff) || 1 };
    const state = {
      cols: [] as RainColumn[],
      width: 0,
      height: 0,
      seed,
      raf: 0,
      live: true,
      frame: 0,
    };
    stateRef.current = state;

    const dpr = Math.max(1, Math.min(2, window.devicePixelRatio || 1));
    const colWidth = cellSize;
    const TUI_TICK_MS = 60;
    const FRAME_MS = 1000 / 60;
    const TICK_TO_FRAME = TUI_TICK_MS / FRAME_MS;

    const reseed = (w: number, h: number) => {
      const numCols = Math.max(1, Math.floor(w / colWidth));
      state.cols = new Array(numCols);
      for (let i = 0; i < numCols; i++) {
        const y = lcg(state.seed) * h;
        const speed = 0.10 + lcg(state.seed) * 0.40;
        const trailLen = 4 + Math.floor(lcg(state.seed) * 6);
        const charOffset = Math.floor(lcg(state.seed) * SPINNER_FRAMES.length);
        const hueOffset = lcg(state.seed) * 30 - 15;
        state.cols[i] = { y, speed, trailLen, charOffset, hueOffset };
      }
    };

    const resize = () => {
      const rect = canvas.getBoundingClientRect();
      if (rect.width === 0 || rect.height === 0) return;
      state.width = rect.width;
      state.height = rect.height;
      canvas.width = Math.max(1, Math.floor(rect.width * dpr));
      canvas.height = Math.max(1, Math.floor(rect.height * dpr));
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      reseed(rect.width, rect.height);
    };

    const tick = () => {
      if (!state.live) return;
      const { width, height, cols, frame } = state;
      if (width === 0 || height === 0) {
        state.raf = requestAnimationFrame(tick);
        return;
      }

      const effectiveOpacity = theme.rainOpacity * opacity;

      ctx.clearRect(0, 0, width, height);

      // === Animation parameters ===
      const brailleOffset = Math.floor(frame / 2) % SPINNER_FRAMES.length;
      const breathePhase = (frame * Math.PI * 2) / 240;
      const globalHueOffset = Math.sin(frame * 0.0035) * 10;
      const baseHue = theme.accentHsl.h;
      const baseSat = Math.min(100, theme.accentHsl.s);

      // === Layout ===
      const totalCols = Math.floor(width / cellSize);
      const totalRows = Math.floor(height / cellSize);
      const canShowLogo = totalCols >= LOGO_WIDTH && totalRows >= LOGO_ROWS + 6;
      const logoStartCol = canShowLogo ? Math.floor((totalCols - LOGO_WIDTH) / 2) : 0;
      const logoStartRow = canShowLogo ? Math.max(2, Math.floor(totalRows * 0.20) - 1) : 0;

      const titlePadX = cellSize * 0.4;
      const titlePadY = cellSize * 0.4;
      const titleX = logoStartCol * cellSize - titlePadX;
      const titleY = logoStartRow * cellSize - titlePadY;
      const titleW = LOGO_WIDTH * cellSize + 2 * titlePadX;
      const titleH = LOGO_ROWS * cellSize + 2 * titlePadY;
      const titleRadius = 6;

      const inLogoZone = (col: number, row: number) =>
        canShowLogo &&
        row >= logoStartRow && row < logoStartRow + LOGO_ROWS &&
        col >= logoStartCol && col < logoStartCol + LOGO_WIDTH;

      const logoCharAt = (col: number, row: number): string => {
        if (!inLogoZone(col, row)) return ' ';
        if (row - logoStartRow !== LETTER_ROW) return ' '; // frame rows are not rain
        const lc = col - logoStartCol;
        return LETTERS[lc] || ' ';
      };

      // === Rain (skips logo cells — negative space) ===
      // Skip drawing entirely when the theme says no rain.
      const drawRain = effectiveOpacity > 0;
      if (drawRain) {
        ctx.font = `${cellSize}px ${MONO_STACK}`;
        ctx.textBaseline = 'top';

        for (let c = 0; c < cols.length; c++) {
          const col = cols[c];
          const x = c * colWidth;
          const hue = baseHue + globalHueOffset + (col.hueOffset || 0);

          for (let r = 0; r < col.trailLen; r++) {
            const y = col.y - r * cellSize;
            if (y < -cellSize || y > height) continue;

            const rowIdx = Math.floor(y / cellSize);
            if (logoCharAt(c, rowIdx) !== ' ') continue;

            let ch: string;
            let lightness: number;
            let alpha: number;

            if (r === 0) {
              ch = '\u2593';
              lightness = 65;
              alpha = 0.85 * effectiveOpacity;
            } else if (r === 1) {
              ch = '\u2591';
              lightness = 50;
              alpha = 0.55 * effectiveOpacity;
            } else {
              const t = r / col.trailLen;
              const idx = ((col.charOffset | 0) + r + brailleOffset) % SPINNER_FRAMES.length;
              ch = SPINNER_FRAMES[idx] || SAFE_FALLBACK_CH;
              lightness = 28 + 22 * (1 - t);
              alpha = (0.40 * (1 - t) + 0.08 * t) * effectiveOpacity;
            }

            ctx.fillStyle = `hsla(${hue}, ${baseSat}%, ${lightness}%, ${alpha})`;
            ctx.fillText(ch, x, y);
          }

          col.y += (col.speed / TICK_TO_FRAME) * cellSize;
          if (col.y - col.trailLen * cellSize > height) {
            col.y = -lcg(state.seed) * cellSize * 6;
            col.speed = 0.10 + lcg(state.seed) * 0.40;
          }
        }
      } else {
        // Even when rain is off, advance the column state so toggling rain on
        // later doesn't reveal columns frozen at y=0.
        for (let c = 0; c < cols.length; c++) {
          const col = cols[c];
          col.y += (col.speed / TICK_TO_FRAME) * cellSize;
          if (col.y - col.trailLen * cellSize > height) {
            col.y = -lcg(state.seed) * cellSize * 6;
            col.speed = 0.10 + lcg(state.seed) * 0.40;
          }
        }
      }

      // === Title background: opaque surface-elevated panel (blocks rain) ===
      if (canShowLogo) {
        ctx.fillStyle = theme.surfaceElevated;
        ctx.beginPath();
        if (typeof (ctx as any).roundRect === 'function') {
          (ctx as any).roundRect(titleX, titleY, titleW, titleH, titleRadius);
        } else {
          const r = titleRadius;
          const x = titleX, y = titleY, w = titleW, h = titleH;
          ctx.moveTo(x + r, y);
          ctx.lineTo(x + w - r, y);
          ctx.arcTo(x + w, y, x + w, y + r, r);
          ctx.lineTo(x + w, y + h - r);
          ctx.arcTo(x + w, y + h, x + w - r, y + h, r);
          ctx.lineTo(x + r, y + h);
          ctx.arcTo(x, y + h, x, y + h - r, r);
          ctx.lineTo(x, y + r);
          ctx.arcTo(x, y, x + r, y, r);
          ctx.closePath();
        }
        ctx.fill();
      }

      // === Frame lines: drawn as fillRect (no font dependency) ===
      if (canShowLogo) {
        const frameLightness = Math.min(75, theme.accentHsl.l + 5);
        const frameAlpha = 0.6;
        ctx.fillStyle = `hsla(${baseHue}, ${baseSat * 0.6}%, ${frameLightness}%, ${frameAlpha})`;
        const frameH = Math.max(1.5, cellSize * 0.1);
        const topY = (logoStartRow + FRAME_ROW_TOP) * cellSize + cellSize * 0.5 - frameH / 2;
        const botY = (logoStartRow + FRAME_ROW_BOTTOM) * cellSize + cellSize * 0.5 - frameH / 2;
        ctx.fillRect(logoStartCol * cellSize, topY, LOGO_WIDTH * cellSize, frameH);
        ctx.fillRect(logoStartCol * cellSize, botY, LOGO_WIDTH * cellSize, frameH);
      }

      // === Letters: sans-serif, breathing, per-letter hue shift ===
      if (canShowLogo) {
        const letterFontSize = Math.floor(cellSize * 1.5);
        ctx.font = `600 ${letterFontSize}px ${SANS_STACK}`;
        ctx.textBaseline = 'top';
        const letterRowY = (logoStartRow + LETTER_ROW) * cellSize + (cellSize - letterFontSize) * 0.5;
        for (let lc = 0; lc < LOGO_WIDTH; lc++) {
          const ch = LETTERS[lc];
          if (ch === ' ') continue;
          const x = (logoStartCol + lc) * cellSize;
          const letterT = lc / LOGO_WIDTH;
          const letterHue = baseHue + (letterT - 0.5) * 30;
          const breathe = 0.60 + 0.30 * (0.5 + 0.5 * Math.sin(breathePhase));
          ctx.fillStyle = `hsla(${letterHue}, ${baseSat}%, ${breathe * 78}%, 0.96)`;
          ctx.fillText(ch, x, letterRowY);
        }
      }

      // === Subtitle: sans-serif with letter spacing ===
      if (canShowLogo) {
        const subtitleFontSize = Math.max(12, Math.floor(cellSize * 0.9));
        ctx.font = `500 ${subtitleFontSize}px ${SANS_STACK}`;
        try { (ctx as any).letterSpacing = '2.5px'; } catch {}
        const subtitleWidth = ctx.measureText(SUBTITLE).width;
        const subtitleX = (width - subtitleWidth) / 2;
        const subtitleY = titleY + titleH + cellSize * 1.0;
        const subHsl = theme.subtitleHsl;
        const subBreathe = 0.70 + 0.22 * (0.5 + 0.5 * Math.sin(breathePhase + Math.PI * 0.4));
        ctx.fillStyle = `hsla(${subHsl.h}, ${subHsl.s}%, ${subHsl.l}%, ${subBreathe * 0.95})`;
        ctx.fillText(SUBTITLE, subtitleX, subtitleY);
        try { (ctx as any).letterSpacing = '0px'; } catch {}
      }

      state.frame++;
      state.raf = requestAnimationFrame(tick);
    };

    resize();
    const ro = new ResizeObserver(resize);
    ro.observe(canvas);
    state.raf = requestAnimationFrame(tick);

    return () => {
      state.live = false;
      cancelAnimationFrame(state.raf);
      ro.disconnect();
      themeObserver.disconnect();
    };
  }, [cellSize, opacity]);

  return (
    <canvas
      ref={canvasRef}
      aria-hidden
      className="absolute inset-0 w-full h-full pointer-events-none"
    />
  );
}
