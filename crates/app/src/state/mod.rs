//! Global UI state: authentication, watchlist, settings, and rate limit.
//!
//! These are provided via Leptos context (see AGENTS.md §3.4) and never
//! duplicated locally in components. Pages read them with `expect_context`.

pub mod auth;
pub mod rate_limit;
pub mod settings;
pub mod watchlist;

pub use auth::{AuthState, AuthStatus};
pub use rate_limit::RateLimitState;
pub use settings::{RefreshInterval, SettingsState, Theme};
pub use watchlist::{RepoRef, WatchlistState};
