//! `GET /user` and `GET /repos/{owner}/{repo}` endpoints.

use gloo_net::http::Headers;
use models::{Repo, User};

use crate::client::{HeaderCapture, headers_to_map, map_status};
use crate::error::ApiError;
use crate::http;
use crate::pagination::Pagination;

/// Fetches the authenticated user (validates the token).
pub async fn get_user(
  base: &str,
  headers: Headers,
  capture: &HeaderCapture<'_>,
) -> Result<User, ApiError> {
  let url = format!("{base}/user");
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
  let url = format!("{base}/repos/{owner}/{repo}");
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
