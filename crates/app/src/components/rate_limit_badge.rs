//! Small rate-limit indicator shown in the app chrome (AGENTS.md §5.4).

use leptos::prelude::*;

use crate::state::RateLimitState;
use crate::time::now;

/// Renders a compact rate-limit meter reflecting the remaining API budget.
///
/// Shows a small status dot (green / amber / red / grey) plus the remaining
/// request count, with a tooltip revealing when the window resets. Reads
/// exclusively from [`RateLimitState`].
#[component]
pub fn RateLimitBadge() -> impl IntoView {
  let rate = expect_context::<RateLimitState>();

  let meter = move || {
    let (level, text, title) = rate.limit.get().map_or_else(
      || {
        (
          "rl-meter-unknown".to_string(),
          "API limit ?".to_string(),
          String::new(),
        )
      },
      |rl| {
        let pct = rl.fraction_remaining();
        let level = if rl.remaining == 0 || pct < 0.1 {
          "rl-meter-red".to_string()
        } else if pct < 0.5 {
          "rl-meter-amber".to_string()
        } else {
          "rl-meter-green".to_string()
        };
        let reset_secs = rl.reset as i64;
        let reset_label = if reset_secs > 0 {
          let delta = reset_secs.saturating_sub(now().timestamp());
          if delta <= 0 {
            "resets now".to_string()
          } else if delta < 60 {
            format!("resets in {delta}s")
          } else if delta < 3600 {
            format!("resets in {}m", delta / 60)
          } else {
            format!("resets in {}h", delta / 3600)
          }
        } else {
          String::new()
        };
        let title = format!(
          "{} of {} requests left — {}",
          rl.remaining, rl.limit, reset_label
        );
        (level, format!("{} left", rl.remaining), title)
      },
    );
    view! {
      <span class=level title=title>
        <span class="rl-dot"></span>
        <span class="rl-text">{text}</span>
      </span>
    }
    .into_any()
  };

  view! { <span class="rl-badge-wrap">{move || meter()}</span> }
}
