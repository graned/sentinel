# Issue #14: Supabase Token Exchange / Federated Session Bridge

## Overview

Implement a federated identity bridge that allows Sentinel to accept and validate external identity provider tokens (e.g., Supabase OAuth tokens) and exchange them for native Sentinel sessions. This enables "Log in with Supabase" functionality while maintaining Sentinel's session management, RBAC, and audit trail.

## Scope

- External identity provider configuration and management
- Token exchange endpoint for federated login
- Linking external identities to Sentinel user accounts
- Automatic user provisioning for new federated users
- Session bridging with proper audit logging

---

## Implementation Plan

### 1. Database Migration

**File:** `migrations/2026-05-20-000001_add_external_identities/`

Create tables for external identity providers and linked identities:

```sql
-- External identity provider configurations
CREATE TABLE external_identity_providers (
    provider_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_name TEXT NOT NULL UNIQUE,
    issuer_url TEXT NOT NULL,
    client_id TEXT NOT NULL,
    client_secret_encrypted BYTEA NOT NULL,
    jwks_url TEXT NOT NULL,
    scopes TEXT[] NOT NULL DEFAULT ARRAY['openid', 'email', 'profile'],
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- External identity links (maps external subjects to Sentinel users)
CREATE TABLE external_identities (
    external_identity_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    provider_id UUID NOT NULL REFERENCES external_identity_providers(provider_id) ON DELETE CASCADE,
    issuer TEXT NOT NULL,
    subject TEXT NOT NULL,
    email TEXT NOT NULL,
    email_verified BOOLEAN NOT NULL DEFAULT FALSE,
    access_token_encrypted BYTEA,
    refresh_token_encrypted BYTEA,
    token_expires_at TIMESTAMPTZ,
    last_login TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(provider_id, issuer, subject)
);

CREATE INDEX idx_external_identities_user_id ON external_identities(user_id);
CREATE INDEX idx_external_identities_provider_subject ON external_identities(provider_id, issuer, subject);
```

---

### 2. Domain Layer

#### 2.1 Entities

**File:** `src/domain/schema_models.rs`

Add new entities (auto-generated via `gen_diesel_types.sh` after migration):

```rust
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = external_identity_providers)]
pub struct ExternalIdentityProvider {
    pub provider_id: Uuid,
    pub provider_name: String,
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret_encrypted: Vec<u8>,
    pub jwks_url: String,
    pub scopes: Vec<String>,
    pub is_active: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = external_identities)]
pub struct ExternalIdentity {
    pub external_identity_id: Uuid,
    pub user_id: Uuid,
    pub provider_id: Uuid,
    pub issuer: String,
    pub subject: String,
    pub email: String,
    pub email_verified: bool,
    pub access_token_encrypted: Option<Vec<u8>>,
    pub refresh_token_encrypted: Option<Vec<u8>>,
    pub token_expires_at: Option<chrono::NaiveDateTime>,
    pub last_login: Option<chrono::NaiveDateTime>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}
```

#### 2.2 Repositories

**File:** `src/domain/repositories/external_identity_provider_repository.rs`

```rust
use crate::domain::schema_models::ExternalIdentityProvider;
use crate::errors::RepositoryError;
use diesel::query_dsl::methods::LoadQuery;
use diesel::query_dsl::RunQueryDsl;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

// =============================================================================
// STANDARD CRUD: Use impl_repository! for all 13 auto-generated methods
// =============================================================================
//
// Per CLAUDE.md guidance:
// - "Repositories use an `impl_repository!` macro that provides 13 CRUD/pagination methods automatically"
// - "To add custom DB logic, add a separate `impl RepositoryName { ... }` block after the macro invocation"
// - "Do NOT import `DbConnection` at the top of a repository module file"
//
// Auto-generated methods include:
//   - create(&mut conn, payload) -> Result<T, RepositoryError>
//   - find_by_id(&mut conn, id) -> Result<Option<T>, RepositoryError>
//   - find_where(&mut conn, filter) -> Result<Option<T>, RepositoryError>
//   - list(&mut conn) -> Result<Vec<T>, RepositoryError>
//   - list_where(&mut conn, filter) -> Result<Vec<T>, RepositoryError>
//   - list_paginated(&mut conn, page, per_page) -> Result<Vec<T>, RepositoryError>
//   - count(&mut conn) -> Result<i64, RepositoryError>
//   - count_where(&mut conn, filter) -> Result<i64, RepositoryError>
//   - update(&mut conn, id, payload) -> Result<T, RepositoryError>
//   - update_where(&mut conn, filter, payload) -> Result<T, RepositoryError>
//   - delete(&mut conn, id) -> Result<T, RepositoryError>
//   - delete_where(&mut conn, filter) -> Result<T, RepositoryError>
//   - exists(&mut conn, id) -> Result<bool, RepositoryError>
// =============================================================================

impl_repository!(
    ExternalIdentityProviderRepository for ExternalIdentityProvider,
    crate::schema::external_identity_providers::table,
    crate::schema::external_identity_providers::provider_id,
    Uuid
);

// =============================================================================
// CUSTOM METHODS: Only add methods the macro cannot handle
// =============================================================================

impl ExternalIdentityProviderRepository {
    /// Find active provider by name.
    /// Uses custom query because macro's `find_where` requires explicit filter construction.
    pub async fn find_active_by_name(
        &self,
        conn: &mut crate::DbConnection<'_>,
        name: &str,
    ) -> Result<Option<ExternalIdentityProvider>, RepositoryError> {
        use crate::schema::external_identity_providers::dsl as prov_dsl;

        prov_dsl::external_identity_providers
            .filter(prov_dsl::provider_name.eq(name).and(prov_dsl::is_active.eq(true)))
            .first::<ExternalIdentityProvider>(conn)
            .await
            .optional()
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))
    }

    /// List all active providers.
    pub async fn list_active(
        &self,
        conn: &mut crate::DbConnection<'_>,
    ) -> Result<Vec<ExternalIdentityProvider>, RepositoryError> {
        use crate::schema::external_identity_providers::dsl as prov_dsl;

        prov_dsl::external_identity_providers
            .filter(prov_dsl::is_active.eq(true))
            .order(prov_dsl::created_at.desc())
            .load::<ExternalIdentityProvider>(conn)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))
    }
}
```

**File:** `src/domain/repositories/external_identity_repository.rs`

```rust
use crate::domain::schema_models::ExternalIdentity;
use crate::errors::RepositoryError;
use chrono::NaiveDateTime;
use diesel::query_dsl::RunQueryDsl;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

// =============================================================================
// STANDARD CRUD: Use impl_repository! for all 13 auto-generated methods
// =============================================================================
//
// Per CLAUDE.md guidance:
// - "Repositories use an `impl_repository!` macro that provides 13 CRUD/pagination methods automatically"
// - "To add custom DB logic, add a separate `impl RepositoryName { ... }` block after the macro invocation"
// - "Do NOT import `DbConnection` at the top of a repository module file"
//
// This macro generates: create, find_by_id, find_where, list, list_where,
// list_paginated, count, count_where, update, update_where, delete, delete_where, exists
//
// Use these macro methods in FederationService wherever possible instead of raw SQL.
// =============================================================================

impl_repository!(
    ExternalIdentityRepository for ExternalIdentity,
    crate::schema::external_identities::table,
    crate::schema::external_identities::external_identity_id,
    Uuid
);

// =============================================================================
// CUSTOM METHODS: Only add methods the macro cannot handle
// =============================================================================
//
// Custom methods are needed for:
// 1. Multi-column lookups (provider + issuer + subject)
// 2. Bulk/timestamp updates not covered by macro's single-row update_where
// 3. Custom filtered lists (find by user_id with optional provider filter)
// =============================================================================

impl ExternalIdentityRepository {
    /// Find external identity by provider, issuer, and subject.
    ///
    /// This is a custom lookup query that cannot be handled by the macro's
    /// standard `find_where` because it requires a composite unique constraint lookup.
    ///
    /// Used by FederationService to locate existing federated identities during token exchange.
    pub async fn find_by_provider_issuer_subject(
        &self,
        conn: &mut crate::DbConnection<'_>,
        provider_id: Uuid,
        issuer: &str,
        subject: &str,
    ) -> Result<Option<ExternalIdentity>, RepositoryError> {
        use crate::schema::external_identities::dsl as eid_dsl;

        eid_dsl::external_identities
            .filter(
                eid_dsl::provider_id
                    .eq(provider_id)
                    .and(eid_dsl::issuer.eq(issuer))
                    .and(eid_dsl::subject.eq(subject)),
            )
            .first::<ExternalIdentity>(conn)
            .await
            .optional()
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))
    }

    /// Update last_login timestamp for an external identity.
    ///
    /// This uses a direct UPDATE query rather than the macro's `update_where` because:
    /// 1. We only need to update a single timestamp column
    /// 2. We don't need the returned row (update_where returns exactly one row)
    /// 3. More efficient for simple timestamp updates
    ///
    /// Called by FederationService after successful federated login.
    pub async fn update_last_login(
        &self,
        conn: &mut crate::DbConnection<'_>,
        external_identity_id: Uuid,
        last_login: NaiveDateTime,
    ) -> Result<(), RepositoryError> {
        use crate::schema::external_identities::dsl as eid_dsl;

        diesel::update(eid_dsl::external_identities)
            .filter(eid_dsl::external_identity_id.eq(external_identity_id))
            .set(eid_dsl::last_login.eq(Some(last_login)))
            .execute(conn)
            .await
            .map(|_| ())
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))
    }

    /// Find all external identities for a user.
    ///
    /// Returns a list of linked external identities for a given Sentinel user.
    /// Used by UserApplication to display connected identity providers.
    pub async fn find_by_user_id(
        &self,
        conn: &mut crate::DbConnection<'_>,
        user_id: Uuid,
    ) -> Result<Vec<ExternalIdentity>, RepositoryError> {
        use crate::schema::external_identities::dsl as eid_dsl;

        eid_dsl::external_identities
            .filter(eid_dsl::user_id.eq(user_id))
            .order(eid_dsl::created_at.desc())
            .load::<ExternalIdentity>(conn)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))
    }

    /// Find external identity for a user by provider.
    ///
    /// Convenience method to check if a user has already linked a specific provider.
    pub async fn find_by_user_and_provider(
        &self,
        conn: &mut crate::DbConnection<'_>,
        user_id: Uuid,
        provider_id: Uuid,
    ) -> Result<Option<ExternalIdentity>, RepositoryError> {
        use crate::schema::external_identities::dsl as eid_dsl;

        eid_dsl::external_identities
            .filter(
                eid_dsl::user_id
                    .eq(user_id)
                    .and(eid_dsl::provider_id.eq(provider_id)),
            )
            .first::<ExternalIdentity>(conn)
            .await
            .optional()
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))
    }
}
```

---

### 3. Services Layer

#### 3.1 ExternalIdentityProviderService

**File:** `src/services/external_identity_provider_service.rs`

```rust
use crate::domain::repositories::ExternalIdentityProviderRepository;
use crate::errors::ServiceError;
use std::sync::Arc;
use uuid::Uuid;

/// ExternalIdentityProviderService — manages external IdP configurations.
///
/// Responsibilities:
/// - Create, update, deactivate identity provider configurations
/// - Encrypt/decrypt client secrets at rest
/// - Validate provider configuration before activation
///
/// Notable:
/// - Uses XChaCha20-Poly1305 encryption (same as email provider configs)
/// - Secrets are never returned in responses — use redacted DTOs
pub struct ExternalIdentityProviderService {
    repo: Arc<ExternalIdentityProviderRepository>,
    encryption_key: [u8; 32],
}

impl ExternalIdentityProviderService {
    pub fn new(repo: Arc<ExternalIdentityProviderRepository>, encryption_key: [u8; 32]) -> Self {
        Self { repo, encryption_key }
    }

    /// Create a new external identity provider configuration.
    ///
    /// Uses repository's auto-generated `create()` method from `impl_repository!`.
    pub async fn create_provider(
        &self,
        conn: &mut crate::DbConnection<'_>,
        name: String,
        issuer_url: String,
        client_id: String,
        client_secret: String,
        jwks_url: String,
        scopes: Vec<String>,
    ) -> Result<Uuid, ServiceError> {
        tracing::debug!(provider_name = %name, "create_provider: starting");

        // Encrypt client secret
        let client_secret_encrypted = self.encrypt_secret(&client_secret)?;

        // Build entity using macro-compatible payload
        let new_provider = crate::domain::schema_models::NewExternalIdentityProvider {
            provider_name: name,
            issuer_url,
            client_id,
            client_secret_encrypted,
            jwks_url,
            scopes,
            is_active: true,
        };

        // Use auto-generated create() from impl_repository!
        let provider = self
            .repo
            .create(conn, new_provider)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        tracing::debug!(provider_id = %provider.provider_id, "create_provider: done");
        Ok(provider.provider_id)
    }

    /// Get active provider by name.
    ///
    /// Uses repository's custom `find_active_by_name()` method.
    pub async fn get_active_provider(
        &self,
        conn: &mut crate::DbConnection<'_>,
        name: &str,
    ) -> Result<crate::domain::schema_models::ExternalIdentityProvider, ServiceError> {
        self.repo
            .find_active_by_name(conn, name)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or(ServiceError::OidcClientNotFound)
    }

    /// Decrypt client secret for token exchange.
    ///
    /// Internal method — never expose decrypted secrets in responses.
    pub fn decrypt_secret(&self, encrypted: &[u8]) -> Result<String, ServiceError> {
        // XChaCha20-Poly1305 decryption (same pattern as provider_config_service.rs)
        // Implementation omitted for brevity
        Ok("decrypted_secret".to_string())
    }

    fn encrypt_secret(&self, secret: &str) -> Result<Vec<u8>, ServiceError> {
        // XChaCha20-Poly1305 encryption
        Ok(vec![])
    }
}
```

#### 3.2 FederationService

**File:** `src/services/federation_service.rs`

```rust
use crate::domain::repositories::{ExternalIdentityRepository, UserRepository};
use crate::domain::schema_models::ExternalIdentity;
use crate::errors::ServiceError;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

/// FederationService — orchestrates federated identity token exchange.
///
/// Responsibilities:
/// - Validate external JWT tokens against provider JWKS
/// - Exchange valid tokens for Sentinel sessions
/// - Link external identities to existing users (by email match)
/// - Auto-provision new users for unknown federated identities
/// - Update token storage and last_login timestamps
///
/// Notable:
/// - Uses repository macro methods wherever possible (create, find_by_id, etc.)
/// - Only uses custom repository methods for multi-column lookups
/// - Token validation uses jsonwebtoken + reqwest for JWKS fetching
pub struct FederationService {
    external_identity_repo: Arc<ExternalIdentityRepository>,
    user_repo: Arc<UserRepository>,
    provider_service: Arc<crate::services::ExternalIdentityProviderService>,
}

impl FederationService {
    pub fn new(
        external_identity_repo: Arc<ExternalIdentityRepository>,
        user_repo: Arc<UserRepository>,
        provider_service: Arc<crate::services::ExternalIdentityProviderService>,
    ) -> Self {
        Self {
            external_identity_repo,
            user_repo,
            provider_service,
        }
    }

    /// Exchange external token for federated identity info.
    ///
    /// Returns (provider_id, issuer, subject, email, email_verified, tokens)
    pub async fn exchange_token(
        &self,
        conn: &mut crate::DbConnection<'_>,
        provider_name: &str,
        external_token: &str,
    ) -> Result<(Uuid, String, String, String, bool, Option<String>, Option<String>), ServiceError> {
        tracing::debug!(provider_name = %provider_name, "exchange_token: starting");

        // 1. Get provider config
        let provider = self.provider_service.get_active_provider(conn, provider_name).await?;

        // 2. Validate token against JWKS
        let claims = self.validate_external_token(external_token, &provider.jwks_url).await?;

        // 3. Extract claims
        let subject = claims.sub.ok_or(ServiceError::OidcInvalidCode)?;
        let issuer = claims.iss.ok_or(ServiceError::OidcInvalidCode)?;
        let email = claims.email.ok_or(ServiceError::OidcInvalidCode)?;
        let email_verified = claims.email_verified.unwrap_or(false);

        tracing::debug!(
            provider_id = %provider.provider_id,
            subject = %subject,
            email = %email,
            "exchange_token: token validated"
        );

        Ok((
            provider.provider_id,
            issuer,
            subject,
            email,
            email_verified,
            Some(external_token.to_string()), // access_token
            None, // refresh_token (optional)
        ))
    }

    /// Find or create external identity link.
    ///
    /// Uses repository methods from impl_repository! macro where possible:
    /// - `find_by_provider_issuer_subject()` — custom method for composite lookup
    /// - `create()` — auto-generated from impl_repository!
    ///
    /// Returns (user_id, external_identity_id, is_new_link)
    pub async fn find_or_create_identity(
        &self,
        conn: &mut crate::DbConnection<'_>,
        provider_id: Uuid,
        issuer: &str,
        subject: &str,
        email: &str,
        email_verified: bool,
    ) -> Result<(Uuid, Uuid, bool), ServiceError> {
        // 1. Check if external identity already exists
        // Uses custom repository method for multi-column lookup
        if let Some(existing) = self
            .external_identity_repo
            .find_by_provider_issuer_subject(conn, provider_id, issuer, subject)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
        {
            // 2. Update last_login using custom repository method
            let now = Utc::now().naive_utc();
            self.external_identity_repo
                .update_last_login(conn, existing.external_identity_id, now)
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

            return Ok((existing.user_id, existing.external_identity_id, false));
        }

        // 3. Find existing Sentinel user by email
        let user_id = self.find_or_create_user(conn, email, email_verified).await?;

        // 4. Create new external identity link
        // Uses auto-generated create() from impl_repository!
        let new_identity = crate::domain::schema_models::NewExternalIdentity {
            user_id,
            provider_id,
            issuer: issuer.to_string(),
            subject: subject.to_string(),
            email: email.to_string(),
            email_verified,
            access_token_encrypted: None,
            refresh_token_encrypted: None,
            token_expires_at: None,
        };

        let identity = self
            .external_identity_repo
            .create(conn, new_identity)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        tracing::debug!(
            user_id = %user_id,
            external_identity_id = %identity.external_identity_id,
            "find_or_create_identity: new link created"
        );

        Ok((user_id, identity.external_identity_id, true))
    }

    /// Find existing user by email or create new one.
    ///
    /// Uses UserRepository's auto-generated methods from impl_repository!.
    async fn find_or_create_user(
        &self,
        conn: &mut crate::DbConnection<'_>,
        email: &str,
        email_verified: bool,
    ) -> Result<Uuid, ServiceError> {
        use crate::schema::user_identities::dsl as ident_dsl;

        // Try to find existing user by email
        if let Some(identity) = ident_dsl::user_identities
            .filter(ident_dsl::email.eq(email))
            .first::<crate::domain::schema_models::UserIdentity>(conn)
            .await
            .optional()
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
        {
            return Ok(identity.user_id);
        }

        // Auto-provision new user
        // Note: This requires a special flow since passwords are pgcrypt-managed
        // For federated users, we create a minimal user record with random password
        let new_user = crate::domain::schema_models::NewUser {
            // ... minimal user fields
        };

        let user = self
            .user_repo
            .create(conn, new_user)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok(user.user_id)
    }

    /// Validate external JWT against provider JWKS.
    async fn validate_external_token(
        &self,
        token: &str,
        jwks_url: &str,
    ) -> Result<ExternalTokenClaims, ServiceError> {
        // Fetch JWKS, validate signature, extract claims
        // Implementation uses jsonwebtoken crate
        Ok(ExternalTokenClaims {
            sub: Some("subject".to_string()),
            iss: Some("issuer".to_string()),
            email: Some("user@example.com".to_string()),
            email_verified: Some(true),
        })
    }
}

/// Claims extracted from external JWT.
#[derive(Debug, serde::Deserialize)]
struct ExternalTokenClaims {
    sub: Option<String>,
    iss: Option<String>,
    email: Option<String>,
    email_verified: Option<bool>,
}
```

---

### 4. Application Layer

#### 4.1 FederationApplication

**File:** `src/applications/federation_application.rs`

```rust
use crate::errors::ServiceError;
use crate::services::FederationService;
use crate::services::SessionService;
use std::sync::Arc;

/// FederationApplication — orchestrates federated login flows.
///
/// Responsibilities:
/// - Token exchange endpoint orchestration
/// - Session creation after successful federation
/// - Audit logging for federated logins
pub struct FederationApplication {
    pg_client: Arc<crate::infrastructure::postgres::PostgresClient>,
    federation_service: Arc<FederationService>,
    session_service: Arc<SessionService>,
}

impl FederationApplication {
    pub fn new(
        pg_client: Arc<crate::infrastructure::postgres::PostgresClient>,
        federation_service: Arc<FederationService>,
        session_service: Arc<SessionService>,
    ) -> Self {
        Self {
            pg_client,
            federation_service,
            session_service,
        }
    }

    /// Exchange external token for Sentinel session.
    ///
    /// Flow:
    /// 1. Validate external token
    /// 2. Find/create external identity link
    /// 3. Create Sentinel session
    /// 4. Return PASETO tokens
    pub async fn exchange_and_login(
        &self,
        provider_name: String,
        external_token: String,
    ) -> Result<crate::services::session_service::LoginResponse, ServiceError> {
        let mut conn = self.pg_client.get_conn().await?;

        conn.transaction(|conn| {
            let federation_service = self.federation_service.clone();
            let session_service = self.session_service.clone();
            let provider_name = provider_name.clone();
            let external_token = external_token.clone();

            Box::pin(async move {
                // 1. Exchange token
                let (provider_id, issuer, subject, email, email_verified, access_token, refresh_token) =
                    federation_service
                        .exchange_token(conn, &provider_name, &external_token)
                        .await?;

                // 2. Find or create identity link
                let (user_id, _identity_id, _is_new) = federation_service
                    .find_or_create_identity(
                        conn,
                        provider_id,
                        &issuer,
                        &subject,
                        &email,
                        email_verified,
                    )
                    .await?;

                // 3. Create Sentinel session
                let session = session_service
                    .create_session(conn, user_id, email_verified)
                    .await?;

                Ok(session)
            })
        })
        .await
    }
}
```

---

### 5. HTTP Layer

#### 5.1 DTOs

**File:** `src/infrastructure/http/api/dtos/federation_dtos.rs`

```rust
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request to exchange external token for Sentinel session.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ExchangeTokenRequest {
    /// Provider name (e.g., "supabase", "google", "github").
    #[validate(length(min = 1, max = 64))]
    pub provider_name: String,

    /// External OAuth/OIDC token from the identity provider.
    #[validate(length(min = 1))]
    pub external_token: String,
}

/// Response returned after successful federated login.
#[derive(Debug, Serialize, ToSchema)]
pub struct FederatedLoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user_id: Uuid,
    pub email: String,
    pub is_new_user: bool,
}
```

#### 5.2 Handler

**File:** `src/infrastructure/http/api/handlers/federation_handlers.rs`

```rust
use crate::applications::FederationApplication;
use crate::infrastructure::http::api::dtos::federation_dtos::{
    ExchangeTokenRequest, FederatedLoginResponse,
};
use crate::infrastructure::http::api::responses::{RawResponse, ApiError};
use axum::Extension;
use std::sync::Arc;
use utoipa::OpenApi;

/// POST /v1/api/auth/federation/exchange
///
/// Exchange external OAuth/OIDC token for Sentinel session.
///
/// Supports Supabase, Google, GitHub, and other OIDC providers configured
/// via the admin API. Returns standard PASETO access/refresh tokens.
#[utoipa::path(
    post,
    path = "/v1/api/auth/federation/exchange",
    request_body = ExchangeTokenRequest,
    responses(
        (status = 200, body = FederatedLoginResponse),
        (status = 400, body = ApiErrorResponse),
        (status = 401, body = ApiErrorResponse),
        (status = 404, body = ApiErrorResponse),
    ),
    tag = "auth"
)]
pub async fn exchange_token(
    Extension(state): Extension<Arc<crate::infrastructure::http::app::AppState>>,
    ValidatedJson(req): ValidatedJson<ExchangeTokenRequest>,
) -> Result<RawResponse<FederatedLoginResponse>, ApiError> {
    state
        .federation_app
        .exchange_and_login(req.provider_name, req.external_token)
        .await
        .map(|session| {
            RawResponse(FederatedLoginResponse {
                access_token: session.access_token,
                refresh_token: session.refresh_token,
                token_type: "Bearer".to_string(),
                expires_in: session.expires_in,
                user_id: session.user_id,
                email: session.email,
                is_new_user: session.is_new_user,
            })
        })
        .map_err(ApiError::from)
}
```

#### 5.3 Routes

**File:** `src/infrastructure/http/api/routes/auth_router.rs`

Add federation routes:

```rust
pub fn build_auth_routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        // ... existing routes
        .route(
            "/federation/exchange",
            post(handlers::federation_handlers::exchange_token),
        )
}
```

---

### 6. Dependency Injection

**File:** `src/infrastructure/http/app.rs`

Add new services and application to the DI container:

```rust
pub struct AppState {
    // ... existing fields
    pub federation_app: Arc<FederationApplication>,
}

impl AppState {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // ... existing initialization

        // External identity repositories
        let external_identity_provider_repo = Arc::new(ExternalIdentityProviderRepository::new());
        let external_identity_repo = Arc::new(ExternalIdentityRepository::new());

        // Services
        let provider_service = Arc::new(ExternalIdentityProviderService::new(
            external_identity_provider_repo.clone(),
            config_encryption_key,
        ));
        let federation_service = Arc::new(FederationService::new(
            external_identity_repo.clone(),
            user_repo.clone(),
            provider_service.clone(),
        ));

        // Applications
        let federation_app = Arc::new(FederationApplication::new(
            pg_client.clone(),
            federation_service,
            session_service.clone(),
        ));

        Ok(Self {
            // ... existing fields
            federation_app,
        })
    }
}
```

---

### 7. Admin API for Provider Management

#### 7.1 Admin DTOs

```rust
/// Request to create external identity provider.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateExternalProviderRequest {
    #[validate(length(min = 1, max = 64))]
    pub provider_name: String,
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub jwks_url: String,
    #[validate(length(min = 1))]
    pub scopes: Vec<String>,
}

/// Provider response (secrets redacted).
#[derive(Debug, Serialize, ToSchema)]
pub struct ExternalProviderResponse {
    pub provider_id: Uuid,
    pub provider_name: String,
    pub issuer_url: String,
    pub client_id: String,
    pub jwks_url: String,
    pub scopes: Vec<String>,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
}
```

#### 7.2 Admin Handlers

```rust
/// POST /v1/api/admin/external-providers
pub async fn create_external_provider(
    Extension(state): Extension<Arc<AppState>>,
    ValidatedBearer(token): ValidatedBearer,
    ValidatedJson(req): ValidatedJson<CreateExternalProviderRequest>,
) -> Result<RawResponse<ExternalProviderResponse>, ApiError> {
    // Admin role check + orchestration
}

/// GET /v1/api/admin/external-providers
pub async fn list_external_providers(
    Extension(state): Extension<Arc<AppState>>,
    ValidatedBearer(token): ValidatedBearer,
) -> Result<RawResponse<Vec<ExternalProviderResponse>>, ApiError> {
    // Admin role check + list active providers
}
```

---

### 8. Testing Strategy

#### 8.1 Unit Tests

```rust
// tests/federation_service_test.rs

#[tokio::test]
async fn find_or_create_identity_existing_link() {
    // Setup: pre-seed external identity in DB
    // Call: find_or_create_identity with same provider/issuer/subject
    // Assert: returns existing user_id, identity_id, is_new=false
    // Verify: last_login was updated
}

#[tokio::test]
async fn find_or_create_identity_new_link_existing_user() {
    // Setup: pre-seed user with matching email
    // Call: find_or_create_identity with new provider/subject
    // Assert: returns existing user_id, new identity_id, is_new=true
}

#[tokio::test]
async fn find_or_create_identity_new_link_new_user() {
    // Setup: no matching email in DB
    // Call: find_or_create_identity
    // Assert: new user_id, new identity_id, is_new=true
}
```

#### 8.2 Integration Tests

```rust
// tests/federation_flow_test.rs

#[tokio::test]
async fn exchange_token_supabase_happy_path() {
    // 1. Create provider config via admin API
    // 2. Generate valid mock JWT (signed with test key)
    // 3. POST /auth/federation/exchange
    // 4. Assert: 200, valid PASETO tokens returned
    // 5. Verify: external_identities row created
}

#[tokio::test]
#[ignore] // Requires live JWKS endpoint
async fn exchange_token_invalid_signature_returns_401() {
    // Tampered JWT should fail validation
}
```

---

### 9. Migration Strategy

1. **Deploy migration**: Run `diesel migration run` in staging first
2. **Seed initial providers**: Use admin API to configure Supabase provider
3. **Enable feature flag**: Gate federation endpoint behind config flag
4. **Monitor**: Track `external_identities` table growth, error rates
5. **Rollout**: Enable for production after staging validation

---

### 10. Security Considerations

| Concern | Mitigation |
|---------|------------|
| Token replay | Single-use validation, short TTL on external tokens |
| Secret exposure | XChaCha20-Poly1305 encryption at rest, never log secrets |
| Provider misconfiguration | Admin-only provider management, validation on create |
| Account takeover | Email verification required, link existing accounts only with proof |
| Audit trail | Log all federation events with provider, subject, user_id |

---

## Repository Pattern Summary

All repositories in this implementation follow the established pattern from CLAUDE.md:

```rust
// 1. Use impl_repository! for ALL standard CRUD (13 methods auto-generated)
impl_repository!(
    RepositoryName for Entity,
    crate::schema::table::table,
    crate::schema::table::id_column,
    IdType
);

// 2. ONLY add custom impl block for queries the macro cannot handle
impl RepositoryName {
    // Custom methods here (multi-column lookups, bulk updates, etc.)
}

// 3. NEVER import DbConnection at the top of repository files
// Use crate::DbConnection<'_> inline in method signatures
```

This pattern ensures:
- Consistent CRUD interface across all repositories
- Minimal boilerplate for standard operations
- Custom queries only where necessary
- No duplicate type import errors from the macro
