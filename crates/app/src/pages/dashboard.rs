//! Dashboard page: monitor many repos via a sortable table or compact cards
//! (AGENTS.md §5.1, §5.2; DESIGN.md "Dashboard Pattern").

use futures::future::join_all;
use github_api::{GithubApi, GithubClient, IssueParams, PullParams};
use js_sys::wasm_bindgen::JsCast;
use leptos::prelude::*;
use leptos::reactive::callback::Callable;
use leptos::server::LocalResource;
use web_sys::wasm_bindgen::prelude::Closure;

use models::{Issue, PullRequest, Repo, WorkflowRun};

use crate::components::repo_card::{RepoCard, RepoCardData, count_label};
use crate::state::{AuthState, RateLimitState, RepoRef, SettingsState, WatchlistState};

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
  let settings = expect_context::<SettingsState>();

  // Input box for adding a new repo reference.
  let (new_repo, set_new_repo) = signal(String::new());
  let (add_error, set_add_error) = signal(Option::<String>::None);

  // View / filter / sort state.
  let (view_mode, set_view_mode) = signal(ViewMode::Table);

  // Below the tablet breakpoint (767px, DESIGN.md "Breakpoints") the table
  // is unusable, so the dashboard forces the card view and the toggle is
  // hidden via CSS. Resizing back up restores the user's choice.
  let (is_compact, set_is_compact) = signal(false);
  leptos::prelude::Effect::new(move |_| {
    if is_compact.get() {
      set_view_mode.set(ViewMode::Cards);
    }
  });
  if let Ok(Some(mql)) = window().match_media("(max-width: 767px)") {
    set_is_compact.set(mql.matches());
    let listener = mql.clone();
    let cb = Closure::<dyn FnMut()>::new(move || set_is_compact.set(listener.matches()));
    let _ = mql.add_event_listener_with_callback("change", cb.as_ref().unchecked_ref());
    // The closure is not Send+Sync, so it can't be stored for on_cleanup;
    // forget() keeps it alive for the page lifetime (one per mount).
    cb.forget();
  }
  let (query, set_query) = signal(String::new());
  let (sort_key, set_sort_key) = signal(SortKey::Name);
  let (sort_asc, set_sort_asc) = signal(true);

  // Tracks the success confirmation after a repo is added.
  let (add_success, set_add_success) = signal(Option::<String>::None);

  // Auto-refresh: a version counter that triggers re-fetch when bumped.
  let (refresh_version, set_refresh_version) = signal(0_u32);
  // Number of repos whose entire bundle failed to load on the last fetch.
  let (failed_repos, set_failed_repos) = signal(0_usize);

  // Wire the configured RefreshInterval to an actual timer.
  // The effect only tracks the interval setting; the tick closure reads the
  // token and rate limit untracked, so responses from other pages never
  // restart the countdown.
  {
    let current_id: std::sync::Arc<std::sync::atomic::AtomicI32> =
      std::sync::Arc::new(std::sync::atomic::AtomicI32::new(0));

    {
      let current_id = current_id.clone();
      Effect::new(move |_| {
        let interval = settings.refresh_interval.get();

        // Cancel the previous timer if any.
        let old_id = current_id.swap(0, std::sync::atomic::Ordering::SeqCst);
        if old_id != 0
          && let Some(win) = web_sys::window()
        {
          win.clear_interval_with_handle(old_id);
        }

        if let Some(seconds) = interval.seconds() {
          let millis = (seconds * 1000) as i32;
          if let Some(win) = web_sys::window() {
            let closure = Closure::wrap(Box::new(move || {
              let has_token = auth.token.get_untracked().is_some();
              // Pause while the rate limit is nearly exhausted (< 10 remaining);
              // the next tick resumes automatically once it recovers.
              let rate_ok = rate_limit
                .limit
                .get_untracked()
                .is_none_or(|rl| rl.remaining > 10);
              if has_token && rate_ok {
                set_refresh_version.update(|v| *v += 1);
              }
            }) as Box<dyn FnMut()>);
            if let Ok(id) = win.set_interval_with_callback_and_timeout_and_arguments_0(
              closure.as_ref().unchecked_ref(),
              millis,
            ) {
              closure.forget();
              current_id.store(id, std::sync::atomic::Ordering::SeqCst);
            }
          }
        }
      });
    }

    // Cleanup: cancel the timer when the component is removed.
    on_cleanup(move || {
      let id = current_id.load(std::sync::atomic::Ordering::SeqCst);
      if id != 0
        && let Some(win) = web_sys::window()
      {
        win.clear_interval_with_handle(id);
      }
    });
  }

  let add_repo = Action::new_local(move |input: &String| {
    let raw = input.trim().to_string();
    let watchlist = watchlist;
    async move {
      match RepoRef::parse(&raw) {
        Ok(r) => {
          if watchlist.contains(&r) {
            set_add_error.set(Some(format!("{r} is already in your watchlist")));
          } else if let Some(client) = auth.client() {
            match client.get_repo(&r.owner, &r.name).await {
              Ok(_) => {
                watchlist.add(r.clone());
                let _ = watchlist.save();
                rate_limit.update(&client);
                set_add_error.set(None);
                set_add_success.set(Some(format!("Added {r}")));
              }
              Err(e) => set_add_error.set(Some(e.to_string())),
            }
          }
        }
        Err(e) => set_add_error.set(Some(e.to_string())),
      }
    }
  });

  // A resource that re-fetches whenever the watchlist, token, or refresh timer changes.
  let bundles = LocalResource::new(move || {
    let auth = auth;
    let rate_limit = rate_limit;
    let watchlist = watchlist;
    let _tick = refresh_version.get(); // re-fetch when auto-refresh fires
    async move {
      set_failed_repos.set(0);
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
        .map(|(i, bundle)| {
          if bundle.all_failed {
            set_failed_repos.update(|n| *n += 1);
          }
          RepoCardData {
            r#ref: repos[i].clone(),
            repo: bundle.repo,
            issues: bundle.issues,
            pulls: bundle.pulls,
            ci: bundle.ci,
            total_open_issues: bundle.total_open_issues,
            total_open_prs: bundle.total_open_prs,
            open_issues_estimate: bundle.open_issues_estimate,
            open_prs_estimate: bundle.open_prs_estimate,
          }
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
      items.retain(|d| d.r#ref.to_string().to_lowercase().contains(&q));
    }
    let asc = sort_asc.get();
    items.sort_by(|a, b| {
      let cmp = match sort_key.get() {
        SortKey::Name => a.r#ref.to_string().cmp(&b.r#ref.to_string()),
        SortKey::Issues => a.open_issue_count().cmp(&b.open_issue_count()),
        SortKey::Prs => a.open_prs_count().cmp(&b.open_prs_count()),
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
    let issues: usize = items.iter().map(RepoCardData::open_issue_count).sum();
    let prs: usize = items.iter().map(RepoCardData::open_prs_count).sum();
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
                      "Monitor every repository you watch - open issues, pull requests, and CI status at a glance."
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

          {move || {
              let n = failed_repos.get();
              (n > 0).then(|| {
                  view! {
                      <div class="dash-error">
                          "Couldn't refresh {n} repo(s) - check the token, its scopes, and the rate limit."
                      </div>
                  }
              })
          }}

          <Transition
              fallback=move || view! { <p class="loading-text">"Loading…"</p> }
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
                                       {move || fresh.clone().map_or_else(
                                           || "Not yet synced".to_string(),
                                           |ts| format!("Updated {ts}"),
                                       )}
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
    let os = on_sort;
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
                      .map(|data| {
                          let r = data.r#ref.clone();
                          let (ci_dot, ci_label) = data.ci_badge();
                          let ext = data
                              .repo
                              .as_ref()
                              .map_or_else(|| r.github_url(), |x| x.html_url.clone());
                          let open_issues = count_label(
                              data.open_issue_count(),
                              data.open_issues_estimate,
                          );
                          let open_prs = count_label(
                              data.open_prs_count(),
                              data.open_prs_estimate,
                          );
                          let issues_title = data.open_issues_estimate.then_some(
                              "upper bound - includes pull requests and partial pages",
                          );
                          let stars = data.repo.as_ref().map_or(0, |x| x.stargazers_count);
                          let last_push = data.last_push_label();
                          view! {
                              <tr>
                                  <td>
                                      <span class="repo-name">
                                          <a href=format!("/repo/{r}")>{r.to_string()}</a>
                                      </span>
                                  </td>
                                  <td class="num metric-strong" title=issues_title>{open_issues}</td>
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

/// Result of fetching all data for a single repo on the dashboard.
struct RepoBundle {
  repo: Option<Repo>,
  issues: Vec<Issue>,
  pulls: Vec<PullRequest>,
  ci: Option<WorkflowRun>,
  total_open_issues: usize,
  total_open_prs: usize,
  open_issues_estimate: bool,
  open_prs_estimate: bool,
  /// True when every sub-request for this repo failed (likely an auth or
  /// rate-limit problem affecting the whole dashboard).
  all_failed: bool,
}

/// Fetches metadata, issues, pulls, and latest CI run for one repo,
/// tolerating partial failure on any single piece. The four sub-requests run
/// concurrently so a single repo's bundle costs ~1 round-trip of latency.
///
/// Issue/PR totals are derived from the `Link` header (`Pagination::total_pages`)
/// as upper bounds: the first page is counted exactly (PRs excluded from the
/// issue count), later pages are assumed full. Callers mark them with "+".
async fn fetch_bundle(client: &GithubClient, r: &RepoRef) -> RepoBundle {
  let repo_fut = client.get_repo(&r.owner, &r.name);
  let issue_params = IssueParams::default();
  let issues_fut = client.list_issues(&r.owner, &r.name, &issue_params);
  let pull_params = PullParams::default();
  let pulls_fut = client.list_pulls(&r.owner, &r.name, &pull_params);
  let ci_fut = client.latest_workflow_run(&r.owner, &r.name);

  let (repo, issues, pulls, ci) = futures::join!(repo_fut, issues_fut, pulls_fut, ci_fut,);
  let all_failed = repo.is_err() && issues.is_err() && pulls.is_err() && ci.is_err();

  let per_page = 30_usize;

  // GitHub returns PRs inside the issues endpoint, so the first page is
  // counted exactly (PRs filtered out) and later pages are assumed full.
  let (issues_vec, issues_pagination) = issues.unwrap_or_default();
  let issues_page1_non_pr = issues_vec.iter().filter(|i| !i.is_pr()).count();
  let (total_open_issues, open_issues_estimate) = issues_pagination.total_pages().map_or_else(
    || (issues_page1_non_pr, false),
    |pages| {
      let pages = pages as usize;
      (
        pages.saturating_sub(1) * per_page + issues_page1_non_pr,
        pages > 1,
      )
    },
  );

  let (pulls_vec, pulls_pagination) = pulls.unwrap_or_default();
  let pulls_page1_len = pulls_vec.len();
  let (total_open_prs, open_prs_estimate) = pulls_pagination.total_pages().map_or_else(
    || (pulls_page1_len, false),
    |pages| {
      let pages = pages as usize;
      (
        pages.saturating_sub(1) * per_page + pulls_page1_len,
        pages > 1,
      )
    },
  );

  RepoBundle {
    repo: repo.ok(),
    issues: issues_vec,
    pulls: pulls_vec,
    ci: ci.ok().flatten(),
    total_open_issues,
    total_open_prs,
    open_issues_estimate,
    open_prs_estimate,
    all_failed,
  }
}

/// Compact card grid of monitored repos (used when the user prefers cards).
fn card_grid(items: Vec<RepoCardData>) -> impl IntoView {
  view! {
      <div class="repo-grid">
          {items
              .into_iter()
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
                      items.clone(),
                      sort_key,
                      sort_asc,
                      on_sort,
                  )
                  .into_any()
              } else {
                  card_grid(items.clone()).into_any()
              }
          }}
      </div>
  }
}
