//! Settings page: manage token, watchlist, refresh interval, theme (AGENTS §5.6).

use leptos::prelude::*;

use crate::state::{
  AuthState, RateLimitState, RefreshInterval, RepoRef, SettingsState, WatchlistState,
};

/// The settings page.
#[component]
pub fn SettingsPage() -> impl IntoView {
  let auth = expect_context::<AuthState>();
  let watchlist = expect_context::<WatchlistState>();
  let settings = expect_context::<SettingsState>();
  let rate_limit = expect_context::<RateLimitState>();

  let logout = move |_| {
    auth.logout();
    rate_limit.reset();
  };

  let masked_token = Signal::derive(move || match auth.token.get() {
    Some(t) if t.len() > 4 => format!("{}****{}", &t[..4], &t[t.len() - 4..]),
    Some(_) => "****".to_string(),
    None => "not set".to_string(),
  });

  // Interval selector.
  let on_interval_change = move |ev| {
    let v = event_target_value(&ev);
    let interval = match v.as_str() {
      "1" => RefreshInterval::Every1Min,
      "5" => RefreshInterval::Every5Min,
      "15" => RefreshInterval::Every15Min,
      _ => RefreshInterval::Manual,
    };
    settings.refresh_interval.set(interval);
    if let Err(e) = settings.save_refresh_interval() {
      leptos::logging::error!("failed to save refresh interval: {e}");
    }
  };

  let remove_repo = Action::new_local(move |r: &RepoRef| {
    let r = r.clone();
    let watchlist = watchlist;
    async move {
      watchlist.remove(&r);
      let _ = watchlist.save();
    }
  });

  view! {
      <div class="page page--settings">
          <div class="page-header">
              <div>
                  <h1 class="heading-lg">"Settings"</h1>
                  <p class="page-intro">"Manage your token, watchlist, and how Lepo behaves."</p>
              </div>
          </div>

          <div class="settings-grid">
              <section class="settings-section">
                  <h2 class="heading-md">"Token"</h2>
                  <p class="body-sm">"Stored only in this browser's local storage."</p>
                  <div class="field">
                      <span class="field-label">"Current token"</span>
                      <p class="body-strong">{masked_token}</p>
                  </div>
                  <button class="button-secondary" on:click=logout>"Log out / remove token"</button>
              </section>

              <section class="settings-section">
                  <h2 class="heading-md">"Watchlist"</h2>
                  <ul class="settings-watchlist">
                      {move || {
                          watchlist
                              .repos
                              .get()
                              .into_iter()
                              .map(|r| {
                                  let r2 = r.clone();
                                  view! {
                                      <li>
                                          <span>{r.to_string()}</span>
                                          <button
                                              class="button-tertiary"
                                              on:click=move |_| { remove_repo.dispatch(r2.clone()); }
                                          >
                                              "remove"
                                          </button>
                                      </li>
                                  }
                              })
                              .collect_view()
                      }}
                  </ul>
              </section>

              <section class="settings-section">
                  <h2 class="heading-md">"Auto-refresh"</h2>
                  <div class="field">
                      <span class="field-label">"Refresh interval"</span>
                      <select class="text-input" on:change=on_interval_change>
                          <option value="0">"Manual only"</option>
                          <option value="1">"Every 1 minute"</option>
                          <option value="5">"Every 5 minutes"</option>
                          <option value="15">"Every 15 minutes"</option>
                      </select>
                  </div>
              </section>
          </div>
      </div>
  }
}
