# sentinel-auth-sdk

TypeScript client for [Sentinel Auth](../../apps/sentinel-core). Covers the full Sentinel Auth API surface — authentication, user management, MFA, API tokens, admin operations, and system health — with in-memory session caching, automatic token refresh, and typed error handling.

## Requirements

- Node.js 18+ (native `fetch`)
- Sentinel Auth server running and reachable

## Installation

```bash
# from a workspace root that includes this package
npm install @sentinel/auth-sdk
```

## Quick start

```ts
import { SentinelAuthClient } from '@sentinel/auth-sdk';

const client = new SentinelAuthClient({
  baseUrl: 'https://auth.example.com',
});

// Register
await client.register({
  first_name: 'Alice',
  last_name: 'Smith',
  email: 'alice@example.com',
  password: 'SuperSecret1!',
});

// Login
const result = await client.login({ email: 'alice@example.com', password: 'SuperSecret1!' });

if (result.type === 'mfa_challenge') {
  // MFA is enabled — complete login via client.mfa.verify()
  const session = await client.mfa.verify({
    mfa_session_token: result.mfaSessionToken,
    code: '123456',
  });
} else {
  const { session } = result;

  // User profile
  const profile = await client.user.getMe(session.accessToken);
  console.log(profile.email, profile.roles);

  // Later — get a valid (auto-refreshed) session before any API call
  const live = await client.getValidSession(session.userId);
}
```

## Configuration

```ts
new SentinelAuthClient(config: SentinelConfig)
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `baseUrl` | `string` | — | Base URL of the Sentinel Auth server |
| `refreshBufferMs` | `number` | `300000` (5 min) | How far before expiry `getValidSession()` proactively refreshes |
| `headers` | `Record<string, string>` | `{}` | Default headers merged into every request (useful for `X-Forwarded-For`, auth proxies, etc.) |

## API reference

### Auth (`client.*`)

All auth methods live directly on the `SentinelAuthClient` instance.

| Method | Endpoint | Auth | Description |
|--------|----------|------|-------------|
| `register(body)` | `POST /v1/api/auth/register` | — | Create a new user account |
| `login(credentials)` | `POST /v1/api/auth/login` | — | Log in; returns `session` or `mfa_challenge` |
| `logout(userId)` | `POST /v1/api/auth/logout` | Bearer | Revoke current session; clears cache |
| `logoutAll(userId)` | `POST /v1/api/auth/logout-all` | Bearer | Revoke all sessions; clears cache |
| `refreshSession(userId)` | `POST /v1/api/auth/token/refresh` | — | Exchange refresh token for new tokens |
| `getValidSession(userId)` | — | — | Cache-aware accessor with auto-refresh |
| `getSession(userId)` | — | — | Raw synchronous cache lookup |
| `clearSession(userId)` | — | — | Remove one cached session locally |
| `clearAllSessions()` | — | — | Remove all cached sessions locally |
| `evictExpiredSessions()` | — | — | Purge expired sessions; returns count |
| `authenticate(body)` | `POST /v1/api/auth/authenticate` | — | Validate a token; return auth context |
| `checkAuthorization(body)` | `POST /v1/api/auth/token/authorize` | — | Policy-based authz check |
| `verifyEmail(token)` | `GET /v1/api/auth/verify-email` | — | Confirm email address |
| `resendVerification(body)` | `POST /v1/api/auth/resend-verification` | — | Re-send verification email |
| `getAuthMethods()` | `GET /v1/api/auth/auth-methods` | — | List enabled auth methods |
| `forgotPassword(body)` | `POST /v1/api/auth/password/forgot` | — | Request a password reset email |
| `resetPassword(body)` | `POST /v1/api/auth/password/reset` | — | Set new password via reset token |

#### `login()` return type

```ts
type LoginResult =
  | { type: 'session'; session: Session }
  | { type: 'mfa_challenge'; userId: string; mfaSessionToken: string };
```

---

### User (`client.user.*`)

All methods require a valid Bearer `accessToken`. Obtain one via `getValidSession()`.

```ts
const session = await client.getValidSession(userId);
const profile  = await client.user.getMe(session.accessToken);
```

| Method | Endpoint | Description |
|--------|----------|-------------|
| `getMe(token)` | `GET /v1/api/user/me` | Fetch own profile |
| `updateProfile(token, body)` | `PATCH /v1/api/user/me` | Update own name / avatar |
| `changePassword(token, body)` | `POST /v1/api/user/password/change` | Change password; revokes all sessions |
| `getSessions(token)` | `GET /v1/api/user/sessions` | List all sessions |
| `getSession(token, sessionId)` | `GET /v1/api/user/sessions/{id}` | Get one session detail |
| `getPermissions(token)` | `GET /v1/api/user/permissions` | Get own roles |

---

### MFA (`client.mfa.*`)

```ts
// Enrollment (user must be logged in)
const { otpauth_uri } = await client.mfa.totpStart(accessToken);
// Render otpauth_uri as a QR code, then:
const { recovery_codes } = await client.mfa.totpConfirm(accessToken, { code: '123456' });

// Login with MFA
const session = await client.mfa.verify({ mfa_session_token, code: '123456' });
```

| Method | Endpoint | Auth | Description |
|--------|----------|------|-------------|
| `totpStart(token)` | `POST /v1/api/auth/mfa/totp/start` | Bearer | Begin enrollment; returns `otpauth_uri` |
| `totpConfirm(token, body)` | `POST /v1/api/auth/mfa/totp/confirm` | Bearer | Confirm enrollment; returns recovery codes |
| `verify(body)` | `POST /v1/api/auth/mfa/verify` | — | Complete MFA login; stores session in cache |

---

### API tokens (`client.apiTokens.*`)

Admin role required for all methods.

```ts
const { token } = await client.apiTokens.create(accessToken, { name: 'CI pipeline' });
// token is `sat_<hex>` — store it, it is returned only once
```

| Method | Endpoint | Description |
|--------|----------|-------------|
| `create(token, body)` | `POST /v1/api/auth/api-tokens` | Create token; raw value returned once |
| `list(token)` | `GET /v1/api/auth/api-tokens` | List all tokens (active + revoked) |
| `revoke(token, tokenId)` | `DELETE /v1/api/auth/api-tokens/{id}` | Soft-revoke one token |
| `revokeAll(token)` | `DELETE /v1/api/auth/api-tokens` | Revoke all tokens |

---

### Admin (`client.admin.*`)

Admin role required for all methods.

#### Roles

| Method | Endpoint | Description |
|--------|----------|-------------|
| `createRole(token, body)` | `POST /v1/api/admin/roles` | Create role (`role_type`: `user`/`admin`/`support`) |
| `listRoles(token)` | `GET /v1/api/admin/roles` | List all roles |
| `updateRole(token, roleId, body)` | `PUT /v1/api/admin/roles/{id}` | Update name/description |
| `deleteRole(token, roleId)` | `DELETE /v1/api/admin/roles/{id}` | Delete a role |
| `assignRole(token, userId, body)` | `POST /v1/api/admin/users/{id}/roles` | Assign role to user |
| `removeRole(token, userId, roleName)` | `DELETE /v1/api/admin/users/{id}/roles/{name}` | Remove role from user |
| `getUserPermissions(token, userId)` | `GET /v1/api/admin/users/{id}/permissions` | Get user's roles |
| `getUserAuthInfo(token, userId)` | `GET /v1/api/admin/users/{id}/auth-info` | Get profile + roles |

#### Policies

| Method | Endpoint | Description |
|--------|----------|-------------|
| `createPolicy(token, body)` | `POST /v1/api/admin/policies` | Create RBAC policy with rules |
| `updatePolicyRules(token, policyId, body)` | `PUT /v1/api/admin/policies/{id}/rules` | Replace rules; activates new version |

#### Email templates

| Method | Endpoint | Description |
|--------|----------|-------------|
| `listEmailTemplates(token)` | `GET /v1/api/admin/email-templates` | List all templates |
| `createEmailTemplate(token, body)` | `POST /v1/api/admin/email-templates` | Create template (deactivates previous) |
| `updateEmailTemplate(token, templateId, body)` | `PUT /v1/api/admin/email-templates/{id}` | Update template |

---

### System (`client.system.*`)

| Method | Endpoint | Auth | Description |
|--------|----------|------|-------------|
| `health()` | `GET /v1/api/system/health` | — | Server health check |
| `listProviderConfigs(token)` | `GET /v1/api/system/config/email` | Bearer + admin | List configurations (secrets redacted) |
| `createProviderConfig(token, body)` | `POST /v1/api/system/config/email` | Bearer + admin | Add a provider configuration |
| `updateProviderConfig(token, configId, body)` | `PUT /v1/api/system/config/email/{id}` | Bearer + admin | Update a provider configuration |
| `deleteProviderConfig(token, configId)` | `DELETE /v1/api/system/config/email/{id}` | Bearer + admin | Delete a provider configuration |
| `revealProviderConfig(token, configId)` | `GET /v1/api/system/config/email/{id}/reveal` | Bearer + admin | Get plaintext decrypted config |
| `testProviderConfig(token, configId)` | `POST /v1/api/system/config/email/{id}/test` | Bearer + admin | Test SMTP connection; returns `{ success, message }` |

Provider config body shape:

```ts
// credentials mode (Mailjet, custom SMTP)
{ provider: 'mailjet', config: { host, port, username, password, from_email, use_tls }, is_active: true }

// API key mode (Resend)
{ provider: 'resend', config: { host, port, username, api_key, from_email, use_tls }, is_active: true }
```

The backend accepts either `password` or `api_key` — whichever is present is used as the SMTP credential.

---

## Session shape

```ts
interface Session {
  userId: string;
  accessToken: string;   // PASETO v4.local, ~1 hour TTL
  refreshToken: string;  // PASETO v4.local, longer TTL
  expiresAt: Date;       // access token expiry
}
```

## Error handling

All errors extend `SentinelError`:

```ts
import {
  SentinelAuthClient,
  AuthenticationError,
  EmailNotVerifiedError,
  MfaInvalidCodeError,
  RateLimitError,
  NetworkError,
} from '@sentinel/auth-sdk';

try {
  await client.login({ email, password });
} catch (err) {
  if (err instanceof AuthenticationError) {
    // 401 — wrong credentials
  } else if (err instanceof EmailNotVerifiedError) {
    // 403 — user hasn't clicked the verification link
  } else if (err instanceof RateLimitError) {
    // 429 — 5 req / 15 min limit hit
  } else if (err instanceof NetworkError) {
    // Server unreachable
  } else {
    throw err;
  }
}
```

Every `SentinelError` carries:

| Property | Type | Description |
|----------|------|-------------|
| `code` | `string` | Machine-readable code (e.g. `AUTH_ERROR`) |
| `message` | `string` | Human-readable description |
| `statusCode` | `number` | HTTP status (0 for network/local errors) |
| `requestId` | `string \| undefined` | Server request ID for tracing |
| `details` | `unknown` | Optional extra context from the server |

### Error code reference

| Class | Code | HTTP |
|-------|------|------|
| `AuthenticationError` | `AUTH_ERROR` | 401 |
| `ValidationError` | `VALIDATION_ERROR` | 400 |
| `InvalidTokenError` | `INVALID_TOKEN` | 401 |
| `ExpiredTokenError` | `EXPIRED_TOKEN` | 401 |
| `MissingTokenError` | `MISSING_TOKEN` | 401 |
| `EmailNotVerifiedError` | `EMAIL_NOT_VERIFIED` | 403 |
| `ForbiddenError` | `FORBIDDEN` | 403 |
| `RateLimitError` | `RATE_LIMIT_EXCEEDED` | 429 |
| `MfaInvalidCodeError` | `INVALID_MFA_CODE` | 401 |
| `MfaAttemptLimitError` | `MFA_ATTEMPT_LIMIT_EXCEEDED` | 429 |
| `ApiTokenNotFoundError` | `API_TOKEN_NOT_FOUND` | 404 |
| `InternalServerError` | `INTERNAL_ERROR` | 500 |
| `NetworkError` | `NETWORK_ERROR` | 0 |
| `SessionNotFoundError` | `SESSION_NOT_FOUND` | 0 |

## Build

```bash
npm run build      # compile to dist/ (CJS + ESM + type declarations)
npm run typecheck  # type-check without emitting
npm run dev        # watch mode
```

## Testing

```bash
npm run test:unit          # 15 unit tests — SessionCache only, no Docker needed
npm run test:integration   # integration tests against http://localhost:9000
npm test                   # both suites
```

Docker stack must be running for integration tests:

```bash
docker compose -f docker-compose.dev.yml up -d
```

## Linting & formatting

```bash
npm run lint     # biome lint
npm run format   # biome format --write
npm run check    # lint + format + organise imports in one pass
```
