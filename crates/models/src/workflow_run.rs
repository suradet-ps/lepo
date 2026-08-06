//! Latest GitHub Actions workflow run, from `GET /repos/{owner}/{repo}/actions/runs`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Workflow run status as reported by the GitHub API.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowStatus {
  /// Run completed successfully.
  Completed,
  /// Run is currently executing.
  InProgress,
  /// Run is queued.
  Queued,
  /// Run was requested but not yet queued.
  Requested,
  /// Run was cancelled.
  Cancelled,
  /// Any other status (e.g. `waiting`, `pending`).
  #[serde(other)]
  Other,
}

impl WorkflowStatus {
  /// Returns `true` when the run is in a non-terminal, active state.
  pub const fn is_active(&self) -> bool {
    matches!(self, Self::InProgress | Self::Queued | Self::Requested)
  }
}

/// Conclusion attached to a completed workflow run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowConclusion {
  /// All jobs succeeded.
  Success,
  /// At least one job failed.
  Failure,
  /// Run was cancelled.
  Cancelled,
  /// Run was skipped.
  Skipped,
  /// Run was neutral.
  Neutral,
  /// Run timed out.
  TimedOut,
  /// Any other conclusion.
  #[serde(other)]
  Other,
}

/// The latest workflow run for a repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowRun {
  /// Numeric run id.
  pub id: u64,
  /// Head commit message for the run.
  #[serde(default)]
  pub head_branch: Option<String>,
  /// Run status.
  pub status: WorkflowStatus,
  /// Conclusion (only present for completed runs).
  #[serde(default)]
  pub conclusion: Option<WorkflowConclusion>,
  /// Timestamp the run was created.
  #[serde(default)]
  pub created_at: Option<DateTime<Utc>>,
  /// Timestamp the run finished.
  #[serde(default)]
  pub updated_at: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;

  #[test]
  fn deserialize_completed_success_run() {
    let value = json!({
        "id": 1,
        "head_branch": "main",
        "status": "completed",
        "conclusion": "success",
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:05:00Z"
    });
    let run: WorkflowRun = serde_json::from_value(value).expect("valid run json");
    assert_eq!(run.status, WorkflowStatus::Completed);
    assert_eq!(run.conclusion, Some(WorkflowConclusion::Success));
    assert!(!run.status.is_active());
  }

  #[test]
  fn deserialize_active_run_status() {
    let value = json!({"id": 2, "status": "in_progress"});
    let run: WorkflowRun = serde_json::from_value(value).expect("valid run json");
    assert!(run.status.is_active());
    assert_eq!(run.conclusion, None);
  }

  #[test]
  fn deserialize_unknown_status_is_other() {
    let value = json!({"id": 3, "status": "pending"});
    let run: WorkflowRun = serde_json::from_value(value).expect("valid run json");
    assert_eq!(run.status, WorkflowStatus::Other);
  }
}
