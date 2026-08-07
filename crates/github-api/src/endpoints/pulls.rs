//! `GET /repos/{owner}/{repo}/pulls` endpoint.

use gloo_net::http::Headers;
use models::PullRequest;

use crate::client::{HeaderCapture, headers_to_map, map_status};
use crate::error::ApiError;
use crate::http;
use crate::pagination::{Pagination, PullParams};

/// Builds the pulls list URL for the given parameters.
pub(crate) fn pulls_url(base: &str, owner: &str, repo: &str, params: &PullParams) -> String {
  format!("{base}/repos/{owner}/{repo}/pulls?{}", params.to_query())
}

/// Lists pull requests for a repository.
pub async fn list_pulls(
  base: &str,
  headers: Headers,
  owner: &str,
  repo: &str,
  params: &PullParams,
  capture: &HeaderCapture<'_>,
) -> Result<(Vec<PullRequest>, Pagination), ApiError> {
  let url = pulls_url(base, owner, repo, params);
  let resp = http::get(&url, headers).await?;
  let h = resp.headers();
  capture(&h);
  let hm = headers_to_map(&h);
  map_status(resp.status(), &hm, "pulls not found")?;
  let body: Vec<PullRequest> = resp
    .json()
    .await
    .map_err(|e| ApiError::Parse(e.to_string()))?;
  let pagination = Pagination::from_headers(&hm);
  Ok((body, pagination))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::pagination::PullParams;

  #[test]
  fn pulls_url_includes_default_query() {
    let url = pulls_url(
      "https://api.github.com",
      "rust-lang",
      "lepo",
      &PullParams::default(),
    );
    assert_eq!(
      url,
      "https://api.github.com/repos/rust-lang/lepo/pulls?state=open&per_page=30&sort=updated"
    );
  }

  #[test]
  fn pulls_url_includes_state_and_size() {
    let params = PullParams {
      state: "all".into(),
      sort: "created".into(),
      per_page: 100,
    };
    let url = pulls_url("https://api.github.com", "a", "b", &params);
    assert_eq!(
      url,
      "https://api.github.com/repos/a/b/pulls?state=all&per_page=100&sort=created"
    );
  }
}
