//! Repo detail page: issues / PR tabs with filters and pagination (AGENTS.md §5.3).
//!
//! - Only the active tab triggers a fetch (lazy-load).
//! - "Load more" button follows the `Link` header cursor.
//! - Label, author, and sort filters are exposed in the toolbar.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;

use github_api::{ApiError, GithubApi, GithubClient, IssueParams, Pagination, PullParams};
use models::Issue;

use crate::components::issue_row::IssueRow;
use crate::components::pr_row::PrRow;
use crate::state::{AuthState, RateLimitState, RepoRef};

/// Fetches one page from a raw URL via the shared client so timeouts, error
/// classification, and rate-limit capture behave like every other request.
async fn fetch_next_page<T: serde::de::DeserializeOwned>(
  client: &GithubClient,
  url: &str,
) -> Result<(Vec<T>, Pagination), ApiError> {
  client.get_page(url).await
}

/// The repo detail route. Expects an `owner` and `repo` path param.
#[component]
pub fn RepoDetailPage() -> impl IntoView {
  let params = use_params_map();
  let auth = expect_context::<AuthState>();
  let rate_limit = expect_context::<RateLimitState>();

  // Active tab: issues or pulls.
  let (tab, set_tab) = signal(Tab::Issues);
  // State filter.
  let (state_filter, set_state_filter) = signal(StateFilter::Open);
  // Label filter (comma-separated).
  let (label_filter, set_label_filter) = signal(String::new());
  // Author filter.
  let (author_filter, set_author_filter) = signal(String::new());
  // Sort key.
  let (sort_key, set_sort_key) = signal(SortKey::Updated);

  let r#ref = Signal::derive(move || {
    let p = params.read();
    RepoRef {
      owner: p.get("owner").unwrap_or_default(),
      name: p.get("repo").unwrap_or_default(),
    }
  });

  // --- Issues (fetched only when the Issues tab is active) ---

  let issues_items = RwSignal::<Vec<Issue>>::new(Vec::new());
  let issues_next = RwSignal::<Option<String>>::new(None);
  let issues_loading = RwSignal::new(false);
  let issues_error = RwSignal::new(Option::<String>::None);
  // Bumped on every fetch start; responses only apply their writes when the
  // generation still matches, so out-of-order responses cannot clobber newer
  // ones (e.g. after a quick filter change).
  let issues_gen = RwSignal::new(0_u32);

  // Fetches one page of issues. `next_url` is `None` for the first page.
  let fetch_issues = {
    move |next_url: Option<String>| {
      issues_gen.update(|g| *g += 1);
      let generation = issues_gen.get_untracked();
      issues_loading.set(true);
      let auth = auth;
      let rate_limit = rate_limit;
      let r#ref = r#ref;
      let state_filter = state_filter;
      let label_filter = label_filter;
      let author_filter = author_filter;
      let sort_key = sort_key;
      async move {
        let r = r#ref.get();
        let client = match auth.client() {
          Some(c) => c,
          None => {
            if issues_gen.get_untracked() == generation {
              issues_error.set(Some("not authenticated".into()));
              issues_loading.set(false);
            }
            return;
          }
        };

        let result = if let Some(ref url) = next_url {
          match fetch_next_page::<Issue>(&client, url).await {
            Ok((mut items, pagination)) => {
              rate_limit.update(&client);
              // Filter PRs out — GitHub's issues endpoint returns PRs too.
              items.retain(|i: &Issue| !i.is_pr());
              (
                items,
                Pagination {
                  next: pagination.next,
                  last: None,
                },
              )
            }
            Err(e) => {
              if issues_gen.get_untracked() == generation {
                issues_error.set(Some(e.to_string()));
                issues_loading.set(false);
              }
              return;
            }
          }
        } else {
          issues_error.set(None);
          let params = IssueParams {
            state: state_filter.get().as_str().into(),
            labels: label_filter
              .get()
              .split(',')
              .map(str::trim)
              .filter(|s| !s.is_empty())
              .map(str::to_string)
              .collect(),
            sort: sort_key.get().as_api_str().into(),
            creator: author_filter.get(),
            per_page: 30,
          };
          match client.list_issues(&r.owner, &r.name, &params).await {
            Ok((mut v, p)) => {
              rate_limit.update(&client);
              v.retain(|i| !i.is_pr());
              (v, p)
            }
            Err(e) => {
              if issues_gen.get_untracked() == generation {
                issues_error.set(Some(e.to_string()));
                issues_loading.set(false);
              }
              return;
            }
          }
        };

        if issues_gen.get_untracked() == generation {
          let (items, pagination) = result;
          if next_url.is_none() {
            issues_items.set(items);
          } else {
            issues_items.update(|v| v.extend(items));
          }
          issues_next.set(pagination.next);
          issues_error.set(None);
          issues_loading.set(false);
        }
      }
    }
  };

  // Trigger initial fetch when the issues tab becomes active.
  // Watch for tab changes and reset + fetch when switching to Issues.
  Effect::new(move |_| {
    let current_tab = tab.get();
    let _state = state_filter.get();
    let _labels = label_filter.get();
    let _author = author_filter.get();
    let _sort = sort_key.get();
    if current_tab == Tab::Issues {
      let fut = fetch_issues(None);
      spawn_local(fut);
    }
  });

  // --- Pulls (fetched only when the Pulls tab is active) ---

  let pulls_items = RwSignal::<Vec<models::PullRequest>>::new(Vec::new());
  let pulls_next = RwSignal::<Option<String>>::new(None);
  let pulls_loading = RwSignal::new(false);
  let pulls_error = RwSignal::new(Option::<String>::None);
  let pulls_gen = RwSignal::new(0_u32);

  let fetch_pulls = {
    move |next_url: Option<String>| {
      pulls_gen.update(|g| *g += 1);
      let generation = pulls_gen.get_untracked();
      pulls_loading.set(true);
      let auth = auth;
      let rate_limit = rate_limit;
      let r#ref = r#ref;
      let state_filter = state_filter;
      let sort_key = sort_key;
      async move {
        let r = r#ref.get();
        let client = match auth.client() {
          Some(c) => c,
          None => {
            if pulls_gen.get_untracked() == generation {
              pulls_error.set(Some("not authenticated".into()));
              pulls_loading.set(false);
            }
            return;
          }
        };

        let result = if let Some(ref url) = next_url {
          match fetch_next_page::<models::PullRequest>(&client, url).await {
            Ok((items, pagination)) => {
              rate_limit.update(&client);
              (
                items,
                Pagination {
                  next: pagination.next,
                  last: None,
                },
              )
            }
            Err(e) => {
              if pulls_gen.get_untracked() == generation {
                pulls_error.set(Some(e.to_string()));
                pulls_loading.set(false);
              }
              return;
            }
          }
        } else {
          pulls_error.set(None);
          let params = PullParams {
            state: state_filter.get().as_str().into(),
            sort: sort_key.get().as_api_str().into(),
            per_page: 30,
          };
          match client.list_pulls(&r.owner, &r.name, &params).await {
            Ok((v, p)) => {
              rate_limit.update(&client);
              (v, p)
            }
            Err(e) => {
              if pulls_gen.get_untracked() == generation {
                pulls_error.set(Some(e.to_string()));
                pulls_loading.set(false);
              }
              return;
            }
          }
        };

        if pulls_gen.get_untracked() == generation {
          let (items, pagination) = result;
          if next_url.is_none() {
            pulls_items.set(items);
          } else {
            pulls_items.update(|v| v.extend(items));
          }
          pulls_next.set(pagination.next);
          pulls_error.set(None);
          pulls_loading.set(false);
        }
      }
    }
  };

  // Trigger initial fetch when the pulls tab becomes active.
  Effect::new(move |_| {
    let current_tab = tab.get();
    let _state = state_filter.get();
    let _sort = sort_key.get();
    if current_tab == Tab::Pulls {
      let fut = fetch_pulls(None);
      spawn_local(fut);
    }
  });

  let title = move || {
    let r = r#ref.get();
    r.to_string()
  };

  view! {
      <div class="page page--detail">
          <div class="page-header">
              <h1 class="heading-lg">{title}</h1>
          </div>
          <div class="page-toolbar">
              <div class="tabs">
                  <button
                      class="filter-chip"
                      class:filter-chip-active=move || tab.get() == Tab::Issues
                      on:click=move |_| set_tab.set(Tab::Issues)
                  >
                      "Issues"
                  </button>
                  <button
                      class="filter-chip"
                      class:filter-chip-active=move || tab.get() == Tab::Pulls
                      on:click=move |_| set_tab.set(Tab::Pulls)
                  >
                      "Pull Requests"
                  </button>
              </div>
              <div class="filters">
                  <select
                      class="text-input"
                      on:change=move |ev| {
                          let v = event_target_value(&ev);
                          set_state_filter
                              .set(match v.as_str() {
                                  "closed" => StateFilter::Closed,
                                  "all" => StateFilter::All,
                                  _ => StateFilter::Open,
                              });
                      }
                  >
                      <option value="open">"Open"</option>
                      <option value="closed">"Closed"</option>
                      <option value="all">"All"</option>
                  </select>
                  <select
                      class="text-input"
                      on:change=move |ev| {
                          let v = event_target_value(&ev);
                          set_sort_key
                              .set(match v.as_str() {
                                  "created" => SortKey::Created,
                                  "comments" => SortKey::Comments,
                                  _ => SortKey::Updated,
                              });
                      }
                  >
                      <option value="updated">"Sort: updated"</option>
                      <option value="created">"Sort: created"</option>
                      <option value="comments">"Sort: comments"</option>
                  </select>
                  <input
                      class="text-input"
                      placeholder="label filter"
                      prop:value=move || label_filter.get()
                      on:input=move |ev| {
                          set_label_filter.set(event_target_value(&ev));
                      }
                  />
                  <input
                      class="text-input"
                      placeholder="author filter"
                      prop:value=move || author_filter.get()
                      on:input=move |ev| {
                          set_author_filter.set(event_target_value(&ev));
                      }
                  />
              </div>
          </div>

          <Show when=move || tab.get() == Tab::Issues>
              {move || {
                  let items = issues_items.get();
                  let empty = items.is_empty();
                  let loading = issues_loading.get();
                  let has_more = issues_next.get().is_some();
                  let error = issues_error.get();
                  if loading && empty {
                      view! {
                          <div class="row-list">
                              {(0..5).map(|_| view! {
                                  <div class="row row--issue row-skeleton">
                                      <span class="row-num"><span class="skeleton skeleton-line"></span></span>
                                      <span class="row-title"><span class="skeleton skeleton-line"></span></span>
                                      <span class="row-labels"><span class="skeleton skeleton-line"></span></span>
                                      <span class="row-author"><span class="skeleton skeleton-line"></span></span>
                                      <span class="row-comments"><span class="skeleton skeleton-line"></span></span>
                                  </div>
                              }).collect_view()}
                          </div>
                      }.into_any()
                  } else if !empty {
                      view! {
                          <div class="row-list">
                              {items
                                  .iter()
                                  .cloned()
                                  .map(|i| view! { <IssueRow issue=i/> })
                                  .collect_view()}
                              {if has_more {
                                  view! {
                                      <button
                                          class="button-secondary load-more"
                                          disabled=move || loading
                                          on:click=move |_| {
                                              let fut = fetch_issues(issues_next.get());
                                              spawn_local(fut);
                                          }
                                      >
                                          {move || {
                                              if loading {
                                                  view! { <span class="button-spinner"></span> "Loading…" }.into_any()
                                              } else {
                                                  "Load more".into_any()
                                              }
                                          }}
                                      </button>
                                  }
                                  .into_any()
                              } else {
                                  ().into_any()
                              }}
                              {if let Some(msg) = error {
                                  view! { <p class="row-error">{msg}</p> }.into_any()
                              } else {
                                  ().into_any()
                              }}
                          </div>
                      }.into_any()
                  } else if let Some(msg) = error {
                      view! {
                          <div class="error-state">
                              <p class="body-strong">"Couldn't load issues"</p>
                              <p class="body-sm">{msg}</p>
                          </div>
                      }.into_any()
                  } else {
                      view! {
                          <div class="empty-state">
                              <p class="body-strong">"No issues"</p>
                              <p class="body-sm">"This repository has no issues matching the current filter."</p>
                          </div>
                      }.into_any()
                  }
              }}
          </Show>

          <Show when=move || tab.get() == Tab::Pulls>
              {move || {
                  let items = pulls_items.get();
                  let empty = items.is_empty();
                  let loading = pulls_loading.get();
                  let has_more = pulls_next.get().is_some();
                  let error = pulls_error.get();
                  if loading && empty {
                      view! {
                          <div class="row-list">
                              {(0..5).map(|_| view! {
                                  <div class="row row--pr row-skeleton">
                                      <span class="row-num"><span class="skeleton skeleton-line"></span></span>
                                      <span class="row-title"><span class="skeleton skeleton-line"></span></span>
                                      <span class="row-labels"><span class="skeleton skeleton-line"></span></span>
                                      <span class="row-author"><span class="skeleton skeleton-line"></span></span>
                                      <span class="row-comments"><span class="skeleton skeleton-line"></span></span>
                                  </div>
                              }).collect_view()}
                          </div>
                      }.into_any()
                  } else if !empty {
                      view! {
                          <div class="row-list">
                              {items
                                  .iter()
                                  .cloned()
                                  .map(|p| view! { <PrRow pr=p/> })
                                  .collect_view()}
                              {if has_more {
                                  view! {
                                      <button
                                          class="button-secondary load-more"
                                          disabled=move || loading
                                          on:click=move |_| {
                                              let fut = fetch_pulls(pulls_next.get());
                                              spawn_local(fut);
                                          }
                                      >
                                          {move || {
                                              if loading {
                                                  view! { <span class="button-spinner"></span> "Loading…" }.into_any()
                                              } else {
                                                  "Load more".into_any()
                                              }
                                          }}
                                      </button>
                                  }
                                  .into_any()
                              } else {
                                  ().into_any()
                              }}
                              {if let Some(msg) = error {
                                  view! { <p class="row-error">{msg}</p> }.into_any()
                              } else {
                                  ().into_any()
                              }}
                          </div>
                      }.into_any()
                  } else if let Some(msg) = error {
                      view! {
                          <div class="error-state">
                              <p class="body-strong">"Couldn't load pull requests"</p>
                              <p class="body-sm">{msg}</p>
                          </div>
                      }.into_any()
                  } else {
                      view! {
                          <div class="empty-state">
                              <p class="body-strong">"No pull requests"</p>
                              <p class="body-sm">"This repository has no pull requests matching the current filter."</p>
                          </div>
                      }.into_any()
                  }
              }}
          </Show>
      </div>
  }
}

/// Which tab is active on the detail page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
  Issues,
  Pulls,
}

/// Issue state filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StateFilter {
  Open,
  Closed,
  All,
}

impl StateFilter {
  const fn as_str(self) -> &'static str {
    match self {
      Self::Open => "open",
      Self::Closed => "closed",
      Self::All => "all",
    }
  }
}

/// Sort key for issues and pulls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SortKey {
  Updated,
  Created,
  Comments,
}

impl SortKey {
  const fn as_api_str(self) -> &'static str {
    match self {
      Self::Updated => "updated",
      Self::Created => "created",
      Self::Comments => "comments",
    }
  }
}
