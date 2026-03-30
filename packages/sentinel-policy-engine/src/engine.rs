use crate::compiler::TrieNode;
use crate::types::{CompileError, Segment};

/// The evaluate-time RBAC engine backed by a compiled trie.
///
/// Construct with [`PolicyEngine::from_bytes`] (passing the `compiled_rules`
/// bytes from the DB) and query with [`PolicyEngine::is_allowed`].
///
/// The engine is immutable after construction — a new engine is created each
/// time rules change (via [`PolicyApplication::swap_engine`]).
pub struct PolicyEngine {
    root: TrieNode,
}

impl PolicyEngine {
    /// Deserialize a compiled policy trie from the bytes stored in
    /// `policy_versions.compiled_rules`.
    ///
    /// This is a single `bincode::deserialize` call — no JSON parsing,
    /// no path compilation.  The resulting engine is ready to evaluate requests.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CompileError> {
        tracing::debug!(
            bytes = bytes.len(),
            "Deserializing PolicyEngine from compiled bytes"
        );
        let root = bincode::deserialize(bytes)?;
        tracing::debug!("PolicyEngine deserialized successfully");
        Ok(Self { root })
    }

    /// Check whether any of `roles` is permitted to invoke `method` on `path`.
    ///
    /// The evaluation is O(path depth) — one trie node traversal per segment.
    /// Typical latency is 116–213 ns on modern hardware (see benchmark suite).
    ///
    /// # Matching rules
    ///
    /// Children are tried in precedence order: `Literal` → `Param` → `*` → `**`.
    /// The first child that ultimately reaches an allowed rule wins.
    /// If no non-glob child matches (or a non-glob match doesn't find an allowed
    /// role), the `**` glob fallback is tried.
    ///
    /// Role matching at a leaf node checks the exact method first, then `"*"`.
    pub fn is_allowed(&self, method: &str, path: &str, roles: &[String]) -> bool {
        let method = method.to_uppercase();
        let segments: Vec<&str> = path
            .trim_start_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();

        tracing::debug!(
            method = %method,
            path,
            roles = ?roles,
            segment_count = segments.len(),
            "Evaluating policy rule"
        );

        let result = self.evaluate(&self.root, &segments, &method, roles);

        tracing::debug!(
            method = %method,
            path,
            roles = ?roles,
            allowed = result,
            "Policy evaluation result"
        );

        result
    }

    fn evaluate(&self, node: &TrieNode, segments: &[&str], method: &str, roles: &[String]) -> bool {
        if segments.is_empty() {
            let result = self.check_roles(node, method, roles);
            tracing::trace!(
                method,
                remaining_segments = 0,
                matched = result,
                "Reached leaf node — checking roles"
            );
            return result;
        }

        let current = segments[0];
        let rest = &segments[1..];

        tracing::trace!(
            segment = current,
            remaining = rest.len(),
            children = node.children.len(),
            "Traversing trie node"
        );

        // children is pre-sorted by precedence (literal → param → * → **)
        // so we iterate in order. We track the tightest precedence that
        // structurally matched to avoid bleeding into looser levels.
        let mut tightest_non_glob_precedence: Option<u8> = None;
        let mut non_glob_matched = false;

        for (seg, child) in &node.children {
            if *seg == Segment::Glob {
                // ** is always last in the sorted vec — handle after the loop
                break;
            }

            if !seg_matches(seg, current) {
                continue;
            }

            let prec = seg.precedence();

            // Stop if we've moved past the tightest precedence level
            if let Some(p) = tightest_non_glob_precedence {
                if prec > p {
                    break;
                }
            }

            tightest_non_glob_precedence = Some(prec);
            non_glob_matched = true;

            tracing::trace!(
                segment = current,
                matched_seg = ?seg,
                precedence = prec,
                "Segment matched child node"
            );

            if self.evaluate(child, rest, method, roles) {
                return true;
            }
        }

        // ** fallback — only if no non-glob matched, or there are remaining
        // segments (meaning ** is matching a longer tail than * could)
        if let Some((_, glob_child)) = node.children.iter().find(|(seg, _)| *seg == Segment::Glob) {
            let glob_should_try = !non_glob_matched || !rest.is_empty();
            tracing::trace!(
                segment = current,
                non_glob_matched,
                remaining = rest.len(),
                glob_should_try,
                "Checking glob (**) fallback"
            );
            if glob_should_try {
                return self.check_roles(glob_child, method, roles);
            }
        }

        tracing::trace!(segment = current, "No matching child — denying");
        false
    }

    fn check_roles(&self, node: &TrieNode, method: &str, roles: &[String]) -> bool {
        // Check exact method match
        if let Some(allowed_roles) = node.rules.get(method) {
            let matched = roles.iter().any(|r| allowed_roles.contains(r));
            tracing::trace!(
                method,
                allowed_roles = ?allowed_roles,
                user_roles = ?roles,
                matched,
                "Checking exact method roles"
            );
            if matched {
                return true;
            }
        }
        // Check wildcard method match
        if let Some(allowed_roles) = node.rules.get("*") {
            let matched = roles.iter().any(|r| allowed_roles.contains(r));
            tracing::trace!(
                method,
                allowed_roles = ?allowed_roles,
                user_roles = ?roles,
                matched,
                "Checking wildcard method (*) roles"
            );
            if matched {
                return true;
            }
        }

        tracing::trace!(
            method,
            user_roles = ?roles,
            node_rule_methods = ?node.rules.keys().collect::<Vec<_>>(),
            "No matching role found at this node"
        );
        false
    }
}

/// Returns `true` if `seg` matches the runtime path component `value`.
///
/// `Glob` is never passed here — it is handled by the caller before the loop.
fn seg_matches(seg: &Segment, value: &str) -> bool {
    match seg {
        Segment::Literal(s) => s == value,
        Segment::Param(_) => true, // named param matches any single segment
        Segment::Wildcard => true, // anonymous wildcard matches any single segment
        Segment::Glob => unreachable!("glob is handled separately"),
    }
}
