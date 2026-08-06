//! Token entry form used on first load and on the login page (AGENTS.md §4).

use leptos::prelude::*;

use crate::state::AuthState;

/// A form that accepts a GitHub PAT, validates it via `GET /user`, and reports
/// errors inline. On success the parent re-renders via the auth context.
#[component]
pub fn TokenForm() -> impl IntoView {
  let auth = expect_context::<AuthState>();
  let (token, set_token) = signal(String::new());
  let (error, set_error) = signal(Option::<String>::None);
  let (pending, set_pending) = signal(false);

  let submit = Action::new_local(move |input: &String| {
    let token_value = input.clone();
    let auth = auth;
    async move {
      set_pending.set(true);
      set_error.set(None);
      match auth.login(token_value).await {
        Ok(_) => {
          set_pending.set(false);
          Ok(())
        }
        Err(e) => {
          set_pending.set(false);
          set_error.set(Some(e.to_string()));
          Err(())
        }
      }
    }
  });

  view! {
      <div class="tokenform">
          <p class="tokenformcopy">
              "Paste a GitHub fine-grained Personal Access Token with read-only access to Issues, Pull requests, Metadata, Contents, and Actions."
          </p>
          <a
              class="tokencreate"
              href="https://github.com/settings/personal-access-tokens/new"
              target="_blank"
              rel="noopener noreferrer"
          >
              "Create a token"
          </a>
           <input
               class="text-input"
               type="password"
               placeholder="ghp_..."
               prop:value=move || token.get()
               on:input=move |ev| set_token.set(event_target_value(&ev))
           />
           <button
               class="button-primary"
               class:button-loading=move || pending.get()
               on:click=move |_| { submit.dispatch(token.get()); }
               disabled=move || pending.get()
           >
              {move || if pending.get() {
                  view! { <span class="button-spinner"></span> "Validating…" }.into_any()
              } else {
                  "Save token".into_any()
              }}
          </button>
          {move || {
              error
                  .get()
                  .map(|msg| view! { <p class="formerror">{msg}</p> })
          }}
      </div>
  }
}
