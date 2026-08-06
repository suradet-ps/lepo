//! Rate limit information parsed from response headers.
//!
//! GitHub reports the current rate-limit budget on every REST response via the
//! `x-ratelimit-*` headers. [`RateLimit`] mirrors those, and
//! [`RateLimit::from_headers`] parses them without any network dependency so it
//! can be unit-tested offline.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Current GitHub API rate-limit budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RateLimit {
  /// Remaining requests in the current window.
  pub remaining: u32,
  /// Total request allowance in the current window.
  pub limit: u32,
  /// Unix epoch seconds at which the window resets.
  pub reset: u64,
}

impl RateLimit {
  /// Builds a [`RateLimit`] from a header map, reading the `x-ratelimit-*`
  /// keys. Returns `Ok` only when all three headers are present and parse.
  pub fn from_headers(headers: &HashMap<String, String>) -> Option<Self> {
    let remaining = headers.get("x-ratelimit-remaining")?.parse().ok()?;
    let limit = headers.get("x-ratelimit-limit")?.parse().ok()?;
    let reset = headers.get("x-ratelimit-reset")?.parse().ok()?;
    Some(Self {
      remaining,
      limit,
      reset,
    })
  }

  /// Fraction (0.0–1.0) of the budget remaining.
  pub fn fraction_remaining(&self) -> f64 {
    if self.limit == 0 {
      return 0.0;
    }
    self.remaining as f64 / self.limit as f64
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn header_map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
      .iter()
      .map(|(k, v)| (k.to_string(), v.to_string()))
      .collect()
  }

  #[test]
  fn from_headers_parses_all_three() {
    let h = header_map(&[
      ("x-ratelimit-remaining", "42"),
      ("x-ratelimit-limit", "60"),
      ("x-ratelimit-reset", "1700000000"),
    ]);
    let rl = RateLimit::from_headers(&h).expect("headers present");
    assert_eq!(rl.remaining, 42);
    assert_eq!(rl.limit, 60);
    assert_eq!(rl.reset, 1_700_000_000);
    assert!((rl.fraction_remaining() - 0.7).abs() < 1e-9);
  }

  #[test]
  fn from_headers_missing_key_returns_none() {
    let h = header_map(&[("x-ratelimit-remaining", "42")]);
    assert!(RateLimit::from_headers(&h).is_none());
  }

  #[test]
  fn from_headers_unparseable_value_returns_none() {
    let h = header_map(&[
      ("x-ratelimit-remaining", "abc"),
      ("x-ratelimit-limit", "60"),
      ("x-ratelimit-reset", "1700000000"),
    ]);
    assert!(RateLimit::from_headers(&h).is_none());
  }
}
