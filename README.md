# Lepo

```
██╗     ███████╗██████╗  ██████╗
██║     ██╔════╝██╔══██╗██╔═══██╗
██║     █████╗  ██████╔╝██║   ██║
██║     ██╔══╝  ██╔═══╝ ██║   ██║
███████╗███████╗██║     ╚██████╔╝
╚══════╝╚══════╝╚═╝ ╚═════╝
```

---

## ◆ PULSE

One dashboard, many repositories, zero tab-hopping. Lepo watches every
repo you care about at once - open issues, pull requests, CI status,
recent activity - and answers the daily question: *what needs me today?*
Click any row and the real issue or PR opens on github.com. v1 is
strictly read-only: it observes, never writes back. A monitor's job is
to look, and Lepo does not blink.

| 1-2 ▣ | 3 ▢ | 4 ▣ | 5-8 ☐ |
|---|---|---|---|

*The honest dashboard and its offline test suite are sealed; the
responsive pass is sealed. The look is half-forged - the warm-cream
identity landed, the DESIGN.md redo still waits. Keyboard navigation,
accessibility, offline cache, and the v1.0 gate stand open.*

> Built with Rust 2024 + Leptos 0.8, fetched through `gloo-net`, kept in
> `gloo-storage`, shipped as a static site - no backend of its own, ever.
>
> **suradet-ps**, artifact keeper

---

## ◆ IGNITION

One target, one tool, one command.

```
⟫ rustup target add wasm32-unknown-unknown
⟫ cargo install trunk
⟫ trunk serve --port 3000 --open
```

The release artifact:

```
⟫ trunk build --release
```

The bundle lands in `dist/` - deploy the folder to any static host.
On Vercel: build command `trunk build --release`, output `dist`; the app
calls `api.github.com` directly from the browser (CORS-enabled), so no
proxy is needed.

<details>
<summary>Prerequisites</summary>

- [Rust](https://rustup.rs/) - stable toolchain (pinned in `rust-toolchain.toml`)
- [Trunk](https://trunkrs.dev/) - installed above

</details>

---

## ◆ ANATOMY

One page, several honest views, a boundary that never bends.

- **Watches** - the watchlist lives in `localStorage` as `owner/repo`
  entries; every card shows open-issue and open-PR counts plus star and
  fork stats, with the last commit time surfaced where it matters.
- **Pages** - the dashboard follows the `Link` header instead of guessing
  page numbers; past a reasonable count, the number itself ("200+ open
  issues") is more honest than a truncated list. Repo detail grows a
  "Load more" button, never a guess.
- **Filters** - the inactive tab is never fetched; switching tabs fires
  the fetch. Label, author, and sort filters expose exactly the slice of
  reality the user asked for.
- **Breathes** - auto-refresh fires at the configured interval (manual /
  1 / 5 / 15 min) and pauses when the rate limit is nearly exhausted,
  resuming when it recovers. The live `x-ratelimit-*` badge says what the
  quota says.
- **Guards** - the GitHub token never leaves the browser; CSP allows only
  `https://api.github.com`, `X-Frame-Options: DENY`, no `unsafe` code at
  the workspace level, and every API error maps to a human-readable
  message - the dashboard reports, it never mumbles.

---

## ◆ RITUALS

**The core ceremony** - the morning scan:

1. Open Lepo. The dashboard answers first: which repos need attention
   today.
2. Add a repo as `owner/repo`. It appears on the board, stored locally.
3. Scan a card. Open issues, open PRs, stars, forks, last commit - all
   in one glance. Click a row to stand in front of the real thing on
   github.com.
4. Refresh on your rhythm: manual, or the interval you chose. When the
   quota runs thin, Lepo holds its breath and tells you why.

**The ceremony of restraint** - v1 reads and never writes. No issues
created, no comments left, no merges made - the read-only boundary is a
security feature, not a missing feature.

**The ceremony of the token** - the PAT lives in your browser and dies
with your clearing of `localStorage`. It is never sent to any server of
ours, because there is no server of ours.

---

## ◆ ECHOES

**Where this artifact is heading**

```
1-2  ▸ honest dashboard: pagination, lazy tabs, filters + tests ──── ▸ sealed
3    ▸ own look: tokens + brand mark done; DESIGN.md redo ─────────── ▸ forging
4    ▸ responsive: table-to-card, touch targets ────────────────────── ▸ sealed
5    ▸ keyboard navigation ─────────────────────────────────────────── ▸ open
6    ▸ accessibility pass, WCAG AA, reduced motion ─────────────────── ▸ open
7    ▸ offline cache and PWA shell ─────────────────────────────────── ▸ open
8    ▸ v1.0.0: audit, branch protection, tag ───────────────────────── ▸ open
```

**Raising the artifact** - the full scope lives in `AGENTS.md`; the
honest path in `docs/ROADMAP.md`; the look in `docs/DESIGN.md`.
Vulnerabilities go to `docs/SECURITY.md`. Gates before any PR:
`cargo fmt --all --check`, clippy with correctness + suspicious lints,
and `cargo test --workspace --exclude app` - no test ever hits
`api.github.com`.

**Status** - CI checks every push; deployment is a Vercel connect away.
[Watch the gates](.github/workflows).

---

```
  ─────────────────────────────────────────
   A monitor that lies is worse than
   no monitor at all.
  ─────────────────────────────────────────
```

Lepo is released under the [MIT License](LICENSE).