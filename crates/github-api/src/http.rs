//! Single HTTP entry point with a timeout applied to every request.
//!
//! AGENTS.md §9 requires a timeout on every API call. On wasm the request is
//! raced against `gloo_timers::future::TimeoutFuture`; on native (unit tests)
//! it is awaited directly because `gloo-timers` is a wasm-only dependency.

use gloo_net::http::{Headers, Request, Response};

use crate::error::ApiError;

/// Sends a GET request, mapping transport errors and enforcing the timeout.
pub async fn get(url: &str, headers: Headers) -> Result<Response, ApiError> {
  let request = Request::get(url).headers(headers).send();
  #[cfg(target_arch = "wasm32")]
  {
    use futures::future::select;
    use gloo_timers::future::TimeoutFuture;

    /// How long to wait for a single HTTP response before giving up.
    const REQUEST_TIMEOUT_MS: u32 = 15_000;

    match select(
      Box::pin(request),
      Box::pin(TimeoutFuture::new(REQUEST_TIMEOUT_MS)),
    )
    .await
    {
      futures::future::Either::Left((result, _)) => {
        result.map_err(|e| ApiError::Request(e.to_string()))
      }
      futures::future::Either::Right(_) => Err(ApiError::Timeout),
    }
  }
  #[cfg(not(target_arch = "wasm32"))]
  {
    request.await.map_err(|e| ApiError::Request(e.to_string()))
  }
}
