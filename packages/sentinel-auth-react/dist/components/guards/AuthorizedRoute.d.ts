/**
 * Route guard that checks whether the current session is authorised to access
 * admin routes by calling POST /v1/api/auth/token/authorize via the SDK's
 * authenticateAndAuthorize helper.
 *
 * - While the check is in-flight: renders nothing (prevents flash of protected content).
 * - Forbidden (ForbiddenError): redirects to the unauthorized page.
 * - Other errors (401, network): renders nothing and lets QueryCache.onError handle
 *   token refresh / redirect to login.
 * - Authorised: renders the nested routes via <Outlet />.
 */
export declare function AuthorizedRoute(): import("react/jsx-runtime").JSX.Element | null;
//# sourceMappingURL=AuthorizedRoute.d.ts.map