//! Login page: token entry when no token is present (AGENTS.md §4).

use leptos::prelude::*;

use crate::components::token_form::TokenForm;

/// The login page shown when the user is logged out.
#[component]
pub fn LoginPage() -> impl IntoView {
  view! {
      <div class="page page--login">
          <svg viewBox="0 0 24 24" width="64" height="64" fill="none" xmlns="http://www.w3.org/2000/svg">
              <path d="M12 23C14 23 16 20 16 18C16 15.7909 14.2091 14 12 14C9.79086 14 8 15.7909 8 18C8 20 10 23 12 23Z" fill="#F87171"/>
              <path d="M12 1C7 1 4 7 4 13C4 15.5 5.5 17 7 17H17C18.5 17 20 15.5 20 13C20 7 17 1 12 1Z" fill="#F1F5F9" stroke="#0F172A" stroke-width="2.2" stroke-linejoin="round"/>
              <path d="M4 13L1 16V19H5L7 17" stroke="#0F172A" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/>
              <path d="M20 13L23 16V19H19L17 17" stroke="#0F172A" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/>
              <circle cx="12" cy="7.5" r="2.5" fill="#38BDF8" stroke="#0F172A" stroke-width="1.8"/>
          </svg>
          <h1 class="heading-lg">"Lepo"</h1>
          <p class="body-md">"Monitor your GitHub repositories in one place."</p>
          <TokenForm/>
      </div>
  }
}
