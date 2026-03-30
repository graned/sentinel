//! `sentinel-policy-engine` — a two-phase RBAC rules engine.
//!
//! # Overview
//!
//! The engine separates **compile time** from **evaluate time** so that the
//! expensive JSON → trie transformation happens once (on policy update) and
//! every incoming request pays only a fast trie traversal (~116–213 ns).
//!
//! ## Phase 1 — Compile (`compiler::compile`)
//!
//! Input: a [`PolicyBundle`] (JSON rules with `method`, `path`, and `roles`).
//!
//! 1. Parse each `path` string into a [`Vec<Segment>`] where each segment is
//!    a `Literal`, `Param` (`:name`), `Wildcard` (`*`), or `Glob` (`**`).
//! 2. Insert all `(method, segments, roles)` triples into a prefix trie
//!    ([`compiler::TrieNode`]).
//! 3. Sort each child list by segment precedence (Literal → Param → `*` → `**`)
//!    so the evaluator always tries the most specific match first.
//! 4. Serialize the trie to a `Vec<u8>` with `bincode` — store these bytes in
//!    `policy_versions.compiled_rules`.
//!
//! ## Phase 2 — Evaluate (`PolicyEngine::is_allowed`)
//!
//! Input: compiled bytes (from the DB), plus `method`, `path`, and `roles`.
//!
//! 1. Deserialize the bytes back into a `TrieNode` (one allocation, no JSON).
//! 2. Walk the trie depth-first, consuming one path segment per level.
//! 3. At a leaf node, check whether the caller's roles intersect with the
//!    allowed roles for the given method (or the wildcard method `"*"`).
//!
//! # Segment precedence
//!
//! When multiple child nodes match a segment, the engine tries them in
//! precedence order and returns `true` on the first match:
//!
//! | Priority | Segment type | Example |
//! |----------|-------------|---------|
//! | 0 (highest) | `Literal` | `/users` |
//! | 1 | `Param` | `/:id` |
//! | 2 | `Wildcard` | `/*` |
//! | 3 (lowest) | `Glob` | `/**` |

pub mod compiler;
pub mod engine;
pub mod types;

#[cfg(test)]
mod tests;

pub use compiler::compile;
pub use engine::PolicyEngine;
pub use types::{CompileError, PolicyBundle, Rule, Segment};
