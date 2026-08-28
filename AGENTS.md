# Lepo - GitHub Repo Monitor Dashboard

> Guide for AI agents (Claude, GPT, etc.) working on this project.
> Read this entire file before writing any code.

---

## 1. Project Overview

**Project name**: Lepo

**What it does**: A single-page application that monitors the status of multiple
GitHub repositories at once (issues, pull requests, CI status) in an easy-to-read
dashboard. Clicking an item jumps straight to the real issue/PR page on github.com.

**Target user**: Developers/maintainers who look after several repos at once and
want an overview without opening github.com for each one individually.

**v1 scope (important)**: **Read-only only.** No feature that writes back to GitHub
(creating issues, commenting, merging PRs, etc.) in the first version - full stop.
Even if the user asks for this later, confirm explicitly before implementing.

---

## 2. Tech Stack (do not change without asking)

| Layer | Technology |
|---|---|
| Framework | Leptos **v0.8**, CSR mode (Client-Side Rendering) only - no SSR, no server |
| Build tool | Trunk |
| Language | Pure Rust + WASM (no JS/TS in business logic) |
| HTTP client | `gloo-net` (fetch wrapper for wasm) |
| Serialization | `serde` + `serde_json` |
| Storage | `gloo-storage` (localStorage wrapper) |
| Time/format | `chrono` or `time` (pick one, don't mix) |
| Routing | `leptos_router` (if multiple pages are needed, e.g. Settings, Repo detail) |
| Error handling | No `unwrap()` / `expect()` outside `#[cfg(test)]` - use `Result` + a project-specific error type |
| Icons | Pick one pure SVG/Rust icon lib (no CDN JS icon fonts) - specified in DESIGN.md |

**No backend server of our own** - this app is a static site that can be deployed
anywhere (GitHub Pages, Cloudflare Pages, Netlify) and calls the GitHub REST API
directly from the browser.

---

## 3. Architecture Rules

### 3.1 Overall architecture: Workspace + Ports-lite (not full Hexagonal)

This project uses a **Cargo workspace split into crates** from day one, but does
**not** implement full Hexagonal Architecture (no separate use-case/domain layer),
because v1 talks to only one external system (the GitHub REST API) and the
business logic is very light. Forcing a port/adapter for every entity from the
start would add ceremony without real benefit.

Instead: the `github-api` crate is split out from the `app` crate and wired
through a **`GithubApi` trait** - so `app` doesn't depend on a concrete
implementation directly, making it easy to mock/test, and reusable later for a
CLI or Tauri app if needed. (If the project grows to need multiple adapters -
e.g. adding GitLab - refactor up to full hexagonal at that point; don't build it
preemptively.)

### 3.2 Data Flow (mandatory - never skip a layer)

```
UI (Components/Pages) → State → API (github-api crate)
```

- **Components must never call the GitHub API directly.** They must go through
  the state layer only.
- Pages talk to state only (via context).
- State talks to the API layer only (via the `GithubApi` trait).
- Every HTTP request must go through `GithubClient` (no `gloo_net::http::Request`
  calls scattered across component or page files).

### 3.3 Async Convention

- **Fetching data (GET)**: use `Resource` (or `LocalResource` if the data is tied
  to browser-only state, e.g. the token in localStorage)
- **User-triggered mutations/actions** (e.g. clicking refresh, adding/removing a
  repo): use `Action`
- Avoid calling `spawn_local` directly in a component unless truly necessary
  (e.g. a side-effect that isn't data fetching) - if used, add a comment
  explaining why.

### 3.4 State Management

Use Leptos Context (`provide_context` / `expect_context`) as the primary global
state, split into exactly these pieces - do not create duplicate state:

- `AuthState` - token, current user, login status
- `WatchlistState` - the list of repos being monitored
- `SettingsState` - refresh interval, theme
- `RateLimitState` - current remaining/limit/reset (updated automatically every
  time `GithubClient` receives a response - pages should never check this
  themselves)

Any page/component that needs these values should call
`expect_context::<XState>()` - never create a duplicate signal locally.

### 3.5 API Layer Pattern

```rust
// crates/github-api/src/client.rs
pub trait GithubApi {
    async fn get_repo(&self, owner: &str, repo: &str) -> Result<Repo, ApiError>;
    async fn list_issues(&self, owner: &str, repo: &str, params: IssueParams) -> Result<Vec<Issue>, ApiError>;
    async fn list_pulls(&self, owner: &str, repo: &str, params: PullParams) -> Result<Vec<PullRequest>, ApiError>;
    async fn latest_workflow_run(&self, owner: &str, repo: &str) -> Result<Option<WorkflowRun>, ApiError>;
    async fn rate_limit(&self) -> Result<RateLimit, ApiError>;
}

pub struct GithubClient { /* token, base headers */ }
impl GithubApi for GithubClient { /* ... */ }
```

Individual files (`issues.rs`, `pulls.rs`, ...) must not each call `fetch`
independently - every endpoint is a method on `GithubClient` implementing this
trait.

### 3.6 Pagination

GitHub's API returns a `Link` header indicating the next page (`rel="next"`).
**Do not manually guess/increment the page number (`page += 1`) without
checking it.** Parse the `Link` header and stop once there's no `rel="next"`
entry left.

---

## 4. Authentication with GitHub

### Approach used (v1): Personal Access Token (fine-grained)

- Rationale: the GitHub REST API (`api.github.com`) already supports CORS for
  browser fetch requests with an `Authorization` header (confirmed via GitHub's
  own CORS/JSONP documentation), so API calls can go straight from the client
  without any backend to exchange tokens.
- OAuth App Authorization Code flow **cannot be used** with this architecture,
  since it requires a `client_secret`, which cannot be safely stored in a static
  site - **do not implement this flow**.
- OAuth Device Flow is theoretically possible (no client_secret needed for the
  token exchange step), but CORS support on `github.com/login/device/code` and
  `github.com/login/oauth/access_token` for direct browser calls is not
  guaranteed to be stable - treat this as **Phase 3 (optional/experimental)
  only**. Do not make it the primary auth method.

### v1 UI flow

1. On first load with no token present → show a form asking the user to paste a
   PAT, with instructions (link to
   `https://github.com/settings/personal-access-tokens/new`).
2. Clearly state the required scopes in the copy: **Repository permissions →
   Issues: Read-only, Pull requests: Read-only, Metadata: Read-only,
   Contents: Read-only (for commits/CI), Actions: Read-only (for workflow status)**.
3. Store the token in `localStorage` (e.g. key `gh_monitor_token`).
4. On the Settings page, tell the user the token is stored only in this browser's
   local storage, is never sent to any server of ours, and recommend using the
   narrowest fine-grained token scope necessary.
5. Always provide a "Log out / remove token" button that clears all related
   localStorage keys.

### Validating the token

- When the user submits a token, call `GET /user` first to validate it and fetch
  the username to display.
- On 401 → show a clear error that the token is invalid or expired.

---

## 5. Core Features (Feature Scope)

### 5.1 Repo Watchlist (managing which repos to monitor)

- User adds repos manually in `owner/repo` format (do not auto-pull an entire org).
- Validate that the repo exists and the token can read it
  (`GET /repos/{owner}/{repo}`).
- Store the watchlist in localStorage (e.g. key `gh_monitor_watchlist`, as a
  `Vec<String>` or `Vec<RepoRef>` JSON array).
- Allow removing repos from the watchlist.

### 5.2 Dashboard (landing page after login)

Render one card per repo, containing:

- Repo name + star/fork count (optional, lightweight)
- Open issue count (excluding PRs - the GitHub API returns PRs as issues too,
  so filter them out by checking the `pull_request` field in the `/issues`
  response)
- Open pull request count
- Latest CI status (from `GET /repos/{owner}/{repo}/actions/runs?per_page=1`) -
  success/failure/in_progress with a colored badge
- Last commit time on the default branch
- Clicking the card → navigates to the Repo Detail page (within the app, not
  github.com)
- Clicking a corner icon → opens the repo on github.com in a new tab

### 5.3 Repo Detail Page

- Issues tab / Pull Requests tab (clearly separated)
- Paginated list following the `Link` header (see 3.6) - **do not guess/increment
  the page number manually**
- Each row shows: title, number (#123), author avatar+name, colored labels,
  last updated date, comment count
- **Clicking a row → always opens `github.com/{owner}/{repo}/issues/{number}` or
  `/pull/{number}` in a new tab** (per the original requirement - do not attempt
  to render the full issue content inside our own app in v1)
- Filters: state (open/closed/all), label, author
- Sort: created, updated, comments

### 5.4 Rate Limit Awareness

- `GithubClient` must automatically update `RateLimitState` (global context -
  see 3.4) every time a response comes back, from the `x-ratelimit-remaining` /
  `x-ratelimit-reset` headers - **every page reads from this same state; do not
  call `/rate_limit` separately in each page.**
- Show a small indicator in the top corner of the app (green/yellow/red based
  on remaining %).
- If remaining drops below a threshold (e.g. < 10) → pause auto-refresh
  temporarily and notify the user.

### 5.5 Auto-refresh

- Configurable interval in Settings (options: manual only / 1 min / 5 min / 15 min).
- Use `gloo-timers` or `leptos::set_interval`.
- Show a "last updated" timestamp per card/page.

### 5.6 Settings Page

- Manage the token (view/remove - **never display the full token value**, show
  it masked, e.g. `ghp_****1234`).
- Manage the watchlist.
- Configure refresh interval.
- Configure theme (light/dark) - tied to tokens defined in DESIGN.md.

### Out of scope for v1 (do not implement unless the user explicitly asks)

- Any write-back to GitHub (create/edit/close issue, merge PR, comment)
- Webhooks / real-time push notifications (polling is sufficient)
- Multi-user / multiple accounts at once (v1 is a single-user browser session)
- Full OAuth App flow (with a backend to exchange tokens)
- Cross-repo full-text search (can use the GitHub search API later if needed)

---

## 6. GitHub REST API Endpoints Used

Base URL: `https://api.github.com` - every request must include these headers:
```
Authorization: Bearer {token}
Accept: application/vnd.github+json
X-GitHub-Api-Version: 2022-11-28
```

| Endpoint | Purpose |
|---|---|
| `GET /user` | Validate token, fetch username |
| `GET /repos/{owner}/{repo}` | Validate repo, fetch default_branch, stars, description |
| `GET /repos/{owner}/{repo}/issues?state=open` | List issues (filter out entries with a `pull_request` field if counting issues only) |
| `GET /repos/{owner}/{repo}/pulls?state=open` | List pull requests |
| `GET /repos/{owner}/{repo}/actions/runs?per_page=1` | Latest CI status |
| `GET /repos/{owner}/{repo}/commits?per_page=1` | Latest commit |
| `GET /rate_limit` | Check current rate limit (call once on startup; after that, rely on headers from every other response to keep `RateLimitState` updated instead) |

> CORS note: all endpoints above are under `api.github.com`, which already
> supports CORS - no proxy is required.

---

## 7. Cargo Workspace Structure

```
github-monitor/
  Cargo.toml               // [workspace] members = ["crates/*"]
  Trunk.toml
  index.html
  DESIGN.md                // separate file - all CSS/design tokens reference this
  crates/
    app/                   // Leptos UI (CSR) - the only crate aware of Trunk/wasm entry
      Cargo.toml
      src/
        main.rs
        app.rs              // root component + router
        state/
          mod.rs
          auth.rs           // AuthState
          watchlist.rs      // WatchlistState
          settings.rs       // SettingsState
          rate_limit.rs     // RateLimitState
        storage/
          mod.rs             // wrapper around gloo-storage, all keys defined here in one place
        components/
          repo_card.rs
          rate_limit_badge.rs
          issue_row.rs
          pr_row.rs
          token_form.rs
        pages/
          dashboard.rs
          repo_detail.rs
          settings.rs
          login.rs
        error.rs             // AppError for the UI layer

    github-api/             // does not depend on Leptos - reusable for a future CLI/Tauri app
      Cargo.toml
      src/
        lib.rs
        client.rs            // GithubClient + trait GithubApi
        endpoints/
          issues.rs
          pulls.rs
          repos.rs
          actions.rs
        pagination.rs         // parses the Link header
        error.rs              // ApiError

    models/                  // structs shared between app and github-api
      Cargo.toml
      src/
        lib.rs
        issue.rs
        pull_request.rs
        repo.rs
        workflow_run.rs
        rate_limit.rs
```

`app` depends on `github-api` and `models`. `github-api` depends only on
`models` (must never depend back on `app` or import anything from Leptos).

---

## 8. Design / CSS

- **Agents must not invent design tokens.** All colors, spacing, fonts,
  breakpoints, and component styles must reference `DESIGN.md` (provided
  separately). If `DESIGN.md` doesn't cover a needed component, ask the user
  first - do not guess additional styling.
- **No inline styles (`style="..."`) and no hardcoded colors in code.** Only use
  CSS variables declared in `DESIGN.md`.

---

## 9. Error Handling & Code Quality

- No `.unwrap()` / `.expect()` outside tests - use a custom `AppError` (in
  `app`) and `ApiError` (in `github-api`) together with `thiserror`.
- Every API call must have a timeout and show an error state in the UI (no
  silent failures).
- Retry logic: if a 403 is due to rate limiting (check headers), don't retry
  immediately - wait until the reset time.
- Every place that calls the API needs a loading state (skeleton or spinner,
  per DESIGN.md).

### Logging

- Use `tracing` + `tracing-wasm` for logging on wasm (never `println!`).
- Install `console_error_panic_hook` from `main.rs` so panic messages appear
  readably in the browser console instead of an opaque wasm trap.

### Testing

- `cargo test` must run without any network access or a real token.
- `models` crate: write serde tests (deserialize sample JSON from the GitHub
  API against the structs).
- `github-api` crate: write tests for parsing logic (Link header, rate limit
  headers) using mocked HTTP responses - no real network calls in unit tests.
- **No test may hit `api.github.com` for real** in the CI/unit test suite.

---

## 10. Development Phases

**Phase 1 - MVP**
- Set up the workspace (`app`, `github-api`, `models`) + the `GithubApi` trait
- Token input + validation
- Add/remove repos in watchlist (stored in localStorage)
- Dashboard showing cards with issue count and PR count only (no CI status yet)

**Phase 2 - Full Dashboard**
- Add CI status and last commit to cards
- Repo Detail page with issue/PR lists + filter/sort + pagination via the Link
  header
- Clicking a row opens github.com in a new tab
- Rate limit badge (reading from `RateLimitState`) + auto-refresh

**Phase 3 - Polish / Optional**
- Full Settings page (theme, interval)
- OAuth Device Flow (experimental, only if CORS proves reliable in testing)
- Cross-repo search within the watchlist
- (If the project genuinely grows) consider refactoring to full hexagonal
  architecture once there's more than one adapter (e.g. adding GitLab)

Agents should always ask which phase to work on if the user hasn't specified.
Default to starting with Phase 1.

---

## 11. Common Commands

```bash
trunk serve                                          # dev server + hot reload (from workspace root)
trunk build --release                                # production build (from workspace root)
cargo check --workspace --target wasm32-unknown-unknown
cargo clippy --workspace --target wasm32-unknown-unknown -- -D warnings
cargo test --workspace --exclude app                 # test only models/github-api (no wasm target needed)
```

All commands run from the workspace root: the Trunk assets (`index.html`,
`Trunk.toml`, `favicon.svg`) live at the root and point at
`crates/app/Cargo.toml`, so no `cd crates/app` is ever needed.

---

## 12. Notes for AI Agents Continuing This Work

- If a workspace/global `AGENTS-RUST.md` file exists, follow those conventions
  too (naming, module style, testing conventions). This file is project-specific
  scope only.
- Before adding a new dependency, verify it actually compiles for
  `wasm32-unknown-unknown` (many crates that depend on a full `tokio` runtime
  won't work on wasm) - `github-api` and `models` must compile on both native
  (for `cargo test`) and wasm.
- Do not add features listed under "Out of scope" without asking, even if they
  seem useful.
- Do not skip the data flow defined in 3.2 (UI → State → API), even if it looks
  like a faster shortcut.
