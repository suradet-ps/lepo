//! `GET /repos/{owner}/{repo}/issues` endpoint. PRs are filtered out by callers
//! using [`models::Issue::is_pr`].

use gloo_net::http::Headers;
use models::Issue;

use crate::client::{HeaderCapture, headers_to_map, map_status};
use crate::error::ApiError;
use crate::http;
use crate::pagination::{IssueParams, Pagination};

/// Builds the issues list URL for the given parameters.
pub(crate) fn issues_url(base: &str, owner: &str, repo: &str, params: &IssueParams) -> String {
  format!("{base}/repos/{owner}/{repo}/issues?{}", params.to_query())
}

/// Lists issues for a repository, following one page of the `Link` cursor.
pub async fn list_issues(
  base: &str,
  headers: Headers,
  owner: &str,
  repo: &str,
  params: &IssueParams,
  capture: &HeaderCapture<'_>,
) -> Result<(Vec<Issue>, Pagination), ApiError> {
  let url = issues_url(base, owner, repo, params);
  let resp = http::get(&url, headers).await?;
  let h = resp.headers();
  capture(&h);
  let hm = headers_to_map(&h);
  map_status(resp.status(), &hm, "issues not found")?;
  let body: Vec<Issue> = resp
    .json()
    .await
    .map_err(|e| ApiError::Parse(e.to_string()))?;
  let pagination = Pagination::from_headers(&hm);
  Ok((body, pagination))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::pagination::IssueParams;

  #[test]
  fn issues_url_includes_default_query() {
    let url = issues_url(
      "https://api.github.com",
      "rust-lang",
      "lepo",
      &IssueParams::default(),
    );
    assert_eq!(
      url,
      "https://api.github.com/repos/rust-lang/lepo/issues?state=open&per_page=30&sort=updated"
    );
  }

  #[test]
  fn issues_url_includes_every_parameter() {
    let params = IssueParams {
      state: "closed".into(),
      labels: vec!["bug".into(), "needs-triage".into()],
      sort: "created".into(),
      creator: "octocat".into(),
      per_page: 10,
    };
    let url = issues_url("https://api.github.com", "a", "b", &params);
    assert_eq!(
      url,
      "https://api.github.com/repos/a/b/issues?state=closed&per_page=10&labels=bug%2Cneeds-triage&sort=created&creator=octocat"
    );
  }
}
