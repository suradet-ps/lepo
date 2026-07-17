//! Settings state: refresh interval and theme, persisted to localStorage.

use leptos::prelude::*;

use crate::error::AppError;
use crate::storage;

/// Auto-refresh cadence (see AGENTS.md §5.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RefreshInterval {
  /// No automatic refresh.
  Manual,
  /// Refresh every minute.
  Every1Min,
  /// Refresh every five minutes.
  Every5Min,
  /// Refresh every fifteen minutes.
  Every15Min,
}

impl RefreshInterval {
  /// Number of seconds between refreshes, or `None` for manual.
  #[allow(dead_code)]
  pub fn seconds(&self) -> Option<u64> {
    match self {
      RefreshInterval::Manual => None,
      RefreshInterval::Every1Min => Some(60),
      RefreshInterval::Every5Min => Some(300),
      RefreshInterval::Every15Min => Some(900),
    }
  }
}

/// UI theme selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Theme {
  /// Light theme.
  Light,
  /// Dark theme.
  Dark,
}

impl Theme {
  /// The value used for the `data-theme` attribute on the app root.
  pub fn as_attr(self) -> &'static str {
    match self {
      Theme::Light => "light",
      Theme::Dark => "dark",
    }
  }
}

/// Global settings state held in context.
#[derive(Clone, Copy)]
pub struct SettingsState {
  /// Auto-refresh interval.
  pub refresh_interval: RwSignal<RefreshInterval>,
  /// Active theme.
  pub theme: RwSignal<Theme>,
}

impl SettingsState {
  /// Loads settings from localStorage, falling back to defaults.
  pub fn from_storage() -> SettingsState {
    let refresh_interval = storage::load_json::<RefreshInterval>(storage::KEY_REFRESH_INTERVAL)
      .unwrap_or(RefreshInterval::Manual);
    let theme = storage::load_json::<Theme>(storage::KEY_THEME).unwrap_or(Theme::Light);
    SettingsState {
      refresh_interval: RwSignal::new(refresh_interval),
      theme: RwSignal::new(theme),
    }
  }

  /// Persists the current refresh interval.
  pub fn save_refresh_interval(&self) -> Result<(), AppError> {
    storage::save_json(storage::KEY_REFRESH_INTERVAL, &self.refresh_interval.get())
  }

  /// Persists the current theme.
  pub fn save_theme(&self) -> Result<(), AppError> {
    storage::save_json(storage::KEY_THEME, &self.theme.get())
  }
}
