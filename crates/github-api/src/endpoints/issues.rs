//! `GET /repos/{owner}/{repo}/issues` endpoint. PRs are filtered out by callers
//! using [`models::Issue::is_pr`].

use gloo_net::http::{Headers, Request};
use models::Issue;

use crate::client::{HeaderCapture, headers_to_map, map_status};
use crate::error::ApiError;
use crate::pagination::{IssueParams, Pagination};

/// Lists issues for a repository, following one page of the `Link` cursor.
pub async fn list_issues(
  base: &str,
  headers: Headers,
  owner: &str,
  repo: &str,
  params: &IssueParams,
  capture: &HeaderCapture<'_>,
) -> Result<(Vec<Issue>, Pagination), ApiError> {
  let url = format!("{base}/repos/{owner}/{repo}/issues?{}", params.to_query());
  let resp = Request::get(&url)
    .headers(headers)
    .send()
    .await
    .map_err(|e| ApiError::Request(e.to_string()))?;
  let h = resp.headers();
  let hm = headers_to_map(&h);
  map_status(resp.status(), &hm, "issues not found")?;
  capture(&h);
  let body: Vec<Issue> = resp
    .json()
    .await
    .map_err(|e| ApiError::Parse(e.to_string()))?;
  let pagination = Pagination::from_headers(&hm);
  Ok((body, pagination))
}
