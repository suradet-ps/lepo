//! Thin endpoint builders. Each function performs one HTTP request and returns
//! a typed result. They are called exclusively by [`crate::client::GithubClient`]
//! so that all network access flows through a single client implementation.

pub mod actions;
pub mod issues;
pub mod pulls;
pub mod repos;
