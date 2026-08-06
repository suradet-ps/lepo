//! A GitHub issue as returned by `GET /repos/{owner}/{repo}/issues`.
//!
//! Note: the GitHub API returns pull requests through the same endpoint. A
//! genuine issue has no `pull_request` field, so the [`Issue::is_pr`] helper
//! lets callers filter PRs out when counting "issues only".

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A label attached to an issue or pull request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Label {
  /// Label display name.
  pub name: String,
  /// Hex color (e.g. `"0e8a16"`) without a leading `#`.
  pub color: String,
}

/// Author of an issue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Author {
  /// Login handle.
  pub login: String,
  /// Avatar image URL.
  #[serde(default)]
  pub avatar_url: Option<String>,
}

/// A GitHub issue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Issue {
  /// Issue number (unique within the repo).
  pub number: u64,
  /// Issue title.
  pub title: String,
  /// Author of the issue.
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
  /// Attached labels.
  #[serde(default)]
  pub labels: Vec<Label>,
  /// Present only when this entry is actually a pull request.
  #[serde(default)]
  pub pull_request: Option<serde_json::Value>,
  /// HTML URL on github.com.
  pub html_url: String,
}

impl Issue {
  /// Returns `true` when this entry is actually a pull request (the GitHub
  /// API returns PRs inside the issues endpoint).
  pub const fn is_pr(&self) -> bool {
    self.pull_request.is_some()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;

  #[test]
  fn deserialize_issue_excludes_pr_field() {
    let value = json!({
        "number": 42,
        "title": "Bug fix",
        "user": {"login": "alice", "avatar_url": "https://a/aa.png"},
        "state": "open",
        "comments": 3,
        "updated_at": "2024-02-01T00:00:00Z",
        "created_at": "2024-01-01T00:00:00Z",
        "labels": [{"name": "bug", "color": "d73a4a"}],
        "html_url": "https://github.com/o/r/issues/42"
    });
    let issue: Issue = serde_json::from_value(value).expect("valid issue json");
    assert!(!issue.is_pr());
    assert_eq!(issue.labels.len(), 1);
    assert_eq!(issue.labels[0].color, "d73a4a");
    assert_eq!(issue.user.unwrap().login, "alice");
  }

  #[test]
  fn deserialize_issue_detects_pr() {
    let value = json!({
        "number": 7,
        "title": "Add feature",
        "state": "open",
        "html_url": "https://github.com/o/r/issues/7",
        "pull_request": {"url": "https://api.github.com/..."}
    });
    let issue: Issue = serde_json::from_value(value).expect("valid pr-as-issue json");
    assert!(issue.is_pr());
    assert!(issue.labels.is_empty());
  }
}
