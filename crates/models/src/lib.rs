//! Shared domain types for the Lepo GitHub monitor.
//!
//! These structs are the single source of truth for data passed between the
//! `github-api` crate (which deserializes GitHub REST responses) and the `app`
//! crate (which renders them). They are intentionally free of any Leptos or
//! wasm dependency so they compile natively (for `cargo test`) and on wasm.

pub mod issue;
pub mod pull_request;
pub mod rate_limit;
pub mod repo;
pub mod user;
pub mod workflow_run;

pub use issue::{Author, Issue, Label};
pub use pull_request::PullRequest;
pub use rate_limit::RateLimit;
pub use repo::{Owner, Repo};
pub use user::User;
pub use workflow_run::{WorkflowRun, WorkflowStatus};
