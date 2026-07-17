# Security Policy

## Supported Versions

Lepo is currently in **v1 (read-only)**. The latest `main` / `master` branch is the only
supported version.

| Version | Supported |
|---------|-----------|
| main    | ✅ Yes    |
| < 1.0   | ❌ No     |

## Reporting a Vulnerability

If you discover a security vulnerability in Lepo, please **do not open a public GitHub
issue**. Instead, report it privately so we can triage and release a fix before details
are disclosed.

Please include:

- A clear description of the vulnerability and its impact
- Steps to reproduce (or a proof-of-concept)
- The affected version / commit
- Any suggested mitigation, if known

We will acknowledge receipt within a few business days and keep you informed of progress
toward a fix and disclosure.

## Scope & Threat Model

Lepo is a **static, client-side WASM app** with no backend of our own. Relevant concerns:

- **Token handling:** the GitHub Personal Access Token is stored only in the browser's
  `localStorage` and is sent **only** to `https://api.github.com` (GitHub's CORS-enabled
  REST API). It is never transmitted to any server operated by the Lepo project.
- **CSP:** `vercel.json` ships a `Content-Security-Policy` that restricts `connect-src` to
  `https://api.github.com` and disables inline script execution beyond what WASM requires.
- **No `unsafe`:** the workspace denies `unsafe_code` at the lint level.
- **Out of scope (v1):** any write-back to GitHub (creating/editing issues, merging PRs,
  commenting) is intentionally **not implemented**. If you find a path that enables it,
  that is a vulnerability.

## Hardening Checklist (for deployments)

- Serve over HTTPS only.
- Keep the `vercel.json` security headers intact (CSP, `X-Frame-Options: DENY`,
  `X-Content-Type-Options: nosniff`, `Referrer-Policy`).
- Use the narrowest fine-grained GitHub token scope necessary (read-only on Issues, Pull
  requests, Metadata, Contents, Actions).
