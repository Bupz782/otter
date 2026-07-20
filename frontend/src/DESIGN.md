# Otter Design System

## Principles

- Dark, typography-driven interface.
- One warm accent used sparingly.
- No neon, no heavy shadows, no colored glows.
- Glassmorphism is subtle and functional.
- Motion is ambient and restrained.

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
| `--color-accent`        | `#c8a46c`                   | CTAs, active links, highlights |
| `--color-accent-dark`   | `#a6834d`                   | Hover states                   |
| `--color-accent-subtle` | `rgba(200, 164, 108, 0.12)` | Ghost hover, badge bg          |
| `--color-ring`          | `rgba(200, 164, 108, 0.35)` | Focus rings                    |

Use the accent for primary actions and small status highlights only. Never use it for large backgrounds or glows.

### Status

- Positive: emerald (`#34d399`)
- Negative: rose (`#fb7185`)
- Warning: amber (`#fbbf24`)

## Glassmorphism

Use the `.glass` utility on cards:

```
bg-card backdrop-blur-md border border-border/60
```

Use `.glass-strong` on the fixed navigation:

```
bg-background/75 backdrop-blur-xl border-b border-border/50
```

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

- `AmbientBackground`: fixed slow-drifting gradient layer.
- `PageTransition`: wraps route content for enter/exit fade + slide.
- `DemoPreview`: demo widget embedded on the home page (`#demo` anchor).

Keep motion subtle. Avoid parallax tied to mouse movement. Respect `prefers-reduced-motion` via Framer Motion's `MotionConfig` and CSS media queries.

## Components

### Button

- `default`: accent background, dark text.
- `outline`: accent border/text, subtle hover fill.
- `ghost`: muted text, accent hover.
- `link`: accent underline.

### Card

Always uses glassmorphism. Title uses `font-heading`.

### Badge

- `default`: accent fill.
- `secondary`: subtle accent background.

## File map

| File                                   | Purpose                                 |
| -------------------------------------- | --------------------------------------- |
| `src/styles/tokens.css`                | Primitive tokens                        |
| `src/index.css`                        | Semantic tokens, utilities, animations  |
| `src/components/ui/*`                  | shadcn/ui components styled to system   |
| `src/components/AmbientBackground.tsx` | Ambient motion layer                    |
| `src/components/PageTransition.tsx`    | Route transition wrapper                |
| `src/components/DemoPreview.tsx`       | Home page demo widget                   |
| `src/components/demo/*`                | Demo widget intent simulator components |
| `src/components/app/PageHeader.tsx`    | App page title/subtitle/action header   |
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
- Status visuals come from `src/lib/status.ts` only.
