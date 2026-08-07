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

/// Fetches the most recent workflow run (if any) for a repository.
pub async fn latest_workflow_run(
  base: &str,
  headers: Headers,
  owner: &str,
  repo: &str,
  capture: &HeaderCapture<'_>,
) -> Result<Option<WorkflowRun>, ApiError> {
  let url = format!("{base}/repos/{owner}/{repo}/actions/runs?per_page=1");
  let resp = http::get(&url, headers).await?;
  let h = resp.headers();
  let hm = headers_to_map(&h);
  map_status(resp.status(), &hm, "workflow runs not found")?;
  capture(&h);
  let body: RunsResponse = resp
    .json()
    .await
    .map_err(|e| ApiError::Parse(e.to_string()))?;
  Ok(body.workflow_runs.into_iter().next())
}
