//! Application-level error type for the Leptos UI layer.

/// Errors surfaced to the user from the UI/state layer.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AppError {
  /// An underlying GitHub API error, forwarded to the UI.
  #[error("{0}")]
  Api(String),
  /// The user is not authenticated (no token in storage).
  #[error("not authenticated")]
  #[allow(dead_code)]
  NotAuthenticated,
  /// A local storage operation failed.
  #[error("storage error: {0}")]
  Storage(String),
  /// A watchlist entry was malformed.
  #[error("invalid repo reference: {0}")]
  InvalidRepoRef(String),
}

impl From<github_api::ApiError> for AppError {
  fn from(e: github_api::ApiError) -> Self {
    Self::Api(e.to_string())
  }
}
