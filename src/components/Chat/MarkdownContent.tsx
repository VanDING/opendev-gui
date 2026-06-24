import { createHighlighterCore, type HighlighterCore } from 'shiki/core';
import { createJavaScriptRegexEngine } from 'shiki/engine/javascript';
import MarkdownIt from 'markdown-it';

let darkHighlighter: HighlighterCore | null = null;
let lightHighlighter: HighlighterCore | null = null;
let highlighterPromise: Promise<void> | null = null;

function getActiveHighlighter(): HighlighterCore | null {
  const theme = document.documentElement.dataset.theme;
  return theme === 'light-default' ? lightHighlighter : darkHighlighter;
}

function ensureHighlighter(): Promise<void> {
  if (highlighterPromise) return highlighterPromise;
  const jsEngine = createJavaScriptRegexEngine();
  const langImports = [
    import('@shikijs/langs/typescript'),
    import('@shikijs/langs/javascript'),
    import('@shikijs/langs/tsx'),
    import('@shikijs/langs/jsx'),
    import('@shikijs/langs/python'),
    import('@shikijs/langs/rust'),
    import('@shikijs/langs/bash'),
    import('@shikijs/langs/shell'),
    import('@shikijs/langs/json'),
    import('@shikijs/langs/yaml'),
    import('@shikijs/langs/toml'),
    import('@shikijs/langs/html'),
    import('@shikijs/langs/css'),
    import('@shikijs/langs/scss'),
    import('@shikijs/langs/sql'),
    import('@shikijs/langs/markdown'),
    import('@shikijs/langs/go'),
    import('@shikijs/langs/java'),
    import('@shikijs/langs/diff'),
  ];
  highlighterPromise = Promise.all([
    createHighlighterCore({ themes: [import('@shikijs/themes/dark-plus')], langs: langImports, engine: jsEngine }),
    createHighlighterCore({ themes: [import('@shikijs/themes/light-plus')], langs: langImports, engine: jsEngine }),
  ]).then(([dark, light]) => {
    darkHighlighter = dark;
    lightHighlighter = light;
  });
  return highlighterPromise;
}

ensureHighlighter();

function escapeHtml(code: string): string {
  return code
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');
}

const md = new MarkdownIt({
  html: true,
  breaks: true,
  linkify: true,
  highlight(code: string, lang: string): string {
    const safeLang = lang || 'text';
    const hl = getActiveHighlighter();
    if (hl?.getLoadedLanguages().includes(safeLang as any)) {
      const themeName = document.documentElement.dataset.theme === 'light-default' ? 'light-plus' : 'dark-plus';
      return hl.codeToHtml(code, { lang: safeLang, theme: themeName });
    }
    return `<pre class="shiki-code-block"><code>${escapeHtml(code)}</code></pre>`;
  },
});

interface MarkdownContentProps {
  content: string;
  className?: string;
}

export function MarkdownContent({ content, className }: MarkdownContentProps) {
  const html = md.render(content);

  return (
    <div
      className={className}
      dangerouslySetInnerHTML={{ __html: html }}
    />
  );
}
