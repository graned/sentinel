# sentinel-policy-engine

A compiled, high-performance authorization policy engine written in Rust.
Part of the `sentinel-auth` monorepo.

---

## What is this?

`sentinel-policy-engine` is an RBAC (Role-Based Access Control) engine that works like this:

1. You define authorization rules in JSON (which method + path requires which roles)
2. The engine **compiles** those rules into a trie sorted by precedence at compile time and serializes it to bytes
3. The compiled bytes are **stored in Postgres** (`policy_versions.compiled_rules`)
4. At runtime the bytes are **loaded into memory** and evaluated on every request — fast

This is the same model used by Auth0 and Supabase: separate the slow compile step (write-time) from the fast evaluation step (request-time).

---

## Repository location

```
sentinel-auth/
├── Cargo.toml
├── apps/
│   ├── sentinel-core/            # Main backend (Axum)
│   └── ui/
└── packages/
    └── sentinel-policy-engine/   # ← this package
        ├── src/
        │   ├── lib.rs
        │   ├── types.rs          # Rule, PolicyBundle, Segment, CompileError
        │   ├── compiler.rs       # compile() — builds and serializes the trie
        │   ├── engine.rs         # PolicyEngine — loads and evaluates
        │   └── tests.rs          # Unit tests
        ├── benches/
        │   └── policy_engine.rs  # Criterion benchmarks
        └── Cargo.toml
```

---

## Rule format

Rules are defined in JSON and stored in the `policy_versions.rules` column (JSONB):

```json
{
  "rules": [
    { "method": "GET",    "path": "/users/:id",  "roles": ["user", "admin"] },
    { "method": "DELETE", "path": "/users/**",   "roles": ["admin"] },
    { "method": "*",      "path": "/public/**",  "roles": ["guest"] }
  ]
}
```

Each rule has three fields:

| Field    | Description                                               |
|----------|-----------------------------------------------------------|
| `method` | HTTP method (`GET`, `POST`, `PUT`, `DELETE`, `*` for any) |
| `path`   | URL path with optional segment types (see below)          |
| `roles`  | List of roles that are allowed                            |

---

## Path segment types

The path is split on `/` and each segment is one of four types:

| Syntax  | Type        | Matches                                |
|---------|-------------|----------------------------------------|
| `users` | Literal     | Exactly the string `users`             |
| `:id`   | Named param | Any single segment (e.g. `123`, `abc`) |
| `*`     | Wildcard    | Any single segment                     |
| `**`    | Glob        | Any number of remaining segments       |

### Precedence (most → least specific)

When multiple rules could match the same path, the most specific one wins:

```
Literal  >  :param  >  *  >  **
```

This means a rule for `/users/me` (literal) will always take precedence over
`/users/:id` (param) for the path `/users/me`, even if the param rule has a
more permissive role list. Precedence is enforced at **compile time** — children
in the trie are sorted by precedence when `compile()` is called, so `evaluate()`
never sorts at runtime.

---

## Public API

### `compile(bundle: &PolicyBundle) -> Result<Vec<u8>, CompileError>`

Takes a `PolicyBundle` (deserialized from JSON) and returns the compiled trie
as a byte vector. Store these bytes in `policy_versions.compiled_rules`.

```rust
let bundle: PolicyBundle = serde_json::from_str(json)?;
let bytes = compile(&bundle)?;
// store bytes in DB
```

### `PolicyEngine::from_bytes(bytes: &[u8]) -> Result<PolicyEngine, CompileError>`

Deserializes the compiled bytes back into a `PolicyEngine` ready for evaluation.
Call this once at startup or when a new policy version is activated.

```rust
let engine = PolicyEngine::from_bytes(&bytes_from_db)?;
```

### `engine.is_allowed(method, path, roles) -> bool`

Evaluates a single authorization request. This is the hot path — call it on
every incoming request. O(depth) complexity.

```rust
let allowed = engine.is_allowed("GET", "/users/123", &["user".to_string()]);
```

---

## How the trie works

At compile time each path is split into segments and inserted into a trie.
After all rules are inserted, the entire trie is sorted by segment precedence
before serialization:

```
/users/:id/posts   →   ["users", ":id", "posts"]
/users/**          →   ["users", "**"]
```

The trie for these two rules looks like:

```
root
 └── "users" (Literal)
      ├── :id (Param)              ← more specific, sorted first
      │    └── "posts" (Literal)  →  rules: { GET: ["user"] }
      └── ** (Glob)               ← least specific, sorted last
               →  rules: { DELETE: ["admin"] }
```

At evaluation time the path is split and walked down the trie in O(depth).
Because children are pre-sorted, `evaluate()` just iterates — no sorting,
no allocation on the hot path. `**` is always last in the sorted vec and is
only tried as a fallback when no other segment type matched, or when there
are remaining segments that `*` could not consume.


## Performance

Benchmarked with Criterion on a policy of 200 rules. The key optimization is
**compile-time sorting**: trie children are sorted by precedence once during
`compile()`, so `evaluate()` never allocates or sorts at runtime.

| Operation                        | Time     | vs initial |
|----------------------------------|----------|------------|
| `is_allowed` — literal hit       | ~116 ns  | -38%       |
| `is_allowed` — param hit         | ~135 ns  | -37%       |
| `is_allowed` — deep path (3 lvl) | ~182 ns  | -46%       |
| `is_allowed` — glob shallow      | ~165 ns  | -35%       |
| `is_allowed` — glob deep         | ~213 ns  | -32%       |
| `is_allowed` — miss              | ~110 ns  | -15%       |
| 10 mixed requests (500 rules)    | ~1.52 µs | -37%       |
| `compile` 1000 rules             | ~414 µs  | -19%       |
| `load_from_bytes` 1000 rules     | ~137 µs  | —          |

`is_allowed` is sub-microsecond across all scenarios. At ~152ns average per
request you get roughly **6.5 million authorization checks per second on a
single thread**.

`load_from_bytes` is ~3x faster than recompiling, which validates the
compile-once-load-many architecture.

---

## Running tests

```bash
# Unit + integration tests
cargo test -p sentinel-policy-engine

# Benchmarks (generates HTML report in target/criterion/)
cargo bench -p sentinel-policy-engine
```

---

## Dependencies

| Crate       | Purpose                                        |
|-------------|------------------------------------------------|
| `serde`     | Derive `Serialize`/`Deserialize` on trie nodes |
| `bincode`   | Fast binary serialization of the compiled trie |
| `thiserror` | Ergonomic error types                          |

Dev only:

| Crate        | Purpose                     |
|--------------|-----------------------------|
| `serde_json` | Parse rule bundles in tests |
| `criterion`  | Benchmarking                |
