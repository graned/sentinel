use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::{CompileError, PolicyBundle, Segment};

/// A node in the compiled policy trie.
///
/// Each node represents one path segment.  The `rules` map holds the
/// method-to-roles mapping for requests that terminate at this depth.
/// `children` holds the next segment level, pre-sorted by [`Segment::precedence`]
/// so the evaluator tries the most-specific match first without extra sorting.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TrieNode {
    /// HTTP method (uppercase) mapped to the set of role strings allowed at this node.
    /// The special key `"*"` matches any HTTP method.
    pub rules: HashMap<String, Vec<String>>,
    /// Child nodes, sorted by segment precedence (ascending) at compile time.
    /// Iteration order determines match priority: Literal > Param > Wildcard > Glob.
    pub children: Vec<(Segment, TrieNode)>,
}

impl TrieNode {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Compile a [`PolicyBundle`] into a binary trie representation.
///
/// The output `Vec<u8>` is a `bincode`-serialized [`TrieNode`] tree.  Store it
/// in `policy_versions.compiled_rules` and pass it to [`PolicyEngine::from_bytes`]
/// to evaluate requests.
///
/// # Errors
/// Returns [`CompileError::InvalidPath`] if any rule has an invalid path (empty
/// segment, misplaced `**`).  Returns [`CompileError::Serialization`] if `bincode`
/// fails (should be unreachable in practice).
pub fn compile(bundle: &PolicyBundle) -> Result<Vec<u8>, CompileError> {
    let rule_count = bundle.rules.len();
    tracing::debug!(rule_count, "Compiling policy bundle");

    let mut root = TrieNode::new();

    for (i, rule) in bundle.rules.iter().enumerate() {
        tracing::trace!(
            index = i,
            method = %rule.method,
            path = %rule.path,
            roles = ?rule.roles,
            "Inserting rule into trie"
        );
        let segments = parse_path(&rule.path)?;
        insert(&mut root, &segments, &rule.method.to_uppercase(), &rule.roles);
    }

    // Sort the entire trie by precedence before serializing
    sort_trie(&mut root);

    let bytes = bincode::serialize(&root)?;
    tracing::debug!(
        rule_count,
        compiled_bytes = bytes.len(),
        "Policy bundle compiled successfully"
    );
    Ok(bytes)
}

/// Recursively sort all children vectors by segment precedence
fn sort_trie(node: &mut TrieNode) {
    node.children.sort_by_key(|(seg, _)| seg.precedence());
    for (_, child) in &mut node.children {
        sort_trie(child);
    }
}

/// Parse a URL path string into an ordered list of [`Segment`]s.
///
/// - Leading `/` is stripped.
/// - The root path `"/"` or `""` returns an empty `Vec` (matches at the root node).
/// - `**` is only valid as the final segment; any other position is an error.
pub fn parse_path(path: &str) -> Result<Vec<Segment>, CompileError> {
    let path = path.trim_start_matches('/');

    if path.is_empty() {
        tracing::trace!(path = "/", "Parsed root path into 0 segments");
        return Ok(vec![]);
    }

    let segments = path
        .split('/')
        .map(|s| {
            if s.is_empty() {
                Err(CompileError::InvalidPath(format!(
                    "Empty segment in path: {}",
                    path
                )))
            } else {
                Ok(Segment::parse(s))
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    for (i, seg) in segments.iter().enumerate() {
        if *seg == Segment::Glob && i != segments.len() - 1 {
            return Err(CompileError::InvalidPath(format!(
                "** must be the last segment in path: {}",
                path
            )));
        }
    }

    tracing::trace!(path, segments = ?segments, "Parsed path segments");
    Ok(segments)
}

fn insert(node: &mut TrieNode, segments: &[Segment], method: &str, roles: &[String]) {
    if segments.is_empty() {
        node.rules
            .entry(method.to_string())
            .or_default()
            .extend(roles.iter().cloned());
        return;
    }

    // Find existing child with this segment or create a new one
    let pos = node.children.iter().position(|(seg, _)| *seg == segments[0]);
    let child = if let Some(i) = pos {
        &mut node.children[i].1
    } else {
        node.children.push((segments[0].clone(), TrieNode::new()));
        &mut node.children.last_mut().unwrap().1
    };

    insert(child, &segments[1..], method, roles);
}
