//! A single issue row shown on the repo detail page (AGENTS.md §5.3).
//!
//! Clicking the row opens the issue on github.com in a new tab.

use leptos::prelude::*;

use models::Issue;

/// Renders one issue as a clickable row.
#[component]
pub fn IssueRow(issue: Issue) -> impl IntoView {
  let title = issue.title.clone();
  let number = issue.number;
  let author = issue
    .user
    .as_ref()
    .map(|u| u.login.clone())
    .unwrap_or_default();
  let comments = issue.comments;
  let labels = issue.labels.clone();
  let url = issue.html_url.clone();

  view! {
      <a class="row row--issue" href=url target="_blank" rel="noopener noreferrer">
          <span class="row-num">"#" {number}</span>
          <span class="row-title">{title}</span>
          <span class="row-labels">
              {labels
                  .into_iter()
                  .map(|l| view! { <span class="label">{l.name}</span> })
                  .collect_view()}
          </span>
          <span class="row-author">{author}</span>
          <span class="row-comments">{comments} "comments"</span>
      </a>
  }
}
