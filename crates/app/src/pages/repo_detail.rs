//! Repo detail page: issues / PR tabs with filters and pagination (AGENTS.md §5.3).
//!
//! - Only the active tab triggers a fetch (lazy-load).
//! - "Load more" button follows the `Link` header cursor.
//! - Label, author, and sort filters are exposed in the toolbar.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;

use github_api::{GithubApi, IssueParams, Pagination, PullParams};
use models::Issue;

use crate::components::issue_row::IssueRow;
use crate::components::pr_row::PrRow;
use crate::state::{AuthState, RateLimitState, RepoRef};

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

  // Fetches one page of issues. `next_url` is `None` for the first page.
  let fetch_issues = {
    move |next_url: Option<String>| {
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
          None => return,
        };
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

        let result = if let Some(url) = next_url {
          // Fetch the next page directly via URL (avoids reconstructing params).
          use gloo_net::http::Request;
          let headers = client.auth_headers();
          match Request::get(&url).headers(headers).send().await {
            Ok(resp) => {
              let status = resp.status();
              let h = resp.headers();
              let hm = github_api::client::headers_to_map(&h);
              if !(200..300).contains(&status) {
                // Best-effort error handling — silently stop pagination.
                issues_loading.set(false);
                return;
              }
              let body: Vec<Issue> = resp.json().await.unwrap_or_default();
              let pagination = Pagination::from_headers(&hm);
              rate_limit.update(&client);
              // Filter PRs out.
              let filtered: Vec<Issue> = body.into_iter().filter(|i| !i.is_pr()).collect();
              (filtered, pagination)
            }
            Err(_) => {
              issues_loading.set(false);
              return;
            }
          }
        } else {
          match client.list_issues(&r.owner, &r.name, &params).await {
            Ok((mut v, p)) => {
              rate_limit.update(&client);
              // Filter PRs out.
              v.retain(|i| !i.is_pr());
              (v, p)
            }
            Err(_) => {
              issues_loading.set(false);
              return;
            }
          }
        };

        let (items, pagination) = result;
        issues_items.update(|v| v.extend(items));
        issues_next.set(pagination.next);
        issues_loading.set(false);
      }
    }
  };

  // Trigger initial fetch when the issues tab becomes active.
  // Watch for tab changes and reset + fetch when switching to Issues.
  let issues_fetched = RwSignal::new(false);
  Effect::new(move |_| {
    let current_tab = tab.get();
    let _state = state_filter.get();
    let _labels = label_filter.get();
    let _author = author_filter.get();
    let _sort = sort_key.get();
    if current_tab == Tab::Issues {
      // Reset and re-fetch on any filter change.
      issues_items.set(Vec::new());
      issues_next.set(None);
      issues_fetched.set(false);
      issues_loading.set(true);
      let fut = fetch_issues(None);
      spawn_local(fut);
      issues_fetched.set(true);
    }
  });

  // --- Pulls (fetched only when the Pulls tab is active) ---

  let pulls_items = RwSignal::<Vec<models::PullRequest>>::new(Vec::new());
  let pulls_next = RwSignal::<Option<String>>::new(None);
  let pulls_loading = RwSignal::new(false);

  let fetch_pulls = {
    move |next_url: Option<String>| {
      let auth = auth;
      let rate_limit = rate_limit;
      let r#ref = r#ref;
      let state_filter = state_filter;
      let sort_key = sort_key;
      async move {
        let r = r#ref.get();
        let client = match auth.client() {
          Some(c) => c,
          None => return,
        };
        let params = PullParams {
          state: state_filter.get().as_str().into(),
          sort: sort_key.get().as_api_str().into(),
          per_page: 30,
        };

        let result = if let Some(url) = next_url {
          use gloo_net::http::Request;
          let headers = client.auth_headers();
          match Request::get(&url).headers(headers).send().await {
            Ok(resp) => {
              let status = resp.status();
              let h = resp.headers();
              let hm = github_api::client::headers_to_map(&h);
              if !(200..300).contains(&status) {
                pulls_loading.set(false);
                return;
              }
              let body: Vec<models::PullRequest> = resp.json().await.unwrap_or_default();
              let pagination = Pagination::from_headers(&hm);
              rate_limit.update(&client);
              (body, pagination)
            }
            Err(_) => {
              pulls_loading.set(false);
              return;
            }
          }
        } else {
          match client.list_pulls(&r.owner, &r.name, &params).await {
            Ok((v, p)) => {
              rate_limit.update(&client);
              (v, p)
            }
            Err(_) => {
              pulls_loading.set(false);
              return;
            }
          }
        };

        let (items, pagination) = result;
        pulls_items.update(|v| v.extend(items));
        pulls_next.set(pagination.next);
        pulls_loading.set(false);
      }
    }
  };

  // Trigger initial fetch when the pulls tab becomes active.
  let pulls_fetched = RwSignal::new(false);
  Effect::new(move |_| {
    let current_tab = tab.get();
    let _state = state_filter.get();
    let _sort = sort_key.get();
    if current_tab == Tab::Pulls && !pulls_fetched.get() {
      pulls_items.set(Vec::new());
      pulls_next.set(None);
      pulls_loading.set(true);
      let fut = fetch_pulls(None);
      spawn_local(fut);
      pulls_fetched.set(true);
    }
  });

  // Reset pulls cache when switching away from pulls tab.
  Effect::new(move |_| {
    let current_tab = tab.get();
    if current_tab == Tab::Issues && pulls_fetched.get() {
      pulls_fetched.set(false);
      pulls_items.set(Vec::new());
      pulls_next.set(None);
    }
  });

  let title = move || {
    let r = r#ref.get();
    r.as_str()
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
                  view! {
                      <Show
                          when=move || empty && !loading
                          fallback=move || view! {
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
                                                  issues_loading.set(true);
                                                  let fut = fetch_issues(issues_next.get());
                                                  spawn_local(fut);
                                              }
                                          >
                                              {move || {
                                                  if loading {
                                                      "Loading…"
                                                  } else {
                                                      "Load more"
                                                  }
                                              }}
                                          </button>
                                      }
                                      .into_any()
                                  } else {
                                      ().into_any()
                                  }}
                              </div>
                          }
                      >
                          <div class="empty-state">
                              <p class="body-strong">"No issues"</p>
                              <p class="body-sm">"This repository has no issues matching the current filter."</p>
                          </div>
                      </Show>
                  }
              }}
          </Show>

          <Show when=move || tab.get() == Tab::Pulls>
              {move || {
                  let items = pulls_items.get();
                  let empty = items.is_empty();
                  let loading = pulls_loading.get();
                  let has_more = pulls_next.get().is_some();
                  view! {
                      <Show
                          when=move || empty && !loading
                          fallback=move || view! {
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
                                                  pulls_loading.set(true);
                                                  let fut = fetch_pulls(pulls_next.get());
                                                  spawn_local(fut);
                                              }
                                          >
                                              {move || {
                                                  if loading {
                                                      "Loading…"
                                                  } else {
                                                      "Load more"
                                                  }
                                              }}
                                          </button>
                                      }
                                      .into_any()
                                  } else {
                                      ().into_any()
                                  }}
                              </div>
                          }
                      >
                          <div class="empty-state">
                              <p class="body-strong">"No pull requests"</p>
                              <p class="body-sm">"This repository has no open pull requests."</p>
                          </div>
                      </Show>
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
