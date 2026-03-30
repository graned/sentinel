# Sentinel Admin UI

React 19 management dashboard for the Sentinel Auth server. Provides a web interface for administering roles, policies, API tokens, email templates, and user sessions. Auth UI (login, register, MFA, password reset, email verification) is provided by `@sentinel/auth-react`.

## Stack

- **React 19** with TypeScript
- **Vite 7** — dev server with HMR and proxy to the backend
- **TanStack React Query** — data fetching and cache invalidation
- **`@sentinel/auth-react`** — complete auth UI bundle (login, register, MFA, password flows, route guards)
- **`@sentinel/auth-sdk`** — all API communication goes through the SDK client

## Source layout

```
src/
  lib/
    sdkClient.ts          # singleton SentinelAuthClient (from @sentinel/auth-sdk)
    withAuthRetry.ts      # wraps admin API calls with 401→refresh retry
  api/
    auth.ts               # thin wrappers: sentinelClient.login / logout / user.getMe
    admin.ts              # thin wrappers: sentinelClient.admin.* / apiTokens.* / system.*
  types/
    index.ts              # re-exports SDK types (RoleData, ApiTokenData, etc.)
  pages/
    dashboard/            # DashboardPage — overview stats
    roles/                # RolesPage — create / delete roles
    sessions/             # SessionsPage — list user sessions
    tokens/               # TokensPage — create / revoke API tokens
    email/                # EmailTemplatesPage — manage email templates
    policies/             # PoliciesPage — create policies and define rules
    providers/            # ProvidersPage — manage SMTP provider configurations
    users/                # UsersPage — list users, assign/remove roles
  components/
    layout/
      AppShell.tsx        # navigation sidebar + top bar (uses useAuthStore from @sentinel/auth-react)
```

Auth pages (login, register, verify-email, forgot/reset password, change-password, setup-mfa,
unauthorized) are mounted via `<SentinelAuthRoutes />` from `@sentinel/auth-react` — they are
not authored in this app.

## Auth integration

`App.tsx` mounts the full auth flow with three components from `@sentinel/auth-react`:

```tsx
import {
  SentinelAuthProvider,
  SentinelAuthRoutes,
  ProtectedRoute,
  AuthorizedRoute,
  createSentinelQueryClient,
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
            <Route element={<AppShell />}>
              {/* admin routes */}
            </Route>
          </Route>
        </Route>
      </Routes>
    </BrowserRouter>
  </QueryClientProvider>
</SentinelAuthProvider>
```

`createSentinelQueryClient` pre-wires:
- **401** → token refresh attempt; on failure clears session and redirects to `/login`
- **`EmailNotVerifiedError`** → redirects to `/verify-email`
- **403 `MUST_CHANGE_PASSWORD`** → redirects to `/change-password`
- **403 other** → redirects to `/unauthorized`

Auth state (`isAuthenticated`, `accessToken`, `userId`, etc.) comes from
`useAuthStore()` exported by `@sentinel/auth-react`.

## ProvidersPage

`src/pages/providers/ProvidersPage.tsx` manages SMTP email provider configurations:

- **Provider presets** — selecting Resend, Mailjet, or Custom SMTP auto-fills host/port/username
- **Auth type toggle** — "Username & Password" or "API Key" mode; Resend defaults to API Key
- **Test button** — calls `POST .../test` to verify the SMTP handshake
- **Send test email** — calls `POST .../send-test` to verify end-to-end delivery
- **Reveal** — decrypts and shows the plaintext config in a modal

## Prerequisites

- Node.js 18+
- Sentinel Auth backend running (Docker Compose stack or standalone)

## Development

The easiest way to run everything together is via Docker Compose from the repo root:

```bash
docker compose -f docker-compose.dev.yml up -d
```

The UI is then available at **http://localhost:3000**. Vite proxies `/v1/*` requests to
the backend, so no CORS configuration is needed in dev.

To run the UI standalone (backend must already be running):

```bash
# Install dependencies (including the local @sentinel/auth-react and @sentinel/auth-sdk)
npm install --legacy-peer-deps

# Start dev server
npm run dev
```

Set `VITE_API_URL` to override the backend base URL (default: `http://localhost:8080`):

```bash
VITE_API_URL=http://localhost:9000 npm run dev
```

> **Note:** `--legacy-peer-deps` is required because the workspace uses React 19 /
> react-router-dom v7 / zustand v5 which don't perfectly satisfy all `>=N` peer dep ranges.

## Build

```bash
npm run build    # TypeScript compile + Vite production build → dist/
npm run preview  # Serve the production build locally
```

## Linting & formatting

```bash
npm run lint         # oxlint
npm run lint:fix     # oxlint --fix
npm run format       # prettier --write
npm run format:check # prettier --check
```

## Local package dependencies

This app references two local packages as `file:` dependencies:

| Package | Local path |
|---------|-----------|
| `@sentinel/auth-sdk` | `packages/sentinel-auth-sdk` |
| `@sentinel/auth-react` | `packages/sentinel-auth-react` |

If you modify either package's source, rebuild it before running the UI:

```bash
# SDK
cd ../../packages/sentinel-auth-sdk && npm run build

# Auth React
cd ../../packages/sentinel-auth-react && npm run build
```

`vite.config.ts` sets `resolve.preserveSymlinks: true` so Vite resolves peer deps
from `sentinel-ui/node_modules/` rather than following the symlink to the package's
real path (where peer deps are not installed).
