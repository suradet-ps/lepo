//! Dashboard page: one card per monitored repo (AGENTS.md §5.1, §5.2).

use leptos::prelude::*;
use leptos::server::LocalResource;

use github_api::{GithubApi, GithubClient, IssueParams, PullParams};
use models::{Issue, PullRequest, Repo};

use crate::components::repo_card::{RepoCard, RepoCardData};
use crate::state::{AuthState, RateLimitState, RepoRef, WatchlistState};

/// The main dashboard shown after login.
#[component]
pub fn DashboardPage() -> impl IntoView {
  let auth = expect_context::<AuthState>();
  let watchlist = expect_context::<WatchlistState>();
  let rate_limit = expect_context::<RateLimitState>();

  // Input box for adding a new repo reference.
  let (new_repo, set_new_repo) = signal(String::new());
  let (add_error, set_add_error) = signal(Option::<String>::None);

  let add_repo = Action::new_local(move |input: &String| {
    let raw = input.trim().to_string();
    let watchlist = watchlist;
    async move {
      match RepoRef::parse(&raw) {
        Ok(r) => {
          if watchlist.contains(&r) {
            set_add_error.set(Some(format!("{} is already in your watchlist", r.as_str())));
          } else {
            // Validate the repo exists before persisting.
            if let Some(client) = auth.client() {
              match client.get_repo(&r.owner, &r.name).await {
                Ok(_) => {
                  watchlist.add(r);
                  let _ = watchlist.save();
                  rate_limit.update(&client);
                  set_add_error.set(None);
                }
                Err(e) => set_add_error.set(Some(e.to_string())),
              }
            }
          }
        }
        Err(e) => set_add_error.set(Some(e.to_string())),
      }
    }
  });

  // A resource that re-fetches whenever the watchlist (or token) changes.
  let bundles = LocalResource::new(move || {
    let auth = auth;
    let rate_limit = rate_limit;
    let watchlist = watchlist;
    async move {
      let repos = watchlist.repos.get();
      let client = match auth.client() {
        Some(c) => c,
        None => return Vec::new(),
      };
      let mut out = Vec::with_capacity(repos.len());
      for r in repos.iter() {
        let (repo, issues, pulls) = fetch_bundle(&client, r).await;
        rate_limit.update(&client);
        out.push(RepoCardData {
          r#ref: r.clone(),
          repo,
          issues,
          pulls,
        });
      }
      out
    }
  });

  view! {
      <div class="page page--dashboard">
          <div class="page-header">
              <div>
                  <h1 class="heading-lg">"Dashboard"</h1>
                  <p class="page-intro">
                      "An overview of every repository you monitor — open issues, pull requests, and activity at a glance."
                  </p>
              </div>
              <div class="add-repo">
                  <input
                      class="text-input"
                      placeholder="owner/repo"
                      prop:value=move || new_repo.get()
                      on:input=move |ev| set_new_repo.set(event_target_value(&ev))
                  />
                  <button class="button-primary" on:click=move |_| { add_repo.dispatch(new_repo.get()); }>
                      "Add repo"
                  </button>
              </div>
          </div>
          {move || {
              add_error
                  .get()
                  .map(|msg| view! { <span class="add-repo-error">{msg}</span> })
          }}
          <Transition
              fallback=move || view! {
                  <div class="repo-grid">
                      {(0..3).map(|_| view! {
                          <div class="skeleton-card">
                              <div class="skeleton skeleton-line skeleton-line--lg"></div>
                              <div class="skeleton skeleton-line skeleton-line--sm"></div>
                              <div class="skeleton skeleton-block"></div>
                          </div>
                      }).collect_view()}
                  </div>
              }
          >
              {move || {
                  bundles.get().map(|cards| {
                      let empty = cards.is_empty();
                      let cards2 = cards.clone();
                      view! {
                          <Show
                              when=move || empty
                              fallback=move || view! {
                                  <div class="repo-grid">
                                      {cards2
                                          .iter()
                                          .cloned()
                                          .map(|data| view! { <RepoCard data/> })
                                          .collect_view()}
                                  </div>
                              }
                          >
                              <div class="empty-state">
                                  <p class="body-strong">"No repositories yet"</p>
                                  <p class="body-sm">"Add a repo above (e.g. <code>leptos-rs/leptos</code>) to start monitoring."</p>
                              </div>
                          </Show>
                      }
                  })
              }}
          </Transition>
      </div>
  }
}

/// Fetches metadata, issues, and pulls for one repo, tolerating partial failure.
async fn fetch_bundle(
  client: &GithubClient,
  r: &RepoRef,
) -> (Option<Repo>, Vec<Issue>, Vec<PullRequest>) {
  let repo = client.get_repo(&r.owner, &r.name).await.ok();
  let issues = client
    .list_issues(
      &r.owner,
      &r.name,
      &IssueParams {
        state: "open".into(),
        labels: vec![],
        sort: String::new(),
        per_page: 30,
      },
    )
    .await
    .map(|(v, _)| v)
    .unwrap_or_default();
  let pulls = client
    .list_pulls(
      &r.owner,
      &r.name,
      &PullParams {
        state: "open".into(),
        sort: String::new(),
        per_page: 30,
      },
    )
    .await
    .map(|(v, _)| v)
    .unwrap_or_default();
  (repo, issues, pulls)
}
