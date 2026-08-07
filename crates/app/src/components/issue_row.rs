//! A single issue row shown on the repo detail page (AGENTS.md §5.3).
//!
//! Clicking the row opens the issue on github.com in a new tab.

use leptos::prelude::*;

use models::Issue;

fn format_updated(updated_at: Option<chrono::DateTime<chrono::Utc>>) -> String {
  updated_at
    .map(|ts| ts.format("%Y-%m-%d").to_string())
    .unwrap_or_default()
}

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
  let avatar = issue
    .user
    .as_ref()
    .and_then(|u| u.avatar_url.clone())
    .unwrap_or_default();
  let comments = issue.comments;
  let labels = issue.labels.clone();
  let url = issue.html_url;
  let updated = format_updated(issue.updated_at);

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
          <span class="row-author">
              {(!avatar.is_empty()).then(|| {
                  view! { <img class="row-avatar" src=avatar alt="" /> }
              })}
              {author}
          </span>
          <span class="row-comments">{comments} "comments"</span>
          <span class="row-updated">{updated}</span>
      </a>
  }
}
