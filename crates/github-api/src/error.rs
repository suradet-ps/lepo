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
  Status {
    /// HTTP status code.
    status: u16,
    /// Error message from the API.
    message: String,
  },

  /// The token was rejected (401 / 403 auth failure).
  #[error("authentication failed: {0}")]
  Auth(String),

  /// The requested resource does not exist (404).
  #[error("resource not found: {0}")]
  NotFound(String),

  /// The rate limit was exceeded (403 with rate-limit headers).
  #[error("rate limit exceeded; resets at unix {reset}")]
  RateLimited {
    /// Unix epoch seconds when the window resets.
    reset: u64,
  },

  /// The request did not complete within the configured timeout.
  #[error("request timed out")]
  Timeout,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn request_error_display() {
    let e = ApiError::Request("connection refused".into());
    assert_eq!(e.to_string(), "network request failed: connection refused");
  }

  #[test]
  fn parse_error_display() {
    let e = ApiError::Parse("invalid json".into());
    assert_eq!(e.to_string(), "failed to parse response: invalid json");
  }

  #[test]
  fn status_error_display() {
    let e = ApiError::Status {
      status: 500,
      message: "server error".into(),
    };
    assert_eq!(e.to_string(), "github returned 500: server error");
  }

  #[test]
  fn auth_error_display() {
    let e = ApiError::Auth("token rejected".into());
    assert_eq!(e.to_string(), "authentication failed: token rejected");
  }

  #[test]
  fn not_found_error_display() {
    let e = ApiError::NotFound("repo not found".into());
    assert_eq!(e.to_string(), "resource not found: repo not found");
  }

  #[test]
  fn rate_limited_error_display() {
    let e = ApiError::RateLimited { reset: 1700000000 };
    assert_eq!(
      e.to_string(),
      "rate limit exceeded; resets at unix 1700000000"
    );
  }

  #[test]
  fn timeout_error_display() {
    let e = ApiError::Timeout;
    assert_eq!(e.to_string(), "request timed out");
  }
}
