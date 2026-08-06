//! A single pull-request row shown on the repo detail page (AGENTS.md §5.3).

use leptos::prelude::*;

use models::PullRequest;

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
  let comments = pr.comments;
  let draft = pr.draft;
  let labels = pr.labels.clone();
  let url = pr.html_url;

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
          <span class="row-author">{author}</span>
          <span class="row-comments">{comments} "comments"</span>
      </a>
  }
}
