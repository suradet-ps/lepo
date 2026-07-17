//! Watchlist state: the set of repos being monitored, persisted to localStorage.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::storage;

/// A normalized `owner/repo` reference stored in the watchlist.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RepoRef {
  /// Repository owner (user or org login).
  pub owner: String,
  /// Repository name.
  pub name: String,
}

impl RepoRef {
  /// Parses an `owner/repo` string. Rejects anything without exactly one `/`.
  pub fn parse(s: &str) -> Result<RepoRef, AppError> {
    let s = s.trim();
    match s.split_once('/') {
      Some((owner, name)) if !owner.is_empty() && !name.is_empty() => Ok(RepoRef {
        owner: owner.to_string(),
        name: name.to_string(),
      }),
      _ => Err(AppError::InvalidRepoRef(s.to_string())),
    }
  }

  /// Returns the canonical `owner/repo` form.
  pub fn as_str(&self) -> String {
    format!("{}/{}", self.owner, self.name)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_valid_ref() {
    let r = RepoRef::parse("rust-lang/rust").unwrap();
    assert_eq!(r.owner, "rust-lang");
    assert_eq!(r.name, "rust");
    assert_eq!(r.as_str(), "rust-lang/rust");
  }

  #[test]
  fn parse_rejects_missing_slash() {
    assert!(RepoRef::parse("rustlang").is_err());
  }

  #[test]
  fn parse_rejects_empty_parts() {
    assert!(RepoRef::parse("/rust").is_err());
    assert!(RepoRef::parse("rust/").is_err());
  }
}

/// Global watchlist state held in context.
#[derive(Clone, Copy)]
pub struct WatchlistState {
  /// The current list of monitored repos.
  pub repos: RwSignal<Vec<RepoRef>>,
}

impl WatchlistState {
  /// Loads the watchlist from localStorage (empty if none/absent).
  pub fn from_storage() -> WatchlistState {
    let repos = storage::load_json::<Vec<RepoRef>>(storage::KEY_WATCHLIST).unwrap_or_default();
    WatchlistState {
      repos: RwSignal::new(repos),
    }
  }

  /// Persists the current watchlist to localStorage.
  pub fn save(&self) -> Result<(), AppError> {
    storage::save_json(storage::KEY_WATCHLIST, &self.repos.get())
  }

  /// Adds a repo if not already present; returns false if it was a duplicate.
  pub fn add(&self, repo: RepoRef) -> bool {
    let mut added = false;
    self.repos.update(|list| {
      if list.iter().any(|r| r == &repo) {
        added = false;
      } else {
        list.push(repo);
        added = true;
      }
    });
    added
  }

  /// Removes a repo by reference.
  pub fn remove(&self, repo: &RepoRef) {
    self.repos.update(|list| {
      list.retain(|r| r != repo);
    });
  }

  /// Whether the given repo is already in the list.
  pub fn contains(&self, repo: &RepoRef) -> bool {
    self.repos.get_untracked().iter().any(|r| r == repo)
  }
}
