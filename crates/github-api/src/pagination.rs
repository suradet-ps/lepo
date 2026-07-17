//! Pagination and query-parameter helpers.
//!
//! GitHub paginates via the `Link` header rather than page-number query
//! params. [`Pagination::next`] extracts the `rel="next"` URL so callers follow
//! the cursor instead of guessing page numbers (see AGENTS.md §3.6).

use std::collections::HashMap;

/// Query parameters accepted by `list_issues`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IssueParams {
  /// `open`, `closed`, or `all`.
  pub state: String,
  /// Optional label filter.
  pub labels: Vec<String>,
  /// Sort key: `created`, `updated`, or `comments`.
  pub sort: String,
  /// Page size.
  pub per_page: u8,
}

impl IssueParams {
  /// Builds the URL-encoded query string (without the leading `?`).
  pub fn to_query(&self) -> String {
    let mut q = format!(
      "state={}&per_page={}",
      urlencode(&self.state),
      self.per_page
    );
    if !self.labels.is_empty() {
      q.push_str(&format!("&labels={}", urlencode(&self.labels.join(","))));
    }
    if !self.sort.is_empty() {
      q.push_str(&format!("&sort={}", urlencode(&self.sort)));
    }
    q
  }
}

/// Query parameters accepted by `list_pulls`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PullParams {
  /// `open`, `closed`, or `all`.
  pub state: String,
  /// Sort key: `created`, `updated`, or `comments` (mapped to `popularity`).
  pub sort: String,
  /// Page size.
  pub per_page: u8,
}

impl PullParams {
  /// Builds the URL-encoded query string (without the leading `?`).
  pub fn to_query(&self) -> String {
    let mut q = format!(
      "state={}&per_page={}",
      urlencode(&self.state),
      self.per_page
    );
    if !self.sort.is_empty() {
      q.push_str(&format!("&sort={}", urlencode(&self.sort)));
    }
    q
  }
}

/// Minimal pagination cursor parsed from a `Link` header.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Pagination {
  /// Absolute URL of the next page, if any.
  pub next: Option<String>,
}

impl Pagination {
  /// Parses the `Link` header value into a [`Pagination`] cursor.
  ///
  /// The header looks like:
  /// `<https://api.github.com/...&page=2>; rel="next", <...>; rel="last"`.
  /// We only care about `rel="next"`.
  pub fn parse(link_header: Option<&str>) -> Pagination {
    let Some(link) = link_header else {
      return Pagination::default();
    };
    for part in link.split(',') {
      // Each part: <url>; rel="next"
      let mut url = None;
      let mut rel = None;
      for seg in part.split(';') {
        let seg = seg.trim();
        if let Some(stripped) = seg.strip_prefix('<')
          && let Some(u) = stripped.strip_suffix('>')
        {
          url = Some(u.to_string());
        } else if let Some((key, val)) = seg.split_once('=')
          && key.trim() == "rel"
        {
          rel = Some(val.trim().trim_matches('"').to_string());
        }
      }
      if rel.as_deref() == Some("next")
        && let Some(u) = url
      {
        return Pagination { next: Some(u) };
      }
    }
    Pagination::default()
  }

  /// Convenience for tests/headers coming as a map.
  pub fn from_headers(headers: &HashMap<String, String>) -> Pagination {
    Pagination::parse(headers.get("link").map(String::as_str))
  }
}

/// Percent-encodes a query-string component (subset sufficient for GitHub params).
fn urlencode(input: &str) -> String {
  let mut out = String::with_capacity(input.len());
  for byte in input.bytes() {
    match byte {
      b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
        out.push(byte as char);
      }
      _ => out.push_str(&format!("%{byte:02X}")),
    }
  }
  out
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_finds_next_rel() {
    let header = "<https://api.github.com/repos/o/r/issues?page=2>; rel=\"next\", <https://api.github.com/repos/o/r/issues?page=5>; rel=\"last\"";
    let p = Pagination::parse(Some(header));
    assert_eq!(
      p.next.as_deref(),
      Some("https://api.github.com/repos/o/r/issues?page=2")
    );
  }

  #[test]
  fn parse_no_next_returns_none() {
    let header = "<https://api.github.com/repos/o/r/issues?page=5>; rel=\"last\"";
    let p = Pagination::parse(Some(header));
    assert_eq!(p.next, None);
  }

  #[test]
  fn parse_missing_header_is_default() {
    assert_eq!(Pagination::parse(None), Pagination::default());
  }

  #[test]
  fn issue_params_query_string() {
    let p = IssueParams {
      state: "open".into(),
      labels: vec!["bug".into(), "urgent".into()],
      sort: "comments".into(),
      per_page: 30,
    };
    let q = p.to_query();
    assert!(q.contains("state=open"));
    assert!(q.contains("per_page=30"));
    assert!(q.contains("labels=bug%2Curgent") || q.contains("labels=bug,urgent"));
    assert!(q.contains("sort=comments"));
  }

  #[test]
  fn pull_params_query_string() {
    let p = PullParams {
      state: "all".into(),
      sort: "updated".into(),
      per_page: 20,
    };
    let q = p.to_query();
    assert!(q.contains("state=all"));
    assert!(q.contains("sort=updated"));
    assert!(q.contains("per_page=20"));
  }

  #[test]
  fn urlencode_spaces_and_symbols() {
    assert_eq!(urlencode("a b"), "a%20b");
    assert_eq!(urlencode("keep-_.~"), "keep-_.~");
  }
}
