//! GitHub REST API client for the Lepo monitor.
//!
//! This crate is deliberately free of any Leptos or `app`-crate dependency so it
//! can be reused from a future CLI or Tauri app, and so it compiles both natively
//! (for `cargo test`) and on `wasm32-unknown-unknown`. Every endpoint is exposed
//! through the [`GithubApi`] trait and implemented by [`GithubClient`].

pub mod client;
pub mod endpoints;
pub mod error;
pub mod http;
pub mod pagination;

pub use client::{GithubApi, GithubClient};
pub use error::ApiError;
pub use pagination::{IssueParams, Pagination, PullParams};
