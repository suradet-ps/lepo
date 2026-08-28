# Contributing to Lepo

Thanks for your interest in contributing to **Lepo** - a GitHub repository monitor built
with Leptos 0.8 (CSR) and Trunk.

This guide covers how to set up your environment, the conventions we follow, and how to
get your changes merged. For architecture and UI/UX details, read
[AGENTS.md](../AGENTS.md) and [DESIGN.md](./DESIGN.md) first.

## Prerequisites

- [Rust](https://rustup.rs) - the repo pins the `wasm32-unknown-unknown` target via
  `rust-toolchain.toml`. Use a recent stable toolchain (`rustup update stable`).
- The `wasm32-unknown-unknown` target:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```
- [Trunk](https://trunkrs.dev) - the WASM bundler/dev server:
  ```bash
  cargo install trunk
  ```

## Project setup

```bash
# Clone and enter the repo
git clone https://github.com/anomalyco/lepo
cd lepo

# Start the dev server with HMR (http://127.0.0.1:3000)
trunk serve --port 3000 --open
```

## Development workflow

```bash
# Type-check only (no codegen)
cargo check --workspace --target wasm32-unknown-unknown

# Lint
cargo clippy --workspace --target wasm32-unknown-unknown -- -D clippy::correctness -D clippy::suspicious

# Check formatting
cargo fmt --all --check

# Run library unit tests (models / github-api only - no wasm target needed)
cargo test --workspace --exclude app

# Start the dev server with HMR (http://127.0.0.1:3000)
trunk serve --port 3000 --open

# Production build → dist/
trunk build --release
```

> CI (`.github/workflows/ci.yml`) runs `cargo check`, `cargo clippy`, `cargo fmt --check`,
> `cargo test`, and a `trunk build --release` on every push/PR. All jobs must pass before
> a PR can be merged.

## Code style & conventions

- **Formatting:** Run `cargo fmt` before committing. The project uses `rustfmt.toml` with
  2-space indentation and `edition = "2024"`. `cargo fmt --check` is enforced in CI.
- **Lints:** `unsafe_code` and `unused_must_use` are denied at the workspace level; clippy
  `all` / `pedantic` / `nursery` are set to `warn`. Treat clippy warnings as errors for
  correctness/suspicious lints (`-D clippy::correctness -D clippy::suspicious`).
- **Edition:** Rust 2024.
- **No `unsafe`:** the workspace denies `unsafe_code`; keep it that way.
- **Error handling:** no `unwrap()` / `expect()` outside `#[cfg(test)]`. Use the
  `ApiError` / `AppError` types.
- **Pure CSS:** styling lives in `crates/app/src/style.css` with design tokens from
  `DESIGN.md`. No CSS-in-JS, no UI frameworks. No inline `style=` and no hardcoded colors.
- **Architecture (data flow):** UI → State → API. Components never call the GitHub API
  directly; they go through state, which talks to `GithubClient` implementing `GithubApi`.
  See [AGENTS.md](../AGENTS.md) §3.

## Commit messages

We follow [Conventional Commits](https://www.conventionalcommits.org):

```
build:    bump trunk build command
feat:     add CI status to dashboard cards
fix:      correct rate-limit badge fraction
style:    reformat with 2-space indent
refactor: extract github client into github-api crate
test:     add Link-header pagination tests
docs:     document deploy steps in README
ci:       add trunk build --release job
```

## Pull requests

1. Fork and create a feature branch off `main` (e.g. `feat/ci-status`).
2. Keep PRs focused - one logical change per PR.
3. Ensure local `cargo fmt --check`, `cargo clippy`, and `cargo test` all pass.
4. Describe what the change does and why; link any related issues.
5. Add screenshots/GIFs for UI changes where possible.
6. CI must be green before review.

## Reporting bugs & security issues

- For general bugs, open a GitHub issue with reproduction steps, expected vs. actual
  behavior, and your environment (OS, Rust version, browser).
- For security concerns (XSS, auth bypass, token leakage), please report privately
  rather than opening a public issue. See [SECURITY.md](./SECURITY.md).

## License

By contributing, you agree that your contributions will be licensed under the
[MIT License](../LICENSE).
