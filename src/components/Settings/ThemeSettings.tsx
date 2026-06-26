import { useTheme, ALL_THEMES } from '../../contexts/ThemeContext';
import type { Theme } from '../../contexts/ThemeContext';

const THEME_PREVIEW_COLORS: Record<Theme, { surface: string; accent: string; content: string }> = {
  'cyberpunk':     { surface: '#080C18', accent: '#00E5D5', content: '#E0E4EE' },
  'dark-default':  { surface: '#141414', accent: '#FFFFFF', content: '#EDEDED' },
  'light-default': { surface: '#FFFFFF', accent: '#000000', content: '#17181A' },
  'warm':          { surface: '#FAF6F0', accent: '#CC785C', content: '#3D3929' },
  'polar':         { surface: 'rgba(255,255,255,0.90)', accent: '#00E5A0', content: '#0A1A2A' },
  'geek':          { surface: '#0D1117', accent: '#33FF33', content: '#E8ECEA' },
  'afrofuturism':  { surface: '#2A2E45', accent: '#9B4DCA', content: '#E5C687' },
  'sumi-e':        { surface: '#F7F5F0', accent: '#2B2B2B', content: '#2B2B2B' },
  'synthwave':     { surface: '#1A1530', accent: '#FF9F4B', content: '#FF6EC7' },
  'techno':        { surface: '#1C1C1C', accent: '#F4C542', content: '#E0E0E0' },
  'brutalism':     { surface: '#FFFFFF', accent: '#0000FF', content: '#000000' },
  'pixel-by':      { surface: '#F8F4EC', accent: '#FFCE4E', content: '#2D2640' },
};

const THEME_LABELS: Record<Theme, string> = {
  'cyberpunk':     'Cyberpunk',
  'dark-default':  'Dark',
  'light-default': 'Light',
  'warm':          'Warm',
  'polar':         'Polar',
  'geek':          'Geek',
  'afrofuturism':  'Afrofuturism',
  'sumi-e':        'Sumi-e',
  'synthwave':     'Synthwave',
  'techno':        'Techno',
  'brutalism':     'Brutalism',
  'pixel-by':      'Pixel',
};

export function ThemeSettings() {
  const { theme: currentTheme, setTheme } = useTheme();

  return (
    <div>
      <h3 className="text-sm font-semibold text-content-primary mb-1">Theme</h3>
      <p className="text-xs text-content-tertiary mb-5">
        Click a theme to preview and apply it instantly.
      </p>

      <div className="grid grid-cols-3 gap-3">
        {ALL_THEMES.map(t => {
          const isActive = t === currentTheme;
          const colors = THEME_PREVIEW_COLORS[t];

          return (
            <button
              key={t}
              onClick={() => setTheme(t)}
              className={`relative p-3 rounded-xl border-2 text-left transition-all ${
                isActive
                  ? 'border-accent-primary shadow-md'
                  : 'border-transparent hover:border-border-default'
              }`}
              style={{ background: 'var(--color-surface-elevated)' }}
            >
              {isActive && (
                <div
                  className="absolute top-2 right-2 w-2 h-2 rounded-full"
                  style={{ background: 'var(--color-accent-primary)' }}
                />
              )}

              <div className="text-xs font-semibold mb-1.5" style={{ color: 'var(--color-content-primary)' }}>
                {THEME_LABELS[t]}
              </div>

              <div
                className="overflow-hidden rounded-lg p-2"
                style={{
                  background: colors.surface,
                  borderRadius: 'var(--radius-md)',
                }}
              >
                <div
                  className="flex items-center gap-1 px-1.5 py-0.5 mb-1.5 rounded-sm"
                  style={{
                    background: colors.surface,
                    border: `1px solid var(--color-border-default)`,
                  }}
                >
                  <span className="text-[8px] font-semibold" style={{ color: colors.content }}>
                    opendev
                  </span>
                </div>

                <div className="flex gap-1 mb-1.5">
                  <span
                    className="px-1.5 py-0.5 text-[7px] font-medium rounded-sm"
                    style={{ background: colors.accent, color: colors.surface }}
                  >
                    Chat
                  </span>
                  <span
                    className="px-1.5 py-0.5 text-[7px] rounded-sm"
                    style={{ background: '#1F1F1F', color: colors.content, border: '1px solid' }}
                  >
                    Files
                  </span>
                </div>

                <div
                  className="px-1.5 py-1 mb-1.5 rounded-md"
                  style={{ background: colors.surface, border: '1px solid' }}
                >
                  <div className="text-[8px] font-semibold" style={{ color: colors.content }}>
                    Project
                  </div>
                  <div className="text-[7px]" style={{ color: colors.content }}>
                    3 files
                  </div>
                </div>

                <div className="flex gap-1">
                  <span className="px-1 py-[1px] text-[6px] rounded-sm" style={{ background: 'rgba(0,200,150,0.15)', color: '#5EC269' }}>
                    active
                  </span>
                  <span className="px-1 py-[1px] text-[6px] rounded-sm" style={{ background: 'rgba(0,150,255,0.12)', color: '#60A5FA' }}>
                    done
                  </span>
                </div>
              </div>
            </button>
          );
        })}
      </div>
    </div>
  );
}
