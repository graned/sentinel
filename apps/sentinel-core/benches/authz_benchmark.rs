//! HTTP authorization round-trip benchmarks.
//!
//! Measures end-to-end latency of `GET /v1/api/user/canary` through the full
//! middleware stack: bearer extraction → PASETO token decrypt+validate →
//! policy engine trie lookup → handler.
//!
//! Two scenarios are timed:
//!
//! - **authorized (200)** — happy path: valid token + allow policy in cache.
//!   Exercises both `authenticate_middleware` and `authorize_middleware`.
//! - **no token (401)** — baseline: `authenticate_middleware` rejects early,
//!   policy engine is never reached.
//!
//! The difference between the two isolates the cost of the policy engine
//! lookup from the rest of the request pipeline.
//!
//! # Running
//! ```bash
//! docker compose -f docker-compose.dev.yml run --rm sentinel-core \
//!   cargo bench --manifest-path apps/sentinel-core/Cargo.toml
//! ```
//! HTML report is written to `target/criterion/`.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use reqwest::blocking::Client;
use serde_json::{json, Value};
use uuid::Uuid;

// ── server URL ────────────────────────────────────────────────────────────────

fn server_url() -> String {
    dotenvy::dotenv().ok();
    let host = std::env::var("APP_HOST").unwrap_or_else(|_| "sentinel-core".to_string());
    let port = std::env::var("APP_PORT").unwrap_or_else(|_| "8000".to_string());
    format!("http://{}:{}", host, port)
}

// ── setup ─────────────────────────────────────────────────────────────────────

struct AuthSession {
    token: String,
}

/// Register a fresh user, log in, and activate an allow policy for
/// `GET /v1/api/user/canary`.
///
/// Called once per benchmark group — setup cost is NOT included in timings.
fn setup_authorized_session(client: &Client, base: &str) -> AuthSession {
    let email = format!("bench-{}@bench.internal", Uuid::new_v4());
    let password = "bench_password_1!";

    // Register
    let status = client
        .post(format!("{base}/v1/api/auth/register"))
        .json(&json!({
            "first_name": "Bench",
            "last_name":  "User",
            "email":      email,
            "password":   password,
        }))
        .send()
        .expect("register: network error")
        .status()
        .as_u16();
    assert_eq!(status, 200, "register failed — is the server running?");

    // Login → access token
    let body: Value = client
        .post(format!("{base}/v1/api/auth/login"))
        .json(&json!({ "email": email, "password": password }))
        .send()
        .expect("login: network error")
        .json()
        .unwrap();
    let token = body["data"]["access_token"]
        .as_str()
        .expect("missing access_token in login response")
        .to_string();

    // Authenticate the token to discover the user's actual roles
    let body: Value = client
        .post(format!("{base}/v1/api/auth/authenticate"))
        .bearer_auth(&token)
        .send()
        .expect("authenticate: network error")
        .json()
        .unwrap();
    let roles: Vec<String> = body["data"]["roles"]
        .as_array()
        .expect("missing roles array in authenticate response")
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();
    assert!(!roles.is_empty(), "user has no roles — policy would never match");

    // Create a policy that grants GET /v1/api/user/canary for every role the
    // user has. One rule per role, matching the integration test pattern.
    let rules: Vec<Value> = roles
        .iter()
        .map(|role| {
            json!({
                "method": "GET",
                "path":   "/v1/api/user/canary",
                "roles":  [role],
            })
        })
        .collect();

    let body: Value = client
        .post(format!("{base}/v1/api/admin/policies"))
        .json(&json!({
            "name":        format!("bench-allow-{}", Uuid::new_v4()),
            "environment": "bench",
            "rules":       rules,
        }))
        .send()
        .expect("create policy: network error")
        .json()
        .unwrap();
    assert_eq!(
        body["success"], true,
        "policy creation failed: {body}"
    );

    AuthSession { token }
}

// ── benchmarks ────────────────────────────────────────────────────────────────

/// Full happy-path latency: valid token + allow policy in engine cache → 200.
///
/// Measures the combined cost of:
///   1. Bearer token extraction
///   2. PASETO v4.local decryption + claim validation
///   3. Policy trie lookup (cache warm, zero DB calls)
///   4. Handler execution + response serialisation
fn bench_canary_authorized(c: &mut Criterion) {
    let client = Client::new();
    let base = server_url();
    let session = setup_authorized_session(&client, &base);
    let url = format!("{base}/v1/api/user/canary");

    c.bench_function("GET /user/canary authorized (200)", |b| {
        b.iter(|| {
            let status = client
                .get(black_box(url.as_str()))
                .bearer_auth(black_box(session.token.as_str()))
                .send()
                .expect("request failed")
                .status()
                .as_u16();
            assert_eq!(status, 200);
        });
    });
}

/// Baseline latency: no bearer token → 401, policy engine never invoked.
///
/// Isolates the cost of the request pipeline *without* token validation or
/// policy evaluation. Subtract this from the authorized benchmark to get the
/// marginal cost of the auth+authz middleware pair.
fn bench_canary_no_token(c: &mut Criterion) {
    let client = Client::new();
    let url = format!("{}/v1/api/user/canary", server_url());

    c.bench_function("GET /user/canary no token (401)", |b| {
        b.iter(|| {
            let status = client
                .get(black_box(url.as_str()))
                .send()
                .expect("request failed")
                .status()
                .as_u16();
            assert_eq!(status, 401);
        });
    });
}

/// Policy-size sensitivity: how does trie depth affect cache-warm lookup time?
///
/// Creates one allow policy with N rules, all pointing to distinct paths.
/// The canary rule is always the *last* rule inserted, exercising the worst
/// case traversal for each policy size.
fn bench_canary_vs_policy_size(c: &mut Criterion) {
    let client = Client::new();
    let base = server_url();

    let mut group = c.benchmark_group("GET /user/canary authorized — policy size");

    for n_extra_rules in [0usize, 10, 50, 100, 500] {
        let email = format!("bench-size-{}@bench.internal", Uuid::new_v4());
        let password = "bench_password_2!";

        // Register + login (setup, not timed)
        client
            .post(format!("{base}/v1/api/auth/register"))
            .json(&json!({
                "first_name": "Bench", "last_name": "Size",
                "email": email, "password": password,
            }))
            .send()
            .expect("register failed");

        let body: Value = client
            .post(format!("{base}/v1/api/auth/login"))
            .json(&json!({ "email": email, "password": password }))
            .send()
            .expect("login failed")
            .json()
            .unwrap();
        let token = body["data"]["access_token"].as_str().unwrap().to_string();

        let body: Value = client
            .post(format!("{base}/v1/api/auth/authenticate"))
            .bearer_auth(&token)
            .send()
            .expect("authenticate failed")
            .json()
            .unwrap();
        let roles: Vec<String> = body["data"]["roles"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();

        // Build rules: N filler rules for different paths + the canary allow rule
        let resources = [
            "users", "orders", "products", "invoices", "reports",
            "teams", "roles", "sessions", "audit-logs", "webhooks",
        ];
        let methods = ["GET", "POST", "PUT", "DELETE", "PATCH"];

        let mut rules: Vec<Value> = (0..n_extra_rules)
            .map(|i| {
                let path = format!("/v1/api/bench/{}/{}", resources[i % resources.len()], i);
                let method = methods[i % methods.len()];
                json!({ "method": method, "path": path, "roles": &roles })
            })
            .collect();

        // Canary allow rule appended last (worst-case insertion order)
        for role in &roles {
            rules.push(json!({
                "method": "GET",
                "path":   "/v1/api/user/canary",
                "roles":  [role],
            }));
        }

        let body: Value = client
            .post(format!("{base}/v1/api/admin/policies"))
            .json(&json!({
                "name":        format!("bench-size-{}-{}", n_extra_rules, Uuid::new_v4()),
                "environment": "bench",
                "rules":       rules,
            }))
            .send()
            .expect("create policy failed")
            .json()
            .unwrap();
        assert_eq!(body["success"], true, "policy creation failed: {body}");

        let url = format!("{base}/v1/api/user/canary");
        let token_clone = token.clone();

        group.bench_with_input(
            BenchmarkId::from_parameter(n_extra_rules),
            &n_extra_rules,
            |b, _| {
                b.iter(|| {
                    let status = client
                        .get(black_box(url.as_str()))
                        .bearer_auth(black_box(token_clone.as_str()))
                        .send()
                        .expect("request failed")
                        .status()
                        .as_u16();
                    assert_eq!(status, 200);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_canary_no_token,
    bench_canary_authorized,
    bench_canary_vs_policy_size,
);
criterion_main!(benches);
