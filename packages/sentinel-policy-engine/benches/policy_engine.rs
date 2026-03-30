use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use sentinel_policy_engine::{compile, engine::PolicyEngine, types::PolicyBundle};

// ── helpers ───────────────────────────────────────────────────────────────────

fn make_engine(json: &str) -> PolicyEngine {
    let bundle: PolicyBundle = serde_json::from_str(json).unwrap();
    let bytes = compile(&bundle).unwrap();
    PolicyEngine::from_bytes(&bytes).unwrap()
}

fn roles(r: &[&str]) -> Vec<String> {
    r.iter().map(|s| s.to_string()).collect()
}

/// Generate a realistic policy with `n` rules across varied paths and methods
fn generate_policy(n: usize) -> String {
    let methods = ["GET", "POST", "PUT", "DELETE", "PATCH"];
    let resources = [
        "users",
        "orders",
        "products",
        "invoices",
        "reports",
        "teams",
        "roles",
        "permissions",
        "sessions",
        "audit-logs",
    ];
    let role_sets = [
        r#"["admin"]"#,
        r#"["user","admin"]"#,
        r#"["auditor","admin"]"#,
        r#"["admin","superuser"]"#,
    ];

    let mut rules = Vec::with_capacity(n);

    for i in 0..n {
        let method = methods[i % methods.len()];
        let res = resources[i % resources.len()];
        let roles = role_sets[i % role_sets.len()];

        // Cycle through path shapes to exercise all trie node types
        let path = match i % 4 {
            0 => format!("/{res}"),
            1 => format!("/{res}/:id"),
            2 => format!("/{res}/:id/details"),
            _ => format!("/{res}/**"),
        };

        rules.push(format!(
            r#"{{"method":"{method}","path":"{path}","roles":{roles}}}"#
        ));
    }

    format!(r#"{{"rules":[{}]}}"#, rules.join(","))
}

// ── benchmarks ────────────────────────────────────────────────────────────────

/// How long does compile() take for policies of varying sizes?
fn bench_compile(c: &mut Criterion) {
    let mut group = c.benchmark_group("compile");

    for size in [10, 100, 500, 1000] {
        let policy_json = generate_policy(size);
        let bundle: PolicyBundle = serde_json::from_str(&policy_json).unwrap();

        group.bench_with_input(BenchmarkId::from_parameter(size), &bundle, |b, bundle| {
            b.iter(|| compile(black_box(bundle)).unwrap());
        });
    }

    group.finish();
}

/// How long does from_bytes() (deserialization) take?
fn bench_load(c: &mut Criterion) {
    let mut group = c.benchmark_group("load_from_bytes");

    for size in [10, 100, 500, 1000] {
        let policy_json = generate_policy(size);
        let bundle: PolicyBundle = serde_json::from_str(&policy_json).unwrap();
        let bytes = compile(&bundle).unwrap();

        group.bench_with_input(BenchmarkId::from_parameter(size), &bytes, |b, bytes| {
            b.iter(|| PolicyEngine::from_bytes(black_box(bytes)).unwrap());
        });
    }

    group.finish();
}

/// Core eval benchmark — this is the hot path, should be sub-microsecond
fn bench_is_allowed(c: &mut Criterion) {
    let mut group = c.benchmark_group("is_allowed");

    // Representative request scenarios
    let scenarios: &[(&str, &str, &str, &[&str])] = &[
        // (label, method, path, roles)
        ("literal_hit", "GET", "/users", &["admin"]),
        ("param_hit", "GET", "/users/123", &["user"]),
        ("deep_literal_hit", "GET", "/users/123/details", &["admin"]),
        ("glob_shallow", "DELETE", "/audit-logs/abc", &["admin"]),
        ("glob_deep", "DELETE", "/audit-logs/abc/xyz/789", &["admin"]),
        ("literal_miss", "GET", "/nonexistent", &["admin"]),
        ("role_denied", "DELETE", "/users/123", &["guest"]),
        (
            "multi_role_last_wins",
            "POST",
            "/orders",
            &["guest", "user", "admin"],
        ),
    ];

    // Use a fixed realistic policy for eval benchmarks
    let policy_json = generate_policy(200);
    let engine = make_engine(&policy_json);

    for (label, method, path, r) in scenarios {
        let role_vec = roles(r);
        group.bench_function(*label, |b| {
            b.iter(|| engine.is_allowed(black_box(method), black_box(path), black_box(&role_vec)));
        });
    }

    group.finish();
}

/// Stress test: rapid-fire mixed evaluations simulating real traffic
fn bench_eval_throughput(c: &mut Criterion) {
    let policy_json = generate_policy(500);
    let engine = make_engine(&policy_json);

    let requests: Vec<(&str, &str, Vec<String>)> = vec![
        ("GET", "/users", roles(&["admin"])),
        ("GET", "/users/123", roles(&["user"])),
        ("PUT", "/users/123", roles(&["admin"])),
        ("DELETE", "/users/123", roles(&["user"])), // denied
        ("GET", "/orders/456/details", roles(&["user", "admin"])),
        ("POST", "/products", roles(&["admin"])),
        ("GET", "/reports/**", roles(&["auditor"])),
        ("PATCH", "/invoices/789", roles(&["admin"])),
        ("GET", "/nonexistent/path", roles(&["admin"])), // miss
        ("DELETE", "/audit-logs/x/y/z", roles(&["admin"])),
    ];

    c.bench_function("eval_throughput_500_rules", |b| {
        b.iter(|| {
            for (method, path, roles) in &requests {
                black_box(engine.is_allowed(black_box(method), black_box(path), black_box(roles)));
            }
        });
    });
}

criterion_group!(
    benches,
    bench_compile,
    bench_load,
    bench_is_allowed,
    bench_eval_throughput,
);
criterion_main!(benches);
