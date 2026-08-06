//! Root application component and router wiring.

use leptos::prelude::*;
use leptos_router::{
  components::{Route, Router, Routes},
  hooks::use_location,
  path,
};

use crate::components::rate_limit_badge::RateLimitBadge;
use crate::pages::{DashboardPage, LoginPage, RepoDetailPage, SettingsPage};
use crate::state::{AuthState, RateLimitState, SettingsState, Theme, WatchlistState};

/// The root component. Provides global state via context and renders the router.
#[component]
pub fn App() -> impl IntoView {
  // Build and provide global state once.
  provide_context(AuthState::from_storage());
  provide_context(WatchlistState::from_storage());
  provide_context(SettingsState::from_storage());
  provide_context(RateLimitState::new());

  let auth = expect_context::<AuthState>();
  let settings = expect_context::<SettingsState>();

  // Reflect the theme onto the document root so `:root[data-theme]` CSS applies.
  let theme_attr = move || settings.theme.get().as_attr();
  leptos::prelude::Effect::new(move |_| {
    let value = theme_attr();
    if let Some(win) = web_sys::window()
      && let Some(doc) = win.document()
      && let Some(root) = doc.document_element()
    {
      let _ = root.set_attribute("data-theme", value);
    }
  });

  view! {
      <div class="app-shell" data-theme=move || settings.theme.get().as_attr()>
          <Router>
              {move || {
                  let location = use_location();
                  view! {
                      <header class="primary-nav">
                          <div class="primary-nav-inner">
                              <a class="primary-nav-brand" href="/">"Lepo"</a>
                              <nav class="primary-nav-links">
                                  <a
                                      href="/"
                                      class="nav-link"
                                      class:active=move || location.pathname.get() == "/"
                                  >
                                      "Dashboard"
                                  </a>
                                  <a
                                      href="/settings"
                                      class="nav-link"
                                      class:active=move || location.pathname.get() == "/settings"
                                  >
                                      "Settings"
                                  </a>
                              </nav>
                              <div class="primary-nav-rate">
                                  <button
                                      class="theme-toggle"
                                      title="Toggle light / dark theme"
                                      on:click=move |_| {
                                          let next = match settings.theme.get() {
                                              Theme::Light => Theme::Dark,
                                              Theme::Dark => Theme::Light,
                                          };
                                          settings.theme.set(next);
                                          if let Err(e) = settings.save_theme() {
                                              leptos::logging::error!("failed to save theme: {e}");
                                          }
                                      }
                                  >
                                      {move || {
                                          if settings.theme.get() == Theme::Light {
                                              "☀"
                                          } else {
                                              "☾"
                                          }
                                      }}
                                  </button>
                                  <RateLimitBadge/>
                              </div>
                          </div>
                      </header>
                  }
              }}
              <main class="app-main">
                  <Routes
                      fallback=|| view! { <p class="body-md">"Page not found."</p> }
                  >
                  <Route
                      path=path!("/")
                      view=move || {
                          view! {
                              <Show
                                  when=move || matches!(
                                      auth.status.get(),
                                      crate::state::AuthStatus::LoggedIn(_)
                                  )
                                  fallback=|| view! { <LoginPage/> }
                              >
                                  <DashboardPage/>
                              </Show>
                          }
                      }
                  />
                  <Route path=path!("/repo/:owner/:repo") view=RepoDetailPage/>
                  <Route path=path!("/settings") view=SettingsPage/>
              </Routes>
          </main>
          </Router>
      </div>
  }
}
