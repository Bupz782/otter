# Otter Design System

## Principles

- Dark, typography-driven interface with a **painterly art layer** (Midjourney-generated oil/palette-knife artwork is a first-class design material).
- One warm gold accent, sampled from the artwork's firelight.
- No neon, no heavy shadows, no colored glows (the paintings carry the light).
- Motion is ambient and restrained. The `animate-ping` live dot is allowed — it's the product pulse.
- House glyph: a small rotated square (`h-1.5 w-1.5 rotate-45 bg-accent`). No sparkle/AI-cliché icons.

## Typography

- **Headings**: Space Grotesk (600/700), tight tracking.
- **Body / UI**: IBM Plex Sans (400/500/600).

Headings use the `font-heading` utility or are targeted globally via `h1`–`h6`.

## Color

### Base

| Token                | Value                       | Usage            |
| -------------------- | --------------------------- | ---------------- |
| `--color-background` | `#050505`                   | Page background  |
| `--color-foreground` | `#f4f4f5`                   | Primary text     |
| `--color-card`       | `rgba(12, 12, 12, 0.78)`    | Card surfaces    |
| `--color-border`     | `rgba(255, 255, 255, 0.08)` | Borders/dividers |

### Accent

| Token                   | Value                       | Usage                          |
| ----------------------- | --------------------------- | ------------------------------ |
| `--color-accent`        | `#d09a52`                   | CTAs, active links, highlights |
| `--color-accent-dark`   | `#a97638`                   | Hover states                   |
| `--color-accent-subtle` | `rgba(208, 154, 82, 0.12)`  | Ghost hover, badge bg          |
| `--color-ring`          | `rgba(208, 154, 82, 0.35)`  | Focus rings                    |

Use the accent for primary actions and small status highlights only. Never use it for large backgrounds or glows.

### Status

- Positive: emerald (`#34d399`)
- Negative: rose (`#fb7185`)
- Warning: amber (`#fbbf24`)

## Artwork (painterly layer)

All artwork lives in `public/` as webp, generated with Midjourney (v8.2, `--oref` for character consistency, palette-knife oil style, near-black backgrounds, amber visor slit as the otter's signature).

| Asset                    | Usage                                              |
| ------------------------ | -------------------------------------------------- |
| `landing-hero-bg.webp`   | Hero full-bleed scene (otter right, void left)     |
| `otter-404-scene.webp`   | 404 full-bleed scene (otter lost, void above)      |
| `landing-bg.webp`        | Fixed page background (AmbientBackground, ~20%)    |
| `card-texture.webp`      | Card surface grain (under an 84% black veil)       |
| `btn-texture.webp`       | Default button surface (gold-regraded brushwork)   |
| `otter-dive.webp`        | Diving otter, transparent (luminance-keyed cutout) |

Treatment rules:

- Full-bleed scenes ship unretouched; blend edges with CSS (`object-position` + gradient scrims), never by editing the painting.
- Character cutouts on dark backgrounds use a **luminance key** (dark → transparent), not ML matting. Remove specks with a median filter first.
- New poses: same style block, `--oref <master>` (no `--ow`/`--cref` on v8.2), `--no cat, human, blue, green, neon, text, bright background`. Selection criteria: thick tail + amber visor + black background.

## Surfaces

- Landing cards use `.card-painterly` (artwork grain under a dark veil).
- App cards use `.card-painterly-subtle` (same grain, heavier veil) — texture present, data readability first.
- Fixed navigation uses `.glass-strong` (`bg-background/75 backdrop-blur-xl border-b border-border/50`).

## Spacing

Follow Tailwind's default scale. Section vertical padding is `py-28`. Max content width is `max-w-6xl` (`72rem`) or `max-w-5xl` for reading-heavy sections.

## Radius

| Token         | Value      |
| ------------- | ---------- |
| `--radius-sm` | `0.375rem` |
| `--radius-md` | `0.5rem`   |
| `--radius-lg` | `0.75rem`  |
| `--radius-xl` | `1rem`     |

Buttons and pills use `rounded-full` for a softer CTA shape.

## Motion

### Durations

| Token             | Value   |
| ----------------- | ------- |
| `--duration-fast` | `150ms` |
| `--duration-base` | `250ms` |
| `--duration-slow` | `500ms` |

### Easing

- `--ease-out-expo`: `cubic-bezier(0.22, 1, 0.36, 1)`

### Components

- `AmbientBackground`: fixed painterly texture + slow-drifting gradient layer.
- `PageTransition`: wraps route content for enter/exit fade + slide.
- `DemoPreview`: demo widget embedded on the home page (`#demo` anchor); the diving otter floats beside the intent card, decoupled from submission state.

Keep motion subtle. Ambient loops must be smooth (`repeat: Infinity`, easeInOut keyframes) — never switch between animation states on the same element (brutal resets). Respect `prefers-reduced-motion` via Framer Motion's `MotionConfig` and CSS media queries.

## Components

### Button

- `default`: painterly gold texture (`btn-texture.webp`), dark text, brightness hover.
- `outline`: accent border/text, subtle hover fill.
- `ghost`: muted text, accent hover.
- `link`: accent underline.

### Card

Landing: `.card-painterly`. App: flat `bg-card`. Title uses `font-heading`.

### Markers

House glyph (rotated amber square) for decorative markers; semantic lucide icons for real actions (e.g. `GitFork`, `TextSearch`). No `Sparkles`.

### Favicon / touch icon

Synthetic mark: amber visor slit on near-black (`favicon.png`, `apple-touch-icon.png`). Not a crop of the artwork — it must survive 16px.

## File map

| File                                   | Purpose                                 |
| -------------------------------------- | --------------------------------------- |
| `src/styles/tokens.css`                | Primitive tokens                        |
| `src/index.css`                        | Semantic tokens, utilities, animations  |
| `src/components/ui/*`                  | shadcn/ui components styled to system   |
| `src/components/AmbientBackground.tsx` | Fixed painterly page background         |
| `src/components/HeroSection.tsx`       | Full-bleed painted hero                 |
| `src/pages/NotFoundPage.tsx`           | Full-bleed painted 404                  |
| `src/components/PageTransition.tsx`    | Route transition wrapper                |
| `src/components/DemoPreview.tsx`       | Home page demo widget + floating otter  |
| `src/components/demo/*`                | Demo widget intent simulator components |
| `src/components/app/PageHeader.tsx`    | App page title/subtitle/action header   |
| `src/components/app/PageSceneBanner.tsx` | Painterly scene header (Proofs, Solvency) |
| `src/components/app/SectionCard.tsx`   | App section container card              |
| `src/components/app/StatCard.tsx`      | App KPI stat card                       |
| `src/components/app/DataRow.tsx`       | App list row container                  |
| `src/components/app/ErrorState.tsx`    | Load-failure card with retry            |
| `src/lib/status.ts`                    | Shared intent-status presentation       |
| `src/lib/demo-data.ts`                 | Demo fixtures shown when signed out     |

## App (`/app`) conventions

- Pages compose `PageHeader` + `SectionCard` + `StatCard` + `DataRow`; no ad-hoc card markup.
- Data sections always resolve to one of: skeleton, `ErrorState` (with retry), genuine `EmptyState`, or data.
- When no wallet session exists, hooks serve `src/lib/demo-data.ts` fixtures and the header shows an amber "Demo data" pill. Real API data only renders for authenticated sessions.
- When the API itself flags a payload as demonstration data (`demo: true` field / `X-Demo-Data` header, anomaly A2), the affected pages render `DemoDataNotice` with the same amber styling.
- Status visuals come from `src/lib/status.ts` only.
