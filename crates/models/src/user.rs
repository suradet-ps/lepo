//! The authenticated user returned by `GET /user`.

use serde::{Deserialize, Serialize};

/// The currently authenticated GitHub user (from `GET /user`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
  /// Login handle.
  pub login: String,
  /// Avatar image URL.
  #[serde(default)]
  pub avatar_url: Option<String>,
  /// Display name, if set.
  #[serde(default)]
  pub name: Option<String>,
}
