//! The [`GithubClient`] and its [`GithubApi`] trait.
//!
//! All HTTP access to GitHub goes through this single type so that rate-limit
//! headers can be captured centrally and the concrete implementation can be
//! swapped for a mock in tests (via the trait). Endpoint modules in
//! [`crate::endpoints`] perform the actual requests and report response headers
//! back through a capture closure so the client can update `RateLimitState`.

use std::collections::HashMap;

use gloo_net::http::Headers;
use models::{Issue, PullRequest, RateLimit, Repo, User, WorkflowRun};

use crate::error::ApiError;
use crate::pagination::{IssueParams, Pagination, PullParams};

const BASE_URL: &str = "https://api.github.com";
const USER_AGENT: &str = "lepo";
const API_VERSION: &str = "2022-11-28";

/// Callback used by endpoint functions to surface response headers (so the
/// client can capture rate-limit info without owning the response). The captured
/// reference lives as long as the client (`'a`).
pub(crate) type HeaderCapture<'a> = Box<dyn Fn(&Headers) + Send + Sync + 'a>;

/// Converts gloo-net `Headers` into a plain `HashMap` for easier inspection.
pub fn headers_to_map(headers: &Headers) -> HashMap<String, String> {
  headers.entries().collect()
}

/// Abstraction over GitHub API access used by the `app` crate.
///
/// The `app` crate depends only on this trait, never on the concrete
/// [`GithubClient`], so it can be mocked in tests.
#[allow(async_fn_in_trait)]
pub trait GithubApi {
  /// Validates the token and returns the authenticated user.
  async fn get_user(&self) -> Result<User, ApiError>;
  /// Fetches a single repository.
  async fn get_repo(&self, owner: &str, repo: &str) -> Result<Repo, ApiError>;
  /// Lists issues (one page) for a repository.
  async fn list_issues(
    &self,
    owner: &str,
    repo: &str,
    params: &IssueParams,
  ) -> Result<(Vec<Issue>, Pagination), ApiError>;
  /// Lists pull requests (one page) for a repository.
  async fn list_pulls(
    &self,
    owner: &str,
    repo: &str,
    params: &PullParams,
  ) -> Result<(Vec<PullRequest>, Pagination), ApiError>;
  /// Fetches the latest workflow run for a repository.
  async fn latest_workflow_run(
    &self,
    owner: &str,
    repo: &str,
  ) -> Result<Option<WorkflowRun>, ApiError>;
  /// Calls `/rate_limit` directly (used once at startup).
  async fn rate_limit(&self) -> Result<RateLimit, ApiError>;
  /// Captures the rate-limit headers from the last response, if any.
  fn last_rate_limit(&self) -> Option<RateLimit>;
}

// NOTE: `GithubApi` uses native `async fn`, which makes it *not*
// dyn-compatible. That is intentional: the `app` crate always uses the concrete
// [`GithubClient`], and tests construct the client directly. A dyn-compatible
// wrapper is unnecessary for this project.

/// Concrete GitHub REST client backed by `gloo-net` (fetch).
pub struct GithubClient {
  token: String,
  rate_limit: std::sync::RwLock<Option<RateLimit>>,
}

impl GithubClient {
  /// Creates a client authenticated with the given Personal Access Token.
  pub fn new(token: impl Into<String>) -> Self {
    Self {
      token: token.into(),
      rate_limit: std::sync::RwLock::new(None),
    }
  }

  /// Builds the standard GitHub API headers for an authenticated request.
  pub fn auth_headers(&self) -> Headers {
    let h = Headers::new();
    h.set("Authorization", &format!("Bearer {}", self.token));
    h.set("Accept", "application/vnd.github+json");
    h.set("X-GitHub-Api-Version", API_VERSION);
    h.set("User-Agent", USER_AGENT);
    h
  }

  /// Returns a capture closure that stores rate-limit headers into this client.
  fn capture(&self) -> HeaderCapture<'_> {
    let rl = &self.rate_limit;
    Box::new(move |headers: &Headers| {
      let map = headers_to_map(headers);
      if let (Some(parsed), Ok(mut slot)) = (RateLimit::from_headers(&map), rl.write()) {
        *slot = Some(parsed);
      }
    })
  }
}

impl GithubApi for GithubClient {
  async fn get_user(&self) -> Result<User, ApiError> {
    crate::endpoints::repos::get_user(BASE_URL, self.auth_headers(), &self.capture()).await
  }

  async fn get_repo(&self, owner: &str, repo: &str) -> Result<Repo, ApiError> {
    let (repo, _pagination) = crate::endpoints::repos::get_repo(
      BASE_URL,
      self.auth_headers(),
      owner,
      repo,
      &self.capture(),
    )
    .await?;
    Ok(repo)
  }

  async fn list_issues(
    &self,
    owner: &str,
    repo: &str,
    params: &IssueParams,
  ) -> Result<(Vec<Issue>, Pagination), ApiError> {
    crate::endpoints::issues::list_issues(
      BASE_URL,
      self.auth_headers(),
      owner,
      repo,
      params,
      &self.capture(),
    )
    .await
  }

  async fn list_pulls(
    &self,
    owner: &str,
    repo: &str,
    params: &PullParams,
  ) -> Result<(Vec<PullRequest>, Pagination), ApiError> {
    crate::endpoints::pulls::list_pulls(
      BASE_URL,
      self.auth_headers(),
      owner,
      repo,
      params,
      &self.capture(),
    )
    .await
  }

  async fn latest_workflow_run(
    &self,
    owner: &str,
    repo: &str,
  ) -> Result<Option<WorkflowRun>, ApiError> {
    crate::endpoints::actions::latest_workflow_run(
      BASE_URL,
      self.auth_headers(),
      owner,
      repo,
      &self.capture(),
    )
    .await
  }

  async fn rate_limit(&self) -> Result<RateLimit, ApiError> {
    let headers = self.auth_headers();
    let url = format!("{BASE_URL}/rate_limit");
    let resp = crate::http::get(&url, headers).await?;
    let h = resp.headers();
    let hm = headers_to_map(&h);
    map_status(resp.status(), &hm, "rate_limit failed")?;
    (self.capture())(&h);
    let body: RateLimitBody = resp
      .json()
      .await
      .map_err(|e| ApiError::Parse(e.to_string()))?;
    let rl = RateLimit {
      limit: body.resources.core.limit,
      remaining: body.resources.core.remaining,
      reset: body.resources.core.reset,
    };
    if let Ok(mut slot) = self.rate_limit.write() {
      *slot = Some(rl);
    }
    Ok(rl)
  }

  fn last_rate_limit(&self) -> Option<RateLimit> {
    self.rate_limit.read().ok().and_then(|s| *s)
  }
}

/// Maps an HTTP status into an [`ApiError`], consulting rate-limit headers.
#[allow(clippy::implicit_hasher)]
pub fn map_status(
  status: u16,
  headers: &HashMap<String, String>,
  not_found_msg: &str,
) -> Result<(), ApiError> {
  match status {
    200..=299 => Ok(()),
    401 | 403 => {
      if let Some(rl) = RateLimit::from_headers(headers)
        && rl.remaining == 0
      {
        return Err(ApiError::RateLimited { reset: rl.reset });
      }
      if status == 401 {
        Err(ApiError::Auth("token rejected or expired".into()))
      } else {
        Err(ApiError::Auth(
          "forbidden — token may lack required scopes".into(),
        ))
      }
    }
    404 => Err(ApiError::NotFound(not_found_msg.into())),
    s => Err(ApiError::Status {
      status: s,
      message: "unexpected response".into(),
    }),
  }
}

/// The `rate_limit` endpoint body shape.
#[derive(serde::Deserialize)]
struct RateLimitBody {
  resources: Resources,
}

#[derive(serde::Deserialize)]
struct Resources {
  core: LimitPair,
}

#[derive(serde::Deserialize)]
struct LimitPair {
  limit: u32,
  remaining: u32,
  reset: u64,
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::collections::HashMap;

  fn headers(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
      .iter()
      .map(|(k, v)| (k.to_string(), v.to_string()))
      .collect()
  }

  #[test]
  fn map_status_ok_range() {
    assert!(map_status(200, &headers(&[]), "not found").is_ok());
    assert!(map_status(201, &headers(&[]), "not found").is_ok());
    assert!(map_status(299, &headers(&[]), "not found").is_ok());
  }

  #[test]
  fn map_status_401_no_rate_limit_is_auth_error() {
    let result = map_status(401, &headers(&[]), "not found");
    match result {
      Err(ApiError::Auth(msg)) => assert!(msg.contains("token rejected")),
      _ => panic!("expected Auth error"),
    }
  }

  #[test]
  fn map_status_403_no_rate_limit_is_auth_error() {
    let result = map_status(403, &headers(&[]), "not found");
    match result {
      Err(ApiError::Auth(msg)) => assert!(msg.contains("forbidden")),
      _ => panic!("expected Auth error"),
    }
  }

  #[test]
  fn map_status_403_with_zero_remaining_is_rate_limited() {
    let h = headers(&[
      ("x-ratelimit-remaining", "0"),
      ("x-ratelimit-limit", "60"),
      ("x-ratelimit-reset", "1700000000"),
    ]);
    let result = map_status(403, &h, "not found");
    match result {
      Err(ApiError::RateLimited { reset }) => assert_eq!(reset, 1700000000),
      _ => panic!("expected RateLimited error"),
    }
  }

  #[test]
  fn map_status_404_is_not_found() {
    let result = map_status(404, &headers(&[]), "repo missing");
    match result {
      Err(ApiError::NotFound(msg)) => assert_eq!(msg, "repo missing"),
      _ => panic!("expected NotFound error"),
    }
  }

  #[test]
  fn map_status_500_is_unexpected_status() {
    let result = map_status(500, &headers(&[]), "not found");
    match result {
      Err(ApiError::Status { status, .. }) => assert_eq!(status, 500),
      _ => panic!("expected Status error"),
    }
  }

  #[test]
  fn map_status_422_is_unexpected_status() {
    let result = map_status(422, &headers(&[]), "not found");
    match result {
      Err(ApiError::Status { status, .. }) => assert_eq!(status, 422),
      _ => panic!("expected Status error"),
    }
  }
}
