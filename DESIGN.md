# Lepo Design System

Lepo is a monitoring dashboard. It watches many GitHub repositories and surfaces
the signal — open issues, failing CI, stale PRs — without noise. The design
system serves that job: dense, scannable, calm.

## Design Principles

1. **Warm and readable.** A cream-tinted light palette reduces harsh contrast
   while keeping text crisp. Dark mode uses warm near-blacks for the same feel.
2. **Information density first.** Every pixel carries data. Tight spacing, small
   type, tabular numerics. The user scans dozens of repos in seconds — the
   design must not slow them down.
3. **One accent for actions.** Pinterest Red (`{colors.primary}`) is the only
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

- **Primary** (`{colors.primary}` — `#e60023`): Pinterest Red. Links, active
  tab indicator, primary button background. The single interactive accent.
- **Primary Pressed** (`{colors.primary-pressed}` — `#cc001f`): pressed state
  for the primary button.
- **Primary Muted** (`{colors.primary-muted}` — `rgba(230, 0, 35, 0.08)`):
  tinted background for active chips and hover fills.

### Surface (Light Mode Default)

- **Canvas** (`{colors.canvas}` — `#ffffff`): true white. Base surface for
  nav, modals, feature cards, content body.
- **Surface** (`{colors.surface}` — `#fbfbf9`): faintly cream-tinted
  off-white used for the page body wash.
- **Surface Card** (`{colors.surface-card}` — `#f6f6f3`): warm-cream card and
  tile background.
- **Surface Hover** (`{colors.surface-hover}` — `#ededf0`): hover state for
  interactive surfaces.
- **Secondary BG** (`{colors.secondary-bg}` — `#e5e5e0`): gray-cream for
  secondary button fill.
- **Secondary Pressed** (`{colors.secondary-pressed}` — `#c8c8c1`): pressed
  state for secondary button.
- **Hairline** (`{colors.hairline}` — `#dadad3`): 1px row dividers, card
  borders.
- **Hairline Soft** (`{colors.hairline-soft}` — `#e5e5e0`): lighter inline
  divider.

### Text

- **Ink** (`{colors.ink}` — `#000000`): primary headlines, button text, nav
  links.
- **Ink Soft** (`{colors.ink-soft}` — `#211922`): inline-link color in body
  prose.
- **Body** (`{colors.body}` — `#33332e`): default paragraph text.
- **Charcoal** (`{colors.charcoal}` — `#262622`): softer body where pure ink
  is too heavy.
- **Mute** (`{colors.mute}` — `#62625b`): metadata, timestamps, secondary
  captions.
- **Ash** (`{colors.ash}` — `#91918c`): disabled button text, placeholder
  text.
- **Stone** (`{colors.stone}` — `#c8c8c1`): least-emphasis utility text.
- **On Dark** (`{colors.on-dark}` — `#ffffff`): text on dark surfaces.
- **On Primary** (`{colors.on-primary}` — `#ffffff`): text on primary button.
- **On Secondary** (`{colors.on-secondary}` — `#000000`): text on secondary
  button.

### Semantic

- **Error** (`{colors.error}` — `#9e0a0a`): validation messages, destructive
  actions.
- **Error Deep** (`{colors.error-deep}` — `#cc001f`): deepened error
  background.
- **Success** (`{colors.success}` — `#1a7f37`): CI passing, positive status.
- **Success Deep** (`{colors.success-deep}` — `#103c25`): success messaging.
- **Success Pale** (`{colors.success-pale}` — `#c7f0da`): success pill
  background.
- **Warning** (`{colors.warning}` — `#bf8700`): in-progress CI, caution.
- **Focus** (`{colors.focus}` — `#435ee5`): focus ring blue.

### Dark Mode

Dark mode inverts the surface hierarchy. Token names stay the same; values
change to warm near-blacks (`#1b1b18` canvas, `#242420` card) with inverted
text. Semantic colors adjust lightness for contrast on dark surfaces.

## Typography

### Font Family

**Inter** — a variable geometric sans-serif optimized for screens. Weights 400
(regular), 500 (medium), 600 (semibold), 700 (bold). Fallback:
`-apple-system` → `system-ui` → `Segoe UI` → `Roboto` → `Helvetica Neue` →
`Arial`.

### Hierarchy

| Token | Size | Weight | Line Height | Use |
|---|---|---|---|---|
| `{typography.heading-lg}` | 22px | 700 | 1.25 | Page titles, section headings |
| `{typography.heading}` | 20px | 600 | 1.3 | Sub-section headings |
| `{typography.body-md}` | 16px | 400 | 1.4 | Body copy, default paragraph |
| `{typography.body-strong}` | 14px | 600 | 1.5 | Emphasis, repo names, nav links |
| `{typography.body}` | 14px | 400 | 1.5 | Secondary body text |
| `{typography.small}` | 12px | 400 | 1.4 | Metadata, timestamps, captions |
| `{typography.small-strong}` | 12px | 600 | 1.4 | Table headers, stat labels |
| `{typography.caption-md}` | 12px | 500 | 1.5 | Caption text, link metadata |
| `{typography.mono}` | 13px | 400 | 1.4 | Tabular numerics, issue numbers |
| `{typography.button-md}` | 14px | 700 | 1 | Button labels |
| `{typography.button-sm}` | 12px | 700 | 1 | Chip text, compact buttons |

### Principles

- **Tabular numerics** (`font-variant-numeric: tabular-nums`) on every number
  in a column: issue counts, PR counts, star counts, rate-limit remaining.
- **Tight tracking** on headings (-0.4px to -0.8px) for a dense, confident
  feel.
- **Generous line-height** on body text (1.4–1.5) for readability.

## Spacing

Base unit: 8px. Finer 4/6px steps for tight inline gaps.

| Token | Value | Use |
|---|---|---|
| `{spacing.xxs}` | 4px | Inline gaps, icon-to-text |
| `{spacing.xs}` | 6px | Chip padding, tight gaps |
| `{spacing.sm}` | 8px | Card gaps, list item gaps |
| `{spacing.md}` | 12px | Card padding, row padding |
| `{spacing.lg}` | 16px | Section gaps, component separation |
| `{spacing.xl}` | 24px | Page margins, major gaps |
| `{spacing.xxl}` | 32px | Page header to content |
| `{spacing.section}` | 64px | Major section breaks |

### Density

Table rows are 56px tall. Card padding is 12px. Gaps between list items are
8px. The user should see 15-20 repos without scrolling on a 1080p screen.

## Shapes

| Token | Value | Use |
|---|---|---|
| `{rounded-sm}` | 8px | Buttons, inputs, chips |
| `{rounded-md}` | 16px | Cards, panels, table wrapper |
| `{rounded-lg}` | 32px | Modal dialogs, large cards, token form |
| `{rounded.full}` | 9999px | Status dots, avatars, pill badges |

The radius vocabulary is three values: 8px for most things, 32px for large
surfaces and modals, and full for circular elements.

## Elevation

| Level | Treatment | Use |
|---|---|---|
| 0 — Flat | No shadow, hairline border | Default for cards, table wrapper |
| 1 — Soft shadow | `var(--shadow-sm)` + hairline border | Token form, settings sections |
| 2 — Modal | `var(--shadow-lg)` + backdrop scrim | Modal dialogs |

## Components

### Buttons

**`button-primary`** — background `{colors.primary}`, text
`{colors.on-primary}`, font `{typography.button-md}`, height 40px, rounded
`{rounded-md}`. For primary CTAs (Add repo, Save token).

**`button-secondary`** — background `{colors.secondary-bg}`, text
`{colors.on-secondary}`, height 40px, rounded `{rounded-md}`. For secondary
actions (Cancel, Load more).

**`button-tertiary`** — transparent background, text `{colors.ink}`, rounded
`{rounded-md}`. For low-emphasis actions.

### Inputs

**`text-input`** — background `{colors.canvas}`, border 1px
`{colors.ash}`, height 44px, rounded `{rounded-md}`. Focus: border
`{colors-focus}`, soft blue ring.

### Cards

**`surface-card`** — background `{colors.surface-card}`, 1px solid
`{colors.hairline}`, rounded `{rounded-md}`, padding `{spacing.md}`. No shadow
by default.

### Table

**`data-table`** — full-width, font `{typography.body-sm}`. Header:
`{typography.caption-md}`, uppercase, 0.4px letter-spacing, bottom border
`{colors.hairline}`. Rows: 56px, bottom border `{colors.hairline-soft}`, hover
`{colors.surface}`. Numeric columns: right-aligned, tabular-nums.

### Filter Chips

**`filter-chip`** — transparent background, text `{colors.ink}`, font
`{typography.button-md}`, rounded `{rounded-full}`. Active: inverted
(`{colors.ink}` bg, `{colors.on-dark}` text).

### CI Status Badge

- 8px dot + label text in `{typography.caption-md}`.
- Green dot (`{colors.success}`) + "Pass".
- Red dot (`{colors.error}`) + "Fail".
- Amber dot (`{colors-warning}`) + "Running".
- Grey dot (`{colors.ash}`) + "—" for unknown.

### Rate Limit Badge

- Status dot (8px) + remaining count.
- Green when > 50% remaining, amber when 10-50%, red when < 10%.

### Loading

- Muted "Loading…" text (`{colors-mute}`, `{typography.body-md}`),
  centered, while data is being fetched.
- Buttons that trigger work (e.g. "Load more", "Adding…") use the
  inline spinner.

## Layout

### Breakpoints

- **Mobile:** < 480px. Single column, collapsed nav, card view, secondary
  row details hidden.
- **Tablet:** 480–767px. Compact spacing, dashboard forced to card view.
- **Desktop-small:** 768–1023px. Full nav and table restored.
- **Desktop:** ≥ 1024px. Full experience (content max 1280px).

Media queries use these exact pixel values — CSS custom properties cannot
be used inside `@media`, so the values live in `style.css` under
"Responsive breakpoints" and this section documents them.

### Grid

- **Max width:** 1280px, centered.
- **Dashboard summary:** `repeat(auto-fit, minmax(180px, 1fr))`.
- **Repo cards:** `repeat(auto-fill, minmax(220px, 1fr))`.
- **Settings:** `repeat(auto-fit, minmax(340px, 1fr))`.

### Navigation

- Sticky top nav, 64px height, background `{colors-canvas}` with backdrop blur.
- Left: logo + wordmark. Center: nav links. Right: theme toggle + rate badge.
- 1px bottom border `{colors.hairline}`.

## Do's and Don'ts

### Do
- Use `{colors.primary}` for interactive elements only. Never decorative.
- Use `font-variant-numeric: tabular-nums` on every number in a column.
- Keep table rows at 56px. Density is a feature.
- Use semantic status colors only as dots, text, or thin borders.
- Route every color through a CSS custom property. No hardcoded hex.

### Don't
- Don't add drop shadows to cards. Only for modals and floating panels.
- Don't use cool tones (blue-gray, slate). The palette is warm-cream.
- Don't increase spacing beyond `{spacing.xl}` between content sections.
- Don't introduce new accent colors. Red is the only accent.
- Don't use `{colors.error}` for non-critical decoration.

## Logo & Favicon

The Lepo mark is an SVG combining:
- A rounded-rectangle "monitor" shape (the main body)
- A "drop" indicator at the bottom (status pulse)
- A circular "eye" in the center (the monitoring gaze)
- Side "handles" (the tool/connection metaphor)

Colors from the mark inform the brand palette:
- Sky blue (`#38bdf8`) → the eye accent
- Soft red (`#f87171`) → the drop indicator
- Dark stroke (`#0f172a`) → borders and outlines
- Light fill (`#f1f5f9`) → light-mode surface reference

The favicon uses the full mark. The nav logo renders the SVG inline.
