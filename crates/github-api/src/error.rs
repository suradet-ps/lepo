//! Error types for the GitHub API client.

/// Errors that can occur while talking to the GitHub REST API.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
  /// The HTTP request could not be built or sent (network, CORS, etc.).
  #[error("network request failed: {0}")]
  Request(String),

  /// The response could not be parsed as JSON.
  #[error("failed to parse response: {0}")]
  Parse(String),

  /// GitHub returned a non-success status code.
  #[error("github returned {status}: {message}")]
  Status { status: u16, message: String },

  /// The token was rejected (401 / 403 auth failure).
  #[error("authentication failed: {0}")]
  Auth(String),

  /// The requested resource does not exist (404).
  #[error("resource not found: {0}")]
  NotFound(String),

  /// The rate limit was exceeded (403 with rate-limit headers).
  #[error("rate limit exceeded; resets at unix {reset}")]
  RateLimited { reset: u64 },
}
