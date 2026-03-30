# sentinel-auth-react

Drop-in React auth UI for [Sentinel Auth](../../apps/sentinel-core). Provides a complete authentication flow — login, register, email verification, password reset/change, TOTP MFA setup, and route guards — as a single provider and routes bundle. Works with any React bundler: Vite, webpack, Next.js, Parcel, Rollup.

## Requirements

- React 18+
- react-router-dom v6+
- @tanstack/react-query v5+
- zustand v4+
- @sentinel/auth-sdk (Sentinel Auth TypeScript client)

## Installation

```bash
npm install @sentinel/auth-react
# peer dependencies
npm install @sentinel/auth-sdk @tanstack/react-query react-router-dom zustand
```

## Quick start

### Vite / webpack / Parcel

```tsx
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { QueryClientProvider } from '@tanstack/react-query';
import {
  SentinelAuthProvider,
  SentinelAuthRoutes,
  ProtectedRoute,
  createSentinelQueryClient,
} from '@sentinel/auth-react';
import '@sentinel/auth-react/dist/style.css';
import { SentinelAuthClient } from '@sentinel/auth-sdk';

const client = new SentinelAuthClient({ baseUrl: 'https://auth.example.com' });

const redirects = {
  afterLogin:       '/dashboard',
  afterLogout:      '/login',
  login:            '/login',
  register:         '/register',
  verifyEmail:      '/verify-email',
  forgotPassword:   '/forgot-password',
  changePassword:   '/change-password',
  setupMfa:         '/setup-mfa',
  unauthorized:     '/unauthorized',
};

const queryClient = createSentinelQueryClient(redirects);

export default function App() {
  return (
    <SentinelAuthProvider client={client} redirects={redirects}>
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <Routes>
            {/* login, register, verify-email, forgot/reset password, change-password, setup-mfa, unauthorized */}
            <Route path="/*" element={<SentinelAuthRoutes />} />

            {/* protected app routes */}
            <Route element={<ProtectedRoute />}>
              <Route path="/dashboard" element={<Dashboard />} />
            </Route>
          </Routes>
        </BrowserRouter>
      </QueryClientProvider>
    </SentinelAuthProvider>
  );
}
```

### Next.js (App Router)

```tsx
// app/layout.tsx
import '@sentinel/auth-react/dist/style.css';
import { SentinelAuthProvider } from '@sentinel/auth-react';
import { SentinelAuthClient } from '@sentinel/auth-sdk';

const client = new SentinelAuthClient({ baseUrl: process.env.NEXT_PUBLIC_AUTH_URL! });

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html>
      <body>
        <SentinelAuthProvider client={client} redirects={{ afterLogin: '/dashboard' }}>
          {children}
        </SentinelAuthProvider>
      </body>
    </html>
  );
}
```

## Configuration

### `SentinelAuthProvider`

```tsx
<SentinelAuthProvider
  client={sentinelClient}   // required — SentinelAuthClient instance from @sentinel/auth-sdk
  redirects={redirects}     // optional — navigation paths (all have sensible defaults)
  theme={theme}             // optional — branding and color overrides
>
  {children}
</SentinelAuthProvider>
```

#### `redirects`

All fields are optional. Defaults shown.

| Key | Default | Description |
|-----|---------|-------------|
| `afterLogin` | `"/dashboard"` | Navigate here after a successful login |
| `afterLogout` | `"/login"` | Navigate here after logout |
| `afterRegister` | `"/verify-email"` | Navigate here after registration |
| `login` | `"/login"` | Login page path (used by guards and links) |
| `register` | `"/register"` | Register page path |
| `verifyEmail` | `"/verify-email"` | Email verification page path |
| `forgotPassword` | `"/forgot-password"` | Forgot password page path |
| `changePassword` | `"/change-password"` | Forced password change page path |
| `setupMfa` | `"/setup-mfa"` | Forced MFA setup page path |
| `unauthorized` | `"/unauthorized"` | Access-denied page path |

#### `theme`

| Key | Description |
|-----|-------------|
| `appName` | App name in the auth page wordmark (default: `"Sentinel"`) |
| `tagline` | Tagline shown on auth pages |
| `copyright` | Footer copyright line |
| `primaryColor` | Overrides the `--accent-primary` CSS variable |
| `secondaryColor` | Overrides the `--accent-blue` CSS variable |

---

## API reference

### `SentinelAuthRoutes`

Renders a `<Routes>` block covering all auth paths. Mount it under a catch-all route.

```tsx
<Route path="/*" element={<SentinelAuthRoutes />} />
```

Included routes (paths are taken from `redirects` with the defaults above):

| Path | Page | Guard |
|------|------|-------|
| `/login` | `LoginPage` | Public-only |
| `/register` | `RegisterPage` | Public-only |
| `/verify-email` | `VerifyEmailPage` | Open |
| `/forgot-password` | `ForgotPasswordPage` | Open |
| `/reset-password` | `ResetPasswordPage` | Open |
| `/change-password` | `ChangePasswordForcedPage` | Protected |
| `/setup-mfa` | `SetupMfaForcedPage` | Protected |
| `/unauthorized` | `UnauthorizedPage` | Protected |

---

### Route guards

```tsx
import { ProtectedRoute, PublicRoute, AuthorizedRoute } from '@sentinel/auth-react';

// ProtectedRoute — redirects to /login when not authenticated;
// also redirects to /change-password or /setup-mfa when required
<Route element={<ProtectedRoute />}>
  <Route path="/dashboard" element={<Dashboard />} />
</Route>

// PublicRoute — redirects to /dashboard when already authenticated
<Route element={<PublicRoute />}>
  <Route path="/login" element={<LoginPage />} />
</Route>

// AuthorizedRoute — calls checkAuthorization() before rendering;
// redirects to /unauthorized on 403
<Route element={<ProtectedRoute />}>
  <Route element={<AuthorizedRoute />}>
    <Route path="/admin" element={<AdminPage />} />
  </Route>
</Route>
```

---

### `createSentinelQueryClient(redirects?)`

Creates a pre-configured TanStack `QueryClient` whose `QueryCache.onError` handler covers all common auth failure scenarios:

| Error | Action |
|-------|--------|
| `401` | Attempt token refresh; on success invalidate all queries; on failure clear session → redirect to `afterLogout` |
| `EmailNotVerifiedError` | Redirect to `verifyEmail` |
| `403 MUST_CHANGE_PASSWORD` | Redirect to `changePassword` |
| Other `403` | Redirect to `unauthorized` |

```tsx
const qc = createSentinelQueryClient(redirects);
// …
<QueryClientProvider client={qc}>…</QueryClientProvider>
```

---

### `useAuth()`

```tsx
import { useAuth } from '@sentinel/auth-react';

const { login, verifyMfa, logout, isLoading, error } = useAuth();

// Standard login (also handles forced password change / forced MFA setup)
await login({ email: 'alice@example.com', password: 'SuperSecret1!' });

// Complete MFA challenge after login returned { type: 'mfa_challenge' }
await verifyMfa({ mfa_session_token: '…', code: '123456' });

// Logout
await logout(userId);
```

---

### `useAuthStore()`

```tsx
import { useAuthStore } from '@sentinel/auth-react';

const {
  userId,
  accessToken,
  refreshToken,
  isAuthenticated,
  emailVerified,
  isAdmin,
  mustChangePassword,
  mfaSetupRequired,
  userEmail,
  firstName,
  lastName,
} = useAuthStore();
```

State is persisted to `localStorage` under the key `"sentinel-auth"`.

---

### `useSentinelAuth()` / `useSentinelConfig()`

Both are the same hook — `useSentinelConfig` is an alias.

```tsx
import { useSentinelAuth } from '@sentinel/auth-react';

const { client, redirects, theme } = useSentinelAuth();
```

Throws if called outside `<SentinelAuthProvider>`.

---

### `refreshTokens()` / `registerTokenRefreshClient(client)`

Low-level helpers for integrating token refresh outside React context (Axios interceptors, mutation error handlers, etc.). `SentinelAuthProvider` calls `registerTokenRefreshClient` automatically — you only need this if you're wiring up a custom HTTP layer.

```tsx
import { refreshTokens, registerTokenRefreshClient } from '@sentinel/auth-react';

// Call once at startup (Provider handles this automatically)
registerTokenRefreshClient(client);

// Refresh from anywhere — automatically deduplicated across concurrent callers
const refreshed = await refreshTokens();
if (!refreshed) {
  // session could not be restored — user must log in again
}
```

---

### Standalone page components

All pages are individually exported so you can use them outside `<SentinelAuthRoutes>`:

```tsx
import {
  LoginPage,
  RegisterPage,
  ForgotPasswordPage,
  ResetPasswordPage,
  VerifyEmailPage,
  ChangePasswordForcedPage,
  SetupMfaForcedPage,
  UnauthorizedPage,
  Button,
} from '@sentinel/auth-react';
```

Each page must be rendered inside `<SentinelAuthProvider>`.

---

## Styling

Import the bundled stylesheet **once** in your app entry (or in `_app.tsx` / `layout.tsx`):

```tsx
import '@sentinel/auth-react/dist/style.css';
```

The stylesheet uses CSS custom properties. You can override any variable globally:

```css
:root {
  --accent-primary:  #6366f1;   /* indigo instead of cyan */
  --accent-blue:     #8b5cf6;
  --bg-surface:      #18181b;
  --text-primary:    #fafafa;
}
```

Or pass overrides via `<SentinelAuthProvider theme={…}>` — it injects an inline `<style>` tag targeting `:root`, so no separate CSS file is needed for branding.

---

## Build

```bash
npm run build      # Vite library build → dist/ (ESM + CJS + style.css + .d.ts)
npm run typecheck  # type-check without emitting
npm run dev        # watch mode (rebuilds on source changes)
```

After modifying source files, rebuild before running the consuming app so the updated `dist/` is picked up.
