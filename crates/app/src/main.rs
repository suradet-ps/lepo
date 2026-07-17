//! Leptos CSR entry point for the Lepo GitHub monitor.

mod app;
mod components;
mod error;
mod pages;
mod state;
mod storage;
mod time;

use app::App;

fn main() {
  // Surface panic messages in the browser console instead of a wasm trap.
  console_error_panic_hook::set_once();
  // Route wasm logging through tracing + the browser console.
  tracing_wasm::set_as_global_default();

  leptos::mount::mount_to_body(App);
}
