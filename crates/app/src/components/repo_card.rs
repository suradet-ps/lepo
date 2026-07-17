//! A dashboard card summarizing one monitored repo (AGENTS.md §5.2, Phase 1).

use leptos::prelude::*;

use models::{Issue, PullRequest, Repo};

use crate::state::RepoRef;

/// Data needed to render a [`RepoCard`].
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RepoCardData {
  /// The repo reference.
  pub r#ref: RepoRef,
  /// Fetched repo metadata, if available.
  pub repo: Option<Repo>,
  /// Raw issues (PRs included) — filtered by the card.
  pub issues: Vec<Issue>,
  /// Pull requests.
  pub pulls: Vec<PullRequest>,
}

impl RepoCardData {
  /// Open issues excluding pull requests.
  pub fn open_issues(&self) -> usize {
    self.issues.iter().filter(|i| !i.is_pr()).count()
  }

  /// Open pull requests.
  pub fn open_prs(&self) -> usize {
    self.pulls.len()
  }

  /// Human-readable "last push" label from the repo metadata.
  pub fn last_push_label(&self) -> String {
    self
      .repo
      .as_ref()
      .and_then(|r| r.pushed_at.as_ref())
      .map(format_relative)
      .unwrap_or_else(|| "—".to_string())
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
/// (excluding PRs), and open PR count. Clicking navigates to the detail page.
#[component]
pub fn RepoCard(data: RepoCardData) -> impl IntoView {
  let open_issues = data.issues.iter().filter(|i| !i.is_pr()).count();
  let open_prs = data.pulls.len();
  let stars = data.repo.as_ref().map(|r| r.stargazers_count).unwrap_or(0);
  let forks = data.repo.as_ref().map(|r| r.forks_count).unwrap_or(0);
  let ref_str = data.r#ref.as_str();
  let ref_str_clone = ref_str.clone();

  view! {
      <div class="repo-card">
          <div class="repo-card-head">
              <a class="repo-card-title" href=format!("/repo/{}", data.r#ref.as_str())>
                  {ref_str}
              </a>
              <a
                  class="repo-card-ext"
                  href=data
                      .repo
                      .as_ref()
                      .map(|r| r.html_url.clone())
                      .unwrap_or_else(|| format!("https://github.com/{}", ref_str_clone))
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
      </div>
  }
}
