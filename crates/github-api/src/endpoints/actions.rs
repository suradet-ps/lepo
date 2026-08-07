//! `GET /repos/{owner}/{repo}/actions/runs?per_page=1` endpoint.

use gloo_net::http::Headers;
use models::WorkflowRun;

use crate::client::{HeaderCapture, headers_to_map, map_status};
use crate::error::ApiError;
use crate::http;

/// A workflow runs list response wraps the runs in a `workflow_runs` array.
#[derive(serde::Deserialize)]
struct RunsResponse {
  workflow_runs: Vec<WorkflowRun>,
}

/// Builds the workflow-runs URL (one page, newest first).
pub(crate) fn workflow_runs_url(base: &str, owner: &str, repo: &str) -> String {
  format!("{base}/repos/{owner}/{repo}/actions/runs?per_page=1")
}

/// Fetches the most recent workflow run (if any) for a repository.
pub async fn latest_workflow_run(
  base: &str,
  headers: Headers,
  owner: &str,
  repo: &str,
  capture: &HeaderCapture<'_>,
) -> Result<Option<WorkflowRun>, ApiError> {
  let url = workflow_runs_url(base, owner, repo);
  let resp = http::get(&url, headers).await?;
  let h = resp.headers();
  capture(&h);
  let hm = headers_to_map(&h);
  map_status(resp.status(), &hm, "workflow runs not found")?;
  let body: RunsResponse = resp
    .json()
    .await
    .map_err(|e| ApiError::Parse(e.to_string()))?;
  Ok(body.workflow_runs.into_iter().next())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn workflow_runs_url_uses_owner_and_name_with_per_page() {
    assert_eq!(
      workflow_runs_url("https://api.github.com", "rust-lang", "lepo"),
      "https://api.github.com/repos/rust-lang/lepo/actions/runs?per_page=1"
    );
  }
}
