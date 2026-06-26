import { createContext, useContext, useState, useEffect, type ReactNode } from 'react';

type Theme = 'cyberpunk' | 'dark-default' | 'light-default' | 'warm' | 'polar' | 'geek'
  | 'afrofuturism' | 'sumi-e' | 'synthwave'
  | 'techno' | 'brutalism' | 'pixel-by';

const ALL_THEMES: Theme[] = [
  'cyberpunk', 'dark-default', 'light-default', 'warm', 'polar', 'geek',
  'afrofuturism', 'sumi-e', 'synthwave',
  'techno', 'brutalism', 'pixel-by',
];

const THEME_KEY = 'opendev-theme';

function getStoredTheme(): Theme {
  const stored = localStorage.getItem(THEME_KEY);
  if (stored && (ALL_THEMES as readonly string[]).includes(stored)) return stored as Theme;
  if (window.matchMedia('(prefers-color-scheme: light)').matches) return 'light-default';
  return 'cyberpunk';
}

function applyTheme(theme: Theme) {
  document.documentElement.dataset.theme = theme;
  localStorage.setItem(THEME_KEY, theme);
}

const ThemeContext = createContext<{ theme: Theme; setTheme: (t: Theme) => void }>(null!);

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setTheme] = useState<Theme>(getStoredTheme);

  useEffect(() => {
    applyTheme(theme);
  }, [theme]);

  return <ThemeContext value={{ theme, setTheme }}>{children}</ThemeContext>;
}

export const useTheme = () => useContext(ThemeContext);
export { ALL_THEMES };
export type { Theme };
