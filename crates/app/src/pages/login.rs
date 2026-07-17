//! Login page: token entry when no token is present (AGENTS.md §4).

use leptos::prelude::*;

use crate::components::token_form::TokenForm;

/// The login page shown when the user is logged out.
#[component]
pub fn LoginPage() -> impl IntoView {
  view! {
      <div class="page page--login">
          <h1 class="heading-lg">"Lepo"</h1>
          <p class="body-md">"Monitor your GitHub repositories in one place."</p>
          <TokenForm/>
      </div>
  }
}
