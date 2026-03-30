# CLAUDE.md — sentinel-auth-sdk

TypeScript SDK package for the Sentinel Auth server. Covers the full API surface: authentication, user management, MFA, API tokens, admin operations, system health, and provider configuration management.

## Commands

**Install dependencies:**
```bash
cd packages/sentinel-auth-sdk && npm install
```

**Type-check (no emit):**
```bash
cd packages/sentinel-auth-sdk && npx tsc --noEmit
```

**Build (CJS + ESM + .d.ts into `dist/`):**
```bash
cd packages/sentinel-auth-sdk && npm run build
```

**Watch mode:**
```bash
cd packages/sentinel-auth-sdk && npm run dev
```

**Run all tests (unit + integration — requires Docker stack):**
```bash
cd packages/sentinel-auth-sdk && npm test
```

**Run unit tests only (no Docker needed):**
```bash
cd packages/sentinel-auth-sdk && npm run test:unit
```

**Run integration tests only:**
```bash
cd packages/sentinel-auth-sdk && npm run test:integration
```

Docker stack must be running for integration tests:
```bash
docker compose -f docker-compose.dev.yml up -d
```

**Lint (`biome lint`):**
```bash
cd packages/sentinel-auth-sdk && npm run lint
```

**Format with auto-fix (`biome format --write`):**
```bash
cd packages/sentinel-auth-sdk && npm run format
```

**Lint + format + organise imports in one pass (`biome check --write`):**
```bash
cd packages/sentinel-auth-sdk && npm run check
```

## Architecture

```
src/
  types.ts            — all request/response interfaces (mirrors Rust DTOs) + RequestFn type
  errors.ts           — SentinelError base + typed subclasses; createErrorFromCode() factory
  session.ts          — SessionCache: in-memory Map with expiry helpers (no HTTP)
  client.ts           — SentinelAuthClient: auth methods + session cache + sub-client wiring
  user-client.ts      — UserClient: profile, sessions, permissions, password change
  mfa-client.ts       — MfaClient: TOTP enrollment, MFA login verification
  api-token-client.ts — ApiTokenClient: create/list/revoke/revokeAll (admin)
  admin-client.ts     — AdminClient: roles, user roles, policies, email templates (admin)
  system-client.ts    — SystemClient: health check + provider config CRUD + test connection
  index.ts            — public re-exports (all types, errors, clients)
```

### Dependency graph (no circular imports)

```
client.ts           → session.ts, errors.ts, types.ts
                      user-client.ts, mfa-client.ts, api-token-client.ts,
                      admin-client.ts, system-client.ts
user-client.ts      → types.ts
mfa-client.ts       → types.ts
api-token-client.ts → types.ts
admin-client.ts     → types.ts
system-client.ts    → types.ts
session.ts          → types.ts
errors.ts           → (no internal deps)
index.ts            → everything
```

### Sub-client injection pattern

Sub-clients receive the bound `request()` method from `SentinelAuthClient` via a `RequestFn` callback, so they share the same base URL, default headers, and error-handling logic without inheritance:

```ts
// In SentinelAuthClient constructor:
const req = this.request.bind(this);
this.user      = new UserClient(req);
this.apiTokens = new ApiTokenClient(req);
// MfaClient also receives toSession + cacheSession callbacks to store the
// session on verify() without needing direct cache access:
this.mfa = new MfaClient(req, this.toSession.bind(this), (s) => this.cache.set(s.userId, s));
```

The `RequestFn` type is exported from `src/types.ts`:
```ts
type RequestFn = <T>(path: string, options?: RequestInit) => Promise<{ data: T; requestId: string }>;
```

## Tests

```
tests/
  helpers.ts          — uniqueEmail, uniqueIp, registerUser, markEmailVerified, registerAndVerify
  session.test.ts     — 15 unit tests for SessionCache (no HTTP, no Docker)
  login.test.ts       — login validation, auth errors, session caching, logout, getValidSession
  register.test.ts    — register validation + success, getAuthMethods, authenticate,
                        forgotPassword anti-enumeration, logoutAll
  user.test.ts        — getMe, getSessions, getSession, getPermissions, changePassword,
                        system.health
```

### Why `markEmailVerified` uses `pg` directly

The email verification middleware bakes the `ev` claim into PASETO tokens at login time. Tokens issued before DB verification permanently carry `ev: false` and are rejected by all protected endpoints. The only safe approach is to mark the email verified in the DB **before** the first login call — there is no public API endpoint for this in tests.

DB connection defaults: `postgresql://postgres:password@localhost:5432/sentinel_auth`.
Override via `DATABASE_URL` env var.

### Rate-limit bypass in tests

Several endpoints are rate-limited per IP. Each test creates its own `SentinelAuthClient` with a unique `X-Forwarded-For` header via `uniqueIp()` so no two tests share a bucket:

```ts
function makeClient(): SentinelAuthClient {
  return new SentinelAuthClient({
    baseUrl: API_BASE,
    headers: { 'X-Forwarded-For': uniqueIp() },
  });
}
```

Rate limits:
- Login / MFA verify: **5 req / 15 min** (strict)
- Register / forgot-password / resend-verification: **10 req / 15 min** (moderate)

## Key design decisions

### Sub-client namespaces, not a flat interface
Methods are grouped by domain (`client.user`, `client.mfa`, `client.admin`, etc.) to keep the main client navigable as the API surface grows. Auth-flow methods (`login`, `logout`, `register`, etc.) and session cache helpers live directly on `SentinelAuthClient`.

### `login()` returns a discriminated union
`LoginResult` is `{ type: 'session' } | { type: 'mfa_challenge' }`. Callers must handle both branches at the type level. The `mfa_challenge` branch is not cached — the full session is established only after `client.mfa.verify()`.

### `getValidSession()` is the safe accessor
- Returns `null` for missing/expired sessions (never throws for those cases).
- Auto-refreshes when within `refreshBufferMs` of expiry (default 5 min).
- `getSession()` is the raw synchronous accessor — no expiry check, no network call.

### `logout()` / `logoutAll()` clean cache before the network call
Local state is always consistent even when the server is unreachable.

### `SessionCache` is pure (no HTTP)
All network logic lives in `SentinelAuthClient`. `SessionCache` is a plain `Map` wrapper with TTL helpers, making it straightforwardly testable in isolation.

### Authenticated sub-client methods take `accessToken` explicitly
```ts
await client.user.getMe(session.accessToken);
```
This makes the auth contract visible at the call site and decouples sub-clients from the session cache. Callers are expected to call `getValidSession()` first.

### Error factory (`createErrorFromCode`)
Maps server `ApiErrorBody.code` strings to the right typed subclass. Add a new `case` here whenever a new server error code is introduced. The factory is also exported for use in tests.

## Adding new endpoints

1. Add request/response interfaces to `src/types.ts`.
2. Decide which sub-client owns the method (or add it directly to `client.ts` if it's auth-flow).
3. Implement the method using the injected `this.req<T>()` function — pass `Authorization: Bearer ${accessToken}` in `headers` for protected endpoints.
4. If the endpoint returns a new error code, add a subclass in `src/errors.ts` and a `case` in `createErrorFromCode`.
5. Re-export new types/classes from `src/index.ts`.
6. Add integration tests in the relevant `tests/*.test.ts` file.

## Development gotchas

### `moduleResolution: "Bundler"` — no `.js` extensions in imports
`tsconfig.json` uses `"moduleResolution": "Bundler"`. Import paths must **not** include `.js` extensions. tsup handles extension rewriting at build time.

### `fetch` is not polyfilled
The SDK uses the global `fetch` available in Node.js 18+ and modern browsers. If you need to support older environments, inject a polyfill before instantiating the client.

### `Object.setPrototypeOf` in error constructors
Each `SentinelError` subclass calls `Object.setPrototypeOf(this, new.target.prototype)` to fix `instanceof` checks when compiled to ES5. Do not remove this line when adding new error subclasses.

### `SentinelConfig.refreshBufferMs` interaction with short-lived tokens
If `refreshBufferMs` is set larger than the server's access token TTL, `getValidSession()` will attempt a refresh on every call. Keep it well below the token lifetime (default server TTL is ~1 hour; default SDK buffer is 5 min).

### Mocha uses `tsx/cjs` — no `.js` loader flag needed
`.mocharc.yml` uses `require: tsx/cjs`. This patches Node's CJS loader to handle TypeScript on the fly. Do not add `--loader tsx` or `--import tsx/esm` — it conflicts and causes duplicate-module issues.

### Chai v5 types are bundled — do not add `@types/chai`
Chai 5 ships its own TypeScript declarations. Adding `@types/chai` to `devDependencies` will cause type conflicts. `@types/mocha` is still required (mocha does not bundle types).

### Admin sub-client tests require a seeded admin user
`client.admin.*` and `client.apiTokens.*` methods enforce the `admin` role server-side. Integration tests for these paths are not yet included because there is no public seed API to create an admin account. When an admin seed endpoint is available, add tests to a new `tests/admin.test.ts` file following the same `registerAndVerify` + `makeClient` pattern.
