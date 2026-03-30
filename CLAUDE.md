# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

**Run dev environment (Docker — auto-recompiles on file changes):**
```bash
docker compose -f docker-compose.dev.yml up -d
```

**Run tests:**
```bash
docker compose -f docker-compose.dev.yml run sentinel-core cargo test
```

**Run a specific test:**
```bash
docker compose -f docker-compose.dev.yml run sentinel-core cargo test <test_name>
```

**Run tests for a specific crate:**
```bash
docker compose -f docker-compose.dev.yml run sentinel-core cargo test -p sentinel-policy-engine
```

**Run policy engine benchmarks (HTML report in `target/criterion/`):**
```bash
docker compose -f docker-compose.dev.yml run sentinel-core cargo bench -p sentinel-policy-engine
```

**Run database migrations:**
```bash
diesel migration run
```

**Create database migrations:**
```bash
diesel migration generate <name of migration>
```
Ensure tables have a primary key normally in the form of `table_name_id`.

**Generate types with script:**
```bash
apps/sentinel-core/scripts/gen_diesel_types.sh
```

**Run the management UI (standalone, backend must be running):**
```bash
cd apps/sentinel-ui && npm install --legacy-peer-deps && npm run dev
```

Set `VITE_API_URL` to override the backend base URL (default: `http://localhost:8080`).

**Build the TypeScript SDK (required when SDK source changes):**
```bash
cd packages/sentinel-auth-sdk && npm install && npm run build
```

**Build the React auth UI package (required when auth-react source changes):**
```bash
cd packages/sentinel-auth-react && npm install --legacy-peer-deps && npm run build
```

**Type-check the UI:**
```bash
cd apps/sentinel-ui && npx tsc --noEmit
```

**Environment setup:**
Copy `apps/sentinel-core/.env.example` to `apps/sentinel-core/.env` and fill in values.

Required env vars: `DATABASE_URL`, `HEX_KEY` (32-byte hex, session encryption), `CONFIG_ENCRYPTION_KEY` (32-byte hex), `APP_HOST`, `APP_PORT`, `OIDC_ISSUER_URL`, `FRONTEND_URL` (base URL for email verification links, e.g. `http://localhost:3000`), `CORS_ALLOWED_ORIGINS` (comma-separated allowed origins, e.g. `http://localhost:3000`; if unset, CORS is denied for all cross-origin requests).

## Implemented Endpoints

Real handlers (everything else uses the `health_check` stub):

### Auth & User
| Method | Path | Handler |
|--------|------|---------|
| POST | `/v1/api/auth/register` | `register_user` |
| POST | `/v1/api/auth/login` | `basic_auth_login` |
| POST | `/v1/api/auth/authenticate` | `authenticate` |
| POST | `/v1/api/auth/logout` | `logout` |
| POST | `/v1/api/auth/logout-all` | `logout_all` |
| POST | `/v1/api/auth/token/authorize` | `check_authorization` |
| POST | `/v1/api/auth/token/refresh` | `token_refresh` |
| GET  | `/v1/api/auth/verify-email?token=<raw>` | `verify_email` |
| POST | `/v1/api/auth/resend-verification` | `resend_verification` |
| GET  | `/v1/api/auth/auth-methods` | `get_auth_methods` |
| POST | `/v1/api/auth/password/forgot` | `forgot_password` |
| POST | `/v1/api/auth/password/reset` | `reset_password` |
| GET  | `/v1/api/user/me` | `get_me` |
| POST | `/v1/api/user/password/change` | `change_password` (Bearer required) |
| GET  | `/v1/api/user/sessions` | `get_user_sessions` |
| GET  | `/v1/api/user/sessions/{session_id}` | `get_user_session` |
| GET  | `/v1/api/user/permissions` | `get_user_permissions` |
| GET  | `/v1/api/user/canary` | `protected_canary` (auth + authz demo) |

### API Tokens (admin role + Bearer required)
| Method | Path | Handler |
|--------|------|---------|
| POST   | `/v1/api/auth/api-tokens` | `create_api_token` |
| GET    | `/v1/api/auth/api-tokens` | `list_api_tokens` |
| DELETE | `/v1/api/auth/api-tokens/{token_id}` | `revoke_api_token` |
| DELETE | `/v1/api/auth/api-tokens` | `revoke_all_tokens` |

Raw token returned once at creation; only SHA-256 hash persisted. Revocation is a soft-delete (`revoked_at`). Authorization is checked in the application layer (`ctx.roles` must contain `"admin"`).

### MFA (TOTP)
| Method | Path | Handler | Auth |
|--------|------|---------|------|
| POST | `/v1/api/auth/mfa/totp/start` | `mfa_totp_start` | Bearer token required |
| POST | `/v1/api/auth/mfa/totp/confirm` | `mfa_totp_confirm` | Bearer token required |
| POST | `/v1/api/auth/mfa/verify` | `mfa_verify` | MFA challenge token in body |

### Admin
| Method | Path | Handler |
|--------|------|---------|
| POST   | `/v1/api/admin/roles` | `create_role` |
| GET    | `/v1/api/admin/roles` | `list_roles` |
| PUT    | `/v1/api/admin/roles/{role_id}` | `update_role` |
| DELETE | `/v1/api/admin/roles/{role_id}` | `delete_role` |
| POST   | `/v1/api/admin/users/{user_id}/roles` | `assign_role_to_user` |
| DELETE | `/v1/api/admin/users/{user_id}/roles/{role_name}` | `remove_role_from_user` |
| GET    | `/v1/api/admin/users/{user_id}/permissions` | `get_user_permissions_admin` |
| GET    | `/v1/api/admin/users/{user_id}/auth-info` | `get_user_auth_info` |
| POST | `/v1/api/admin/policies` | `create_policy` |
| PUT  | `/v1/api/admin/policies/{policy_id}/rules` | `update_policy_rules` |
| POST | `/v1/api/admin/oidc/clients` | `create_oidc_client` |
| POST | `/v1/api/admin/oidc/keys/generate` | `generate_signing_key` |
| GET  | `/v1/api/admin/email-templates` | `list_email_templates` |
| POST | `/v1/api/admin/email-templates` | `create_email_template` |
| PUT  | `/v1/api/admin/email-templates/{template_id}` | `update_email_template` |

### System
| Method | Path | Handler |
|--------|------|---------|
| GET    | `/v1/api/system/health` | `health_check` |
| POST   | `/v1/api/system/config/email` | `add_provider_config` |
| GET    | `/v1/api/system/config/email` | `list_provider_configs` |
| PUT    | `/v1/api/system/config/email/{config_id}` | `update_provider_config` |
| DELETE | `/v1/api/system/config/email/{config_id}` | `delete_provider_config` |
| GET    | `/v1/api/system/config/email/{config_id}/reveal` | `get_provider_config_decrypted` |
| POST   | `/v1/api/system/config/email/{config_id}/test` | `test_provider_config` |
| POST   | `/v1/api/system/config/email/{config_id}/send-test` | `send_test_provider_email` |

### OIDC / OAuth 2.0 (spec-compliant, no Sentinel envelope)
| Method | Path | Handler |
|--------|------|---------|
| GET  | `/.well-known/openid-configuration` | `openid_configuration` |
| GET  | `/oauth/jwks.json` | `jwks` |
| GET  | `/oauth/authorize` | `authorize` (requires Sentinel Bearer token) |
| POST | `/oauth/token` | `token_exchange` (form-encoded) |

## Architecture

This is a Cargo workspace (two Rust crates) plus two TypeScript packages:
- `apps/sentinel-core` — main Axum HTTP service
- `packages/sentinel-policy-engine` — standalone compiled RBAC rules engine
- `apps/sentinel-ui` — React 19 admin dashboard (Vite + TypeScript)
- `packages/sentinel-auth-sdk` — TypeScript SDK (`@sentinel/auth-sdk`) consumed by `sentinel-ui`

### Clean Architecture Layers (`sentinel-core`)

Dependency rule: inner layers cannot depend on outer ones.

```
Infrastructure (HTTP handlers, DB clients, external clients)
    ↑
Application (use-case orchestrators: AuthApplication, MfaApplication, OidcApplication, PolicyApplication, SystemApplication, UserApplication, ApiTokenApplication, UserPasswordApplication, EmailTemplateApplication)
    ↑
Services (single-responsibility business logic: identity, user, session, policy, role, oidc_client, oidc_auth_code, oidc_key, oidc_token, mfa_totp, api_token, email_verification, email, email_template, password_reset…)
    ↑
Domain (entities from schema_models.rs, repository trait impls in domain/repositories/)
```

The **application layer** is where multi-service flows and transaction boundaries live. Handlers call application structs, not services directly.

The **infrastructure layer** wires everything together: `infrastructure/http/app.rs` is the DI container that constructs all services and applications, then mounts them into the Axum router.

### MFA (TOTP)

Sentinel supports TOTP-based multi-factor authentication as an optional second factor for users.

**Enrollment flow:** `mfa_totp_start` → user scans QR code → `mfa_totp_confirm` (verifies first code, enables MFA).

**Login flow with MFA:** `basic_auth_login` returns a `LoginOutcome` enum (`#[serde(untagged)]`):
- Non-MFA users: `BasicLoginResponse` (access + refresh tokens, unchanged JSON shape)
- MFA-enabled users: `MfaChallengeResponse` (contains a short-lived `mfa_session_token`)

The `mfa_session_token` is a PASETO token (sub = `"Sentinel:MfaChallenge"`, 5-min TTL, no DB row). The client submits it alongside a TOTP code to `mfa_verify`, which returns full session tokens on success. Recovery codes (8 SHA-256-hashed one-time codes) are also accepted at `mfa_verify`.

Key files:
- `src/applications/mfa_application.rs` — enrollment and MFA-login orchestration
- `src/services/mfa_totp_service.rs` — XChaCha20-Poly1305 secret encryption, totp-rs v5 code verification
- `src/infrastructure/http/api/handlers/mfa_handlers.rs` — three MFA handlers
- DB tables: `user_mfa_totp` (encrypted secret, enabled flag), `user_recovery_codes` (hashed codes, `used_at`)

OIDC `amr` claim reflects MFA: `["pwd"]` or `["pwd", "totp"]`.

### API Tokens

Long-lived opaque tokens for programmatic access (CI/CD, scripts). Distinct from short-lived PASETO sessions.

**Token format:** `sat_<64 hex chars>` (32 bytes from `OsRng`, hex-encoded). Only the SHA-256 hash is stored in `api_tokens.token_hash`. The raw token is returned exactly once at creation.

**Revocation:** soft-delete via `revoked_at = now()`. No hard deletes. Bulk revocation uses a custom `revoke_all_for_user` method (not `update_where`, which is single-row only).

**Authorization:** the application layer checks `ctx.roles.iter().any(|r| r == "admin")` before any operation. No separate authz middleware is added; the check lives in `ApiTokenApplication`.

Key files:
- `src/applications/api_token_application.rs` — create/list/revoke/revoke-all orchestration
- `src/services/api_token_service.rs` — token generation + CRUD; `generate_token()` returns `(raw, hash)`
- `src/domain/repositories/api_token_repository.rs` — `impl_repository!` + `find_by_token_hash` + `revoke_all_for_user`
- `src/infrastructure/http/api/handlers/api_token_handlers.rs` — four handlers
- `src/infrastructure/http/api/dtos/api_token_dtos.rs` — `CreateApiTokenRequest`, `CreateApiTokenResponse` (has `token`), `ApiTokenResponse` (no `token`)
- DB table: `api_tokens` (migration `2026-03-01-000003_add_api_tokens`)

### Provider Configuration (Email SMTP)

Admins configure SMTP email providers via `POST /v1/api/system/config/email`. Secrets (password, API key) are encrypted at rest with XChaCha20-Poly1305 (`CONFIG_ENCRYPTION_KEY`). The stored `config_redacted` field masks all values with `"****"` — it is safe to return in list/get responses.

**Auth modes in the config JSON:**
- **Credentials**: `{ host, port, username, password, from_email, use_tls }`
- **API Key**: `{ host, port, username, api_key, from_email, use_tls }` — `email_service.rs` accepts `api_key` as a fallback for `password` so Resend (username=`"resend"`, password=API key) and similar services work without special casing

**Test endpoint**: `POST /config/email/{config_id}/test` — decrypts config, builds an SMTP transport, calls lettre's `test_connection()`, returns `TestProviderConfigResponse { success: bool, message: String }`. Errors are surfaced as `success: false` with the lettre error message rather than HTTP 500.

Key files:
- `src/services/email_service.rs` — `test_connection(&Value)` + `api_key` fallback in `send_with_template`
- `src/services/provider_config_service.rs` — `encrypt_config`, `decrypt_config`, `redact_config`
- `src/applications/system_application.rs` — `test_config`, `send_test_email_config`, `add_provider_config`, `list_configs`, `update_config`, `delete_config`, `get_decrypted_config`
- `src/infrastructure/http/api/handlers/system_handlers.rs` — all provider config handlers
- `src/infrastructure/http/api/dtos/system_dtos.rs` — `CreateProviderConfigRequest`, `UpdateProviderConfigRequest`, `ProviderConfigResponse`, `DecryptedProviderConfigResponse`, `TestProviderConfigResponse`, `SendTestEmailRequest`

`test_provider_config` (`POST .../test`) verifies the SMTP connection only. `send_test_provider_email` (`POST .../send-test`) decrypts the config and sends an actual email to the address in `SendTestEmailRequest { to_email }`. Both return `TestProviderConfigResponse { success: bool, message: String }`.

### Email Verification

New users are registered with `email_verified = false` and `status = PendingVerification`. On successful registration, a verification token is generated and an email sent (if SMTP is configured).

**Token format:** `ev_<64 hex chars>` (32 bytes OsRng). Only the SHA-256 hash is stored in `email_verifications.token_hash`. Tokens expire after 24 hours.

**Verification flow:** `POST /register` → generate `ev_*` token → store hash in DB → email link to `${FRONTEND_URL}/verify-email?token=<raw>` → user clicks → `GET /verify-email?token=<raw>` → `consume_token` validates hash, sets `verified_at` → `mark_email_verified` sets `user_identities.email_verified = true`.

**PASETO token claim:** `"ev"` (bool). Baked into the token at login time from `user_identities.email_verified`. Tokens issued before verification carry `ev: false` permanently — users must re-login after verifying.

**Middleware gate:** `authorize_middleware` checks `ctx.email_verified` after the `bypass_authorization` short-circuit and before policy evaluation. Unverified users get `403 EMAIL_NOT_VERIFIED`. API token paths (`bypass_authorization = true`) skip the check entirely.

**No SMTP configured:** `EmailService::send_verification_email` silently returns `Ok(())` with a `tracing::warn!` — registration always succeeds regardless.

Key files:
- `src/services/email_verification_service.rs` — `create_verification` + `consume_token`
- `src/services/email_service.rs` — reads active SMTP provider config, decrypts it, sends via lettre STARTTLS
- `src/services/provider_config_service.rs` — `decrypt_config` (inverse of `encrypt_config`, XChaCha20-Poly1305)
- `src/domain/repositories/email_verification_repository.rs` — `impl_repository!` for `email_verifications` table
- `src/services/identity_service.rs` — `mark_email_verified` + `find_identity_by_email`
- DB table: `email_verifications` (columns: `verification_id`, `identity_id`, `token_hash`, `expires_at`, `verified_at`)

### Email Templates

Admin-configurable email templates with typed `{{placeholder}}` substitution. When no active template exists for a type, a built-in default is used so the system works out of the box.

**Template types:** `EmailVerification`, `PasswordReset`, `PasswordChanged` (Diesel enum `EmailTemplateType`).

**Rendering:** iterate context `HashMap<&str, &str>`, replace `{{key}}` with value in `subject`, `body_text`, and optionally `body_html`.

**Unique constraint:** `CREATE UNIQUE INDEX ... ON email_templates(template_type) WHERE is_active = TRUE` — at most one active template per type. `EmailTemplateService::create_template` deactivates the previous one before inserting the new one.

Key files:
- `src/services/email_template_service.rs` — `render`, `create_template`, `update_template`, `list_templates`
- `src/domain/repositories/email_template_repository.rs` — `impl_repository!` + custom `list_all`
- `src/applications/email_template_application.rs` — admin role guard + orchestration
- `src/infrastructure/http/api/handlers/email_template_handlers.rs` — three handlers
- DB table: `email_templates` (migration `2026-03-02-000001_add_email_templates`)

### Password Reset / Change

Two separate password flows with full session revocation and notification emails.

**Forgot/Reset flow (public):** `POST /auth/password/forgot` → silently returns 200 if email unknown (anti-enumeration) → generates `pr_*` token → emails reset link → `POST /auth/password/reset` validates token, updates password via DB trigger, revokes all sessions, sends notification email.

**Change flow (authenticated):** `POST /user/password/change` → verifies current password → updates password → revokes all sessions → sends notification email.

**Token format:** `pr_<64 hex chars>` (32 bytes from `OsRng`). Only the SHA-256 hash is stored in `password_reset_tokens.token_hash`. Tokens expire after **1 hour** and are soft-marked via `used_at`.

**Session revocation:** both flows call `SessionService::revoke_all_sessions` so any compromised tokens are immediately invalidated.

Key files:
- `src/services/password_reset_service.rs` — `create_reset_token` + `consume_token`
- `src/domain/repositories/password_reset_token_repository.rs` — `impl_repository!` macro only
- `src/applications/auth_application.rs` — `forgot_password` + `reset_password` methods
- `src/applications/user_password_application.rs` — `change_password` (authenticated)
- `src/infrastructure/http/api/handlers/password_handlers.rs` — three handlers
- DB table: `password_reset_tokens` (migration `2026-03-02-000002_add_password_reset_tokens`)

### OIDC Provider

Sentinel acts as an OIDC IdP (Identity Provider). External apps can use "Log in with Sentinel" via the Authorization Code + PKCE flow. Sentinel issues:
- **PASETO tokens** — internal Sentinel sessions (unchanged)
- **JWT ID tokens + JWT access tokens** — issued to OIDC clients (RS256, signed with stored RSA keys)

Key files:
- `src/applications/oidc_application.rs` — orchestrates the full OIDC flow
- `src/services/oidc_client_service.rs` — client validation (redirect URI, scopes, secret)
- `src/services/oidc_auth_code_service.rs` — auth code creation and PKCE consumption
- `src/services/oidc_key_service.rs` — RSA key generation, encryption (XChaCha20-Poly1305), JWKS
- `src/services/oidc_token_service.rs` — RS256 JWT signing via `jsonwebtoken` + `ring`
- `src/infrastructure/http/api/routes/oauth_router.rs` — `/oauth/*` and `/.well-known/*` routes

RSA keys are stored encrypted with `CONFIG_ENCRYPTION_KEY` (XChaCha20-Poly1305, same pattern as provider configs). Private keys are serialized as **PKCS#1 DER** — `ring`'s `RsaKeyPair::from_der()` (called by `jsonwebtoken::EncodingKey::from_rsa_der`) requires PKCS#1, not PKCS#8.

OIDC endpoints bypass `ResponseWrapperLayer` — they return spec-compliant JSON/redirects, not the Sentinel envelope. They are mounted on the outer router before the api router.

### Policy Engine (`sentinel-policy-engine`)

A two-phase RBAC engine separating compile-time from evaluate-time:

1. **Compile** (`compile(bundle)`) — converts JSON rules into a sorted trie serialized as `Vec<u8>` via bincode. Literal path segments are prioritized over params, wildcards, and globs. Store bytes in `policy_versions.compiled_rules`.
2. **Evaluate** (`PolicyEngine::from_bytes(bytes).is_allowed(method, path, roles)`) — O(path depth), ~116–213 ns per check.

`PolicyApplication` holds an `Arc<RwLock<HashMap<Uuid, PolicyEngine>>>` cache, hot-reloading compiled engines when the active version changes.

### Database

Diesel 2.1 async (diesel-async) with bb8 connection pooling. Schema is auto-generated into `schema.rs`. Entities are autogenerated with `apps/sentinel-core/scripts/gen_diesel_types.sh` and live in `schema_models.rs`. Enums are autogenerated with `apps/sentinel-core/scripts/gen_diesel_types.sh` and live in `schema_enums.rs` and `schema_enums_impls.rs`. Migrations live in `apps/sentinel-core/migrations/`.

Password hashing is done by PostgreSQL's `pgcrypt` extension (`crypt()`), not in application code.

Tokens use PASETO v4_local (authenticated encryption). `SessionService` owns all token encrypt/decrypt logic. PASETO tokens remain cryptographically valid after logout — there is no token blacklist yet; revocation is tracked by `sessions.revoked_at IS NOT NULL` (soft-delete, no hard deletes on sessions).

#### Repository Pattern

Repositories use an `impl_repository!` macro that provides 13 CRUD/pagination methods automatically. To add custom DB logic, add a separate `impl RepositoryName { ... }` block after the macro invocation.

**Important — `update_where` limitation:** `update_where` calls `.get_result()` and returns exactly ONE row. It is not suitable for bulk updates. For bulk operations (e.g., revoking all sessions for a user), add a custom method in the separate `impl` block.

**Import gotcha:** Do NOT import `DbConnection` at the top of a repository module file. The macro already imports it internally; a top-level import causes an `E0252` duplicate-type error. Use `crate::DbConnection<'_>` inline in custom method signatures instead.

`SessionRevocationChangeset` is private to `session_service.rs` — reuse it there, do not move it to the domain layer.

### HTTP Layer

Routes are defined in `infrastructure/http/api/routes/api_router.rs` and grouped under `/v1/api/{auth,user,service,admin,system}`. OIDC routes live in `oauth_router.rs` and are mounted at the root level (outside the Sentinel envelope middleware).

Middleware stack (outer → inner): CORS → RequestId → TraceLayer → CatchPanic → ResponseWrapper → AuthenticateMiddleware (selective).

All `/v1/api/*` responses are wrapped in a standard envelope:
```json
{ "success": bool, "data": {…}, "error": null, "timestamp": "…", "request_id": "…" }
```

OIDC endpoints (`/oauth/*`, `/.well-known/*`) are mounted **before** `ResponseWrapperLayer` and return raw spec-compliant JSON or redirects.

**Custom extractors** in `api/routes/api_validation.rs`:
- `ValidatedJson<T>` — deserializes then runs `T::validate()` from the `validator` crate
- `ValidatedBearer` — extracts `Authorization: Bearer <token>`

DTOs live in `infrastructure/http/api/dtos/` and use `#[derive(Validate)]` for field-level validation. All DTOs exposed in Swagger also derive `ToSchema` (or `IntoParams` for query param structs).

### OpenAPI / Swagger

Swagger UI: `GET /swagger-ui` — OpenAPI spec: `GET /api-docs/openapi.json`

Tags: `auth`, `user`, `admin`, `system`, `oidc`

All handler functions exposed in Swagger have a `#[utoipa::path(...)]` annotation. New handlers must be added to both `paths(...)` and `components(schemas(...))` in `src/infrastructure/http/api/openapi.rs`.

### Error Handling

Errors flow through a conversion chain via `From` impls:

`DomainError` → `RepositoryError` → `ServiceError` → `ApiError` (with HTTP status)

All error types are defined in `apps/sentinel-core/src/errors.rs`. `ServiceError` is the boundary used by the application layer; `ApiError` is what handlers return. Internal errors (DB, pool, token build failures) are always mapped to `500 INTERNAL_ERROR` without leaking details.

OIDC-specific `ServiceError` variants: `OidcClientNotFound`, `OidcInvalidRedirectUri`, `OidcInvalidScope`, `OidcInvalidCode`, `OidcCodeExpired`, `OidcCodeConsumed`, `OidcPkceVerificationFailed`, `OidcNoActiveSigningKey`, `OidcSigningError`.

API token `ServiceError` variant: `ApiTokenNotFound` → 404 `API_TOKEN_NOT_FOUND`.

Email verification `ServiceError` variant: `EmailNotVerified(String)` → 403 `EMAIL_NOT_VERIFIED`.

## Integration Test Structure

```
tests/
  common/
    setup.rs           — URL builders (get_*_url() helpers)
    helpers.rs         — post_json, put_json, read_json, generate_expired_token
    assertions.rs      — assert_api_envelope_shape
    auth_assertions.rs — assert_error_envelope, assert_login_success_envelope
  authenticate_api.rs
  login_test.rs
  register_user_api.rs
  logout_test.rs
  user_me_test.rs
  user_sessions_test.rs — session listing and individual session retrieval
  middleware_canary_test.rs
  create_policy_test.rs
  update_policy_test.rs
  create_provider_config.rs
  provider_config_test.rs — full provider config CRUD, test-connection, and send-test-email
  oidc_flow_test.rs     — end-to-end OIDC Authorization Code + PKCE flow
  oidc_pkce_test.rs     — PKCE challenge/verifier edge cases
  mfa_totp_test.rs      — TOTP enrollment (start → confirm) and MFA login + recovery code flows
  token_refresh_test.rs — refresh token exchange flow
  api_token_test.rs     — 401/403 security tests for all API token endpoints; admin happy-path tests are #[ignore] pending admin seeding API
  email_verification_test.rs — invalid token → 401, unknown email → 404, newly registered user has email_verified=false; happy-path is #[ignore] (requires live SMTP)
  password_reset_test.rs — forgot/reset/change validation + security tests; happy-path tests are #[ignore] (require live SMTP for reset link delivery)
  password_validation_test.rs — password policy enforcement (length, uppercase, digit, special char)
  rate_limit_test.rs    — GCRA rate limiting: strict 5/15 min on login + mfa/verify, moderate 10/15 min on register/forgot/resend
  security_headers_test.rs — X-Content-Type-Options, X-Frame-Options, Referrer-Policy present on responses
  health.rs
  api_response_contrac_test.rs
```

Add new test files at the top level of `tests/`; add shared utilities to `tests/common/`. OIDC URL helpers (`get_oauth_authorize_url`, `get_oauth_token_url`, etc.) are in `tests/common/setup.rs`.

## Security

### Rate Limiting

IP-keyed GCRA rate limiting is applied to sensitive auth endpoints via `governor` (v0.6, DashMap-backed):

| Tier | Limit | Endpoints |
|------|-------|-----------|
| Strict | 5 req / s | `POST /v1/api/auth/login`, `POST /v1/api/auth/mfa/verify` |
| Moderate | 10 req / s | `POST /v1/api/auth/register`, `POST /v1/api/auth/password/forgot`, `POST /v1/api/auth/resend-verification` |

Client IP is read from `X-Forwarded-For` → `X-Real-IP` → fallback `127.0.0.1`. Rate-limited responses return HTTP 429 with `Retry-After: 900` and error code `RATE_LIMIT_EXCEEDED`.

Integration tests inject `X-Forwarded-For` with a unique per-call UUID-derived IP (via `unique_ip()` in `tests/common/helpers.rs`) so they never exhaust real rate limit buckets. The `rate_limit_test.rs` tests use `post_with_ip()` with a consistent per-test IP to trigger the limit intentionally.

### MFA Attempt Limiting

In addition to IP-level rate limiting, `mfa_verify` has an in-memory per-`mfa_session_token` attempt counter (`MfaAttemptEntry` in `mfa_application.rs`). After 5 failed attempts within a 15-minute window the endpoint returns HTTP 429 `MFA_ATTEMPT_LIMIT_EXCEEDED`. The token key is hashed with SHA-256 before storage.

### Response Security Headers

Added automatically by `SetResponseHeaderLayer` in `api_router.rs`:
- `X-Content-Type-Options: nosniff`
- `X-Frame-Options: DENY`
- `Referrer-Policy: strict-origin-when-cross-origin`

HSTS is intentionally omitted — it should be set by the TLS termination layer (reverse proxy).

### Password Policy

Minimum requirements enforced by `validate_password()` in `user_dtos.rs`:
- At least 12 characters
- At least one uppercase letter
- At least one lowercase letter
- At least one digit
- At least one special (non-alphanumeric) character

Applied to registration (`RegisterUserRequest`) and password-change/reset flows. Login does **not** enforce the new policy on existing credentials (uses `length(min = 1)` only).

## Development Gotchas

### 1. Axum 0.8 path parameter syntax
Use `{param}`, **not** `:param`. The old matchit 0.7 colon syntax causes a panic at startup in Axum 0.8 (matchit 0.8).

### 2. Nested router prefix stripping
`Router::nest("/prefix", inner_router)` strips the prefix from `req.uri()` for both handlers and middleware inside the nested router. Middleware added inside a nested router sees the stripped path.

`authorize_middleware` (inside the `/v1/api/user` nest) uses `req.extensions().get::<axum::extract::OriginalUri>()` to get the full pre-strip path. Policy rules must reference full paths (e.g. `/v1/api/user/canary`).

### 3. Repository `DbConnection` import conflict
See the **Repository Pattern** section above. Never add `use crate::DbConnection;` at the top of a repository file — use the fully qualified path inline in custom method signatures.

### 4. RSA key format for OIDC signing
`jsonwebtoken::EncodingKey::from_rsa_der()` uses ring's `RsaKeyPair::from_der()` internally, which requires **PKCS#1 DER** format (RFC 3447 `RSAPrivateKey`). Use `rsa::pkcs1::EncodeRsaPrivateKey::to_pkcs1_der()` — **not** `pkcs8::EncodePrivateKey::to_pkcs8_der()`.

### 5. Diesel schema and `TEXT[]` columns
`diesel migration run` auto-regenerates `schema.rs` and maps PostgreSQL `TEXT[]` to `Array<Nullable<Text>>`. This is fixed in two places:

- **`schema.rs`** — handled automatically via `src/schema.patch` (configured in `diesel.toml` under `patch_file`), which corrects array columns to `Array<Text>` after every schema regeneration.
- **`schema_models.rs`** — handled automatically by `gen_diesel_types.sh`, which maps `Array<Text>` → `Vec<String>` and `Array<Nullable<Text>>` → `Vec<Option<String>>` via its `map_type` function.

If you add new `TEXT[]` columns that must map to `Vec<String>`:
1. Manually correct `schema.rs` (change `Array<Nullable<Text>>` → `Array<Text>` for the new columns)
2. Run `scripts/update_schema_patch.sh` to regenerate `src/schema.patch` from the diff
3. Re-run `gen_diesel_types.sh` — `schema_models.rs` will be correct automatically

### 6. `!Send` in async handlers (ThreadRng)
`rand::thread_rng()` returns `ThreadRng` which is `!Send`. Holding it across an `.await` point makes the future `!Send`, breaking Axum's `Handler` trait bound. Use `rand::rngs::OsRng` instead (zero-sized, `Send`), and wrap all synchronous crypto in a block `{ ... }` so it drops before any `.await`.

### 7. `gen_diesel_types.sh` — adding custom derives to generated enums
The script hard-codes a default derive set for enums. To add extra derives to a specific enum (e.g. `utoipa::ToSchema` on `UserStatus`), add an entry to the `ENUM_EXTRA_DERIVES` associative array near the top of the script:

```bash
declare -A ENUM_EXTRA_DERIVES=(
  ["UserStatus"]="utoipa::ToSchema"
  ["MyNewEnum"]="SomeOtherDerive"
)
```

The key is the **PascalCase Rust enum name**; the value is a comma-separated list appended after the default derives. This survives every regeneration without manual fixups.

### 8. Diesel `find_where` import pattern
When using `find_where` with a specific column, import the column from `crate::schema::table::column` (not from `dsl`) and add `use diesel::ExpressionMethods;`. Using the `dsl` re-export can cause an E0599 "not an iterator" error.

### 9. Lifetime parameter required when passing `&str` to `find_where`
If a custom repository method passes a `&str` (or any borrowed value) to `find_where`, the connection reference and the borrowed value must share a named lifetime. Rust cannot infer that `'_` on the connection outlives `'_` on the string:

```rust
// ✗ fails — lifetime error: '1 must outlive '2
pub async fn find_by_hash(&self, conn: &mut DbConnection<'_>, hash: &str) -> ...

// ✓ correct — both borrows share lifetime 'a
pub async fn find_by_hash<'a>(&self, conn: &mut DbConnection<'a>, hash: &'a str) -> ...
```

### 10. `lettre` requires `smtp-transport` feature
`lettre 0.11` splits transports into separate features. `AsyncSmtpTransport` and `Tokio1Executor` are gated behind `smtp-transport`. Without it, the import compiles but the type is missing at link time:

```toml
# ✓ correct
lettre = { version = "0.11", features = ["smtp-transport", "tokio1-native-tls", "builder"], default-features = false }
```

### 11. `VerifyEmailQuery` uses `IntoParams`, not `ToSchema`
`VerifyEmailQuery` is a query-param struct extracted with `Query<VerifyEmailQuery>`. It must derive `utoipa::IntoParams` (not `ToSchema`) and be referenced in the handler's `#[utoipa::path(params(VerifyEmailQuery))]`. Do **not** add it to the `schemas(...)` list in `openapi.rs` — it will cause a compile error.

### 12. Integration test DB helpers — use `tokio-postgres`, not `psql`
The Rust Docker image (`rust:1.x`) does not include the `psql` binary. Test helpers that need to update the DB directly (e.g. pre-verifying a user's email so `authorize_middleware` doesn't block the test user) must use `tokio-postgres` (added to `[dev-dependencies]`):

```rust
async fn mark_email_verified(email: &str) {
    dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set for test");
    let (client, connection) = tokio_postgres::connect(&db_url, tokio_postgres::NoTls)
        .await.expect("DB connection failed");
    tokio::spawn(async move { if let Err(e) = connection.await { eprintln!("{e}"); } });
    client.execute(
        "UPDATE user_identities SET email_verified = true WHERE email = $1", &[&email],
    ).await.expect("DB update failed");
}
```

### 13. `authorize_middleware` blocks unverified users — pre-verify in tests
After email verification was introduced, any test that registers a user and then hits an `authorize_middleware`-protected endpoint must pre-verify that user's email in the DB before logging in. This is required because the `ev` claim is baked into the PASETO token at login time — logging in before verification produces a token with `ev: false`, which is permanently blocked even after DB verification. Always call the DB helper **before** the login step.

### 14. `EmailTemplateType` Diesel enum follows the same pattern as `RoleType`
`EmailTemplateType` uses `diesel_derive_enum::DbEnum` with `#[ExistingTypePath = "crate::schema::sql_types::EmailTemplateType"]`. It is listed in `ENUM_EXTRA_DERIVES` in `gen_diesel_types.sh` so that `utoipa::ToSchema` is auto-derived on regeneration. When calling `find_where` with an `EmailTemplateType` column, pass the enum value by value (not by reference) — clone it first if you need to reuse it after the call:

```rust
let template_type_clone = req.template_type.clone();
self.repo.find_where(conn, col.eq(template_type_clone).and(...)).await?;
```

## sentinel-ui (React Admin Dashboard)

`apps/sentinel-ui` is a React 19 single-page app built with Vite. It uses `@sentinel/auth-react` (`packages/sentinel-auth-react`) for all auth UI and `@sentinel/auth-sdk` (`packages/sentinel-auth-sdk`) for API communication.

### Key files

| File | Purpose |
|------|---------|
| `src/lib/sdkClient.ts` | Singleton `SentinelAuthClient` configured from `VITE_API_URL` |
| `src/api/auth.ts` | Thin wrappers over `sentinelClient.login/logout/user.getMe/…` |
| `src/api/admin.ts` | Thin wrappers over `sentinelClient.admin.*`, `sentinelClient.apiTokens.*`, and `sentinelClient.system.*` |
| `src/types/index.ts` | Re-exports SDK types (`RoleData`, `ApiTokenData`, `EmailTemplateData`, etc.) |
| `src/App.tsx` | Mounts `SentinelAuthProvider`, `SentinelAuthRoutes`, `createSentinelQueryClient` from `@sentinel/auth-react` |

Auth state (`isAuthenticated`, `accessToken`, `userId`, etc.) comes from `useAuthStore()` exported by `@sentinel/auth-react`. The auth store, `useAuth` hook, token refresh helpers, route guards, and all auth pages live in `packages/sentinel-auth-react` — they are not duplicated in this app.

### Auth integration pattern

```tsx
// src/App.tsx
import {
  SentinelAuthProvider, SentinelAuthRoutes, ProtectedRoute, AuthorizedRoute,
  createSentinelQueryClient, useAuthStore,
} from '@sentinel/auth-react';
import '@sentinel/auth-react/dist/style.css';

const redirects = { afterLogin: '/dashboard', afterLogout: '/login', … };
const queryClient = createSentinelQueryClient(redirects);

<SentinelAuthProvider client={sentinelClient} redirects={redirects}>
  <QueryClientProvider client={queryClient}>
    <BrowserRouter>
      <Routes>
        <Route path="/*" element={<SentinelAuthRoutes />} />
        <Route element={<ProtectedRoute />}>
          <Route element={<AuthorizedRoute />}>
            <Route element={<AppShell />}>{/* admin routes */}</Route>
          </Route>
        </Route>
      </Routes>
    </BrowserRouter>
  </QueryClientProvider>
</SentinelAuthProvider>
```

### SDK type differences from old hand-rolled types

When working on UI pages, use SDK types directly — do not add new types to `src/types/index.ts` unless you are re-exporting from the SDK:

| SDK field | Old UI field | Notes |
|-----------|-------------|-------|
| `ApiTokenData.api_token_id` | `ApiToken.token_id` | Use `api_token_id` everywhere |
| `RoleData.role_type` | `Role.permissions` | SDK exposes `role_type`, not a permissions array |
| `UserSessionData.is_current` | `Session.revoked_at` | Use `is_current` to derive active/inactive status |
| `CreateRoleRequest.role_type` | — | Required field; one of `'user'`, `'admin'`, `'support'` |
| `CreatePolicyRequest.environment` | — | Required field; use `'production'` as default |

### ProvidersPage (`src/pages/providers/ProvidersPage.tsx`)

Manages SMTP email provider configurations. Key design details:

- **Provider presets** (`PRESETS` map): Resend, Mailjet, Custom SMTP — auto-fill host/port/username and `default_auth_type`
- **Auth type toggle** (`AuthType = "credentials" | "api_key"`): switches between username+password fields and a single API key field; Resend defaults to `api_key`, others to `credentials`
- **`openEdit(c)`** detects stored auth type from `config_redacted` keys: presence of `"api_key"` key → `api_key` mode (the redacted value is `"****"` but the key name is preserved)
- **Test button**: calls `adminApi.testProviderConfig(id)` → `POST .../test`; result stored in `testResults` keyed by `configuration_id`; clears on entering edit mode
- **Send test email button**: calls `adminApi.sendTestProviderEmail(id, to_email)` → `POST .../send-test`; sends an actual email to verify delivery end-to-end (distinct from connection-only `/test`)
- All mutations use `buildConfig()` / `buildEditConfig()` which emit either `{ password }` or `{ api_key }` based on the current `authType`/`editAuthType`

### Rebuilding local packages

When you change files in `packages/sentinel-auth-sdk/src/`, rebuild before running the UI:

```bash
cd packages/sentinel-auth-sdk && npm run build
```

When you change files in `packages/sentinel-auth-react/src/`, rebuild before running the UI:

```bash
cd packages/sentinel-auth-react && npm run build
```

Both packages are referenced as `file:` dependencies in `sentinel-ui/package.json`. The UI references their built `dist/` output.

`vite.config.ts` sets `resolve.preserveSymlinks: true` so Vite resolves peer deps from `sentinel-ui/node_modules/` rather than following the symlink to the real path (where peer deps are not installed).

## sentinel-auth-react (React Auth UI Package)

`packages/sentinel-auth-react` (`@sentinel/auth-react`) is a drop-in React auth UI library extracted from `sentinel-ui`. It is framework-agnostic — the compiled output works with any bundler (Vite, webpack, Rollup, Parcel, Next.js).

See `packages/sentinel-auth-react/CLAUDE.md` for full developer reference.

### Key architectural details

- **`SentinelAuthProvider`** — context provider; registers token refresh client, injects CSS variable overrides via `<style>` tag from `theme` prop
- **`SentinelAuthRoutes`** — `<Routes>` block covering all auth paths; derives paths from `redirects` config
- **`createSentinelQueryClient(redirects?)`** — TanStack QueryClient factory; pre-wires 401 → refresh, 403 → redirect, `EmailNotVerifiedError` → redirect
- **`useAuthStore()`** — Zustand store persisted to `localStorage` under key `"sentinel-auth"`; holds `userId`, `accessToken`, `refreshToken`, `isAuthenticated`, MFA/password flags
- **`refreshTokens()`** — deduped module-level helper; called by `SentinelAuthProvider` and available to Axios interceptors without React context
- **CSS distribution** — consumers must import `@sentinel/auth-react/dist/style.css` once in their app entry; the package entry point does not auto-import it (framework-agnostic requirement)
- **Dockerfile** — `apps/sentinel-ui/Dockerfile.dev` has an `auth-react-builder` stage that builds the package so `dist/` exists at the real path inside the Docker image

---

## Code Style & Patterns

This section documents the concrete patterns used throughout `sentinel-core`. Follow them exactly when adding new code.

### Naming Conventions

| Kind | Convention | Example |
|------|-----------|---------|
| Files | `snake_case` | `api_token_service.rs` |
| Structs / Enums | `PascalCase` | `ApiTokenService`, `LoginOutcome` |
| Functions / methods | `snake_case` | `create_api_token`, `revoke_all_for_user` |
| Constants | `SCREAMING_SNAKE_CASE` | `API_TOKEN_PREFIX`, `MAX_ATTEMPTS` |
| HTTP error codes | `SCREAMING_SNAKE_CASE` string | `"API_TOKEN_NOT_FOUND"` |
| Application structs | `<Domain>Application` | `AuthApplication`, `ApiTokenApplication` |
| Service structs | `<Domain>Service` | `SessionService`, `MfaTotpService` |
| Repository structs | `<Domain>Repository` | `ApiTokenRepository`, `UserRepository` |
| Request DTOs | `<Action><Resource>Request` | `CreateApiTokenRequest`, `BasicAuthLoginRequest` |
| Response DTOs | `<Resource>Response` or `<Action><Resource>Response` | `ApiTokenResponse`, `CreateApiTokenResponse` |

### Layer Structure

Every feature follows the same four-layer pattern. Never skip layers or import across boundaries.

```
Handler → Application → Service(s) → Repository
```

- **Handler**: extracts inputs, calls one application method, returns `Result<RawResponse<T>, ApiError>`
- **Application**: orchestrates services, owns transaction boundaries, enforces admin/role guards
- **Service**: single-responsibility business logic; calls one repository and any external clients
- **Repository**: only Diesel queries; no business logic

### Handler Template

```rust
/// POST /v1/api/example/{id}
///
/// One-line description. Brief notes on auth, side-effects, etc.
#[utoipa::path(
    post,
    path = "/v1/api/example/{id}",
    request_body = ExampleRequest,
    params(
        ("id" = Uuid, Path, description = "The resource identifier")
    ),
    responses(
        (status = 200, body = ExampleResponse),
        (status = 400, body = ApiErrorResponse),
        (status = 401, body = ApiErrorResponse),
    ),
    security(("BearerAuth" = [])),
    tag = "example"
)]
pub async fn handler_name(
    Extension(state): Extension<Arc<AppState>>,
    ValidatedBearer(token): ValidatedBearer,
    Path(id): Path<Uuid>,
    ValidatedJson(req): ValidatedJson<ExampleRequest>,
) -> Result<RawResponse<ExampleResponse>, ApiError> {
    state
        .example_app
        .do_thing(token, id, req)
        .await
        .map(RawResponse)
        .map_err(ApiError::from)
}
```

Rules:
- Extractor order: `Extension` → `ValidatedBearer` → `Path` → `Query` → `ValidatedJson`
- Return type is always `Result<RawResponse<T>, ApiError>`
- Handler body is a single expression (no variable bindings unless unavoidable)
- Add handler to `openapi.rs` `paths(...)` and any new DTO to `components(schemas(...))`
- Tag must be one of: `"auth"`, `"user"`, `"admin"`, `"system"`, `"oidc"`

### Application Template

```rust
/// ExampleApplication — orchestrates [Domain] flows.
///
/// Responsibilities: <list>
/// Notable: <anything non-obvious>
pub struct ExampleApplication {
    pg_client: Arc<PostgresClient>,
    example_service: Arc<ExampleService>,
    other_service: Arc<OtherService>,
}

impl ExampleApplication {
    pub fn new(
        pg_client: Arc<PostgresClient>,
        example_service: Arc<ExampleService>,
        other_service: Arc<OtherService>,
    ) -> Self {
        Self { pg_client, example_service, other_service }
    }

    pub async fn do_thing(
        &self,
        token: String,
        id: Uuid,
        req: ExampleRequest,
    ) -> Result<ExampleResponse, ServiceError> {
        let mut conn = self.pg_client.get_conn().await?;
        // ... orchestrate services
    }
}
```

Rules:
- Constructor accepts `Arc<T>` for every dependency; store them as `Arc<T>` fields
- Methods return `Result<T, ServiceError>`, never `ApiError`
- Admin guard: call `require_admin(&ctx)?` at the top of any admin-only method
- Multi-service flows use `conn.transaction(...)` for atomicity
- Clone services before entering a `move` closure: `let svc = self.example_service.clone();`

### Service Template

```rust
pub struct ExampleService {
    repo: Arc<ExampleRepository>,
}

impl ExampleService {
    pub fn new(repo: Arc<ExampleRepository>) -> Self {
        Self { repo }
    }

    pub async fn create(
        &self,
        conn: &mut DbConnection<'_>,
        payload: NewExample,
    ) -> Result<Example, ServiceError> {
        self.repo
            .create(conn, payload)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }
}
```

Rules:
- Services take `&mut DbConnection<'_>` received from the application layer — they never open their own connections
- Use `OsRng` (not `thread_rng()`); wrap synchronous crypto in `{ ... }` blocks to drop before any `.await`
- Log with `tracing::debug!` on entry; `tracing::error!` on failures with context fields

### Repository Template

```rust
impl_repository!(
    ExampleRepository for Example,
    crate::schema::examples::table,
    crate::schema::examples::example_id,
    Uuid
);

impl ExampleRepository {
    pub async fn find_by_hash<'a>(
        &self,
        conn: &mut DbConnection<'a>,
        hash: &'a str,
    ) -> Result<Option<Example>, RepositoryError> {
        use crate::schema::examples::hash_col;
        use diesel::ExpressionMethods;

        self.find_where(conn, hash_col.eq(hash)).await
    }
}
```

Rules:
- **Never** `use crate::DbConnection;` at the top of a repository file (macro already imports it)
- Import columns from `crate::schema::table::column`, not from `dsl`
- Bulk operations need a custom method — `update_where` returns exactly one row
- Borrowed arguments passed to `find_where` need a named lifetime shared with the connection

### DTO Template

```rust
/// Request body for creating an example resource.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateExampleRequest {
    /// Human-readable name (3–64 chars).
    #[validate(length(min = 3, max = 64))]
    pub name: String,

    /// Optional description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Response returned after creating an example resource.
#[derive(Debug, Serialize, ToSchema)]
pub struct ExampleResponse {
    pub example_id: Uuid,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
}
```

Rules:
- Request DTOs: `Deserialize + Validate + ToSchema`
- Response DTOs: `Serialize + ToSchema`
- Query-param structs: `Deserialize + IntoParams` (NOT `ToSchema`; do not add to `schemas(...)` in `openapi.rs`)
- Use `#[serde(skip_serializing_if = "Option::is_none")]` on all optional response fields
- Brief doc comment on each field

### Error Mapping

New `ServiceError` variants must be added to `errors.rs` and mapped in the `From<ServiceError> for ApiError` impl. Follow this pattern:

| Scenario | HTTP status | Error code string |
|----------|------------|-------------------|
| Resource not found | 404 | `"<RESOURCE>_NOT_FOUND"` |
| Caller is not authenticated | 401 | `"AUTH_ERROR"` |
| Caller lacks permission | 403 | `"AUTHORIZATION_ERROR"` |
| Invalid input | 400 | `"VALIDATION_ERROR"` |
| Conflict / duplicate | 409 | `"<RESOURCE>_CONFLICT"` |
| Internal / DB failure | 500 | `"INTERNAL_ERROR"` (no details leaked) |

### Tracing / Logging

```rust
// Entry — log what came in
tracing::debug!(user_id = %ctx.user_id, "create_example: starting");

// Exit — log what went out
tracing::debug!(example_id = %result.example_id, "create_example: done");

// Error — include context
tracing::error!(error = %e, user_id = %ctx.user_id, "create_example: db error");
```

Rules:
- Use `%` (Display) for IDs, status codes, short strings; use `?` (Debug) only for complex types
- Do not log secrets, passwords, or raw tokens — log only IDs and non-sensitive metadata
- INFO level is reserved for the `TraceLayer` (request/response); use `debug!` inside handlers and services

### Integration Test Conventions

```rust
#[tokio::test]
async fn create_example_without_auth_returns_401() {
    let url = get_example_url();
    let body = json!({ "name": "test" });
    let res = post_json(&url, &body, None).await;  // None = no token
    let (status, body, _) = read_json(res).await;
    assert_eq!(status, 401);
    assert_error_envelope(&body, "AUTH_ERROR");
}
```

Rules:
- Test name format: `<action>_<condition>_returns_<outcome>`
- Always inject a unique IP: `post_with_ip(&url, &body, Some(&token), &unique_ip()).await` for rate-limited endpoints
- Pre-verify email in DB **before** login when the test hits `authorize_middleware`-protected routes (see gotcha #13)
- Mark tests that require live SMTP or admin seeding with `#[ignore]`
- Use `tokio_postgres` (not `psql`) for direct DB access in test helpers (see gotcha #12)

### Adding a New Endpoint — Checklist

1. **Migration** (if new table): `diesel migration generate <name>` → write SQL → run → regenerate schema + models
2. **Domain**: add entity to `schema_models.rs` (or run `gen_diesel_types.sh`), add repository in `domain/repositories/`
3. **Service**: create `services/<domain>_service.rs`, add to `services/mod.rs`
4. **Application**: add method to existing application or create `applications/<domain>_application.rs`, add to `applications/mod.rs`
5. **DTOs**: add to `infrastructure/http/api/dtos/<domain>_dtos.rs`, add to `dtos/mod.rs`
6. **Handler**: add to `infrastructure/http/api/handlers/<domain>_handlers.rs` with `#[utoipa::path]`
7. **Routes**: register in the appropriate `build_*_routes()` function in `infrastructure/http/api/routes/`
8. **OpenAPI**: add handler to `paths(...)` and DTOs to `components(schemas(...))` in `openapi.rs`
9. **DI wiring**: if new service/application, construct and pass through `infrastructure/http/app.rs`
10. **Tests**: add integration test in `tests/<domain>_test.rs`
