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

  let (mobile_menu_open, set_mobile_menu_open) = signal(false);

  view! {
      <div class="app-shell" data-theme=move || settings.theme.get().as_attr()>
          <Router>
              {move || {
                  let location = use_location();
                  // Close mobile menu on navigation
                  let path = location.pathname.get();
                  let _ = path; // subscription only
                  set_mobile_menu_open.set(false);
                  view! {
                      <header class="primary-nav">
                          <div class="primary-nav-inner">
                              <a class="primary-nav-brand" href="/">
                                  <svg class="logo-mark" viewBox="0 0 24 24" width="28" height="28" fill="none" xmlns="http://www.w3.org/2000/svg">
                                      <path d="M12 23C14 23 16 20 16 18C16 15.7909 14.2091 14 12 14C9.79086 14 8 15.7909 8 18C8 20 10 23 12 23Z" fill="#F87171"/>
                                      <path d="M12 1C7 1 4 7 4 13C4 15.5 5.5 17 7 17H17C18.5 17 20 15.5 20 13C20 7 17 1 12 1Z" fill="#F1F5F9" stroke="#0F172A" stroke-width="2.2" stroke-linejoin="round"/>
                                      <path d="M4 13L1 16V19H5L7 17" stroke="#0F172A" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/>
                                      <path d="M20 13L23 16V19H19L17 17" stroke="#0F172A" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/>
                                      <circle cx="12" cy="7.5" r="2.5" fill="#38BDF8" stroke="#0F172A" stroke-width="1.8"/>
                                  </svg>
                                  "Lepo"
                              </a>
                              <button
                                  class="nav-hamburger"
                                  class:open=move || mobile_menu_open.get()
                                  on:click=move |_| set_mobile_menu_open.update(|v| *v = !*v)
                                  aria-label="Toggle navigation menu"
                              >
                                  <span class="nav-hamburger-bar"></span>
                                  <span class="nav-hamburger-bar"></span>
                                  <span class="nav-hamburger-bar"></span>
                              </button>
                              <nav class="primary-nav-links" class:open=move || mobile_menu_open.get()>
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
