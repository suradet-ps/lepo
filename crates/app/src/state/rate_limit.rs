//! Rate-limit state, updated automatically from response headers by the API
//! client. Pages read this; they never call `/rate_limit` themselves.

use leptos::prelude::*;

use github_api::GithubApi;
use models::RateLimit;

/// Global rate-limit budget reflected from the last API response.
#[derive(Clone, Copy)]
pub struct RateLimitState {
  /// The most recent rate-limit snapshot, if any response has arrived.
  pub limit: RwSignal<Option<RateLimit>>,
  /// Wall-clock timestamp (RFC3339) of the last successful data sync.
  pub last_updated: RwSignal<Option<String>>,
}

impl RateLimitState {
  /// Creates an empty (unknown) rate-limit state.
  pub fn new() -> RateLimitState {
    RateLimitState {
      limit: RwSignal::new(None),
      last_updated: RwSignal::new(None),
    }
  }

  /// Updates the snapshot from a client after a request, stamping the sync time.
  pub fn update(&self, client: &github_api::GithubClient) {
    if let Some(rl) = client.last_rate_limit() {
      self.limit.set(Some(rl));
      self
        .last_updated
        .set(Some(crate::time::now().format("%H:%M").to_string()));
    }
  }

  /// Clears the snapshot (e.g. on logout).
  pub fn reset(&self) {
    self.limit.set(None);
    self.last_updated.set(None);
  }
}

impl Default for RateLimitState {
  fn default() -> Self {
    Self::new()
  }
}
