# CLAUDE.md — sentinel-auth-react

Developer reference for working on `packages/sentinel-auth-react` (`@sentinel/auth-react`).

## Commands

```bash
# Install dependencies (run from this directory)
npm install --legacy-peer-deps

# Build: ESM + CJS + style.css + .d.ts → dist/
npm run build

# Watch mode — rebuilds on every source change
npm run dev

# Type-check without emitting
npm run typecheck
```

> **Note:** `--legacy-peer-deps` is needed because React 19 / react-router-dom v7 / zustand v5
> dev deps don't perfectly satisfy peer dep ranges that declare `>=18` / `>=6` / `>=4`.
> The compiled output is fully compatible — this is a declaration artifact only.

After any source change, rebuild (`npm run build`) before running a consuming app so the
updated `dist/` is picked up.

---

## Source Layout

```
src/
├── index.ts                              # All public exports
├── types.ts                              # SentinelAuthContextValue, SentinelAuthRedirects, SentinelTheme
├── vite-env.d.ts                         # declare module "*.module.css" (bundler-agnostic)
│
├── context/
│   ├── SentinelAuthContext.ts            # React.createContext<SentinelAuthContextValue | null>
│   └── SentinelAuthProvider.tsx          # Provider: registers refresh client, injects CSS vars
│
├── store/
│   └── authStore.ts                      # Zustand store (persisted to localStorage)
│
├── hooks/
│   ├── useAuth.ts                        # login / verifyMfa / logout (client from context)
│   └── useSentinelConfig.ts             # reads context — throws outside Provider
│
├── lib/
│   ├── tokenRefresh.ts                   # module-level refresh client + deduped refreshTokens()
│   └── createSentinelQueryClient.ts      # TanStack QueryClient factory with 401/403 handlers
│
└── components/
    ├── guards/
    │   ├── ProtectedRoute.tsx            # Redirects to /login when unauthenticated
    │   └── AuthorizedRoute.tsx           # Calls authenticateAndAuthorize(); redirects on 403
    ├── routes/
    │   └── SentinelAuthRoutes.tsx        # <Routes> covering all auth paths
    ├── ui/
    │   └── Button.tsx                    # Shared button component (+ Button.module.css)
    └── pages/
        ├── LoginPage.tsx                 # Email/password login + inline MFA challenge screen
        ├── RegisterPage.tsx              # Registration form
        ├── ForgotPasswordPage.tsx        # Forgot password — sends reset link
        ├── ResetPasswordPage.tsx         # Reset password using pr_* token from email
        ├── VerifyEmailPage.tsx           # Email verification via ?token= query param
        ├── ChangePasswordForcedPage.tsx  # Forced password change (mustChangePassword flag)
        ├── SetupMfaForcedPage.tsx        # Forced MFA setup (mfaSetupRequired flag)
        └── UnauthorizedPage.tsx          # 403 landing page
```

Each page file has a corresponding `PageName.module.css` in the same directory.
The base styles shared by all pages live in `src/styles/defaults.css`.

---

## Build Output (`dist/`)

| File | Description |
|------|-------------|
| `dist/index.js` | ESM bundle |
| `dist/index.cjs` | CommonJS bundle |
| `dist/index.d.ts` | TypeScript declarations |
| `dist/style.css` | All CSS (global base styles + hashed CSS Module classes) |

The `dist/style.css` file must be imported once by the consuming app:

```ts
import '@sentinel/auth-react/dist/style.css';
```

This is **not** auto-imported by the package entry point — it is the consumer's
responsibility, exactly like `react-toastify/dist/ReactToastify.css` and similar libraries.

---

## Key Design Decisions

### 1. CSS side-effect import removed from `src/index.ts`

`src/index.ts` does **not** contain `import "./styles/defaults.css"`.

If it did, `tsc --emitDeclarationOnly` would copy it verbatim into `dist/index.d.ts`,
causing `TS2307: Cannot find module './styles/defaults.css'` in any TypeScript project
that doesn't have a global `declare module "*.css"`. Removing the import from the entry
point keeps the package framework-agnostic.

### 2. `src/vite-env.d.ts` contains no Vite triple-slash reference

The file only declares the CSS Module type:

```ts
declare module "*.module.css" {
  const classes: Record<string, string>;
  export default classes;
}
```

The `/// <reference types="vite/client" />` directive was removed so the package doesn't
force consumers to install Vite as a dependency.

### 3. `"src"` is excluded from `files` in `package.json`

Only `dist/` is published. Shipping `src/` would expose the Vite-specific declarations
above to consumers who resolve source files instead of dist.

### 4. `sideEffects: ["./dist/style.css"]`

Marks the CSS as a side-effect so webpack and Rollup don't tree-shake it away even though
the JS entry point no longer imports it.

### 5. Token refresh outside React context

`tokenRefresh.ts` uses a module-level `_client` variable set by `registerTokenRefreshClient(client)`.
`SentinelAuthProvider` calls this automatically in a `useEffect`, so external callers
(Axios interceptors, mutation error handlers, `createSentinelQueryClient`) can call
`refreshTokens()` without access to React context.

Concurrent callers share one `refreshPromise` singleton — the dedup prevents multiple
parallel refreshes when several queries fail with 401 simultaneously.

### 6. Vite `resolve.preserveSymlinks: true`

`apps/sentinel-ui/vite.config.ts` sets `resolve.preserveSymlinks: true`.

Without it, Vite follows the `node_modules/@sentinel/auth-react` symlink to the real
path `/packages/sentinel-auth-react`, then tries to resolve peer deps (`zustand`,
`@tanstack/react-query`, etc.) from there — where they are not installed. With
`preserveSymlinks: true`, resolution stays at the symlink location and finds the
peer deps in `sentinel-ui/node_modules/`.

### 7. Dockerfile build stage

`apps/sentinel-ui/Dockerfile.dev` has an `auth-react-builder` stage that runs
`npm run build` inside the package before the UI container starts. This ensures
`dist/` exists at the real path `/packages/sentinel-auth-react/dist/` inside the Docker
image, which is what Vite actually reads after following the symlink.

---

## Public API Surface

Everything exported from `src/index.ts`:

### Provider + context

```ts
SentinelAuthProvider  // <SentinelAuthProvider client={…} redirects={…} theme={…}>
SentinelAuthContext   // React context object (use useSentinelAuth instead)
useSentinelAuth()     // returns { client, redirects, theme }
useSentinelConfig()   // alias for useSentinelAuth()
```

### Auth store (Zustand)

```ts
useAuthStore()
// → { userId, accessToken, refreshToken, isAuthenticated,
//     emailVerified, isAdmin, mustChangePassword, mfaSetupRequired,
//     userEmail, firstName, lastName }
```

State is persisted to `localStorage` under the key `"sentinel-auth"`.

### Hooks

```ts
useAuth()
// → { login(creds), verifyMfa({ mfa_session_token, code }), logout(userId), isLoading, error }
```

### Token refresh helpers

```ts
registerTokenRefreshClient(client)  // called automatically by SentinelAuthProvider
refreshTokens()                     // → Promise<boolean>; false = session unrestorable
```

### QueryClient factory

```ts
createSentinelQueryClient(redirects?)
// Preconfigured TanStack QueryClient with:
//   401  → refresh attempt; on failure clear session + redirect to afterLogout
//   EmailNotVerifiedError → redirect to verifyEmail
//   403 MUST_CHANGE_PASSWORD → redirect to changePassword
//   403 other → redirect to unauthorized
```

### Route guards

```ts
ProtectedRoute    // Outlet wrapper — redirects to /login when unauthenticated
PublicRoute       // Outlet wrapper — redirects to /dashboard when already authenticated
AuthorizedRoute   // Outlet wrapper — calls checkAuthorization(); redirects to /unauthorized on 403
```

### Routes bundle

```ts
SentinelAuthRoutes  // <Routes> covering all auth paths; mount under a catch-all route
```

### Standalone page components

```ts
LoginPage, RegisterPage, ForgotPasswordPage, ResetPasswordPage,
VerifyEmailPage, ChangePasswordForcedPage, SetupMfaForcedPage, UnauthorizedPage
```

### UI primitives

```ts
Button  // shared button with variants: "primary" | "secondary" | "danger"
```

### Types

```ts
SentinelAuthContextValue
SentinelAuthRedirects
SentinelTheme
```

---

## Common Gotchas

### `tsc --emitDeclarationOnly` fails on CSS module imports

Symptom: `TS2307: Cannot find module '*.module.css'`

Cause: The project-level `tsconfig.json` doesn't include `src/vite-env.d.ts` in its
compilation scope, or the CSS declaration was accidentally removed.

Fix: Ensure `src/vite-env.d.ts` exists with:
```ts
declare module "*.module.css" {
  const classes: Record<string, string>;
  export default classes;
}
```

### CSS output file name

`vite.config.ts` sets `build.lib.cssFileName: "style"` so the output is `dist/style.css`.
Without this, Vite derives the CSS file name from the library name and would output
`dist/auth-react.css`, breaking the consumer's `import '@sentinel/auth-react/dist/style.css'`.

### `types` must come before `import`/`require` in `package.json` exports

```json
"exports": {
  ".": {
    "types":   "./dist/index.d.ts",   // ← must be first
    "import":  "./dist/index.js",
    "require": "./dist/index.cjs"
  }
}
```

esbuild / TypeScript resolvers process conditions in order — `types` must appear before the
runtime conditions or TypeScript may pick the wrong resolution path.

### `--legacy-peer-deps` requirement

The build toolchain (React 19, react-router-dom v7, zustand v5) uses version numbers
higher than some peer dep `>=N` declarations expect. This does not affect runtime
compatibility. Always use `npm install --legacy-peer-deps` in this directory.

### Pages must be rendered inside `<SentinelAuthProvider>`

Every page component calls `useSentinelAuth()` which throws if called outside the provider.
Wrap all auth page usage in `<SentinelAuthProvider>` in the consuming app.
