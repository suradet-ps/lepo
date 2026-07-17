//! Small rate-limit indicator shown in the app chrome (AGENTS.md §5.4).

use leptos::prelude::*;

use crate::state::RateLimitState;

/// Renders a colored badge reflecting the remaining API budget.
///
/// Green when > 50% remains, yellow between 10–50%, red below 10% (or when
/// exhausted). Reads exclusively from [`RateLimitState`].
#[component]
pub fn RateLimitBadge() -> impl IntoView {
  let rate = expect_context::<RateLimitState>();

  let view = Signal::derive(move || match rate.limit.get() {
    None => view! {
        <span class="rl-badge rl-badge-unknown">{"rate limit ?".to_string()}</span>
    }
    .into_view(),
    Some(rl) => {
      let pct = rl.fraction_remaining();
      let (class, label) = if rl.remaining == 0 {
        ("rl-badge-red", format!("limited (resets @ {})", rl.reset))
      } else if pct < 0.1 {
        ("rl-badge-red", format!("{} left", rl.remaining))
      } else if pct < 0.5 {
        ("rl-badge-yellow", format!("{} left", rl.remaining))
      } else {
        ("rl-badge-green", format!("{} left", rl.remaining))
      };
      view! { <span class=class>{label}</span> }.into_view()
    }
  });

  view! { <span class="rl-badge-wrap">{view}</span> }
}
