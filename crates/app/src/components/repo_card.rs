//! A dashboard card summarizing one monitored repo (AGENTS.md §5.2, Phase 1).

use leptos::prelude::*;

use models::{Issue, PullRequest, Repo, WorkflowConclusion, WorkflowRun, WorkflowStatus};

use crate::state::RepoRef;

/// Data needed to render a [`RepoCard`].
#[derive(Clone, PartialEq, Eq)]
pub struct RepoCardData {
  /// The repo reference.
  pub r#ref: RepoRef,
  /// Fetched repo metadata, if available.
  pub repo: Option<Repo>,
  /// Raw issues (PRs included) - filtered by the card.
  pub issues: Vec<Issue>,
  /// Pull requests.
  pub pulls: Vec<PullRequest>,
  /// Latest CI status (most recent workflow run), if any.
  pub ci: Option<WorkflowRun>,
  /// Estimated number of open issues across all pages (0 = use `issues.len()`).
  pub total_open_issues: usize,
  /// Estimated number of open PRs across all pages (0 = use `pulls.len()`).
  pub total_open_prs: usize,
  /// Whether `total_open_issues` is an upper bound derived from pagination.
  pub open_issues_estimate: bool,
  /// Whether `total_open_prs` is an upper bound derived from pagination.
  pub open_prs_estimate: bool,
}

impl RepoCardData {
  /// Open issues excluding pull requests. Uses the total count when available.
  pub fn open_issue_count(&self) -> usize {
    if self.total_open_issues > 0 {
      self.total_open_issues
    } else {
      self.issues.iter().filter(|i| !i.is_pr()).count()
    }
  }

  /// Open pull requests. Uses the total count when available.
  pub const fn open_prs_count(&self) -> usize {
    if self.total_open_prs > 0 {
      self.total_open_prs
    } else {
      self.pulls.len()
    }
  }

  /// Human-readable "last push" label from the repo metadata.
  pub fn last_push_label(&self) -> String {
    self
      .repo
      .as_ref()
      .and_then(|r| r.pushed_at.as_ref())
      .map_or_else(|| "-".to_string(), format_relative)
  }

  /// CI status as a (dot-class, label) pair for the badge.
  pub fn ci_badge(&self) -> (&'static str, &'static str) {
    self
      .ci
      .as_ref()
      .map_or(("ci-dot--none", "-"), |run| match run.status {
        WorkflowStatus::Completed => match run.conclusion {
          Some(WorkflowConclusion::Success) => ("ci-dot--pass", "Pass"),
          Some(WorkflowConclusion::Failure) => ("ci-dot--fail", "Fail"),
          Some(WorkflowConclusion::Cancelled) => ("ci-dot--run", "Cancel"),
          Some(WorkflowConclusion::Skipped) => ("ci-dot--run", "Skip"),
          Some(WorkflowConclusion::Neutral) => ("ci-dot--run", "Neutral"),
          Some(WorkflowConclusion::TimedOut) => ("ci-dot--fail", "Timeout"),
          Some(WorkflowConclusion::Other) | None => ("ci-dot--run", "Done"),
        },
        WorkflowStatus::InProgress | WorkflowStatus::Queued | WorkflowStatus::Requested => {
          ("ci-dot--run", "Running")
        }
        WorkflowStatus::Cancelled => ("ci-dot--run", "Cancel"),
        WorkflowStatus::Other => ("ci-dot--none", "-"),
      })
  }
}

/// Formats a count, appending "+" when it is an upper bound from pagination.
pub fn count_label(count: usize, estimated: bool) -> String {
  if estimated {
    format!("{count}+")
  } else {
    count.to_string()
  }
}

/// Formats a timestamp as a short relative label (e.g. "3d ago").
fn format_relative(ts: &chrono::DateTime<chrono::Utc>) -> String {
  let now = crate::time::now();
  let diff = now.signed_duration_since(*ts);
  if diff.num_minutes() < 1 {
    "just now".to_string()
  } else if diff.num_hours() < 1 {
    format!("{}m ago", diff.num_minutes())
  } else if diff.num_days() < 1 {
    format!("{}h ago", diff.num_hours())
  } else if diff.num_days() < 30 {
    format!("{}d ago", diff.num_days())
  } else {
    format!("{}mo ago", diff.num_days() / 30)
  }
}

/// A single repo summary card. Shows star/fork counts, open issue count
/// (excluding PRs), open PR count, and last push. Clicking navigates to the
/// detail page.
#[component]
pub fn RepoCard(data: RepoCardData) -> impl IntoView {
  let open_issues = count_label(data.open_issue_count(), data.open_issues_estimate);
  let open_prs = count_label(data.open_prs_count(), data.open_prs_estimate);
  let stars = data.repo.as_ref().map_or(0, |r| r.stargazers_count);
  let forks = data.repo.as_ref().map_or(0, |r| r.forks_count);
  let last_push = data.last_push_label();
  let ref_str = data.r#ref.to_string();

  view! {
      <div class="repo-card">
          <div class="repo-card-head">
              <a class="repo-card-title" href=format!("/repo/{}", data.r#ref)>
                  {ref_str}
              </a>
              <a
                  class="repo-card-ext"
                  href=data
                      .repo
                      .as_ref()
                      .map_or_else(|| data.r#ref.github_url(), |r| r.html_url.clone())
                  target="_blank"
                  rel="noopener noreferrer"
                  title="Open on GitHub"
              >
                  "↗"
              </a>
          </div>
          <div class="repo-card-counts">
              <span>"★ " {stars} "  ⑂ " {forks}</span>
              <span>
                  <span class="val-issue">{open_issues}</span> " issues · "
                  <span class="val-pr">{open_prs}</span> " PRs"
              </span>
          </div>
          <div class="repo-card-ci">
            {move || {
                let (dot, label) = data.clone().ci_badge();
                view! {
                    <span class="ci-badge">
                        <span class=format!("ci-dot {}", dot)></span>
                        <span>{label}</span>
                    </span>
                }
            }}
          </div>
          <div class="repo-card-push">
              <span class="repo-card-push-label">"Last push"</span>
              <span class="repo-card-push-value">{last_push}</span>
          </div>
      </div>
  }
}
