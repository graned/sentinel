use crate::{
    compile,
    engine::PolicyEngine,
    types::{CompileError, PolicyBundle, Segment},
    compiler::parse_path, // make parse_path pub(crate) for testing
};

// ── helpers ──────────────────────────────────────────────────────────────────

fn bundle(json: &str) -> PolicyBundle {
    serde_json::from_str(json).expect("invalid test bundle")
}

fn engine(json: &str) -> PolicyEngine {
    let b = bundle(json);
    let bytes = compile(&b).expect("compile failed");
    PolicyEngine::from_bytes(&bytes).expect("from_bytes failed")
}

fn roles(r: &[&str]) -> Vec<String> {
    r.iter().map(|s| s.to_string()).collect()
}

// ── compile / serialization ───────────────────────────────────────────────────

#[test]
fn test_compile_produces_bytes() {
    let b = bundle(r#"{"rules":[{"method":"GET","path":"/users","roles":["admin"]}]}"#);
    let bytes = compile(&b).unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_roundtrip_from_bytes() {
    let b = bundle(r#"{"rules":[{"method":"GET","path":"/users","roles":["admin"]}]}"#);
    let bytes = compile(&b).unwrap();
    // Should not panic
    PolicyEngine::from_bytes(&bytes).unwrap();
}

#[test]
fn test_from_bytes_invalid_data() {
    let result = PolicyEngine::from_bytes(b"not valid bincode at all");
    assert!(matches!(result, Err(CompileError::Serialization(_))));
}

// ── segment parsing ───────────────────────────────────────────────────────────

#[test]
fn test_parse_literal_segments() {
    let segs = parse_path("/users/profile").unwrap();
    assert_eq!(segs, vec![
        Segment::Literal("users".into()),
        Segment::Literal("profile".into()),
    ]);
}

#[test]
fn test_parse_named_param() {
    let segs = parse_path("/users/:id").unwrap();
    assert_eq!(segs[1], Segment::Param("id".into()));
}

#[test]
fn test_parse_wildcard() {
    let segs = parse_path("/users/*").unwrap();
    assert_eq!(segs[1], Segment::Wildcard);
}

#[test]
fn test_parse_glob() {
    let segs = parse_path("/users/**").unwrap();
    assert_eq!(segs[1], Segment::Glob);
}

#[test]
fn test_parse_root_path() {
    let segs = parse_path("/").unwrap();
    assert_eq!(segs, vec![]);
}

#[test]
fn test_parse_glob_must_be_last() {
    let result = parse_path("/users/**/posts");
    assert!(matches!(result, Err(CompileError::InvalidPath(_))));
}

#[test]
fn test_parse_double_slash_invalid() {
    let result = parse_path("/users//posts");
    assert!(matches!(result, Err(CompileError::InvalidPath(_))));
}

// ── literal matching ──────────────────────────────────────────────────────────

#[test]
fn test_exact_literal_allowed() {
    let e = engine(r#"{"rules":[{"method":"GET","path":"/users","roles":["admin"]}]}"#);
    assert!(e.is_allowed("GET", "/users", &roles(&["admin"])));
}

#[test]
fn test_exact_literal_wrong_role() {
    let e = engine(r#"{"rules":[{"method":"GET","path":"/users","roles":["admin"]}]}"#);
    assert!(!e.is_allowed("GET", "/users", &roles(&["user"])));
}

#[test]
fn test_exact_literal_wrong_method() {
    let e = engine(r#"{"rules":[{"method":"GET","path":"/users","roles":["admin"]}]}"#);
    assert!(!e.is_allowed("POST", "/users", &roles(&["admin"])));
}

#[test]
fn test_no_match_unknown_path() {
    let e = engine(r#"{"rules":[{"method":"GET","path":"/users","roles":["admin"]}]}"#);
    assert!(!e.is_allowed("GET", "/orders", &roles(&["admin"])));
}

#[test]
fn test_deep_literal_path() {
    let e = engine(r#"{"rules":[{"method":"POST","path":"/api/v1/users/create","roles":["admin"]}]}"#);
    assert!(e.is_allowed("POST", "/api/v1/users/create", &roles(&["admin"])));
    assert!(!e.is_allowed("POST", "/api/v1/users", &roles(&["admin"])));
}

// ── named param :param ────────────────────────────────────────────────────────

#[test]
fn test_named_param_matches_any_segment() {
    let e = engine(r#"{"rules":[{"method":"GET","path":"/users/:id","roles":["user"]}]}"#);
    assert!(e.is_allowed("GET", "/users/123",   &roles(&["user"])));
    assert!(e.is_allowed("GET", "/users/abc",   &roles(&["user"])));
    assert!(e.is_allowed("GET", "/users/uuid-x",&roles(&["user"])));
}

#[test]
fn test_named_param_does_not_match_extra_segments() {
    let e = engine(r#"{"rules":[{"method":"GET","path":"/users/:id","roles":["user"]}]}"#);
    assert!(!e.is_allowed("GET", "/users/123/posts", &roles(&["user"])));
}

// ── single wildcard * ─────────────────────────────────────────────────────────

#[test]
fn test_wildcard_matches_single_segment() {
    let e = engine(r#"{"rules":[{"method":"GET","path":"/files/*","roles":["user"]}]}"#);
    assert!(e.is_allowed("GET", "/files/report.pdf", &roles(&["user"])));
}

#[test]
fn test_wildcard_does_not_match_multiple_segments() {
    let e = engine(r#"{"rules":[{"method":"GET","path":"/files/*","roles":["user"]}]}"#);
    assert!(!e.is_allowed("GET", "/files/2024/report.pdf", &roles(&["user"])));
}

// ── glob ** ───────────────────────────────────────────────────────────────────

#[test]
fn test_glob_matches_single_remaining_segment() {
    let e = engine(r#"{"rules":[{"method":"DELETE","path":"/users/**","roles":["admin"]}]}"#);
    assert!(e.is_allowed("DELETE", "/users/123", &roles(&["admin"])));
}

#[test]
fn test_glob_matches_multiple_remaining_segments() {
    let e = engine(r#"{"rules":[{"method":"DELETE","path":"/users/**","roles":["admin"]}]}"#);
    assert!(e.is_allowed("DELETE", "/users/123/posts/456", &roles(&["admin"])));
}

#[test]
fn test_glob_wrong_role() {
    let e = engine(r#"{"rules":[{"method":"DELETE","path":"/users/**","roles":["admin"]}]}"#);
    assert!(!e.is_allowed("DELETE", "/users/123/posts", &roles(&["user"])));
}

// ── method wildcard * ─────────────────────────────────────────────────────────

#[test]
fn test_method_wildcard_matches_any_method() {
    let e = engine(r#"{"rules":[{"method":"*","path":"/public/health","roles":["guest"]}]}"#);
    assert!(e.is_allowed("GET",    "/public/health", &roles(&["guest"])));
    assert!(e.is_allowed("POST",   "/public/health", &roles(&["guest"])));
    assert!(e.is_allowed("DELETE", "/public/health", &roles(&["guest"])));
}

// ── precedence ────────────────────────────────────────────────────────────────

#[test]
fn test_literal_takes_precedence_over_param() {
    // Literal rule allows only admin; param rule allows user
    // A request matching the literal should require admin, not fall through to param
    let e = engine(r#"{"rules":[
        {"method":"GET","path":"/users/me",  "roles":["admin"]},
        {"method":"GET","path":"/users/:id", "roles":["user"]}
    ]}"#);

    // "me" matches literal first — user role should NOT be enough
    assert!(e.is_allowed("GET", "/users/me",  &roles(&["admin"])));
    assert!(!e.is_allowed("GET", "/users/me", &roles(&["user"])));

    // "123" only matches :id — user role is sufficient
    assert!(e.is_allowed("GET", "/users/123", &roles(&["user"])));
}

#[test]
fn test_param_takes_precedence_over_wildcard() {
    let e = engine(r#"{"rules":[
        {"method":"GET","path":"/a/:id","roles":["admin"]},
        {"method":"GET","path":"/a/*",  "roles":["user"]}
    ]}"#);

    // :id is more specific than * — user alone should not be enough
    assert!(e.is_allowed("GET", "/a/123", &roles(&["admin"])));
    assert!(!e.is_allowed("GET", "/a/123", &roles(&["user"])));
}

#[test]
fn test_wildcard_takes_precedence_over_glob() {
    let e = engine(r#"{"rules":[
        {"method":"GET","path":"/a/*", "roles":["admin"]},
        {"method":"GET","path":"/a/**","roles":["user"]}
    ]}"#);

    // Single segment — * should match before ** 
    assert!(e.is_allowed("GET", "/a/x",   &roles(&["admin"])));
    assert!(!e.is_allowed("GET", "/a/x",  &roles(&["user"])));

    // Multi-segment — only ** can match
    assert!(e.is_allowed("GET", "/a/x/y", &roles(&["user"])));
}

// ── multiple roles ────────────────────────────────────────────────────────────

#[test]
fn test_any_matching_role_grants_access() {
    let e = engine(r#"{"rules":[{"method":"GET","path":"/reports","roles":["admin","auditor"]}]}"#);
    assert!(e.is_allowed("GET", "/reports", &roles(&["auditor"])));
    assert!(e.is_allowed("GET", "/reports", &roles(&["admin", "auditor"])));
    assert!(!e.is_allowed("GET", "/reports", &roles(&["user"])));
}

#[test]
fn test_user_with_multiple_roles_one_matches() {
    let e = engine(r#"{"rules":[{"method":"POST","path":"/admin","roles":["admin"]}]}"#);
    assert!(e.is_allowed("POST", "/admin", &roles(&["user", "admin"])));
}

// ── edge cases ────────────────────────────────────────────────────────────────

#[test]
fn test_empty_roles_denied() {
    let e = engine(r#"{"rules":[{"method":"GET","path":"/secure","roles":["admin"]}]}"#);
    assert!(!e.is_allowed("GET", "/secure", &[]));
}

#[test]
fn test_empty_rules_bundle() {
    let e = engine(r#"{"rules":[]}"#);
    assert!(!e.is_allowed("GET", "/anything", &roles(&["admin"])));
}

#[test]
fn test_root_path_rule() {
    let e = engine(r#"{"rules":[{"method":"GET","path":"/","roles":["admin"]}]}"#);
    assert!(e.is_allowed("GET", "/", &roles(&["admin"])));
    assert!(e.is_allowed("GET", "",  &roles(&["admin"])));
}

#[test]
fn test_case_insensitive_method() {
    let e = engine(r#"{"rules":[{"method":"GET","path":"/users","roles":["admin"]}]}"#);
    assert!(e.is_allowed("get", "/users", &roles(&["admin"])));
    assert!(e.is_allowed("Get", "/users", &roles(&["admin"])));
}

#[test]
fn test_duplicate_rules_merge_roles() {
    // Same path+method declared twice with different roles — both should work
    let e = engine(r#"{"rules":[
        {"method":"GET","path":"/data","roles":["admin"]},
        {"method":"GET","path":"/data","roles":["analyst"]}
    ]}"#);
    assert!(e.is_allowed("GET", "/data", &roles(&["admin"])));
    assert!(e.is_allowed("GET", "/data", &roles(&["analyst"])));
}
