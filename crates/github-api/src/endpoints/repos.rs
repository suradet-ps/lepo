//! `GET /user` and `GET /repos/{owner}/{repo}` endpoints.

use gloo_net::http::Headers;
use models::{Repo, User};

use crate::client::{HeaderCapture, headers_to_map, map_status};
use crate::error::ApiError;
use crate::http;
use crate::pagination::Pagination;

/// Builds the authenticated-user URL.
pub(crate) fn user_url(base: &str) -> String {
  format!("{base}/user")
}

/// Builds the repository URL for owner/name.
pub(crate) fn repo_url(base: &str, owner: &str, repo: &str) -> String {
  format!("{base}/repos/{owner}/{repo}")
}

/// Fetches the authenticated user (validates the token).
pub async fn get_user(
  base: &str,
  headers: Headers,
  capture: &HeaderCapture<'_>,
) -> Result<User, ApiError> {
  let url = user_url(base);
  let resp = http::get(&url, headers).await?;
  let h = resp.headers();
  capture(&h);
  map_status(resp.status(), &headers_to_map(&h), "user request failed")?;
  resp
    .json::<User>()
    .await
    .map_err(|e| ApiError::Parse(e.to_string()))
}

/// Fetches a single repository by owner/name.
pub async fn get_repo(
  base: &str,
  headers: Headers,
  owner: &str,
  repo: &str,
  capture: &HeaderCapture<'_>,
) -> Result<(Repo, Pagination), ApiError> {
  let url = repo_url(base, owner, repo);
  let resp = http::get(&url, headers).await?;
  let h = resp.headers();
  capture(&h);
  let hm = headers_to_map(&h);
  map_status(resp.status(), &hm, "repository not found")?;
  let body: Repo = resp
    .json()
    .await
    .map_err(|e| ApiError::Parse(e.to_string()))?;
  let pagination = Pagination::from_headers(&hm);
  Ok((body, pagination))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn user_url_points_at_the_user_endpoint() {
    assert_eq!(
      user_url("https://api.github.com"),
      "https://api.github.com/user"
    );
  }

  #[test]
  fn repo_url_uses_owner_and_name() {
    assert_eq!(
      repo_url("https://api.github.com", "rust-lang", "lepo"),
      "https://api.github.com/repos/rust-lang/lepo"
    );
  }
}
