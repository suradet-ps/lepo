//! A single pull-request row shown on the repo detail page (AGENTS.md §5.3).

use leptos::prelude::*;

use models::PullRequest;

fn format_updated(updated_at: Option<chrono::DateTime<chrono::Utc>>) -> String {
  updated_at
    .map(|ts| ts.format("%Y-%m-%d").to_string())
    .unwrap_or_default()
}

/// Renders one pull request as a clickable row.
#[component]
pub fn PrRow(pr: PullRequest) -> impl IntoView {
  let title = pr.title.clone();
  let number = pr.number;
  let author = pr
    .user
    .as_ref()
    .map(|u| u.login.clone())
    .unwrap_or_default();
  let avatar = pr
    .user
    .as_ref()
    .and_then(|u| u.avatar_url.clone())
    .unwrap_or_default();
  let comments = pr.comments;
  let draft = pr.draft;
  let labels = pr.labels.clone();
  let url = pr.html_url;
  let updated = format_updated(pr.updated_at);

  view! {
      <a class="row row--pr" href=url target="_blank" rel="noopener noreferrer">
          <span class="row-num">"#" {number}</span>
          <span class="row-title">
              {title}
              <Show when=move || draft fallback=|| view! { <span class="row-draft"></span> }>
                  <span class="row-draft">"draft"</span>
              </Show>
          </span>
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
