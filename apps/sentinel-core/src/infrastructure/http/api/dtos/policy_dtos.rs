//! DTOs for policy management endpoints: creating policies, updating rules,
//! and the policy-test token flow.

use crate::Policy;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};

fn validate_checks_len(checks: &[BatchCheckItem]) -> Result<(), ValidationError> {
    if checks.len() > 500 {
        let mut e = ValidationError::new("too_many_checks");
        e.message = Some("checks array must not exceed 500 items".into());
        return Err(e);
    }
    Ok(())
}

// ── PolicyRule ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PolicyRule {
    pub method: String,
    pub path: String,
    pub roles: Vec<String>,
}

// ── Validators ────────────────────────────────────────────────────────────────

fn validate_rules(rules: &[PolicyRule]) -> Result<(), ValidationError> {
    if rules.is_empty() {
        let mut e = ValidationError::new("empty_rules");
        e.message = Some("rules array must not be empty".into());
        return Err(e);
    }

    for (i, rule) in rules.iter().enumerate() {
        if rule.method.is_empty() {
            let mut e = ValidationError::new("invalid_rule");
            e.message = Some(format!("rule[{}]: method must not be empty", i).into());
            return Err(e);
        }

        if !rule.path.starts_with('/') {
            let mut e = ValidationError::new("invalid_rule");
            e.message = Some(format!("rule[{}]: path must start with '/'", i).into());
            return Err(e);
        }

        if rule.path.contains("//") {
            let mut e = ValidationError::new("invalid_rule");
            e.message =
                Some(format!("rule[{}]: path must not contain empty segments (//)", i).into());
            return Err(e);
        }

        if rule.roles.is_empty() {
            let mut e = ValidationError::new("invalid_rule");
            e.message = Some(format!("rule[{}]: roles must not be empty", i).into());
            return Err(e);
        }

        for (j, role) in rule.roles.iter().enumerate() {
            if role.is_empty() {
                let mut e = ValidationError::new("invalid_rule");
                e.message = Some(
                    format!(
                        "rule[{}].roles[{}]: each role must be a non-empty string",
                        i, j
                    )
                    .into(),
                );
                return Err(e);
            }
        }
    }

    Ok(())
}

// ── Create Policy DTOs ────────────────────────────────────────────────────────

/// Request to create a new policy with its initial set of rules.
///
/// ## Example
/// ```json
/// {
///   "name": "default",
///   "environment": "prod",
///   "description": "Main API access policy",
///   "rules": [
///     { "method": "GET",    "path": "/users/:id", "roles": ["user", "admin"] },
///     { "method": "DELETE", "path": "/users/**",  "roles": ["admin"] }
///   ]
/// }
/// ```
#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct CreatePolicyRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,

    #[validate(length(min = 1, max = 50))]
    pub environment: String,

    pub description: Option<String>,

    pub tenant_id: Option<Uuid>,

    #[validate(custom(function = "validate_rules"))]
    #[schema(example = json!([{"method": "GET", "path": "/users/:id", "roles": ["admin"]}]))]
    pub rules: Vec<PolicyRule>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CreatePolicyResponse {
    pub policy_id: Uuid,
    pub name: String,
    pub environment: String,
    pub description: Option<String>,
    pub active_version: i64,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

// ── Get Policy DTOs ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PolicyResponse {
    pub policy_id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub name: String,
    pub environment: String,
    pub description: Option<String>,
    pub active_version: i64,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Policy> for PolicyResponse {
    fn from(p: Policy) -> Self {
        Self {
            policy_id: p.policy_id,
            tenant_id: p.tenant_id,
            name: p.name,
            environment: p.environment,
            description: p.description,
            active_version: p.active_version,
            is_active: p.is_active,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}

// ── Get Policy Rules DTOs ─────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct GetPolicyRulesResponse {
    pub policy_id: Uuid,
    pub version: i64,
    pub rules: Vec<PolicyRule>,
}

// ── Policy Version DTOs ───────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct PolicyVersionResponse {
    pub policy_version_id: Uuid,
    pub policy_id: Uuid,
    pub version: i64,
    /// The original rules JSON — never returns compiled_rules bytes
    pub rules: serde_json::Value,
    pub rules_hash: String,
    pub compiled_hash: String,
    pub compiler_version: String,
    pub compiled_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// ── Update Policy Rules DTOs ──────────────────────────────────────────────────

/// Request to compile and activate a new version of an existing policy's rules.
///
/// ## Example
/// ```json
/// {
///   "rules": [
///     { "method": "GET",  "path": "/users/:id", "roles": ["user", "admin"] },
///     { "method": "POST", "path": "/users",     "roles": ["admin"] }
///   ]
/// }
/// ```
#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct UpdatePolicyRulesRequest {
    #[validate(custom(function = "validate_rules"))]
    #[schema(example = json!([{"method": "GET", "path": "/users/:id", "roles": ["admin"]}]))]
    pub rules: Vec<PolicyRule>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UpdatePolicyRulesResponse {
    pub policy_id: Uuid,
    pub activated_version: i64,
}

// ── Batch Authorization Check DTOs ────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct BatchCheckItem {
    pub method: String,
    pub path: String,
}

/// Request to evaluate multiple method+path combinations against a set of roles in one call.
///
/// ## Example
/// ```json
/// { "roles": ["user"], "checks": [{"method": "GET", "path": "/users/:id"}, {"method": "DELETE", "path": "/users/:id"}] }
/// ```
#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct BatchCheckRequest {
    pub policy_id: Option<Uuid>,

    #[validate(length(min = 1))]
    pub roles: Vec<String>,

    #[validate(custom(function = "validate_checks_len"), length(min = 1))]
    pub checks: Vec<BatchCheckItem>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct BatchCheckResult {
    pub method: String,
    pub path: String,
    pub allowed: bool,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct BatchCheckResponse {
    pub policy_id: Option<Uuid>,
    pub evaluated_version: i64,
    pub results: Vec<BatchCheckResult>,
}

// ── Run Probe DTOs ─────────────────────────────────────────────────────────────

/// Request to probe a live customer app with a test token for all rules in a policy.
///
/// ## Example
/// ```json
/// { "base_url": "https://api.myapp.com", "roles": ["admin"] }
/// ```
#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct RunProbeRequest {
    #[validate(length(min = 1, max = 2048))]
    pub base_url: String,

    #[validate(length(min = 1))]
    pub roles: Vec<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ProbeResult {
    pub method: String,
    pub path: String,
    pub allowed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RunProbeResponse {
    pub policy_id: Uuid,
    pub evaluated_version: i64,
    pub roles_tested: Vec<String>,
    pub base_url: String,
    pub results: Vec<ProbeResult>,
}

// ── Check Authorization DTOs ──────────────────────────────────────────────────

/// Request to evaluate whether a set of roles can perform an action.
///
/// ## Example
/// ```json
/// {
///   "method": "DELETE",
///   "path": "/users/123",
///   "roles": ["user"]
/// }
/// ```
#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct CheckAuthorizationRequest {
    pub policy_id: Option<Uuid>,

    #[validate(length(min = 1))]
    pub method: String,

    #[validate(length(min = 1))]
    pub path: String,

    #[validate(length(min = 1))]
    pub roles: Vec<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CheckAuthorizationResponse {
    pub allowed: bool,
    pub method: String,
    pub path: String,
    pub roles: Vec<String>,
    pub active_version: i64,
}
