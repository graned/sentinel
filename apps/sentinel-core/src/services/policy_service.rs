//! Policy service — RBAC rule compilation and version management.
//!
//! Policies govern which roles may access which HTTP method + path combinations.
//! Each policy has a set of **versions** — immutable snapshots of compiled rules.
//! Only one version is active at a time; activating a new version atomically swaps
//! the in-memory engine without downtime.
//!
//! # Two-phase lifecycle
//!
//! 1. **Compile** (`compile_and_store`) — converts JSON rules into a binary trie
//!    (via `sentinel_policy_engine::compile`), stores the result as `compiled_rules`
//!    in `policy_versions`.  SHA-256 hashes of both the JSON and the compiled bytes
//!    are also stored for integrity checks.
//! 2. **Activate** (`activate_version`) — flips the active version pointer and
//!    notifies `PolicyApplication` to hot-reload the in-memory `PolicyEngine`.
//!
//! See `sentinel_policy_engine` crate docs for the trie format and evaluation logic.

use crate::schema::policies;
use crate::{
    DbConnection, Policy, PolicyRepository, PolicyVersion, PolicyVersionRepository, ServiceError,
};
use chrono::Utc;
use diesel::AsChangeset;
use sentinel_policy_engine::{compile, PolicyBundle, PolicyEngine};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;

/// Diesel changeset that updates only `policies.active_version`.
#[derive(AsChangeset)]
#[diesel(table_name = policies)]
struct ActivateVersionChangeset {
    active_version: i64,
}

/// Orchestrates policy and policy-version persistence and compilation.
pub struct PolicyService {
    policy_repository: Arc<PolicyRepository>,
    policy_version_repository: Arc<PolicyVersionRepository>,
}

impl PolicyService {
    pub fn new(
        policy_repository: Arc<PolicyRepository>,
        policy_version_repository: Arc<PolicyVersionRepository>,
    ) -> Self {
        Self {
            policy_repository,
            policy_version_repository,
        }
    }

    /// Create a policy
    pub async fn create_policy(
        &self,
        conn: &mut DbConnection<'_>,
        policy: &Policy,
    ) -> Result<Policy, ServiceError> {
        self.policy_repository
            .create(conn, policy)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    /// Create a policy version
    pub async fn create_policy_version(
        &self,
        conn: &mut DbConnection<'_>,
        policy_version: &PolicyVersion,
    ) -> Result<PolicyVersion, ServiceError> {
        self.policy_version_repository
            .create(conn, policy_version)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    pub async fn get_policy(
        &self,
        conn: &mut DbConnection<'_>,
        policy_id: Uuid,
    ) -> Result<Option<Policy>, ServiceError> {
        self.policy_repository
            .find_by_id(conn, policy_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    /// Returns all policies ordered by `created_at` ascending.
    pub async fn list_policies(
        &self,
        conn: &mut DbConnection<'_>,
    ) -> Result<Vec<Policy>, ServiceError> {
        self.policy_repository
            .find_all(conn)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    /// Returns the first policy (by `created_at` ASC) when no `policy_id` is specified.
    pub async fn get_default_policy(
        &self,
        conn: &mut DbConnection<'_>,
    ) -> Result<Option<Policy>, ServiceError> {
        self.policy_repository
            .find_first(conn)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    /// Compile raw rules JSON, build a PolicyVersion and persist it.
    /// Returns the new version ready to be activated.
    pub async fn compile_and_store(
        &self,
        conn: &mut DbConnection<'_>,
        policy_id: Uuid,
        rules_json: serde_json::Value,
    ) -> Result<PolicyVersion, ServiceError> {
        // 1. Deserialize rules into PolicyBundle
        let bundle: PolicyBundle = serde_json::from_value(rules_json.clone())
            .map_err(|e| ServiceError::InternalError(format!("Invalid rules format: {}", e)))?;

        // 2. Compile → bytes
        let compiled_rules = compile(&bundle)
            .map_err(|e| ServiceError::InternalError(format!("Compile error: {}", e)))?;

        // 3. Hash both rules and compiled bytes
        let rules_hash = sha256_hex(&rules_json.to_string());
        let compiled_hash = sha256_hex_bytes(&compiled_rules);

        // 4. Determine next version number
        let next_version = self
            .get_latest_policy_version(conn, policy_id)
            .await?
            .map(|v| v.version + 1)
            .unwrap_or(1);

        // 5. Build and persist the PolicyVersion
        // is_active starts false; caller must call activate_version to flip it.
        let policy_version = PolicyVersion {
            policy_version_id: Uuid::new_v4(),
            policy_id,
            version: next_version,
            rules: rules_json,
            rules_hash,
            compiled_rules,
            compiled_hash,
            compiler_version: env!("CARGO_PKG_VERSION").to_string(),
            compiled_at: Utc::now(),
            created_at: Utc::now(),
            is_active: false,
        };

        self.policy_version_repository
            .create(conn, &policy_version)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    /// Sets the active_version on a policy to the given version number.
    /// Also flips `is_active` on policy_versions: deactivates all old versions,
    /// then marks the new one as active.
    /// Returns the updated Policy.
    pub async fn activate_version(
        &self,
        conn: &mut DbConnection<'_>,
        policy_id: Uuid,
        version: i64,
    ) -> Result<Policy, ServiceError> {
        // Deactivate all existing version rows for this policy
        self.policy_version_repository
            .deactivate_all_for_policy(conn, policy_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        // Mark the new version as active
        self.policy_version_repository
            .set_version_active(conn, policy_id, version)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        // Update the policy's active_version pointer
        self.policy_repository
            .update(
                conn,
                policy_id,
                ActivateVersionChangeset {
                    active_version: version,
                },
            )
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    /// Soft-deactivate a policy (sets `is_active = false`).
    /// The policy is hidden from list endpoints and the default-policy lookup.
    pub async fn deactivate_policy(
        &self,
        conn: &mut DbConnection<'_>,
        policy_id: Uuid,
    ) -> Result<(), ServiceError> {
        self.policy_repository
            .deactivate(conn, policy_id)
            .await
            .map(|_| ())
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    /// Load the active compiled policy into a live PolicyEngine.
    /// Call this at startup or when a policy version changes.
    pub async fn load_engine(
        &self,
        conn: &mut DbConnection<'_>,
        policy_id: Uuid,
    ) -> Result<PolicyEngine, ServiceError> {
        // Fetch the active version number
        let policy = self
            .policy_repository
            .find_by_id(conn, policy_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ServiceError::InternalError("Policy not found".to_string()))?;

        // Load that specific version
        let version = self
            .get_version(conn, policy_id, Some(policy.active_version))
            .await?
            .ok_or_else(|| {
                ServiceError::InternalError(format!(
                    "Active version {} not found for policy {}",
                    policy.active_version, policy_id
                ))
            })?;

        // Deserialize compiled bytes into engine
        PolicyEngine::from_bytes(&version.compiled_rules)
            .map_err(|e| ServiceError::InternalError(format!("Engine load error: {}", e)))
    }

    /// Delete a policy by ID (cascades to policy_versions via FK).
    pub async fn delete_policy(
        &self,
        conn: &mut DbConnection<'_>,
        policy_id: Uuid,
    ) -> Result<(), ServiceError> {
        self.policy_repository
            .delete(conn, policy_id)
            .await
            .map(|_| ())
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    /// Get the latest policy version
    pub async fn get_latest_policy_version(
        &self,
        conn: &mut DbConnection<'_>,
        policy_id: Uuid,
    ) -> Result<Option<PolicyVersion>, ServiceError> {
        self.policy_version_repository
            .find_latest_for_policy_version(conn, policy_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    /// Get a specific version. If `version` is None, returns the latest.
    pub async fn get_version(
        &self,
        conn: &mut DbConnection<'_>,
        policy_id: Uuid,
        version: Option<i64>,
    ) -> Result<Option<PolicyVersion>, ServiceError> {
        match version {
            Some(v) => self
                .policy_version_repository
                .find_by_policy_and_version(conn, policy_id, v)
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string())),
            None => self
                .policy_version_repository
                .find_latest_for_policy_version(conn, policy_id)
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string())),
        }
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// SHA-256 hex digest of a UTF-8 string (used for rules integrity hashing).
fn sha256_hex(s: &str) -> String {
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    hex::encode(h.finalize())
}

/// SHA-256 hex digest of raw bytes (used for compiled-rules integrity hashing).
fn sha256_hex_bytes(b: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(b);
    hex::encode(h.finalize())
}
