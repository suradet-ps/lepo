//! Dashboard page: monitor many repos via a sortable table or compact cards
//! (AGENTS.md §5.1, §5.2; DESIGN.md "Dashboard Pattern").

use futures::future::join_all;
use github_api::{GithubApi, GithubClient, IssueParams, PullParams};
use leptos::prelude::*;
use leptos::reactive::callback::Callable;
use leptos::server::LocalResource;

use models::{Issue, PullRequest, Repo, WorkflowRun};

use crate::components::repo_card::{RepoCard, RepoCardData};
use crate::state::{AuthState, RateLimitState, RepoRef, WatchlistState};

/// Which dashboard display mode is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewMode {
  Table,
  Cards,
}

/// Column used for sorting the table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SortKey {
  Name,
  Issues,
  Prs,
  Stars,
}

/// The main dashboard shown after login.
#[component]
pub fn DashboardPage() -> impl IntoView {
  let auth = expect_context::<AuthState>();
  let watchlist = expect_context::<WatchlistState>();
  let rate_limit = expect_context::<RateLimitState>();

  // Input box for adding a new repo reference.
  let (new_repo, set_new_repo) = signal(String::new());
  let (add_error, set_add_error) = signal(Option::<String>::None);

  // View / filter / sort state.
  let (view_mode, set_view_mode) = signal(ViewMode::Table);
  let (query, set_query) = signal(String::new());
  let (sort_key, set_sort_key) = signal(SortKey::Name);
  let (sort_asc, set_sort_asc) = signal(true);

  // Tracks the success confirmation after a repo is added.
  let (add_success, set_add_success) = signal(Option::<String>::None);

  let add_repo = Action::new_local(move |input: &String| {
    let raw = input.trim().to_string();
    let watchlist = watchlist;
    async move {
      match RepoRef::parse(&raw) {
        Ok(r) => {
          if watchlist.contains(&r) {
            set_add_error.set(Some(format!("{} is already in your watchlist", r.as_str())));
          } else if let Some(client) = auth.client() {
            match client.get_repo(&r.owner, &r.name).await {
              Ok(_) => {
                watchlist.add(r.clone());
                let _ = watchlist.save();
                rate_limit.update(&client);
                set_add_error.set(None);
                set_add_success.set(Some(format!("Added {}", r.as_str())));
              }
              Err(e) => set_add_error.set(Some(e.to_string())),
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
      // Fetch every repo's bundle concurrently instead of one-at-a-time.
      // Each bundle already fans out its 4 sub-requests internally, so the
      // whole dashboard now completes in roughly one repo's worth of latency
      // rather than N repos × 4 sequential requests.
      let out: Vec<RepoCardData> = join_all(repos.iter().map(|r| fetch_bundle(&client, r)))
        .await
        .into_iter()
        .enumerate()
        .map(|(i, (repo, issues, pulls, ci))| RepoCardData {
          r#ref: repos[i].clone(),
          repo,
          issues,
          pulls,
          ci,
        })
        .collect();
      rate_limit.update(&client);
      out
    }
  });

  // Derived, filtered + sorted view of the data.
  let display = Signal::derive(move || {
    let q = query.get().trim().to_lowercase();
    let mut items: Vec<RepoCardData> = match bundles.get() {
      Some(v) => v,
      None => return Vec::new(),
    };
    if !q.is_empty() {
      items.retain(|d| d.r#ref.as_str().to_lowercase().contains(&q));
    }
    let asc = sort_asc.get();
    items.sort_by(|a, b| {
      let cmp = match sort_key.get() {
        SortKey::Name => a.r#ref.as_str().cmp(&b.r#ref.as_str()),
        SortKey::Issues => a.open_issues().cmp(&b.open_issues()),
        SortKey::Prs => a.open_prs().cmp(&b.open_prs()),
        SortKey::Stars => a
          .repo
          .as_ref()
          .map(|r| r.stargazers_count)
          .cmp(&b.repo.as_ref().map(|r| r.stargazers_count)),
      };
      if asc { cmp } else { cmp.reverse() }
    });
    items
  });

  // Summary totals across all fetched repos.
  let summary = Signal::derive(move || {
    let items = bundles.get().unwrap_or_default();
    let repos = items.len();
    let issues: usize = items.iter().map(|d| d.open_issues()).sum();
    let prs: usize = items.iter().map(|d| d.open_prs()).sum();
    (repos, issues, prs)
  });

  // Toggle sort: same column flips direction, new column sorts ascending.
  let on_sort = Callback::new(move |key: SortKey| {
    if sort_key.get() == key {
      set_sort_asc.set(!sort_asc.get());
    } else {
      set_sort_key.set(key);
      set_sort_asc.set(true);
    }
  });

  view! {
      <div class="page page--dashboard">
          <div class="page-header">
              <div>
                  <h1 class="heading-lg">"Dashboard"</h1>
                  <p class="page-intro">
                      "Monitor every repository you watch — open issues, pull requests, and CI status at a glance."
                  </p>
              </div>
              <div class="add-repo">
                  <input
                      class="text-input"
                      placeholder="owner/repo"
                      prop:value=move || new_repo.get()
                      on:input=move |ev| {
                          set_new_repo.set(event_target_value(&ev));
                          set_add_error.set(None);
                          set_add_success.set(None);
                      }
                  />
                  <button
                      class="button-primary"
                      class:button-loading=move || add_repo.pending().get()
                      disabled=move || add_repo.pending().get()
                      on:click=move |_| { add_repo.dispatch(new_repo.get()); }
                  >
                      {move || {
                          if add_repo.pending().get() {
                              view! { <span class="button-spinner"></span> "Adding…" }.into_any()
                          } else {
                              view! { "Add repo" }.into_any()
                          }
                      }}
                  </button>
              </div>
          </div>
          {move || {
              add_error
                  .get()
                  .map(|msg| view! { <span class="add-repo-error">{msg}</span> })
          }}
          {move || {
              add_success
                  .get()
                  .map(|msg| view! { <span class="add-repo-success">{msg}</span> })
          }}

          <Transition
              fallback=move || {
                  let is_table = view_mode.get() == ViewMode::Table;
                  view! {
                      <div class="summary-strip">
                          <div class="summary-item">
                              <span class="summary-value skeleton skeleton-line skeleton-line--sm"></span>
                              <span class="summary-label">"Repos"</span>
                          </div>
                          <div class="summary-item">
                              <span class="summary-value skeleton skeleton-line skeleton-line--sm"></span>
                              <span class="summary-label">"Open issues"</span>
                          </div>
                          <div class="summary-item">
                              <span class="summary-value skeleton skeleton-line skeleton-line--sm"></span>
                              <span class="summary-label">"Open PRs"</span>
                          </div>
                          <div class="summary-item">
                              <span class="summary-value skeleton skeleton-line skeleton-line--sm"></span>
                              <span class="summary-label">"Freshness"</span>
                          </div>
                      </div>
                      {if is_table {
                          view! {
                              <div class="repo-table-wrap">
                                  <table class="repo-table">
                                      <thead>
                                          <tr>
                                              <th>"Repository"</th>
                                              <th class="num">"Open Issues"</th>
                                              <th class="num">"Open PRs"</th>
                                              <th class="num">"CI"</th>
                                              <th class="num">"Stars"</th>
                                              <th>"Last push"</th>
                                              <th></th>
                                          </tr>
                                      </thead>
                                      <tbody>
                                          {(0..8).map(|_| view! {
                                              <tr>
                                                  <td><span class="skeleton skeleton-line"></span></td>
                                                  <td class="num"><span class="skeleton skeleton-line skeleton-line--sm"></span></td>
                                                  <td class="num"><span class="skeleton skeleton-line skeleton-line--sm"></span></td>
                                                  <td><span class="skeleton skeleton-line skeleton-line--sm"></span></td>
                                                  <td class="num"><span class="skeleton skeleton-line skeleton-line--sm"></span></td>
                                                  <td><span class="skeleton skeleton-line skeleton-line--sm"></span></td>
                                                  <td class="num"></td>
                                              </tr>
                                          }).collect_view()}
                                      </tbody>
                                  </table>
                              </div>
                          }
                          .into_any()
                      } else {
                          view! {
                              <div class="repo-grid">
                                  {(0..6).map(|_| view! {
                                      <div class="skeleton-card">
                                          <div class="skeleton skeleton-line skeleton-line--lg"></div>
                                          <div class="skeleton skeleton-line skeleton-line--sm"></div>
                                      </div>
                                  }).collect_view()}
                              </div>
                          }
                          .into_any()
                      }}
                  }
              }
          >
              {move || {
                  bundles.get().map(|_| {
                      let (repos, issues, prs) = summary.get();
                      let fresh = rate_limit.last_updated.get();
                      view! {
                          <div class="summary-strip">
                              <div class="summary-item">
                                  <span class="summary-value">{repos}</span>
                                  <span class="summary-label">"Repos"</span>
                              </div>
                              <div class="summary-item">
                                  <span class="summary-value">{issues}</span>
                                  <span class="summary-label">"Open issues"</span>
                              </div>
                              <div class="summary-item">
                                  <span class="summary-value">{prs}</span>
                                  <span class="summary-label">"Open PRs"</span>
                              </div>
                              <div class="summary-item">
                                  <span class="summary-freshness">
                                      {move || match fresh.clone() {
                                          Some(ts) => format!("Updated {}", ts),
                                          None => "Not yet synced".to_string(),
                                      }}
                                  </span>
                                  <span class="summary-label">"Freshness"</span>
                              </div>
                          </div>

                          <div class="dash-toolbar">
                              <div class="dash-search">
                                  <span class="dash-search-icon">"⌕"</span>
                                  <input
                                      class="text-input"
                                      placeholder="Filter repos…"
                                      prop:value=move || query.get()
                                      on:input=move |ev| set_query.set(event_target_value(&ev))
                                  />
                              </div>
                              <div class="view-toggle">
                                  <button
                                      class:active=move || view_mode.get() == ViewMode::Table
                                      on:click=move |_| set_view_mode.set(ViewMode::Table)
                                  >
                                      "Table"
                                  </button>
                                  <button
                                      class:active=move || view_mode.get() == ViewMode::Cards
                                      on:click=move |_| set_view_mode.set(ViewMode::Cards)
                                  >
                                      "Cards"
                                  </button>
                              </div>
                          </div>

                          {move || {
                              render_dashboard(
                                  display.get(),
                                  view_mode.get(),
                                  sort_key,
                                  sort_asc,
                                  on_sort,
                              )
                          }}
                      }
                  })
              }}
          </Transition>
      </div>
  }
}

/// Sortable table of monitored repos. `sort_key`/`sort_asc` drive the active
/// column and direction; `on_sort` toggles them when a header is clicked.
fn repo_table(
  items: Vec<RepoCardData>,
  sort_key: ReadSignal<SortKey>,
  sort_asc: ReadSignal<bool>,
  on_sort: Callback<SortKey>,
) -> impl IntoView {
  let make_header = move || {
    let sk = sort_key;
    let sa = sort_asc;
    let os = on_sort.clone();
    move |key: SortKey, label: &'static str, num: bool| {
      let cls = if sk.get() == key {
        if sa.get() {
          "sorted-asc"
        } else {
          "sorted-desc"
        }
      } else {
        ""
      };
      let num_cls = if num { " num" } else { "" };
      view! {
        <th
          class=format!("{}{}", cls, num_cls)
          on:click=move |_| { os.run(key); }
        >
          {label}
        </th>
      }
    }
  };

  view! {
      <div class="repo-table-wrap">
          <table class="repo-table">
              <thead>
                  <tr>
                      {make_header()(SortKey::Name, "Repository", false)}
                      {make_header()(SortKey::Issues, "Open Issues", true)}
                      {make_header()(SortKey::Prs, "Open PRs", true)}
                      <th class="ci-col">"CI"</th>
                      {make_header()(SortKey::Stars, "Stars", true)}
                      <th class="center">"Last push"</th>
                      <th class="actions"></th>
                  </tr>
              </thead>
              <tbody>
                  {items
                      .iter()
                      .cloned()
                      .map(|data| {
                          let d = data.clone();
                          let r = d.r#ref.clone();
                          let (ci_dot, ci_label) = d.ci_badge();
                          let ext = d
                              .repo
                              .as_ref()
                              .map(|x| x.html_url.clone())
                              .unwrap_or_else(|| format!("https://github.com/{}", r.as_str()));
                          let open_issues = d.open_issues();
                          let open_prs = d.open_prs();
                          let stars = d.repo.as_ref().map(|x| x.stargazers_count).unwrap_or(0);
                          let last_push = d.last_push_label();
                          view! {
                              <tr>
                                  <td>
                                      <span class="repo-name">
                                          <a href=format!("/repo/{}", r.as_str())>{r.as_str()}</a>
                                      </span>
                                  </td>
                                  <td class="num metric-strong">{open_issues}</td>
                                  <td class="num metric-pr">{open_prs}</td>
                                  <td class="ci-col">
                                      <span class="ci-badge">
                                          <span class=format!("ci-dot {}", ci_dot)></span>
                                          <span>{ci_label}</span>
                                      </span>
                                  </td>
                                  <td class="num">{stars}</td>
                                  <td class="center">{last_push}</td>
                                  <td class="actions">
                                      <a
                                          class="repo-card-ext"
                                          href=ext
                                          target="_blank"
                                          rel="noopener noreferrer"
                                          title="Open on GitHub"
                                      >
                                          "↗"
                                      </a>
                                  </td>
                              </tr>
                          }
                      })
                      .collect_view()}
              </tbody>
          </table>
      </div>
  }
}

/// Fetches metadata, issues, pulls, and latest CI run for one repo,
/// tolerating partial failure on any single piece. The four sub-requests run
/// concurrently so a single repo's bundle costs ~1 round-trip of latency.
async fn fetch_bundle(
  client: &GithubClient,
  r: &RepoRef,
) -> (
  Option<Repo>,
  Vec<Issue>,
  Vec<PullRequest>,
  Option<WorkflowRun>,
) {
  let repo_fut = client.get_repo(&r.owner, &r.name);
  let issue_params = IssueParams {
    state: "open".into(),
    labels: vec![],
    sort: String::new(),
    per_page: 30,
  };
  let issues_fut = client.list_issues(&r.owner, &r.name, &issue_params);
  let pull_params = PullParams {
    state: "open".into(),
    sort: String::new(),
    per_page: 30,
  };
  let pulls_fut = client.list_pulls(&r.owner, &r.name, &pull_params);
  let ci_fut = client.latest_workflow_run(&r.owner, &r.name);

  let (repo, issues, pulls, ci) = futures::join!(repo_fut, issues_fut, pulls_fut, ci_fut,);

  (
    repo.ok(),
    issues.map(|(v, _)| v).unwrap_or_default(),
    pulls.map(|(v, _)| v).unwrap_or_default(),
    ci.ok().flatten(),
  )
}

/// Compact card grid of monitored repos (used when the user prefers cards).
fn card_grid(items: Vec<RepoCardData>) -> impl IntoView {
  view! {
      <div class="repo-grid">
          {items
              .iter()
              .cloned()
              .map(|data| view! { <RepoCard data/> })
              .collect_view()}
      </div>
  }
}

/// Renders the list area: empty state, table, or card grid based on state.
fn render_dashboard(
  items: Vec<RepoCardData>,
  mode: ViewMode,
  sort_key: ReadSignal<SortKey>,
  sort_asc: ReadSignal<bool>,
  on_sort: Callback<SortKey>,
) -> impl IntoView {
  let is_empty = items.is_empty();
  let is_table = mode == ViewMode::Table;
  let items_for_table = items.clone();
  let items_for_cards = items;
  view! {
      <div class="dash-list">
          {move || {
              if is_empty {
                  view! {
                      <div class="empty-state">
                          <p class="body-strong">"No repositories match"</p>
                          <p class="body-sm">"Adjust your filter, or add a repo above to start monitoring."</p>
                      </div>
                  }
                  .into_any()
              } else if is_table {
                  repo_table(
                      items_for_table.clone(),
                      sort_key,
                      sort_asc,
                      on_sort,
                  )
                  .into_any()
              } else {
                  card_grid(items_for_cards.clone()).into_any()
              }
          }}
      </div>
  }
}
