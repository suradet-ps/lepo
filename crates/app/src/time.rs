//! Wasm-safe wall-clock access.
//!
//! `chrono::Utc::now()` panics on `wasm32-unknown-unknown` because it falls
//! back to `std::time::SystemTime`, which is unimplemented there. This module
//! provides a single `now()` that uses the browser clock (`js_sys::Date`) when
//! compiling to wasm and `chrono` everywhere else.

use chrono::{DateTime, Utc};

/// Current UTC time, available on both native and wasm targets.
pub fn now() -> DateTime<Utc> {
  #[cfg(target_arch = "wasm32")]
  {
    let millis = js_sys::Date::now();
    let secs = (millis / 1000.0) as i64;
    let nanos = ((millis - (secs as f64) * 1000.0) * 1_000_000.0) as u32;
    DateTime::from_timestamp(secs, nanos).unwrap_or_else(|| DateTime::UNIX_EPOCH)
  }
  #[cfg(not(target_arch = "wasm32"))]
  {
    Utc::now()
  }
}
