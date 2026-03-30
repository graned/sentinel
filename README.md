# Sentinel Auth

A self-hosted authentication and authorization service written in Rust. Sentinel handles user identity, session management, multi-factor authentication, role-based access control, and acts as a full OIDC Identity Provider so external applications can use "Log in with Sentinel" via the standard Authorization Code + PKCE flow.

## Features

- **User registration & login** — email/password with PostgreSQL `pgcrypt` hashing
- **PASETO v4 sessions** — short-lived access tokens + refresh tokens backed by revocable DB sessions
- **Multi-factor authentication** — TOTP (authenticator app) enrollment, login challenge flow, and one-time recovery codes
- **OIDC Identity Provider** — Authorization Code + PKCE flow, RS256 JWT ID tokens and access tokens, JWKS endpoint, OpenID Connect discovery document; `amr` claim reflects MFA usage
- **Policy-based authorization** — compiled RBAC trie engine; rules are defined in JSON, compiled once, and evaluated at sub-microsecond speed on every request
- **Role management** — admin API to create, update, delete, and assign roles to users
- **Email verification** — verification link sent on registration; unverified users are blocked from protected endpoints; `email_verified` claim embedded in PASETO tokens
- **Password reset & change** — forgot/reset flow with short-lived `pr_*` tokens; authenticated change flow; sessions revoked on both; anti-enumeration (always 200 on forgot)
- **Email templates** — admin-configurable templates with `{{placeholder}}` rendering; built-in defaults work out-of-the-box without configuration
- **API tokens** — long-lived opaque tokens for programmatic access (CI/CD, scripts); raw token shown once at creation, only the SHA-256 hash stored in the DB
- **Swagger UI** — interactive API docs at `/swagger-ui`

## Repository Structure

```
sentinel-auth/
├── apps/
│   ├── sentinel-core/          # Main Axum HTTP service
│   │   ├── src/
│   │   │   ├── applications/   # Use-case orchestrators
│   │   │   ├── services/       # Business logic
│   │   │   ├── domain/         # Entities and repositories
│   │   │   └── infrastructure/ # HTTP layer, DI wiring
│   │   ├── migrations/         # Diesel SQL migrations
│   │   └── tests/              # Integration tests
│   └── sentinel-ui/            # React admin dashboard (Vite + TypeScript)
└── packages/
    ├── sentinel-policy-engine/ # Standalone RBAC engine crate
    ├── sentinel-auth-sdk/      # TypeScript SDK for Sentinel Auth (@sentinel/auth-sdk)
    └── sentinel-auth-react/    # Drop-in React auth UI bundle (@sentinel/auth-react)
```

## Prerequisites

- [Docker](https://docs.docker.com/get-docker/) and Docker Compose
- [Diesel CLI](https://diesel.rs/guides/getting-started) (for running migrations outside Docker)
- Node.js 18+ (for the management UI and SDK)

## Setup

**1. Copy the environment file and fill in values:**

```bash
cp apps/sentinel-core/.env.example apps/sentinel-core/.env
```

Required variables:

| Variable | Description |
|---|---|
| `DATABASE_URL` | PostgreSQL connection string |
| `HEX_KEY` | 32-byte hex key for PASETO session encryption |
| `CONFIG_ENCRYPTION_KEY` | 32-byte hex key for encrypting stored secrets (OIDC keys, MFA secrets, SMTP config) |
| `APP_HOST` | Bind host (e.g. `0.0.0.0`) |
| `APP_PORT` | Bind port (e.g. `8000`) |
| `OIDC_ISSUER_URL` | Public base URL used in OIDC tokens (e.g. `http://localhost:9000`) |
| `FRONTEND_URL` | Base URL for verification/reset links sent in emails (e.g. `http://localhost:3000`) |
| `CORS_ALLOWED_ORIGINS` | Comma-separated allowed CORS origins (e.g. `http://localhost:3000`); if unset, all cross-origin requests are denied |

## Running the Dev Environment

The full stack runs in Docker with hot-reload on file changes:

```bash
docker compose -f docker-compose.dev.yml up -d
```

Migrations run automatically via the `migrate` service before `sentinel-core` starts. No manual `diesel migration run` step is needed.

| Service | URL |
|---|---|
| API | http://localhost:9000 |
| Swagger UI | http://localhost:9000/swagger-ui |
| Management UI | http://localhost:3000 |

All services start alongside the backend and auto-reload on file changes. The management UI uses `@sentinel/auth-sdk` to communicate with the API.

## Running Tests

```bash
# All tests
docker compose -f docker-compose.dev.yml run sentinel-core cargo test

# Single test or test file
docker compose -f docker-compose.dev.yml run sentinel-core cargo test <test_name>

# Policy engine only
docker compose -f docker-compose.dev.yml run sentinel-core cargo test -p sentinel-policy-engine

# Policy engine benchmarks (HTML report in target/criterion/)
docker compose -f docker-compose.dev.yml run sentinel-core cargo bench -p sentinel-policy-engine
```

## API Reference

The full interactive reference is available at `/swagger-ui`. All `/v1/api/*` responses use a standard envelope:

```json
{
  "success": true,
  "data": { },
  "error": null,
  "timestamp": "2026-03-01T10:00:00Z",
  "request_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

OIDC endpoints (`/oauth/*`, `/.well-known/*`) return raw spec-compliant JSON and are not wrapped.

### Auth

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/v1/api/auth/register` | Register a new user (triggers verification email if SMTP configured) |
| `POST` | `/v1/api/auth/login` | Login; returns session tokens or an MFA challenge |
| `POST` | `/v1/api/auth/authenticate` | Validate a token and return the full user context |
| `POST` | `/v1/api/auth/logout` | Revoke the current session (Bearer required) |
| `POST` | `/v1/api/auth/logout-all` | Revoke all sessions for the user (Bearer required) |
| `GET`  | `/v1/api/auth/auth-methods` | List configured authentication methods |
| `POST` | `/v1/api/auth/token/authorize` | Check if a token is authorized for a resource |
| `POST` | `/v1/api/auth/token/refresh` | Exchange a refresh token for new session tokens |
| `GET`  | `/v1/api/auth/verify-email` | Verify email address via `?token=<raw>` query param |
| `POST` | `/v1/api/auth/resend-verification` | Resend the verification email |
| `POST` | `/v1/api/auth/password/forgot` | Request a password reset link (always 200 — anti-enumeration) |
| `POST` | `/v1/api/auth/password/reset` | Reset password using a `pr_*` token from the reset email |

On registration a `ev_*` verification token is generated and emailed if SMTP is configured. Endpoints protected by `authorize_middleware` return `403 EMAIL_NOT_VERIFIED` for unverified users. API tokens (`sat_*`) bypass this gate.

Password reset tokens (`pr_*`) expire after 1 hour. On a successful reset, all existing sessions are revoked and a confirmation email is sent.

### MFA (TOTP)

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/v1/api/auth/mfa/totp/start` | Begin TOTP enrollment — returns QR code URI (Bearer required) |
| `POST` | `/v1/api/auth/mfa/totp/confirm` | Confirm enrollment with first TOTP code (Bearer required) |
| `POST` | `/v1/api/auth/mfa/verify` | Complete MFA login with a TOTP code or recovery code |

Login with MFA enabled returns a short-lived `mfa_session_token` instead of full session tokens. Submit it alongside a TOTP code (or one of the 8 one-time recovery codes) to `/mfa/verify` to receive the full access and refresh tokens.

### API Tokens (admin role + Bearer required)

Long-lived opaque tokens for programmatic access. The raw token (`sat_<64 hex chars>`) is returned exactly once at creation — only its SHA-256 hash is stored in the database. Revocation is a soft-delete via `revoked_at`.

| Method | Path | Description |
|--------|------|-------------|
| `POST`   | `/v1/api/auth/api-tokens` | Create a new API token |
| `GET`    | `/v1/api/auth/api-tokens` | List all API tokens for the authenticated user |
| `DELETE` | `/v1/api/auth/api-tokens/{token_id}` | Revoke a specific token |
| `DELETE` | `/v1/api/auth/api-tokens` | Revoke all tokens for the authenticated user |

### User (Bearer required)

| Method | Path | Description |
|--------|------|-------------|
| `GET`  | `/v1/api/user/me` | Get the authenticated user's profile |
| `POST` | `/v1/api/user/password/change` | Change password (requires current password; revokes all sessions) |
| `GET`  | `/v1/api/user/sessions` | List all active sessions |
| `GET`  | `/v1/api/user/sessions/{session_id}` | Get details for a specific session |
| `GET`  | `/v1/api/user/permissions` | Get the authenticated user's assigned roles |

### Admin (admin role + Bearer required)

All admin endpoints require a valid Bearer token whose user has the `admin` role. The check is enforced in the application layer, not as middleware.

**Role management:**

| Method | Path | Description |
|--------|------|-------------|
| `POST`   | `/v1/api/admin/roles` | Create a new role |
| `GET`    | `/v1/api/admin/roles` | List all roles |
| `PUT`    | `/v1/api/admin/roles/{role_id}` | Update a role's name or description |
| `DELETE` | `/v1/api/admin/roles/{role_id}` | Delete a role |

**User role management:**

| Method | Path | Description |
|--------|------|-------------|
| `POST`   | `/v1/api/admin/users/{user_id}/roles` | Assign a role to a user |
| `DELETE` | `/v1/api/admin/users/{user_id}/roles/{role_name}` | Remove a role from a user |
| `GET`    | `/v1/api/admin/users/{user_id}/permissions` | List a user's assigned roles |
| `GET`    | `/v1/api/admin/users/{user_id}/auth-info` | Get full profile + identity + roles for a user |

**Policy management:**

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/v1/api/admin/policies` | Create an authorization policy |
| `PUT`  | `/v1/api/admin/policies/{policy_id}/rules` | Update policy rules (recompiles the trie) |

**OIDC management:**

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/v1/api/admin/oidc/clients` | Register a new OIDC client application |
| `POST` | `/v1/api/admin/oidc/keys/generate` | Generate a new RSA signing key |

**Email templates:**

| Method | Path | Description |
|--------|------|-------------|
| `GET`  | `/v1/api/admin/email-templates` | List all email templates |
| `POST` | `/v1/api/admin/email-templates` | Create a template (deactivates the previous active template of the same type) |
| `PUT`  | `/v1/api/admin/email-templates/{template_id}` | Update a template |

Template types: `EmailVerification`, `PasswordReset`, `PasswordChanged`. Built-in defaults are used when no active template exists, so the system works without configuration.

### System (admin role + Bearer required unless noted)

| Method | Path | Description |
|--------|------|-------------|
| `GET`  | `/v1/api/system/health` | Health check (public) |
| `POST` | `/v1/api/system/config/email` | Add an email provider configuration |
| `GET`  | `/v1/api/system/config/email` | List all provider configurations (secrets redacted) |
| `PUT`  | `/v1/api/system/config/email/{config_id}` | Update a provider configuration |
| `DELETE` | `/v1/api/system/config/email/{config_id}` | Delete a provider configuration |
| `GET`  | `/v1/api/system/config/email/{config_id}/reveal` | Return the decrypted (plaintext) config |
| `POST` | `/v1/api/system/config/email/{config_id}/test` | Test the SMTP connection for a configuration |
| `POST` | `/v1/api/system/config/email/{config_id}/send-test` | Send a test email through a configured provider |

Provider config supports two authentication modes:
- **Credentials**: `username` + `password` fields (Mailjet, custom SMTP)
- **API Key**: single `api_key` field used as the SMTP password (Resend — username is preset to `"resend"`)

Both `password` and `api_key` are accepted by the backend; the email service reads whichever is present.

### OIDC / OAuth 2.0

These endpoints follow the OAuth 2.0 and OpenID Connect specifications and return standard JSON, not the Sentinel response envelope.

| Method | Path | Description |
|--------|------|-------------|
| `GET`  | `/.well-known/openid-configuration` | OIDC discovery document |
| `GET`  | `/oauth/jwks.json` | Public RSA signing keys (JWKS) |
| `GET`  | `/oauth/authorize` | Start Authorization Code + PKCE flow (requires Sentinel session) |
| `POST` | `/oauth/token` | Exchange auth code for JWT ID token + access token (form-encoded) |

## Authorization Policy Engine

Sentinel uses a compiled RBAC engine (`sentinel-policy-engine`) to authorize requests. Rules are defined in JSON:

```json
{
  "rules": [
    { "method": "GET",    "path": "/v1/api/user/me",    "roles": ["user", "admin"] },
    { "method": "*",      "path": "/v1/api/admin/**",   "roles": ["admin"] }
  ]
}
```

Rules are compiled into a sorted trie at write time and stored in the database. At request time authorization is evaluated in O(path depth) — around 150 ns per check. Literal path segments are prioritized over parameters, wildcards, and globs. See [`packages/sentinel-policy-engine/README.md`](packages/sentinel-policy-engine/README.md) for full documentation.

## Architecture

Sentinel follows Clean Architecture with a strict dependency rule — inner layers never import from outer ones:

```
Infrastructure  (HTTP handlers, DB pool, router, middleware)
      ↑
Application     (use-case orchestrators: auth, mfa, oidc, policy, system,
                 user, api_token, user_password, email_template, admin)
      ↑
Services        (single-responsibility: session, identity, user_role,
                 mfa_totp, oidc_key, oidc_token, email, email_template, …)
      ↑
Domain          (entities, repository trait implementations)
```

The application layer is where multi-service flows and transaction boundaries live. Handlers call application structs, not services directly. The DI container (`infrastructure/http/app.rs`) wires everything together.

## TypeScript SDK

`packages/sentinel-auth-sdk` (`@sentinel/auth-sdk`) is a full-featured TypeScript client for the Sentinel Auth API. It is used by `sentinel-ui` and can be consumed by any external Node.js or browser application.

```ts
import { SentinelAuthClient } from '@sentinel/auth-sdk';

const client = new SentinelAuthClient({ baseUrl: 'https://auth.example.com' });
const result = await client.login({ email: 'alice@example.com', password: 'SuperSecret1!' });
```

See [`packages/sentinel-auth-sdk/README.md`](packages/sentinel-auth-sdk/README.md) for the full API reference.

## React Auth UI

`packages/sentinel-auth-react` (`@sentinel/auth-react`) is a drop-in React auth UI library. It provides a complete authentication flow — login, register, email verification, password reset/change, TOTP MFA setup, and route guards — as a single provider and routes bundle. Works with any React bundler: Vite, webpack, Next.js, Parcel, Rollup.

```tsx
import { SentinelAuthProvider, SentinelAuthRoutes, ProtectedRoute, createSentinelQueryClient } from '@sentinel/auth-react';
import '@sentinel/auth-react/dist/style.css';

const queryClient = createSentinelQueryClient(redirects);

<SentinelAuthProvider client={sentinelClient} redirects={redirects}>
  <QueryClientProvider client={queryClient}>
    <BrowserRouter>
      <Routes>
        <Route path="/*" element={<SentinelAuthRoutes />} />
        <Route element={<ProtectedRoute />}>
          <Route path="/dashboard" element={<Dashboard />} />
        </Route>
      </Routes>
    </BrowserRouter>
  </QueryClientProvider>
</SentinelAuthProvider>
```

See [`packages/sentinel-auth-react/README.md`](packages/sentinel-auth-react/README.md) for the full API reference.

## Management UI

`apps/sentinel-ui` is a React 19 admin dashboard built with Vite. It provides a web interface for managing roles, policies, API tokens, email templates, provider configurations, and sessions. It uses `@sentinel/auth-react` for all auth UI and `@sentinel/auth-sdk` for API communication.

See [`apps/sentinel-ui/README.md`](apps/sentinel-ui/README.md) for setup and development details.

For contributor guidance, internal conventions, and development gotchas see [`CLAUDE.md`](CLAUDE.md).

## License

Sentinel is licensed under the **GNU Affero General Public License v3.0 (AGPL-3.0-only)**.

Why AGPL?
- We want Sentinel to remain open and community-driven.
- If you modify Sentinel and distribute it, you must keep those changes open.
- If you run a modified version of Sentinel as a network service, you must also make the corresponding source code available under the same license.

This project is intentionally licensed to preserve openness, prevent closed-source forks of the core, and ensure that improvements made by others remain available to the community.

Please read the full license in [`LICENSE`](./LICENSE).

## Project Mission

Sentinel exists to make authentication and authorization more accessible, transparent, and available to everyone.
We want a healthy open-source ecosystem where improvements are shared back, security issues are handled responsibly, and the community can build together in the open.

## Attribution

If you redistribute or modify Sentinel, you must preserve the existing copyright notices and license notices.
