/**
 * Wraps protected routes. Redirects unauthenticated users to the login page.
 * Redirects authenticated-but-unverified users to the verify-email page.
 * Redirects users with mustChangePassword=true to the change-password page.
 */
export declare function ProtectedRoute(): import("react/jsx-runtime").JSX.Element;
/**
 * Wraps public-only routes (e.g. /login, /register).
 * Redirects fully authenticated + verified users to the after-login path.
 * Redirects authenticated-but-unverified users to the verify-email page.
 */
export declare function PublicRoute(): import("react/jsx-runtime").JSX.Element;
//# sourceMappingURL=ProtectedRoute.d.ts.map