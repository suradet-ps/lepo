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
          <div class="repo-card-stats">
              <span class="repo-card-stat">"★ " {stars}</span>
              <span class="repo-card-stat">"⑂ " {forks}</span>
          </div>
          <div class="repo-card-counts">
              <div class="repo-card-metric repo-card-metric--issue">
                  <span class="repo-card-metric-value">{open_issues}</span>
                  <span class="repo-card-metric-label">"Open issues"</span>
              </div>
              <div class="repo-card-metric repo-card-metric--pr">
                  <span class="repo-card-metric-value">{open_prs}</span>
                  <span class="repo-card-metric-label">"Open PRs"</span>
              </div>
          </div>
      </div>
  }
}
