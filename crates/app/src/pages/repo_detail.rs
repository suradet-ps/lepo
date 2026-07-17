//! Repo detail page: issues / PR tabs with filters (AGENTS.md §5.3).

use leptos::prelude::*;
use leptos::server::LocalResource;
use leptos_router::hooks::use_params_map;

use github_api::{GithubApi, IssueParams, PullParams};
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
  // State filter for issues.
  let (state_filter, set_state_filter) = signal(StateFilter::Open);

  let r#ref = Signal::derive(move || {
    let p = params.read();
    RepoRef {
      owner: p.get("owner").unwrap_or_default(),
      name: p.get("repo").unwrap_or_default(),
    }
  });

  let issues = LocalResource::new(move || {
    let auth = auth;
    let rate_limit = rate_limit;
    let r#ref = r#ref;
    let state_filter = state_filter;
    async move {
      let r = r#ref.get();
      let state = state_filter.get().as_str().to_string();
      let client = match auth.client() {
        Some(c) => c,
        None => return Vec::new(),
      };
      let out = client
        .list_issues(
          &r.owner,
          &r.name,
          &IssueParams {
            state,
            labels: vec![],
            sort: "updated".into(),
            per_page: 30,
          },
        )
        .await
        .map(|(v, _)| v)
        .unwrap_or_default();
      rate_limit.update(&client);
      // Filter PRs out of the issues list.
      out
        .into_iter()
        .filter(|i| !i.is_pr())
        .collect::<Vec<Issue>>()
    }
  });

  let pulls = LocalResource::new(move || {
    let auth = auth;
    let rate_limit = rate_limit;
    let r#ref = r#ref;
    async move {
      let r = r#ref.get();
      let client = match auth.client() {
        Some(c) => c,
        None => return Vec::new(),
      };
      let out = client
        .list_pulls(
          &r.owner,
          &r.name,
          &PullParams {
            state: "open".into(),
            sort: "updated".into(),
            per_page: 30,
          },
        )
        .await
        .map(|(v, _)| v)
        .unwrap_or_default();
      rate_limit.update(&client);
      out
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
              <Show when=move || tab.get() == Tab::Issues>
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
                  </div>
              </Show>
          </div>
          <Show when=move || tab.get() == Tab::Issues>
              <Transition
                  fallback=move || view! {
                      <div class="row-list">
                          {(0..5).map(|_| view! { <div class="skeleton skeleton-row"></div> }).collect_view()}
                      </div>
                  }
              >
                  {move || {
                      issues.get().map(|list| {
                          let empty = list.is_empty();
                          let list2 = list.clone();
                          view! {
                              <Show
                                  when=move || empty
                                  fallback=move || view! {
                                      <div class="row-list">
                                          {list2
                                              .iter()
                                              .cloned()
                                              .map(|i| view! { <IssueRow issue=i/> })
                                              .collect_view()}
                                      </div>
                                  }
                              >
                                  <div class="empty-state">
                                      <p class="body-strong">"No issues"</p>
                                      <p class="body-sm">"This repository has no issues matching the current filter."</p>
                                  </div>
                              </Show>
                          }
                      })
                  }}
              </Transition>
          </Show>
          <Show when=move || tab.get() == Tab::Pulls>
              <Transition
                  fallback=move || view! {
                      <div class="row-list">
                          {(0..5).map(|_| view! { <div class="skeleton skeleton-row"></div> }).collect_view()}
                      </div>
                  }
              >
                  {move || {
                      pulls.get().map(|list| {
                          let empty = list.is_empty();
                          let list2 = list.clone();
                          view! {
                              <Show
                                  when=move || empty
                                  fallback=move || view! {
                                      <div class="row-list">
                                          {list2
                                              .iter()
                                              .cloned()
                                              .map(|p| view! { <PrRow pr=p/> })
                                              .collect_view()}
                                      </div>
                                  }
                              >
                                  <div class="empty-state">
                                      <p class="body-strong">"No pull requests"</p>
                                      <p class="body-sm">"This repository has no open pull requests."</p>
                                  </div>
                              </Show>
                          }
                      })
                  }}
              </Transition>
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
  fn as_str(self) -> &'static str {
    match self {
      StateFilter::Open => "open",
      StateFilter::Closed => "closed",
      StateFilter::All => "all",
    }
  }
}
