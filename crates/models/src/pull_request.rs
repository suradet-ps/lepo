//! A GitHub pull request as returned by `GET /repos/{owner}/{repo}/pulls`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::issue::{Author, Label};

/// A GitHub pull request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PullRequest {
  /// Pull request number (unique within the repo).
  pub number: u64,
  /// Pull request title.
  pub title: String,
  /// Author of the PR.
  #[serde(default)]
  pub user: Option<Author>,
  /// Open/closed state.
  pub state: String,
  /// Number of comments.
  #[serde(default)]
  pub comments: u32,
  /// Last update timestamp.
  #[serde(default)]
  pub updated_at: Option<DateTime<Utc>>,
  /// Created timestamp.
  #[serde(default)]
  pub created_at: Option<DateTime<Utc>>,
  /// Whether the PR is a draft.
  #[serde(default)]
  pub draft: bool,
  /// Attached labels.
  #[serde(default)]
  pub labels: Vec<Label>,
  /// HTML URL on github.com.
  pub html_url: String,
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;

  #[test]
  fn deserialize_pull_request() {
    let value = json!({
        "number": 99,
        "title": "Draft: refactor",
        "user": {"login": "bob"},
        "state": "open",
        "comments": 1,
        "updated_at": "2024-03-01T00:00:00Z",
        "created_at": "2024-02-15T00:00:00Z",
        "draft": true,
        "labels": [],
        "html_url": "https://github.com/o/r/pull/99"
    });
    let pr: PullRequest = serde_json::from_value(value).expect("valid pr json");
    assert_eq!(pr.number, 99);
    assert!(pr.draft);
    assert_eq!(pr.user.unwrap().login, "bob");
  }
}
