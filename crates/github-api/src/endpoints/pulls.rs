//! `GET /repos/{owner}/{repo}/pulls` endpoint.

use gloo_net::http::Headers;
use models::PullRequest;

use crate::client::{HeaderCapture, headers_to_map, map_status};
use crate::error::ApiError;
use crate::http;
use crate::pagination::{Pagination, PullParams};

/// Lists pull requests for a repository.
pub async fn list_pulls(
  base: &str,
  headers: Headers,
  owner: &str,
  repo: &str,
  params: &PullParams,
  capture: &HeaderCapture<'_>,
) -> Result<(Vec<PullRequest>, Pagination), ApiError> {
  let url = format!("{base}/repos/{owner}/{repo}/pulls?{}", params.to_query());
  let resp = http::get(&url, headers).await?;
  let h = resp.headers();
  let hm = headers_to_map(&h);
  map_status(resp.status(), &hm, "pulls not found")?;
  capture(&h);
  let body: Vec<PullRequest> = resp
    .json()
    .await
    .map_err(|e| ApiError::Parse(e.to_string()))?;
  let pagination = Pagination::from_headers(&hm);
  Ok((body, pagination))
}
