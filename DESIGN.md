# Aether Design System

## 1. Atmosphere & Identity

Aether feels like a quiet operations console for AI infrastructure. It should be dense enough for repeated admin work, but still calm and legible. The visual signature is warm technical restraint: Claude-inspired ivory surfaces, muted slate text, low-contrast borders, and compact controls that make system state easy to scan.

## 2. Color

### Palette

| Role | Token | Light | Dark | Usage |
|------|-------|-------|------|-------|
| Surface/page | `--background` | `oklch(0.9818 0.0054 95.0986)` | dark theme override | App background |
| Surface/card | `--card` | `oklch(0.9818 0.0054 95.0986)` | dark theme override | Panels and cards |
| Surface/muted | `--muted` | `oklch(0.9341 0.0153 90.239)` | dark theme override | Quiet fills and empty states |
| Text/primary | `--foreground` | `oklch(0.3438 0.0269 95.7226)` | dark theme override | Main copy |
| Text/muted | `--muted-foreground` | `oklch(0.6059 0.0075 97.4233)` | dark theme override | Captions and metadata |
| Border/default | `--border` | `oklch(0.8847 0.0069 97.3627)` | dark theme override | Cards, separators, inputs |
| Accent/primary | `--primary` | `oklch(0.6171 0.1375 39.0427)` | dark theme override | Primary actions and active states |
| Accent/text | `--primary-foreground` | `oklch(1 0 0)` | dark theme override | Text over primary |
| Status/error | `--destructive` | `oklch(0.1908 0.002 106.5859)` | dark theme override | Destructive actions and failures |

### Rules

Use Tailwind semantic tokens such as `bg-card`, `text-foreground`, `text-muted-foreground`, `border-border`, `bg-primary`, and `bg-destructive`. Do not add raw hex, rgb, or one-off color families in components. Yellow warning badges already exist in `Badge` and are allowed for warning status only.

## 3. Typography

### Scale

| Level | Size | Weight | Line Height | Tracking | Usage |
|-------|------|--------|-------------|----------|-------|
| Page title | `text-2xl` to `text-3xl` | 600-700 | normal | 0 | Page headers |
| Section title | `text-base` to `text-lg` | 600 | normal | 0 | Card section titles |
| Card title | `text-sm` | 500 | normal | 0 | Dense admin rows |
| Body | `text-sm` | 400 | 1.5 | 0 | Admin form and table text |
| Caption | `text-xs` | 400-500 | 1.4 | 0 | Metadata and descriptions |

### Font Stack

Primary is `var(--sans-serif)`, backed by system UI and CJK sans fonts. Serif is reserved for public/editorial content. Mono is reserved for code, IDs, keys, and technical literals.

### Rules

Admin panels use compact `text-sm` and `text-xs` scales. Do not use hero-sized text inside dashboards, settings panels, cards, or tables.

## 4. Spacing & Layout

### Base Unit

All spacing follows the Tailwind 4px base.

| Token | Value | Usage |
|-------|-------|-------|
| `gap-1` / `p-1` | 4px | Tight icon spacing |
| `gap-2` / `p-2` | 8px | Button internals and inline groups |
| `gap-3` / `p-3` | 12px | Compact rows |
| `gap-4` / `p-4` | 16px | Card padding and row groups |
| `gap-6` / `p-6` | 24px | Section spacing |

### Grid

Management pages use responsive one-column mobile layouts and two or three columns at `md`/`lg` when repeated cards are directly comparable. `PageContainer` owns page width; sections should not create nested page cards.

### Rules

Cards stay at `rounded-lg` or below unless reusing an existing component that already sets a radius. Use stable grid tracks and fixed icon button dimensions so loading labels and dynamic values do not shift layout.

## 5. Components

### CardSection

Structure: a section header with optional actions and one content body. It is the standard admin settings grouping. Use semantic Tailwind tokens only. Actions belong in the `#actions` slot and should be compact buttons or switches.

States: loading states should keep the same section footprint; error and empty states use `border-border`, `bg-muted/30`, and `text-muted-foreground`.

### Badge

Use `success` for succeeded/ready states, `warning` for running or queued states, `destructive` for failed states, `secondary` or `outline` for neutral states. Status text must be short and scannable.

### Admin Record List

Use compact bordered rows or the shared table components. Each row should expose status, primary timestamp, and one supporting detail. Empty states are unframed or lightly bordered inside the section, not a card inside a card.

## 6. Motion & Interaction

Transitions are subtle, 150-300ms, and limited to color, opacity, and transform. Buttons must keep hover, focus, disabled, and loading states. Respect existing component behavior instead of adding custom animation.

## 7. Depth & Surface

Depth strategy is mixed but restrained: admin settings primarily use borders and tonal shifts, with existing soft shadows only where components already define them. Do not add decorative gradients, orbs, oversized shadows, or nested floating cards.
