//! Pagination and query-parameter helpers.
//!
//! GitHub paginates via the `Link` header rather than page-number query
//! params. [`Pagination::next`] extracts the `rel="next"` URL so callers follow
//! the cursor instead of guessing page numbers (see AGENTS.md §3.6).

use std::collections::HashMap;
use std::fmt::Write;

/// Query parameters accepted by `list_issues`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueParams {
  /// `open`, `closed`, or `all`.
  pub state: String,
  /// Optional label filter.
  pub labels: Vec<String>,
  /// Sort key: `created`, `updated`, or `comments`.
  pub sort: String,
  /// Filter by author (GitHub `creator` param).
  pub creator: String,
  /// Page size.
  pub per_page: u8,
}

impl Default for IssueParams {
  fn default() -> Self {
    Self {
      state: "open".into(),
      labels: Vec::new(),
      sort: "updated".into(),
      creator: String::new(),
      per_page: 30,
    }
  }
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
      let _ = write!(q, "&labels={}", urlencode(&self.labels.join(",")));
    }
    if !self.sort.is_empty() {
      let _ = write!(q, "&sort={}", urlencode(&self.sort));
    }
    if !self.creator.is_empty() {
      let _ = write!(q, "&creator={}", urlencode(&self.creator));
    }
    q
  }
}

/// Query parameters accepted by `list_pulls`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PullParams {
  /// `open`, `closed`, or `all`.
  pub state: String,
  /// Sort key: `created`, `updated`, or `comments` (mapped to `popularity`).
  pub sort: String,
  /// Page size.
  pub per_page: u8,
}

impl Default for PullParams {
  fn default() -> Self {
    Self {
      state: "open".into(),
      sort: "updated".into(),
      per_page: 30,
    }
  }
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
      let _ = write!(q, "&sort={}", urlencode(&self.sort));
    }
    q
  }
}

/// Minimal pagination cursor parsed from a `Link` header.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Pagination {
  /// Absolute URL of the next page, if any.
  pub next: Option<String>,
  /// Absolute URL of the last page, if any.
  pub last: Option<String>,
}

impl Pagination {
  /// Parses the `Link` header value into a [`Pagination`] cursor.
  ///
  /// The header looks like:
  /// `<https://api.github.com/...&page=2>; rel="next", <...>; rel="last"`.
  /// We only care about `rel="next"`.
  pub fn parse(link_header: Option<&str>) -> Self {
    let Some(link) = link_header else {
      return Self::default();
    };
    let mut next = None;
    let mut last = None;
    for part in link.split(',') {
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
      match rel.as_deref() {
        Some("next") if url.is_some() => next = url,
        Some("last") if url.is_some() => last = url,
        _ => {}
      }
    }
    Self { next, last }
  }

  /// Convenience for tests/headers coming as a map.
  pub fn from_headers(headers: &HashMap<String, String>) -> Self {
    Self::parse(headers.get("link").map(String::as_str))
  }

  /// Extracts the total page count from the `last` URL's `page=` parameter.
  /// Returns `None` when there is no `last` link or the page number cannot be
  /// parsed.
  pub fn total_pages(&self) -> Option<u32> {
    let last_url = self.last.as_ref()?;
    let page_str = last_url
      .split('?')
      .nth(1)?
      .split('&')
      .find(|p| p.starts_with("page="))?
      .strip_prefix("page=")?;
    page_str.parse().ok()
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
      _ => {
        let _ = write!(out, "%{byte:02X}");
      }
    }
  }
  out
}

#[cfg(test)]
mod tests {
  use super::*;

  fn headers(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
      .iter()
      .map(|(k, v)| (k.to_string(), v.to_string()))
      .collect()
  }

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
  fn parse_finds_last_rel() {
    let header = "<https://api.github.com/repos/o/r/issues?page=2>; rel=\"next\", <https://api.github.com/repos/o/r/issues?page=5>; rel=\"last\"";
    let p = Pagination::parse(Some(header));
    assert_eq!(
      p.last.as_deref(),
      Some("https://api.github.com/repos/o/r/issues?page=5")
    );
  }

  #[test]
  fn parse_no_next_returns_none() {
    let header = "<https://api.github.com/repos/o/r/issues?page=5>; rel=\"last\"";
    let p = Pagination::parse(Some(header));
    assert_eq!(p.next, None);
    assert!(p.last.is_some());
  }

  #[test]
  fn parse_missing_header_is_default() {
    let p = Pagination::parse(None);
    assert_eq!(p.next, None);
    assert_eq!(p.last, None);
  }

  #[test]
  fn total_pages_from_last_url() {
    let p = Pagination {
      next: Some("https://api.github.com/repos/o/r/issues?page=2".into()),
      last: Some("https://api.github.com/repos/o/r/issues?page=5".into()),
    };
    assert_eq!(p.total_pages(), Some(5));
  }

  #[test]
  fn total_pages_none_when_no_last() {
    let p = Pagination::default();
    assert_eq!(p.total_pages(), None);
  }

  #[test]
  fn total_pages_none_when_unparseable() {
    let p = Pagination {
      last: Some("https://api.github.com/repos/o/r/issues".into()),
      ..Default::default()
    };
    assert_eq!(p.total_pages(), None);
  }

  #[test]
  fn issue_params_query_string() {
    let p = IssueParams {
      state: "open".into(),
      labels: vec!["bug".into(), "urgent".into()],
      sort: "comments".into(),
      creator: "alice".into(),
      per_page: 30,
    };
    let q = p.to_query();
    assert!(q.contains("state=open"));
    assert!(q.contains("per_page=30"));
    assert!(q.contains("labels=bug%2Curgent") || q.contains("labels=bug,urgent"));
    assert!(q.contains("sort=comments"));
    assert!(q.contains("creator=alice"));
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

  #[test]
  fn parse_only_next_without_last() {
    let header = "<https://api.github.com/repos/o/r/issues?page=2>; rel=\"next\"";
    let p = Pagination::parse(Some(header));
    assert_eq!(
      p.next.as_deref(),
      Some("https://api.github.com/repos/o/r/issues?page=2")
    );
    assert_eq!(p.last, None);
  }

  #[test]
  fn parse_multiple_pages() {
    let header = "<https://api.github.com/repos/o/r/issues?page=3>; rel=\"next\", <https://api.github.com/repos/o/r/issues?page=10>; rel=\"last\"";
    let p = Pagination::parse(Some(header));
    assert_eq!(p.total_pages(), Some(10));
    assert_eq!(
      p.next.as_deref(),
      Some("https://api.github.com/repos/o/r/issues?page=3")
    );
  }

  #[test]
  fn parse_empty_string_is_default() {
    let p = Pagination::parse(Some(""));
    assert_eq!(p.next, None);
    assert_eq!(p.last, None);
  }

  #[test]
  fn total_pages_single_page() {
    let p = Pagination {
      next: None,
      last: Some("https://api.github.com/repos/o/r/issues?page=1".into()),
    };
    assert_eq!(p.total_pages(), Some(1));
  }

  #[test]
  fn from_headers_extracts_link() {
    let h = headers(&[(
      "link",
      "<https://api.github.com/repos/o/r/issues?page=2>; rel=\"next\"",
    )]);
    let p = Pagination::from_headers(&h);
    assert!(p.next.is_some());
  }

  #[test]
  fn from_headers_missing_link_is_default() {
    let h = headers(&[("content-type", "application/json")]);
    let p = Pagination::from_headers(&h);
    assert_eq!(p, Pagination::default());
  }
}
