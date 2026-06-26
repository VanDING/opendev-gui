---
version: 1.0
name: OpenDev Desktop
description: >
  A developer-first AI assistant desktop application. Dark-first,
  precision-driven design language. Near-pure black canvas with white/gray
  accent system. Surface ladder replaces drop shadows for clean depth.
  Inter for body/display, JetBrains Mono for code. Compact 8px component
  radius with 12px container radius.
---

## 1. Visual Theme & Atmosphere

OpenDev Desktop is a **professional developer tool** — the interface should feel precise,
calm, and quietly powerful. No decorative fluff. No atmospheric gradients. The dark
canvas does the heavy lifting; the surface ladder carries hierarchy without shadows.

**Key characteristics:**
- Near-pure black canvas (`#010102`) — deeper than typical "dark mode"
- White/gray accent system — no chromatic brand color; the UI itself is the brand
- Hairline-border depth — surface color steps, not drop shadows
- Inter at weights 400/500/600 with negative tracking on headings
- JetBrains Mono on every code surface
- Generous 80px section rhythm with 20-24px card padding
- Unified radius: 8px interactive, 12px containers, 16px modals

---

## 2. Color Palette

### Canvas & Surfaces

| Token | Value | Use |
|-------|-------|-----|
| `--color-canvas` | `#010102` | Page background — deepest surface |
| `--color-surface-1` | `#0b0c0d` | Cards, panels, sidebar backgrounds |
| `--color-surface-2` | `#121315` | Elevated cards, hovered surfaces, modal bg |
| `--color-surface-3` | `#191a1c` | Dropdowns, popovers, tooltips |
| `--color-surface-4` | `#1e1f21` | Highest elevation (rare) |
| `--color-surface-input` | `#0b0c0d` | Text input, select, textarea backgrounds |
| `--color-surface-overlay` | `rgba(0, 0, 0, 0.60)` | Modal backdrop scrim |

### Text

| Token | Value | Use |
|-------|-------|-----|
| `--color-ink` | `#eeeff0` | Headlines, emphasized body, primary text |
| `--color-ink-secondary` | `#a8abb0` | Default body text, descriptions |
| `--color-ink-tertiary` | `#696c73` | Metadata, captions, placeholder text |
| `--color-ink-disabled` | `#3d3f44` | Disabled text |
| `--color-ink-inverse` | `#010102` | Text on white accent (e.g., primary button label) |
| `--color-ink-link` | `#8ba8d0` | Hyperlinks, subtle blue tint |

### Borders (Hairlines)

| Token | Value | Use |
|-------|-------|-----|
| `--color-hairline` | `rgba(255, 255, 255, 0.06)` | Default card/panel borders |
| `--color-hairline-strong` | `rgba(255, 255, 255, 0.10)` | Modal borders, focused rings |
| `--color-hairline-emphasis` | `rgba(255, 255, 255, 0.16)` | Input focus borders, active states |

### Accent (White/Gray System)

| Token | Value | Use |
|-------|-------|-----|
| `--color-accent` | `#ffffff` | Primary CTA fill, active indicators |
| `--color-accent-hover` | `#e0e0e0` | Primary button hover |
| `--color-accent-active` | `#c8c8c8` | Primary button press |
| `--color-accent-muted` | `rgba(255, 255, 255, 0.08)` | Accent tint backgrounds |
| `--color-accent-fg` | `#ffffff` | Accent foreground (icons on dark) |

### Semantic

| Token | Value | Use |
|-------|-------|-----|
| `--color-success` | `#27a644` | Success indicators, connected status |
| `--color-success-muted` | `rgba(39, 166, 68, 0.12)` | Success backgrounds |
| `--color-danger` | `#e5484d` | Error, destructive actions |
| `--color-danger-muted` | `rgba(229, 72, 77, 0.12)` | Error backgrounds |
| `--color-warning` | `#f5a623` | Warnings, caution |
| `--color-warning-muted` | `rgba(245, 166, 35, 0.12)` | Warning backgrounds |
| `--color-info` | `#54a0ff` | Informational highlights |
| `--color-info-muted` | `rgba(84, 160, 255, 0.12)` | Info backgrounds |

### Shadow (Minimal — Only for Modals/Popovers)

| Token | Value | Use |
|-------|-------|-----|
| `--shadow-none` | `none` | Default — everything flat |
| `--shadow-popover` | `0px 8px 24px rgba(0, 0, 0, 0.45)` | Modals, dropdowns only |

---

## 3. Typography

### Font Families

| Role | Font | Fallback |
|------|------|----------|
| Display & Body | **Inter** | `-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif` |
| Code & Mono | **JetBrains Mono** | `"Fira Code", "Cascadia Code", "Consolas", monospace` |

### Type Scale

| Token | Size | Weight | Line Height | Letter Spacing | Use |
|-------|------|--------|-------------|----------------|-----|
| `text-display` | 28px | 600 | 1.2 | -0.6px | Page hero, landing |
| `text-title-lg` | 22px | 600 | 1.25 | -0.4px | Section headers |
| `text-title` | 18px | 600 | 1.3 | -0.2px | Card titles, modal titles |
| `text-subhead` | 16px | 500 | 1.4 | 0 | Lead text, emphasized body |
| `text-body` | 14px | 400 | 1.5 | 0 | Default body text |
| `text-body-sm` | 13px | 400 | 1.45 | 0 | Descriptions, secondary |
| `text-caption` | 12px | 400 | 1.4 | 0 | Metadata, labels, footnotes |
| `text-caption-xs` | 11px | 500 | 1.3 | 0.3px | Uppercase labels, badges |
| `text-button` | 14px | 500 | 1.2 | 0 | All button labels |
| `text-code` | 13px | 400 | 1.5 | 0 | Code blocks — JetBrains Mono |

### Principles

- Headings at weight 600 with negative letter-spacing; body at 400 with neutral spacing
- Never use weight 700+ — it's too heavy for a developer tool
- Code must ALWAYS use JetBrains Mono — never the body font
- `--font-sans` and `--font-mono` CSS variables control the global switch

### Font Loading

Inter and JetBrains Mono should be self-hosted (bundled in the Tauri app).
For development preview via Vite, load from Google Fonts CDN.

---

## 4. Elevation & Depth

**Rule: No drop shadows on containers.** Depth comes from surface color steps + hairlines.

| Level | Surface | Border | Use |
|-------|---------|--------|-----|
| 0 — Canvas | `canvas` `#010102` | none | Page background, body areas |
| 1 — Card | `surface-1` `#0b0c0d` | 1px `hairline` | Cards, sidebar, input backgrounds |
| 2 — Elevated | `surface-2` `#121315` | 1px `hairline-strong` | Modal content, hovered cards |
| 3 — Popover | `surface-3` `#191a1c` | 1px `hairline-strong` + `shadow-popover` | Dropdowns, tooltips |
| 4 — Peak | `surface-4` `#1e1f21` | 1px `hairline-emphasis` + `shadow-popover` | Highest priority overlays (rare) |

Only Level 3 and 4 get a shadow, and only `shadow-popover` — a single 24px blur, no multi-layer stacking.

---

## 5. Border Radius

Unified across all themes. No per-theme radius variation.

| Token | Value | Use |
|-------|-------|-----|
| `--radius-sm` | 4px | Inline tags, status badges, small chips |
| `--radius-md` | 8px | **All buttons, all inputs, all form elements** |
| `--radius-lg` | 12px | Cards, panels, sidebars, table containers |
| `--radius-xl` | 16px | Modals, dialogs, large containers |
| `--radius-full` | 9999px | Pills, avatars, toggle switches |

**Critical rule:** Interactive elements = 8px. Containers = 12px. Modals = 16px.
Never mix. Never use 0px (sharp corners) in the default theme.

---

## 6. Spacing & Layout

### Base Unit: 4px

| Token | Value | Use |
|-------|-------|-----|
| `--space-1` | 4px | Tight inline gaps |
| `--space-2` | 8px | Icon+label gaps, list item gaps |
| `--space-3` | 12px | Component internal gaps |
| `--space-4` | 16px | Standard element spacing |
| `--space-5` | 20px | Card interior padding |
| `--space-6` | 24px | Section padding, generous card padding |
| `--space-8` | 32px | Large section gaps |
| `--space-12` | 48px | Major section separation |
| `--space-section` | 80px | Page-level section rhythm |

### Layout Principles

- **Section rhythm:** 80px vertical between major page sections
- **Card padding:** 20px (24px for feature cards)
- **Sidebar:** 240px default width, collapsible to 48px (icons only)
- **Content max-width:** 960px for reading, fluid for tools
- **Modal sizing:** 80vw × 80vh for settings, 560px max-width for confirmations

### Whitespace Philosophy

Dark canvas IS the whitespace. Sections breathe with `--space-section` (80px) gaps.
Within cards, 20-24px padding gives content room. Buttons and inputs use generous
internal padding (8-10px vertical, 16px horizontal).

---

## 7. Components

### Button

**`button-primary`** — White CTA. The strongest action on screen.
- `bg`: `accent` `#ffffff`, `text`: `ink-inverse` `#010102`
- `font`: `text-button` (14px / 500 / 1.2)
- `padding`: 8px 16px (`py-2 px-4`), `height`: 40px
- `radius`: `--radius-md` (8px)
- `hover`: `bg` → `accent-hover` `#e0e0e0`
- `active`: `bg` → `accent-active` `#c8c8c8`
- `disabled`: `opacity: 0.40`, `cursor: not-allowed`

**`button-secondary`** — Surface lift button. Paired alternative.
- `bg`: `surface-1` `#0b0c0d`, `text`: `ink-secondary` `#a8abb0`
- `border`: 1px `hairline`
- `padding`: 8px 16px, `height`: 40px, `radius`: 8px
- `hover`: `bg` → `surface-2` `#121315`, `text` → `ink` `#eeeff0`
- `active`: `bg` → `surface-3`

**`button-ghost`** — Text-only button for tertiary actions.
- `bg`: transparent, `text`: `ink-secondary`
- `padding`: 6px 12px, `radius`: 8px
- `hover`: `bg` → `accent-muted` (8% white)

**`button-destructive`** — Danger action.
- `bg`: `danger` `#e5484d`, `text`: white
- Same sizing as primary
- `hover`: `bg` → `#cf3c41`, `active`: `bg` → `#ba3035`

**Button Sizes:**
| Size | Class | Height | Padding | Font |
|------|-------|--------|---------|------|
| sm | `button-sm` | 32px | 4px 12px | 13px / 500 |
| md | (default) | 40px | 8px 16px | 14px / 500 |
| lg | `button-lg` | 48px | 10px 24px | 16px / 500 |

### Input

**`input`** — Text input, select, textarea.
- `bg`: `surface-input` `#0b0c0d`, `text`: `ink`
- `border`: 1px `hairline`, `radius`: 8px
- `padding`: 8px 12px, `height`: 40px
- `placeholder`: `ink-tertiary`
- `focus`: `border` → `hairline-emphasis`, `ring`: 0 (no glow — hairline change only)
- `disabled`: `opacity: 0.40`, `cursor: not-allowed`
- `error`: `border` → `danger`

**`textarea`** — Same styling as input. Min-height 80px. Resize vertical only.

### Card

**`card`** — Standard content container.
- `bg`: `surface-1` `#0b0c0d`
- `border`: 1px `hairline`
- `radius`: `--radius-lg` (12px)
- `padding`: 20px (24px for feature-style cards)

**`card-elevated`** — Hovered or featured card.
- `bg`: `surface-2` `#121315`
- `border`: 1px `hairline-strong`

No shadow on either variant. The surface lift alone signals elevation.

### Modal / Dialog

**`modal`** — Settings, confirmations, forms.
- `bg`: `surface-2` `#121315`
- `border`: 1px `hairline-strong`
- `radius`: `--radius-xl` (16px)
- `shadow`: `shadow-popover` (single shadow — the ONLY place shadows appear)
- `overlay`: `surface-overlay` (60% black scrim)
- `title`: `text-title` (18px / 600), padding 24px 24px 0
- `body`: padding 24px
- `width`: settings = 80vw (max 1200px), confirm = 480px

### Sidebar

**`sidebar`** — Left navigation panel.
- `bg`: `surface-1` `#0b0c0d`
- `border-right`: 1px `hairline`
- `width`: 240px expanded, 48px collapsed
- Item padding: 8px 12px
- Active item: `bg` → `accent-muted`, `text` → `ink`
- Hover item: `bg` → `surface-2`

### Tooltip

**`tooltip`** — Hover info, keyboard shortcuts.
- `bg`: `surface-3` `#191a1c`
- `text`: `ink`, `font`: `text-caption` (12px)
- `border`: 1px `hairline-strong`
- `radius`: 8px, `padding`: 6px 10px
- `shadow`: `shadow-popover` (Level 3)

### Dropdown Menu

**`dropdown`** — Context menus, select options.
- `bg`: `surface-3` `#191a1c`
- `border`: 1px `hairline-strong`
- `radius`: 12px, `padding`: 4px (container)
- Item: 8px 12px padding, `radius`: 6px
- Item hover: `bg` → `accent-muted`
- `shadow`: `shadow-popover`

### Badge / Tag / Pill

**`badge`** — Status indicators, labels.
- `bg`: `accent-muted` (8% white), `text`: `ink-secondary`
- `font`: `text-caption-xs` (11px / 500 / 0.3px tracking, uppercase)
- `radius`: `--radius-full` (pill), `padding`: 2px 8px

**`badge-success`**: `bg` → `success-muted`, `text` → `success`
**`badge-danger`**: `bg` → `danger-muted`, `text` → `danger`

### Table

**`table`** — Data grids (MCP servers, settings lists).
- Header: `bg` `surface-2`, `text` `ink-secondary`, `font` `text-caption-xs` uppercase
- Row: `bg` `surface-1`, `border-bottom` 1px `hairline`
- Row hover: `bg` → `surface-2`
- Cell padding: 12px 16px

### Scrollbar

- Width: 8px
- Track: transparent
- Thumb: `surface-3` `#191a1c`, `radius`: `--radius-full`
- Thumb hover: `surface-4`

---

## 8. Interaction States

| State | Visual Treatment |
|-------|-----------------|
| **Default** | Rest state as defined per component |
| **Hover** | Surface lift (+1 level) or bg color shift toward lighter |
| **Focus** | `hairline-emphasis` border (no ring, no glow) |
| **Active/Press** | Deeper color shift (10-15% darker than hover) |
| **Disabled** | 40% opacity, `cursor: not-allowed` |
| **Loading** | Skeleton shimmer or subtle spinner, no blocking |

**Focus rule:** No `outline` or `box-shadow` rings. Use `hairline-emphasis` border color shift. Clean and minimal.

**Transition:** `150ms ease` on color/background changes. `200ms ease` on opacity. No motion on layout properties.

---

## 9. Do's and Don'ts

### Do

- Use `canvas` `#010102` as the page floor — never pure `#000`
- Use the surface ladder (canvas → surface-1 → surface-2 → surface-3) for hierarchy
- Apply 1px translucent hairlines to separate surfaces — never multi-layer shadows
- Use 8px radius on ALL interactive elements; 12px on containers; 16px on modals
- Use weight 600 with negative letter-spacing on headings
- Use JetBrains Mono for every code surface
- Use `shadow-popover` ONLY on modals and dropdowns — nowhere else
- Keep section rhythm at 80px vertical gaps
- Use 20-24px interior padding on cards
- Transition colors/backgrounds at 150ms ease — snappy but visible

### Don't

- Don't use drop shadows on cards, buttons, sidebars, or any static surface
- Don't use `#000000` true black — `#010102` is the canvas
- Don't introduce a chromatic brand color (orange, blue, purple) — white/gray IS the accent
- Don't use font weight 700+ — it's too aggressive for a developer tool
- Don't use 0px border radius on interactive elements
- Don't use gradients, glows, or atmospheric background effects
- Don't add `box-shadow` focus rings — use border color shift
- Don't cram elements with `< 8px` gaps — breathe

---

## 10. Responsive Behavior

Applicable when running in browser (Vite dev mode). The Tauri window is fixed-size.

| Breakpoint | Width | Behavior |
|------------|-------|----------|
| Desktop | ≥1024px | Full sidebar (240px), full layout |
| Tablet | 768-1023px | Collapsed sidebar (48px icons), single-column chat |
| Mobile | <768px | Hidden sidebar (toggle), stacked layout |

- Touch targets: minimum 40px height
- Modal: `80vw` width at all breakpoints, `80vh` height
- Cards: fluid width within container

---

## 11. Theme Engine Integration

This DESIGN.md defines the **design specification**. Implementation lives in:

```
src/index.css       →  @theme tokens + [data-theme="dark-default"] CSS variables
src/contexts/
  ThemeContext.tsx   →  Runtime theme switching (no changes needed)
src/components/ui/   →  Components consume CSS variables, not hardcoded values
```

**Token mapping (spec → CSS variable):**

| DESIGN.md Token | CSS Variable |
|-----------------|-------------|
| `canvas` | `--color-surface-primary` (page bg) |
| `surface-1` | `--color-surface-elevated` (cards, sidebar) |
| `surface-2` | `--color-surface-2` (modal, hover) |
| `surface-3` | `--color-surface-3` (dropdowns) |
| `ink` | `--color-content-primary` |
| `ink-secondary` | `--color-content-secondary` |
| `ink-tertiary` | `--color-content-tertiary` |
| `hairline` | `--color-border-default` |
| `hairline-strong` | `--color-border-emphasis` |
| `accent` | `--color-accent-primary` |
| `accent-hover` | `--color-accent-primary-hover` |
| `accent-muted` | `--color-accent-primary-muted` |

CSS variable names remain unchanged — only the **values** in `[data-theme="dark-default"]` need updating to match the spec above.
