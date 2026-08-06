//! Auth state: token, current user, and login status.

use leptos::prelude::*;

use github_api::{GithubApi, GithubClient};
use models::User;

use crate::error::AppError;
use crate::storage;

/// Login status derived from the presence of a validated token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthStatus {
  /// No token present / not yet validated.
  LoggedOut,
  /// Token present and validated; we hold the authenticated [`User`].
  LoggedIn(Option<User>),
}

/// Global authentication state held in context.
#[derive(Clone, Copy)]
pub struct AuthState {
  /// The stored token (kept in memory; source of truth is localStorage).
  pub token: RwSignal<Option<String>>,
  /// Current login status.
  pub status: RwSignal<AuthStatus>,
}

impl AuthState {
  /// Builds an `AuthState` from any token found in localStorage.
  pub fn from_storage() -> Self {
    let token = storage::load_json::<String>(storage::KEY_TOKEN);
    let status = if token.is_some() {
      AuthStatus::LoggedIn(None)
    } else {
      AuthStatus::LoggedOut
    };
    Self {
      token: RwSignal::new(token),
      status: RwSignal::new(status),
    }
  }

  /// Returns a configured API client if a token is present.
  pub fn client(&self) -> Option<GithubClient> {
    self.token.get().map(GithubClient::new)
  }

  /// Submits and validates a token via `GET /user`, persisting on success.
  pub async fn login(&self, token: String) -> Result<User, AppError> {
    let client = GithubClient::new(token.clone());
    let user = client.get_user().await?;
    storage::save_json(storage::KEY_TOKEN, &token)?;
    self.token.set(Some(token));
    self.status.set(AuthStatus::LoggedIn(Some(user.clone())));
    Ok(user)
  }

  /// Clears the token and resets status (logout). Also clears rate-limit state
  /// via [`crate::state::RateLimitState::reset`].
  pub fn logout(&self) {
    storage::clear_all();
    self.token.set(None);
    self.status.set(AuthStatus::LoggedOut);
  }
}
