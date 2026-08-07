# Lepo

![Rust](https://img.shields.io/badge/Rust-2024-000000?style=flat-square&logo=rust&logoColor=white)
![Leptos](https://img.shields.io/badge/Leptos-0.8-543e7c?style=flat-square&logo=leptos&logoColor=white)
![Vercel](https://img.shields.io/badge/Vercel-black?style=flat-square&logo=vercel&logoColor=white)

**Lepo** is a single-page dashboard that monitors the status of multiple GitHub
repositories at once — open issues, pull requests, CI status, and recent activity — in one
easy-to-read view. Click any item to jump straight to the real issue/PR on github.com.

> **v1 is read-only.** No writes back to GitHub (no creating issues, commenting, or
> merging). See [AGENTS.md](./AGENTS.md) for the full scope.

## Features

| Feature | Description |
|---------|-------------|
| **Repo Watchlist** | Add repos manually in `owner/repo` form; stored in `localStorage` |
| **Dashboard** | One card per repo with open-issue / open-PR counts and star/fork stats |
| **Repo Detail** | Issues / PR tabs with state filters; rows open github.com in a new tab |
| **Rate-Limit Awareness** | Live badge from GitHub's `x-ratelimit-*` headers |
| **Auto-refresh** | Configurable interval (manual / 1 / 5 / 15 min) |
| **Theme** | Light / dark, persisted to `localStorage` |

## Tech Stack

- **Language:** Rust (edition 2024, stable toolchain)
- **Framework:** Leptos 0.8 (CSR, `wasm32-unknown-unknown`)
- **Build:** Trunk (WASM bundler / dev server)
- **HTTP:** `gloo-net` (browser `fetch`)
- **Storage:** `gloo-storage` (`localStorage`)
- **Styling:** Pure CSS with design tokens (no UI framework)
- **Deployment:** Vercel (with CSP headers + SPA fallback)

## Documentation

- [AGENTS.md](./AGENTS.md) — architecture, data flow, and agent guidelines
- [DESIGN.md](./DESIGN.md) — UI/UX design system and tokens
- [CONTRIBUTING.md](./CONTRIBUTING.md) — setup, conventions, and PR guide
- [SECURITY.md](./SECURITY.md) — vulnerability reporting policy

## Project Structure

```
crates/
├── app/            # Leptos CSR UI (the only crate aware of Trunk/wasm)
├── github-api/     # GitHub REST client (wasm + native, no Leptos)
└── models/         # Shared serde domain structs
```

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs) — stable toolchain (see `rust-toolchain.toml`)
- `wasm32-unknown-unknown` target:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```
- [Trunk](https://trunkrs.dev):
  ```bash
  cargo install trunk
  ```

### Development

```bash
trunk serve --port 3000 --open
```

### Build (production)

```bash
trunk build --release
```

Output is in `dist/` — deploy the folder to any static host.

## Scripts

| Command | Description |
|---------|-------------|
| `cargo check --workspace --target wasm32-unknown-unknown` | Type-check |
| `cargo clippy --workspace --target wasm32-unknown-unknown -- -D clippy::correctness -D clippy::suspicious` | Lint |
| `cargo fmt --all --check` | Format check (2-space indent, edition 2024) |
| `cargo test --workspace --exclude app` | Run library unit tests |
| `trunk serve --port 3000 --open` | Dev server with HMR |
| `trunk build --release` | Production build |

## Security

- **CSP** via `Content-Security-Policy` header in `vercel.json` (allows only
  `https://api.github.com` for fetches)
- **X-Frame-Options: DENY** — clickjacking protection
- **X-Content-Type-Options: nosniff** — MIME-sniffing protection
- **No `unsafe` code** — `unsafe_code` denied at the workspace level
- **Token stays local** — the GitHub PAT lives only in the browser's `localStorage`; it is
  never sent to any server of ours

To report a vulnerability privately, see [SECURITY.md](./SECURITY.md).

## Accessibility

- Semantic HTML + ARIA
- `:focus-visible` outlines
- `aria-label` on icon-only buttons
- `prefers-reduced-motion` support

## Deployment (Vercel)

1. Connect the repository to Vercel.
2. Set build command: `trunk build --release`
3. Set output directory: `dist`
4. The app calls `api.github.com` directly from the browser (CORS-enabled), so no backend
   or proxy is required.

## License

Lepo is released under the [MIT License](./LICENSE).
