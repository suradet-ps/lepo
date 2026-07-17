//! A GitHub repository summary returned by `GET /repos/{owner}/{repo}`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A repository as returned by the GitHub REST API.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Repo {
  /// Repository owner login (e.g. `"rust-lang"`).
  pub owner: Owner,
  /// Repository name (e.g. `"rust"`).
  pub name: String,
  /// Human-readable description, or `None` when the repo has none.
  #[serde(default)]
  pub description: Option<String>,
  /// Default branch name (e.g. `"main"`).
  pub default_branch: String,
  /// Star count.
  pub stargazers_count: u32,
  /// Fork count.
  pub forks_count: u32,
  /// Whether the repository is archived.
  #[serde(default)]
  pub archived: bool,
  /// Whether the repository is private.
  #[serde(default)]
  pub private: bool,
  /// HTML URL on github.com.
  pub html_url: String,
  /// ISO-8601 timestamp of the last push.
  #[serde(default)]
  pub pushed_at: Option<DateTime<Utc>>,
}

/// Minimal owner account information embedded in a [`Repo`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Owner {
  /// Owner login (user or org name).
  pub login: String,
  /// Avatar image URL.
  #[serde(default)]
  pub avatar_url: Option<String>,
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;

  #[test]
  fn deserialize_repo_full_payload() {
    let value = json!({
        "owner": {"login": "rust-lang", "avatar_url": "https://avatars/a.png"},
        "name": "rust",
        "description": "A language.",
        "default_branch": "master",
        "stargazers_count": 90000,
        "forks_count": 12000,
        "archived": false,
        "private": false,
        "html_url": "https://github.com/rust-lang/rust",
        "pushed_at": "2024-01-02T03:04:05Z"
    });
    let repo: Repo = serde_json::from_value(value).expect("valid repo json");
    assert_eq!(repo.owner.login, "rust-lang");
    assert_eq!(repo.name, "rust");
    assert_eq!(repo.default_branch, "master");
    assert_eq!(repo.stargazers_count, 90000);
    assert_eq!(
      repo.pushed_at.map(|d| d.to_rfc3339()),
      Some("2024-01-02T03:04:05+00:00".to_string())
    );
  }

  #[test]
  fn deserialize_repo_missing_optional_fields() {
    let value = json!({
        "owner": {"login": "octocat"},
        "name": "hello",
        "default_branch": "main",
        "stargazers_count": 1,
        "forks_count": 2,
        "html_url": "https://github.com/octocat/hello"
    });
    let repo: Repo = serde_json::from_value(value).expect("valid repo json");
    assert_eq!(repo.description, None);
    assert!(!repo.archived);
    assert!(!repo.private);
    assert_eq!(repo.pushed_at, None);
    assert_eq!(repo.owner.avatar_url, None);
  }
}
