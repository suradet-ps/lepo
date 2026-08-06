# Lepo Roadmap

Lepo is a Rust/WASM dashboard for monitoring multiple GitHub repositories
from one view. No backend, no server — static WASM talking directly to
`api.github.com`. The token stays in `localStorage`; the data stays yours.

This roadmap is written from what the code actually does today, not from
what it was supposed to do. Every gap listed here was verified against the
source.

---

## What's broken today

Reading the repo reveals several features that are declared but not working,
or working incorrectly:

**The dashboard lies.** Pagination exists and parses the `Link` header
correctly, but both `DashboardPage` and `RepoDetailPage` throw the result
away. Every repo shows at most 30 issues and 30 PRs. A repo with 200 open
issues looks identical to one with 5. This is the most damaging bug: the
tool's one job is to show you what needs attention, and it can't do that
with incomplete data.

**Auto-refresh is dead code.** The interval is defined, the settings UI has
a dropdown, the timer dependency is in `Cargo.toml`. But nothing wires them
together — the dashboard never re-fetches on its own.

**No latest commit time.** The spec says dashboard cards should show last
commit time. The repo metadata already carries `pushed_at`, but nothing
displays it.

**Repo detail wastes rate limit.** Both issues and PRs fire on mount
regardless of which tab is active. Looking at issues also pays for a PRs
fetch you won't see.

**Missing filters.** Only state (open/closed/all) is filterable on repo
detail. Label, author, and sort are not wired — the data structures support
them, the UI doesn't expose them.

**Borrowed visual identity.** DESIGN.md describes Pinterest's marketing
surfaces. The CSS tokens are Pinterest's values. Lepo has no look of its own.

**Single breakpoint.** Only 768px. No tablet, no narrow-mobile handling.

**No tests for logic that matters.** 19 unit tests exist, all serde
deserialization. The API layer has no mocks. Error classification, rate-limit
edge cases, and conversions are untested.

---

## What to do about it

The work is ordered by impact on daily use, not by architectural elegance.
A monitoring tool that shows incomplete data and doesn't refresh is broken
regardless of how good its design system is.

### 1. Make the dashboard honest

Fix the things that make Lepo show wrong information.

- [ ] **Paginate the dashboard.** Fetch more than 30 items per repo. Follow
  the `Link` header for subsequent pages. Cap at a reasonable limit — beyond
  a certain count, the number itself ("200+ open issues") is more useful
  than the full list.
- [ ] **Paginate repo detail.** "Load more" button driven by the `Link`
  header. Never guess the page number.
- [ ] **Wire auto-refresh.** Connect the existing `RefreshInterval` setting
  to an actual timer. Pause when the rate limit is nearly exhausted;
  resume when it recovers.
- [ ] **Show last commit time.** The repo metadata already has `pushed_at`.
  Surface it on dashboard cards.
- [ ] **Lazy-load the inactive tab.** Only fetch the tab the user is
  looking at. Switching tabs triggers the fetch.
- [ ] **Wire the missing filters.** Expose label, author, and sort on the
  repo detail page.

**Acceptance:** a repo with many issues shows them all (paginated);
auto-refresh fires at the configured interval; switching tabs doesn't waste
a fetch; label/author/sort filters exist on repo detail.

### 2. Test the things that will silently break

The pagination refactor and filter additions touch the API layer heavily.
Tests catch regressions before users do.

- [ ] **Mock the API layer.** Hand-rolled mock of the `GithubApi` trait for
  tests. Cover URL construction, query-string encoding, error
  classification, and pagination edge cases. All tests run offline.
- [ ] **Test error conversions.** Every API error variant maps to the
  correct app error variant with a human-readable message.
- [ ] **Test edge cases in core types.** Input validation, rate-limit
  math at boundary values.

**Acceptance:** `cargo test --workspace --exclude app` passes with
mock-based API tests. No test hits `api.github.com`.

### 3. Give Lepo its own look

A monitoring dashboard needs to feel dense, fast-scanning, and calm — not
like a social media platform.

- [ ] **Rewrite `DESIGN.md`.** Dark-first, information-dense, warm grays,
  tight type, tabular numerics for data. One accent color for actions.
  Document why these choices serve a monitoring surface.
- [ ] **Retune CSS tokens.** Replace Pinterest's values. Keep token names.
  Dark mode is default; light mode inverts.
- [ ] **Eliminate hardcoded hex.** Every color routes through a token.
  CI enforces this.
- [ ] **Distinct favicon and wordmark.**

**Acceptance:** DESIGN.md describes Lepo; zero inline hex in CSS (CI
enforced); both themes render from tokens alone.

### 4. Responsive and mobile

Most developers check dashboards on desktop, but a quick phone check should
work too.

- [ ] **Breakpoint system.** Mobile, tablet, desktop-small, desktop.
- [ ] **Table → card fallback.** The sortable table is unusable on small
  screens. Force card view below tablet width.
- [ ] **Responsive navigation.** Top nav on desktop; collapsed menu on
  mobile.
- [ ] **Touch targets.** All interactive elements meet WCAG AA sizing.

**Acceptance:** usable from phone to ultrawide; table auto-converts to
cards on mobile.

### 5. Keyboard navigation

A developer's hands are on the keyboard. Reaching for the mouse to scan
through repos is friction.

- [ ] **Navigate lists with keys.** Move through dashboard rows and
  issue/PR lists without a mouse.
- [ ] **Open items with Enter.** Selected row opens on github.com.
- [ ] **Quick actions.** Switch tabs, focus search, trigger refresh — all
  from the keyboard.
- [ ] **Visible focus ring** on all interactive elements, all themes.

**Acceptance:** every action reachable via keyboard; focus ring visible.

### 6. Accessibility pass

- [ ] **Screen-reader test** with VoiceOver + NVDA. Verify summary numbers,
  table structure, tab switching, and loading states are announced.
- [ ] **`prefers-reduced-motion`.** Disable animations when the OS requests it.
- [ ] **Color contrast.** All themes pass WCAG AA.
- [ ] **ARIA on custom widgets.** Tabs, filter chips, and any dialogs
  correctly labeled.
- [ ] **Log findings** in `docs/a11y-notes.md`.

**Acceptance:** SR pass logged; all themes pass AA; reduced-motion honored.

### 7. Offline cache

Not critical for a monitoring tool, but nice: show the last-known state
when the network is down instead of a blank screen.

- [ ] **PWA shell.** Cache the app shell. Make Lepo installable.
- [ ] **Cache dashboard data.** Last-known state shown immediately;
  re-fetch in background when online.
- [ ] **Offline indicator.** Calm banner showing when the cached data is
  from.

**Acceptance:** app installs as PWA; launches from cache; cached data
shown offline.

### 8. Prepare for v1.0.0

- [ ] **Reproducible build documented.** Exact toolchain → same `dist/`
  from a given commit.
- [ ] **`cargo audit` + `cargo deny`** in CI, kept green.
- [ ] **Branch protection on `main`.** Required status checks, no
  force-push.
- [ ] **Getting-started docs.** Token creation → paste into Lepo → add
  repos → monitor.
- [ ] **Privacy statement.** No telemetry. Token in localStorage.
- [ ] **`v1.0.0` tag** with CHANGELOG via `git-cliff`.

**Acceptance:** tagged release; branch protection live; audit green; docs
match the app.

---

## What this roadmap does not include

These are real features that belong in a different product:

- **Write-back to GitHub** (create/edit/close issues, comment, merge PRs).
  The read-only boundary is a security feature.
- **OAuth Device Flow.** CORS is not guaranteed. Experimental post-1.0.
- **Multi-forge support** (GitLab, Forgejo). Feasible via the trait
  architecture, but each adapter is significant work. Post-1.0.
- **Team features.** Lepo is one developer's surface.
- **AI insights / summaries.** Adds network dependency and cost/privacy
  surface.
- **Telemetry.** Explicitly never.
- **Native mobile apps.** PWA is the mobile story.

---

## What changes if the user base grows

These aren't phases — they're signals that Lepo is becoming something
bigger, and the architecture should adapt at that point, not before:

- **Multiple accounts / orgs** → the single-token model breaks.
- **Collaborative watchlists** → shared monitoring for a team; needs a
  backend.
- **Webhook support** → replace polling with push; needs a relay.
- **GitLab / Forgejo adapters** → refactor to a forge-agnostic trait.
- **CLI companion** → the `github-api` crate is already Leptos-free; a
  CLI could reuse it directly.
