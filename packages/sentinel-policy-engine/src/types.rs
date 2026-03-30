use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The top-level input to the compiler — deserialized from the JSONB `rules` column
/// stored in `policy_versions.rules`.
///
/// Example JSON:
/// ```json
/// {
///   "rules": [
///     { "method": "GET",  "path": "/v1/api/user/me",  "roles": ["user", "admin"] },
///     { "method": "*",    "path": "/v1/api/admin/**", "roles": ["admin"] }
///   ]
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct PolicyBundle {
    pub rules: Vec<Rule>,
}

/// A single RBAC rule: which roles are allowed to call a given method on a path.
///
/// - `method`: HTTP verb (`"GET"`, `"POST"`, etc.) or `"*"` to match any method.
/// - `path`: URL path with optional segments:
///   - `/literal` — exact match
///   - `/:param`  — matches any single segment
///   - `/*`       — matches any single segment (anonymous)
///   - `/**`      — matches any number of remaining segments (must be last)
/// - `roles`: list of role strings that are permitted to access this route.
#[derive(Debug, Deserialize)]
pub struct Rule {
    pub method: String,
    pub path: String,
    pub roles: Vec<String>,
}

/// A parsed path segment stored as a node key in the compiled trie.
///
/// Segments are sorted by [`Segment::precedence`] at compile time so the evaluator
/// always tries the most-specific match first (Literal before Param before wildcards).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Segment {
    /// An exact path component, e.g. `"users"` in `/users/:id`.
    Literal(String),
    /// A named path parameter, e.g. `:id` in `/users/:id`. Matches any single segment.
    Param(String),
    /// Anonymous single-segment wildcard (`*`). Matches any one path segment.
    Wildcard,
    /// Multi-segment glob (`**`). Matches zero or more remaining segments.
    /// Must be the last segment in a path.
    Glob,
}

impl Segment {
    /// Parse a single path component string into a [`Segment`].
    ///
    /// - `"**"` → `Glob`
    /// - `"*"`  → `Wildcard`
    /// - `":name"` → `Param("name")`
    /// - anything else → `Literal`
    pub fn parse(s: &str) -> Self {
        if s == "**" {
            Segment::Glob
        } else if s == "*" {
            Segment::Wildcard
        } else if let Some(name) = s.strip_prefix(':') {
            Segment::Param(name.to_string())
        } else {
            Segment::Literal(s.to_string())
        }
    }

    /// Numeric precedence used to sort trie children — lower value = higher priority.
    ///
    /// The evaluator tries children in ascending precedence order so that a
    /// more-specific match always wins over a less-specific one.
    pub fn precedence(&self) -> u8 {
        match self {
            Segment::Literal(_) => 0,
            Segment::Param(_) => 1,
            Segment::Wildcard => 2,
            Segment::Glob => 3,
        }
    }
}

/// Errors that can occur during the compile phase.
#[derive(Debug, Error)]
pub enum CompileError {
    /// A path string contained an empty segment or a misplaced `**`.
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    /// `bincode` serialization of the trie failed.
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
}
