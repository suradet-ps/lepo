//! Centralized localStorage access. All keys for the app live here so they can
//! be cleared consistently (e.g. on logout). No other module reads/writes
//! storage keys directly.

use gloo_storage::{LocalStorage, Storage};
use leptos::logging::error;

use crate::error::AppError;

/// Key under which the GitHub PAT is stored.
pub const KEY_TOKEN: &str = "gh_monitor_token";
/// Key under which the watchlist (`Vec<RepoRef>`) is stored as JSON.
pub const KEY_WATCHLIST: &str = "gh_monitor_watchlist";
/// Key under which the refresh-interval preference is stored.
pub const KEY_REFRESH_INTERVAL: &str = "gh_monitor_refresh_interval";
/// Key under which the theme preference is stored.
pub const KEY_THEME: &str = "gh_monitor_theme";

/// Reads a JSON-serializable value from localStorage.
pub fn load_json<T: serde::de::DeserializeOwned>(key: &str) -> Option<T> {
  match LocalStorage::get::<T>(key) {
    Ok(v) => Some(v),
    Err(e) => {
      error!("failed to read {key}: {e}");
      None
    }
  }
}

/// Writes a JSON-serializable value to localStorage.
pub fn save_json<T: serde::Serialize>(key: &str, value: &T) -> Result<(), AppError> {
  LocalStorage::set(key, value).map_err(|e| AppError::Storage(e.to_string()))
}

/// Removes a single key from localStorage.
pub fn remove(key: &str) {
  LocalStorage::delete(key);
}

/// Clears every Lepo-owned key (used on logout).
pub fn clear_all() {
  remove(KEY_TOKEN);
  remove(KEY_WATCHLIST);
  remove(KEY_REFRESH_INTERVAL);
  remove(KEY_THEME);
}
