# Lepo Design System

Lepo is a monitoring dashboard. It watches many GitHub repositories and surfaces
the signal — open issues, failing CI, stale PRs — without noise. The design
system serves that job: dense, scannable, calm.

## Design Principles

1. **Dark by default.** Monitoring happens at all hours. A dark canvas reduces
   eye strain and makes status colors (green/red/amber) pop without fighting a
   bright background.
2. **Information density first.** Every pixel carries data. Tight spacing, small
   type, tabular numerics. The user scans dozens of repos in seconds — the
   design must not slow them down.
3. **One accent for actions.** Sky blue (`{colors.primary}`) is the only
   interactive accent. It marks links, active states, and CTAs. Everything else
   is neutral.
4. **Status colors are semantic, not decorative.** Green = pass. Red = fail or
   error. Amber = in progress or warning. These never appear as background fills
   or decorative tints — only as dots, text, and thin borders.
5. **No chrome that doesn't earn its space.** No drop shadows on cards. No
   gradients. No decorative borders. Flat surfaces, hairline dividers, and
   spacing do the work.

## Colors

### Brand & Accent

- **Primary** (`{colors.primary}` — `#38bdf8`): sky blue. Links, active tab
  indicator, primary button background, focus ring. The single interactive
  accent.
- **Primary Hover** (`{colors.primary-hover}` — `#7dd3fc`): lighter sky blue
  for hover states.
- **Primary Muted** (`{colors.primary-muted}` — `rgba(56, 189, 248, 0.15)`):
  tinted background for active chips and hover fills.

### Surface (Dark Mode Default)

- **Canvas** (`{colors.canvas}` — `#0f1117`): the base page background. Near-
  black with a cool undertone.
- **Surface** (`{colors.surface}` — `#161921`): cards, panels, and floating
  surfaces. Slightly lighter than canvas.
- **Surface Raised** (`{colors.surface-raised}` — `#1e2130`): elevated cards,
  dropdown menus, modals. One step above surface.
- **Surface Hover** (`{colors.surface-hover}` — `#252838`): hover state for
  interactive surfaces.
- **Hairline** (`{colors.hairline}` — `#2a2d3a`): 1px dividers, card borders,
  table row separators.
- **Hairline Strong** (`{colors.hairline-strong}` — `#3a3d4a`): emphasis
  dividers (section breaks, table header bottom).

### Text

- **Text** (`{colors.text}` — `#e2e4e9`): primary text on dark surfaces.
  Headings, labels, body copy.
- **Text Secondary** (`{colors.text-secondary}` — `#8b8fa3`): metadata,
  captions, timestamps, muted labels.
- **Text Tertiary** (`{colors.text-tertiary}` — `#5c5f72`): placeholder text,
  disabled states, least-emphasis utility text.
- **Text On Primary** (`{colors.text-on-primary}` — `#0f1117`): text on
  `{colors.primary}` backgrounds (buttons, active chips).

### Semantic

- **Success** (`{colors.success}` — `#34d399`): CI passing, positive status.
- **Error** (`{colors.error}` — `#f87171`): CI failure, validation errors,
  destructive actions.
- **Warning** (`{colors.warning}` — `#fbbf24`): in-progress CI, rate-limit
  amber, caution states.
- **Success Muted** (`{colors.success-muted}` — `rgba(52, 211, 153, 0.12)`):
  subtle success background tint.
- **Error Muted** (`{colors.error-muted}` — `rgba(248, 113, 113, 0.12)`):
  subtle error background tint.
- **Warning Muted** (`{colors.warning-muted}` — `rgba(251, 191, 36, 0.12)`):
  subtle warning background tint.

### Focus

- **Focus Ring** (`{colors.focus}` — `#38bdf8`): 2px outline on focused
  interactive elements. Matches `{colors.primary}`.

## Typography

### Font Family

**Inter** — a variable geometric sans-serif optimized for screens. Weights 400
(regular), 500 (medium), 600 (semibold), 700 (bold). Fallback stack:
`-apple-system` → `system-ui` → `Segoe UI` → `Roboto` → `Helvetica Neue` →
`Arial`.

### Hierarchy

| Token | Size | Weight | Line Height | Letter Spacing | Use |
|---|---|---|---|---|---|
| `{typography.display}` | 28px | 700 | 1.2 | -0.5px | Page titles (rare — only login, settings header) |
| `{typography.heading}` | 20px | 600 | 1.3 | -0.2px | Section headings, card titles |
| `{typography.body}` | 14px | 400 | 1.5 | 0 | Default text, descriptions, form labels |
| `{typography.body-strong}` | 14px | 600 | 1.5 | 0 | Emphasis, repo names, nav links |
| `{typography.small}` | 12px | 400 | 1.4 | 0 | Metadata, timestamps, captions |
| `{typography.small-strong}` | 12px | 600 | 1.4 | 0 | Table headers, stat labels |
| `{typography.mono}` | 13px | 400 | 1.4 | 0 | Tabular numerics, issue/PR numbers, rate-limit counts |
| `{typography.button}` | 13px | 600 | 1 | 0 | Button labels, chip text |

### Principles

- **Tabular numerics** (`font-variant-numeric: tabular-nums`) on every number
  that appears in a column: issue counts, PR counts, star counts, rate-limit
  remaining. This keeps columns aligned when values change.
- **Tight tracking** on headings (-0.2px to -0.5px) for a dense, confident
  feel. Body text uses default tracking for readability.
- **No display-size typography.** This is a monitoring tool, not a marketing
  site. The largest text on any page is 28px.

## Spacing

Base unit: 4px. All spacing is a multiple of 4.

| Token | Value | Use |
|---|---|---|
| `{spacing.xs}` | 4px | Inline gaps, icon-to-text spacing |
| `{spacing.sm}` | 8px | Tight component padding, chip padding |
| `{spacing.md}` | 12px | Card padding, row padding, input padding |
| `{spacing.lg}` | 16px | Section gaps, component separation |
| `{spacing.xl}` | 24px | Page margins, major section gaps |
| `{spacing.xxl}` | 32px | Page header to content gap |

### Density

The dashboard is **dense by default**. Table rows are 44px tall. Card padding is
12px. Gaps between list items are 8px. The user should see 15-20 repos without
scrolling on a 1080p screen.

## Shapes

| Token | Value | Use |
|---|---|---|
| `{rounded.sm}` | 4px | Buttons, inputs, chips, small cards |
| `{rounded.md}` | 8px | Cards, panels, table wrapper |
| `{rounded.lg}` | 12px | Modal dialogs, large cards |
| `{rounded.full}` | 9999px | Status dots, avatar circles, pill badges |

Two radius values cover 95% of cases: 4px for interactive elements, 8px for
surfaces. No rounded corners larger than 12px except full-radius pills.

## Elevation

| Level | Treatment | Use |
|---|---|---|
| 0 — Flat | No shadow, hairline border | Default for cards, table wrapper |
| 1 — Raised | `0 2px 8px rgba(0,0,0,0.3)` + hairline border | Dropdowns, tooltips, floating panels |
| 2 — Modal | `0 8px 32px rgba(0,0,0,0.5)` + backdrop scrim | Modal dialogs (settings, confirm) |

Shadows are cool-toned (`rgba(0,0,0,...)`) to match the dark canvas. No warm
shadows.

## Components

### Buttons

**`button-primary`**
- Background `{colors.primary}`, text `{colors.text-on-primary}`, font
  `{typography.button}`, padding `6px 14px`, height 36px, rounded
  `{rounded.sm}`.
- Used for: Add repo, Save token, primary actions.

**`button-secondary`**
- Background `{colors.surface}`, text `{colors.text}`, 1px solid
  `{colors.hairline}`, font `{typography.button}`, padding `6px 14px`, height
  36px, rounded `{rounded.sm}`.
- Used for: Cancel, Load more, secondary actions.

**`button-tertiary`**
- Background transparent, text `{colors.text-secondary}`, font
  `{typography.button}`, rounded `{rounded.sm}`.
- Used for: Low-emphasis actions (remove repo, log out).

### Inputs

**`text-input`**
- Background `{colors.surface}`, text `{colors.text}`, 1px solid
  `{colors.hairline}`, font `{typography.body}`, padding `8px 12px`, height
  36px, rounded `{rounded.sm}`.
- Focus: border-color `{colors.primary}`, box-shadow `0 0 0 2px
  {colors.primary-muted}`.

### Cards

**`surface-card`**
- Background `{colors.surface}`, 1px solid `{colors.hairline}`, rounded
  `{rounded.md}`, padding `{spacing.md}`.
- No shadow by default. Gains `{elevation-1}` on hover when interactive.

### Table

**`data-table`**
- Full-width, border-collapse, font `{typography.body}`.
- Header: `{typography.small-strong}`, text `{colors.text-secondary}`, uppercase,
  0.4px letter-spacing, bottom border `{colors.hairline-strong}`.
- Rows: 44px height, bottom border `{colors.hairline}`, hover
  `{colors.surface-hover}`.
- Numeric columns: right-aligned, `font-variant-numeric: tabular-nums`.

### Filter Chips

**`filter-chip`**
- Background transparent, text `{colors.text-secondary}`, font
  `{typography.button}`, rounded `{rounded.full}`, padding `6px 14px`.
- Active: background `{colors.primary-muted}`, text `{colors.primary}`.

### CI Status Badge

- 8px dot + label text in `{typography.small}`.
- Green dot (`{colors.success}`) + "Pass" for passing.
- Red dot (`{colors.error}`) + "Fail" for failing.
- Amber dot (`{colors.warning}`) + "Running" for in-progress.
- Grey dot (`{colors.text-tertiary}`) + "—" for unknown/no CI.

### Rate Limit Badge

- Compact: status dot (8px) + remaining count in `{typography.mono}`.
- Green when > 50% remaining, amber when 10-50%, red when < 10%.
- Tooltip shows exact count and reset time.

### Skeleton Loading

- Background `{colors.surface}` with a subtle shimmer animation (left-to-right
  gradient using `{colors.surface-raised}`).
- Shape matches the content it replaces (rectangular for table rows, card-shaped
  for repo cards).

## Layout

### Grid

- **Max width:** 1280px, centered.
- **Dashboard summary:** 4-column grid (`repeat(auto-fit, minmax(180px, 1fr)`).
- **Repo cards:** `repeat(auto-fill, minmax(220px, 1fr))`.
- **Settings:** 2-column grid (`repeat(auto-fit, minmax(340px, 1fr))`).

### Navigation

- Sticky top nav, 56px height, background `{colors.canvas}` with `backdrop-
  filter: blur(14px)`.
- Left: logo + "Lepo" wordmark. Center: nav links. Right: theme toggle + rate
  limit badge.
- 1px bottom border `{colors.hairline}`.

### Page Structure

- Page header: title (`{typography.display}`) + description + action button.
  Flex row, space-between.
- Content area: `{spacing.xl}` (24px) top margin from header.

## Light Mode

Light mode inverts the surface hierarchy. Token names stay the same; values
change:

| Token | Dark | Light |
|---|---|---|
| `{colors.canvas}` | `#0f1117` | `#ffffff` |
| `{colors.surface}` | `#161921` | `#f5f5f7` |
| `{colors.surface-raised}` | `#1e2130` | `#ffffff` |
| `{colors.surface-hover}` | `#252838` | `#ededf0` |
| `{colors.hairline}` | `#2a2d3a` | `#d1d1d6` |
| `{colors.hairline-strong}` | `#3a3d4a` | `#b0b0b8` |
| `{colors.text}` | `#e2e4e9` | `#1a1a2e` |
| `{colors.text-secondary}` | `#8b8fa3` | `#6b6b80` |
| `{colors.text-tertiary}` | `#5c5f72` | `#a0a0b0` |
| `{colors.text-on-primary}` | `#0f1117` | `#ffffff` |

The primary accent (`{colors.primary}`) stays `#38bdf8` in both themes.
Semantic colors (success/error/warning) keep their hue but adjust lightness for
contrast on light surfaces.

## Do's and Don'ts

### Do
- Use `{colors.primary}` for interactive elements only (links, active states,
  CTAs). Never for decorative purposes.
- Use `font-variant-numeric: tabular-nums` on every number in a column.
- Keep table rows at 44px. Density is a feature.
- Use semantic status colors (green/red/amber) only as dots, text, or thin
  borders — never as large background fills.
- Route every color through a CSS custom property. No hardcoded hex in
  component styles.

### Don't
- Don't add drop shadows to cards. The only shadows are for modals and
  floating panels.
- Don't use warm tones (cream, beige, warm-gray). The palette is cool-neutral.
- Don't increase spacing beyond `{spacing.xl}` between content sections. This
  is a dense tool, not a marketing page.
- Don't introduce new accent colors. Sky blue is the only accent.
- Don't use `{colors.error}` for non-critical decoration. Red means something
  is wrong.

## Logo & Favicon

The Lepo mark is an SVG combining:
- A rounded-rectangle "monitor" shape (the main body)
- A "drop" indicator at the bottom (status pulse)
- A circular "eye" in the center (the monitoring gaze)
- Side "handles" (the tool/connection metaphor)

Colors from the mark inform the brand palette:
- Sky blue (`#38bdf8`) → `{colors.primary}`
- Soft red (`#f87171`) → `{colors.error}`
- Dark stroke (`#0f172a`) → borders and outlines
- Light fill (`#f1f5f9`) → light-mode surface reference

The favicon uses the full mark at 120×120. The nav logo uses a simplified
version rendered inline as an `<img>` tag.
