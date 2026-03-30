//! Policy application layer — RBAC rule management and in-process policy evaluation.
//!
//! # Policy engine cache
//!
//! `PolicyApplication` owns an `Arc<RwLock<HashMap<Uuid, CachedEngine>>>` keyed by
//! policy ID.  On every `is_allowed` call the cached version is compared to the DB
//! version; a mismatch triggers a background recompile so authorisation checks
//! remain accurate without a full reload.
//!
//! # Two-phase compile / evaluate lifecycle
//!
//! 1. `create_policy` — stores the JSON rule bundle and compiles it to a binary trie
//!    (bincode-encoded `Vec<u8>`) in a single transaction.
//! 2. `update_policy_rules` — creates a new `policy_version`, compiles, and marks
//!    it active — the cache is invalidated on the next authorisation call.
//! 3. `is_allowed(policy_id, method, path, roles)` — O(path depth) in-memory check.

use crate::{
    http::api::dtos::{
        AuthenticatedUserContext, BatchCheckRequest, BatchCheckResponse, BatchCheckResult,
        CreatePolicyRequest, CreatePolicyResponse, GetPolicyRulesResponse, PolicyRule,
        ProbeResult, RunProbeRequest, RunProbeResponse, UpdatePolicyRulesRequest,
        UpdatePolicyRulesResponse,
    },
    Policy, PolicyService, PostgresClient, ServiceError, SessionService,
};
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::AsyncConnection;
use futures::future::join_all;
use sentinel_policy_engine::{compile, PolicyBundle, PolicyEngine, Rule};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

// ── Cache ─────────────────────────────────────────────────────────────────────

/// The cached engine and the version it represents.
/// Stored together so the version check and engine access are atomic under the RwLock.
struct CachedEngine {
    version: i64,
    engine: PolicyEngine,
}

// ── PolicyApplication ─────────────────────────────────────────────────────────

pub struct PolicyApplication {
    // DB access for transactional compile → store → activate flows
    pg_client: Arc<PostgresClient>,

    // Owns policy + version CRUD and compile orchestration
    policy_service: Arc<PolicyService>,

    // Used to issue short-lived test tokens for the live probe feature
    session_service: Arc<SessionService>,

    // Per-policy engine cache keyed by policy_id.
    // RwLock: many concurrent readers (every request), rare writers (policy activation).
    //
    // Zero-downtime policy updates: in-flight requests using the old engine
    // complete naturally. The next request that detects a cache miss
    // acquires the write lock, inserts the new engine, and serves the new policy.
    //
    // TODO: Add LRU eviction for large policy counts.
    // TODO: Use Postgres LISTEN/NOTIFY for proactive cross-instance invalidation.
    engine: Arc<RwLock<HashMap<Uuid, CachedEngine>>>,
}

impl PolicyApplication {
    pub fn new(
        pg_client: Arc<PostgresClient>,
        policy_service: Arc<PolicyService>,
        session_service: Arc<SessionService>,
    ) -> Self {
        Self {
            pg_client,
            policy_service,
            session_service,
            engine: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // ── List path ─────────────────────────────────────────────────────────────

    /// List all policies. Requires admin role.
    pub async fn list_policies(
        &self,
        ctx: &AuthenticatedUserContext,
    ) -> Result<Vec<Policy>, ServiceError> {
        if !ctx.roles.iter().any(|r| r == "admin") {
            return Err(ServiceError::AuthorizationError("Admin role required".to_string()));
        }
        let mut conn = self.pg_client.get_conn().await?;
        self.policy_service.list_policies(&mut conn).await
    }

    /// Delete a policy and all its versions. Requires admin role.
    pub async fn delete_policy(
        &self,
        ctx: &AuthenticatedUserContext,
        policy_id: Uuid,
    ) -> Result<(), ServiceError> {
        if !ctx.roles.iter().any(|r| r == "admin") {
            return Err(ServiceError::AuthorizationError("Admin role required".to_string()));
        }
        let mut conn = self.pg_client.get_conn().await?;
        self.policy_service
            .get_policy(&mut conn, policy_id)
            .await?
            .ok_or_else(|| ServiceError::NotFoundError("Policy not found".to_string()))?;
        self.policy_service.deactivate_policy(&mut conn, policy_id).await?;
        // Remove only this policy's cache entry.
        self.engine.write().await.remove(&policy_id);
        tracing::info!(policy_id = %policy_id, "Policy deactivated — engine cache entry removed");
        Ok(())
    }

    /// Get the active version's rules for a policy. Requires admin role.
    pub async fn get_active_rules(
        &self,
        ctx: &AuthenticatedUserContext,
        policy_id: Uuid,
    ) -> Result<GetPolicyRulesResponse, ServiceError> {
        if !ctx.roles.iter().any(|r| r == "admin") {
            return Err(ServiceError::AuthorizationError("Admin role required".to_string()));
        }
        let mut conn = self.pg_client.get_conn().await?;

        let policy = self
            .policy_service
            .get_policy(&mut conn, policy_id)
            .await?
            .ok_or_else(|| ServiceError::NotFoundError(format!("Policy {} not found", policy_id)))?;

        let version = self
            .policy_service
            .get_version(&mut conn, policy_id, Some(policy.active_version))
            .await?
            .ok_or_else(|| {
                ServiceError::InternalError("Active policy version not found".to_string())
            })?;

        // rules stored as { "rules": [...] } — extract inner array
        let rules: Vec<PolicyRule> = serde_json::from_value(
            version
                .rules
                .get("rules")
                .cloned()
                .unwrap_or(serde_json::Value::Array(vec![])),
        )
        .map_err(|e| ServiceError::InternalError(format!("Invalid rules format: {}", e)))?;

        Ok(GetPolicyRulesResponse {
            policy_id,
            version: version.version,
            rules,
        })
    }

    // ── Write path ────────────────────────────────────────────────────────────

    /// Compile rules, store as a new policy version, and activate it.
    /// This is the full write path: JSON rules → compiled bytes → DB → active.
    ///
    /// Steps:
    /// 1. Deserialize rules JSON into PolicyBundle
    /// 2. compile() → Vec<u8>
    /// 3. Hash rules and compiled bytes
    /// 4. Determine next version number
    /// 5. INSERT policy_version
    /// 6. UPDATE policies.active_version
    /// 7. Hot-reload the in-memory engine cache
    pub async fn compile_and_activate(
        &self,
        policy_id: Uuid,
        rules: serde_json::Value,
    ) -> Result<i64, ServiceError> {
        let mut conn = self.pg_client.get_conn().await?;

        // Deserialize rules into a PolicyBundle
        let bundle: PolicyBundle = serde_json::from_value(rules.clone())
            .map_err(|e| ServiceError::InternalError(format!("Invalid rules format: {}", e)))?;

        // Compile rules into bytes
        let compiled_bytes = compile(&bundle)
            .map_err(|e| ServiceError::InternalError(format!("Compile error: {}", e)))?;

        // Hash both for integrity checks
        let rules_hash = sha256_hex(&rules.to_string());
        let compiled_hash = sha256_hex_bytes(&compiled_bytes);

        // Determine next version number
        let next_version = match self
            .policy_service
            .get_version(&mut conn, policy_id, None)
            .await?
        {
            Some(latest) => latest.version + 1,
            None => 1,
        };

        // Build the new PolicyVersion record
        // is_active starts false; activate_version will flip it.
        let new_version = crate::PolicyVersion {
            policy_version_id: Uuid::new_v4(),
            policy_id,
            version: next_version,
            rules,
            rules_hash,
            compiled_rules: compiled_bytes.clone(),
            compiled_hash,
            compiler_version: env!("CARGO_PKG_VERSION").to_string(),
            compiled_at: chrono::Utc::now(),
            created_at: chrono::Utc::now(),
            is_active: false,
        };

        // Store the new version
        self.policy_service
            .create_policy_version(&mut conn, &new_version)
            .await?;

        // Activate the new version on the policy
        self.policy_service
            .activate_version(&mut conn, policy_id, next_version)
            .await?;

        // Hot-reload the cache from the freshly compiled bytes
        self.swap_engine(policy_id, next_version, &compiled_bytes).await?;

        tracing::info!(
            policy_id = %policy_id,
            version = next_version,
            "Policy compiled and activated"
        );

        Ok(next_version)
    }

    // ── Read path ─────────────────────────────────────────────────────────────

    /// Evaluate whether the given roles can perform method on path.
    ///
    /// Strategy:
    /// - Fast path (specific policy_id): cache hit → evaluate with zero DB calls.
    /// - Slow path (no policy_id or cache miss): resolve policy from DB, warm
    ///   per-policy cache slot, then evaluate.
    ///
    /// Cache freshness is guaranteed by the write path: every `create_policy` and
    /// `update_policy_rules` call `swap_engine` after committing, so the cache is
    /// always up to date — no version comparison against DB is needed on reads.
    pub async fn is_allowed(
        &self,
        policy_id: Option<Uuid>,
        method: &str,
        path: &str,
        roles: &[String],
    ) -> Result<(bool, i64), ServiceError> {
        // Fast path: specific policy_id + cache is warm — zero DB calls
        if let Some(id) = policy_id {
            let cache = self.engine.read().await;
            if let Some(cached) = cache.get(&id) {
                tracing::debug!(
                    policy_id = %id,
                    cached_version = cached.version,
                    method,
                    path,
                    roles = ?roles,
                    "Policy cache hit (fast path)"
                );
                return Ok((cached.engine.is_allowed(method, path, roles), cached.version));
            }
        } // read lock dropped

        // Cold path: need DB to resolve policy_id and/or load compiled rules
        tracing::info!(
            policy_id = ?policy_id,
            method,
            path,
            "Policy cache miss — loading from DB"
        );

        let mut conn = self.pg_client.get_conn().await?;

        // When a specific policy_id is given, evaluate only that policy.
        // When policy_id is None, evaluate ALL active policies with OR semantics:
        // access is granted if any policy allows the request.
        if let Some(resolved_id) = policy_id {
            let policy = self
                .policy_service
                .get_policy(&mut conn, resolved_id)
                .await?
                .ok_or_else(|| ServiceError::InternalError("Policy not found".to_string()))?;

            let active_version = policy.active_version;

            let policy_version = self
                .policy_service
                .get_version(&mut conn, resolved_id, Some(active_version))
                .await?
                .ok_or_else(|| {
                    ServiceError::InternalError(format!(
                        "Active policy version {} not found",
                        active_version
                    ))
                })?;

            self.swap_engine(resolved_id, active_version, &policy_version.compiled_rules)
                .await?;

            let cache = self.engine.read().await;
            let cached = cache
                .get(&resolved_id)
                .ok_or_else(|| ServiceError::InternalError("Engine failed to load".to_string()))?;

            return Ok((cached.engine.is_allowed(method, path, roles), active_version));
        }

        // No specific policy_id — evaluate all active policies; allow if any permits.
        let policies = self.policy_service.list_policies(&mut conn).await?;
        if policies.is_empty() {
            return Err(ServiceError::InternalError("No policy found".to_string()));
        }

        for policy in &policies {
            let resolved_id = policy.policy_id;
            let active_version = policy.active_version;

            // Per-policy cache check
            {
                let cache = self.engine.read().await;
                if let Some(cached) = cache.get(&resolved_id) {
                    if cached.version == active_version {
                        if cached.engine.is_allowed(method, path, roles) {
                            return Ok((true, cached.version));
                        }
                        continue;
                    }
                }
            }

            // Cache miss for this policy — load compiled rules from DB
            let policy_version = self
                .policy_service
                .get_version(&mut conn, resolved_id, Some(active_version))
                .await?
                .ok_or_else(|| {
                    ServiceError::InternalError(format!(
                        "Active policy version {} not found",
                        active_version
                    ))
                })?;

            self.swap_engine(resolved_id, active_version, &policy_version.compiled_rules)
                .await?;

            let cache = self.engine.read().await;
            if let Some(cached) = cache.get(&resolved_id) {
                if cached.engine.is_allowed(method, path, roles) {
                    return Ok((true, cached.version));
                }
            }
        }

        Ok((false, 0))
    }

    // ── Write path ────────────────────────────────────────────────────────────────

    /// Create a new policy and compile + activate its initial rules in one transaction.
    ///
    /// Steps:
    /// 1. INSERT policy (with active_version = 1)
    /// 2. compile() rules → Vec<u8>
    /// 3. INSERT policy_version (version = 1)
    /// 4. Warm up the in-memory cache
    pub async fn create_policy(
        &self,
        request: CreatePolicyRequest,
    ) -> Result<CreatePolicyResponse, ServiceError> {
        tracing::debug!("Creating policy '{}'", request.name);
        let mut conn = self.pg_client.get_conn().await?;

        let policy_service = self.policy_service.clone();

        // Compile outside the transaction — pure CPU work, no DB needed
        let bundle = PolicyBundle {
            rules: request.rules.iter().map(|r| Rule {
                method: r.method.clone(),
                path: r.path.clone(),
                roles: r.roles.clone(),
            }).collect(),
        };

        let compiled_bytes = compile(&bundle)
            .map_err(|e| ServiceError::InternalError(format!("Compile error: {}", e)))?;

        // Wrap as PolicyBundle JSON for DB storage and hashing
        let rules = serde_json::json!({ "rules": &request.rules });
        let rules_hash = sha256_hex(&rules.to_string());
        let compiled_hash = sha256_hex_bytes(&compiled_bytes);
        let compiled_bytes_clone = compiled_bytes.clone();

        let policy_id = Uuid::new_v4();

        let response = conn
            .transaction(move |trx| {
                let policy_service = policy_service.clone();
                let compiled_bytes = compiled_bytes_clone.clone();

                async move {
                    // INSERT policy — starts at active_version = 1
                    let new_policy = crate::Policy {
                        policy_id,
                        tenant_id: request.tenant_id,
                        environment: request.environment.clone(),
                        name: request.name.clone(),
                        description: request.description.clone(),
                        active_version: 1,
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        is_active: true,
                    };
                    let policy = policy_service.create_policy(trx, &new_policy).await?;

                    // INSERT policy_version version = 1
                    // is_active starts false; activate_version (called below) flips it.
                    let new_version = crate::PolicyVersion {
                        policy_version_id: Uuid::new_v4(),
                        policy_id: policy.policy_id,
                        version: 1,
                        rules,
                        rules_hash,
                        compiled_rules: compiled_bytes,
                        compiled_hash,
                        compiler_version: env!("CARGO_PKG_VERSION").to_string(),
                        compiled_at: chrono::Utc::now(),
                        created_at: chrono::Utc::now(),
                        is_active: false,
                    };
                    policy_service
                        .create_policy_version(trx, &new_version)
                        .await?;

                    // Activate version 1 — flips policy_versions.is_active
                    policy_service.activate_version(trx, policy.policy_id, 1).await?;

                    tracing::info!(
                        policy_id = %policy.policy_id,
                        "Policy '{}' created with version 1",
                        policy.name
                    );

                    Ok::<CreatePolicyResponse, ServiceError>(CreatePolicyResponse {
                        policy_id: policy.policy_id,
                        name: policy.name,
                        environment: policy.environment,
                        description: policy.description,
                        active_version: policy.active_version,
                        is_active: policy.is_active,
                        created_at: policy.created_at,
                    })
                }
                .scope_boxed()
            })
            .await?;

        // Warm up cache outside the transaction — non-critical, log if it fails
        if let Err(e) = self.swap_engine(policy_id, 1, &compiled_bytes).await {
            tracing::warn!("Policy created but cache warm-up failed: {}", e);
        }

        Ok(response)
    }

    /// Compile new rules, store as a new version, and activate it in one transaction.
    ///
    /// Steps:
    /// 1. Load latest version to determine next version number
    /// 2. compile() new rules → Vec<u8>
    /// 3. INSERT policy_version (version = latest + 1)
    /// 4. UPDATE policies.active_version
    /// 5. Hot-reload in-memory cache
    pub async fn update_policy_rules(
        &self,
        policy_id: Uuid,
        request: UpdatePolicyRulesRequest,
    ) -> Result<UpdatePolicyRulesResponse, ServiceError> {
        tracing::debug!("Updating rules for policy {}", policy_id);
        let mut conn = self.pg_client.get_conn().await?;

        let policy_service = self.policy_service.clone();

        // Verify policy exists
        policy_service
            .get_policy(&mut conn, policy_id)
            .await?
            .ok_or_else(|| {
                ServiceError::NotFoundError(format!("Policy {} not found", policy_id))
            })?;

        // Determine next version number
        let next_version = match policy_service
            .get_version(&mut conn, policy_id, None)
            .await?
        {
            Some(latest) => latest.version + 1,
            None => 1,
        };

        // Compile outside the transaction
        let bundle = PolicyBundle {
            rules: request.rules.iter().map(|r| Rule {
                method: r.method.clone(),
                path: r.path.clone(),
                roles: r.roles.clone(),
            }).collect(),
        };

        let compiled_bytes = compile(&bundle)
            .map_err(|e| ServiceError::InternalError(format!("Compile error: {}", e)))?;

        // Wrap as PolicyBundle JSON for DB storage and hashing
        let rules = serde_json::json!({ "rules": &request.rules });
        let rules_hash = sha256_hex(&rules.to_string());
        let compiled_hash = sha256_hex_bytes(&compiled_bytes);
        let compiled_bytes_clone = compiled_bytes.clone();

        let response = conn
            .transaction(move |trx| {
                let policy_service = policy_service.clone();
                let compiled_bytes = compiled_bytes_clone.clone();

                async move {
                    // INSERT new policy_version
                    // is_active starts false; activate_version (called below) flips it.
                    let new_version = crate::PolicyVersion {
                        policy_version_id: Uuid::new_v4(),
                        policy_id,
                        version: next_version,
                        rules,
                        rules_hash,
                        compiled_rules: compiled_bytes,
                        compiled_hash,
                        compiler_version: env!("CARGO_PKG_VERSION").to_string(),
                        compiled_at: chrono::Utc::now(),
                        created_at: chrono::Utc::now(),
                        is_active: false,
                    };
                    policy_service
                        .create_policy_version(trx, &new_version)
                        .await?;

                    // UPDATE policies.active_version
                    policy_service
                        .activate_version(trx, policy_id, next_version)
                        .await?;

                    tracing::info!(
                        policy_id = %policy_id,
                        version = next_version,
                        "Policy rules updated and version {} activated",
                        next_version
                    );

                    Ok::<UpdatePolicyRulesResponse, ServiceError>(UpdatePolicyRulesResponse {
                        policy_id,
                        activated_version: next_version,
                    })
                }
                .scope_boxed()
            })
            .await?;

        // Hot-reload cache outside the transaction
        if let Err(e) = self.swap_engine(policy_id, next_version, &compiled_bytes).await {
            tracing::warn!(
                "Policy {} version {} activated in DB but cache reload failed: {}",
                policy_id,
                next_version,
                e
            );
        }

        Ok(response)
    }

    // ── Batch Authorization Check ─────────────────────────────────────────────

    /// Evaluate multiple method+path checks against a set of roles in one call.
    /// Public endpoint — no auth context required (same as check_authorization).
    pub async fn check_authorization_batch(
        &self,
        request: BatchCheckRequest,
    ) -> Result<BatchCheckResponse, ServiceError> {
        let mut results = Vec::with_capacity(request.checks.len());
        let mut evaluated_version = 0i64;

        for check in &request.checks {
            let (allowed, version) = self
                .is_allowed(request.policy_id, &check.method, &check.path, &request.roles)
                .await?;
            evaluated_version = version;
            results.push(BatchCheckResult {
                method: check.method.clone(),
                path: check.path.clone(),
                allowed,
            });
        }

        Ok(BatchCheckResponse {
            policy_id: request.policy_id,
            evaluated_version,
            results,
        })
    }

    // ── Live Endpoint Probe ───────────────────────────────────────────────────

    /// Fetch the active policy rules, issue an internal test token, and probe each
    /// rule against the customer's live app at `base_url`. Requires admin role.
    pub async fn run_probe(
        &self,
        ctx: &AuthenticatedUserContext,
        policy_id: Uuid,
        request: RunProbeRequest,
    ) -> Result<RunProbeResponse, ServiceError> {
        if !ctx.roles.iter().any(|r| r == "admin") {
            return Err(ServiceError::AuthorizationError("Admin role required".to_string()));
        }

        // Fetch the active rules — these define the test cases
        let rules_response = self.get_active_rules(ctx, policy_id).await?;
        let version = rules_response.version;

        // Issue a short-lived internal test token (never returned to the caller)
        let test_token = self
            .session_service
            .generate_test_token(&request.roles, policy_id, version)?;

        let base_url = request.base_url.trim_end_matches('/').to_string();

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| ServiceError::InternalError(format!("Failed to build HTTP client: {e}")))?;

        // Fan-out all rule probes concurrently
        let probe_futures: Vec<_> = rules_response
            .rules
            .iter()
            .map(|rule| {
                let client = client.clone();
                let url = format!("{}{}", base_url, rule.path);
                let method_str = rule.method.clone();
                let path = rule.path.clone();
                let token = test_token.clone();

                async move {
                    let method = reqwest::Method::from_bytes(method_str.to_uppercase().as_bytes())
                        .unwrap_or(reqwest::Method::GET);

                    match client.request(method, &url).bearer_auth(&token).send().await {
                        Ok(resp) => {
                            let status = resp.status().as_u16();
                            ProbeResult {
                                method: method_str,
                                path,
                                allowed: status == 200,
                                status_code: Some(status),
                                error: None,
                            }
                        }
                        Err(e) => ProbeResult {
                            method: method_str,
                            path,
                            allowed: false,
                            status_code: None,
                            error: Some(e.to_string()),
                        },
                    }
                }
            })
            .collect();

        let results = join_all(probe_futures).await;

        Ok(RunProbeResponse {
            policy_id,
            evaluated_version: version,
            roles_tested: request.roles,
            base_url: request.base_url,
            results,
        })
    }

    // ── Internal ──────────────────────────────────────────────────────────────

    /// Deserialize compiled bytes into a PolicyEngine and insert/replace the
    /// per-policy cache slot. Acquires the write lock briefly.
    async fn swap_engine(
        &self,
        policy_id: Uuid,
        version: i64,
        compiled_bytes: &[u8],
    ) -> Result<(), ServiceError> {
        let engine = PolicyEngine::from_bytes(compiled_bytes)
            .map_err(|e| ServiceError::InternalError(format!("Engine load error: {}", e)))?;

        let mut cache = self.engine.write().await;
        let prev_version = cache.get(&policy_id).map(|c| c.version);
        cache.insert(policy_id, CachedEngine { version, engine });

        tracing::info!(
            policy_id = %policy_id,
            new_version = version,
            prev_version = ?prev_version,
            compiled_bytes = compiled_bytes.len(),
            cached_policies = cache.len(),
            "Policy engine cache swapped (hot-reload)"
        );

        Ok(())
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn sha256_hex(s: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    hex::encode(h.finalize())
}

fn sha256_hex_bytes(b: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(b);
    hex::encode(h.finalize())
}
