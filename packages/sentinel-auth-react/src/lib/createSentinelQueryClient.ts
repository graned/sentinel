/**
 * Factory that creates a pre-configured TanStack QueryClient for Sentinel apps.
 *
 * The QueryCache `onError` handler provides automatic session recovery:
 *
 * | Error | Action |
 * |-------|--------|
 * | `401` | Attempt token refresh; on success invalidate all queries; on failure clear session + redirect to login |
 * | `EmailNotVerifiedError` | Redirect to `verifyEmail` path |
 * | `403 MUST_CHANGE_PASSWORD` | Redirect to `changePassword` path |
 * | `403` (other) | Redirect to `unauthorized` path |
 *
 * The `retry` option is set to never retry 401 errors — the query function already
 * attempted one refresh-and-retry internally, so a second attempt would cascade.
 *
 * @param redirects - Optional redirect path overrides.  Defaults match the standard
 *   Sentinel auth route paths (`/login`, `/verify-email`, `/change-password`, `/unauthorized`).
 */
import { QueryClient, QueryCache } from "@tanstack/react-query";
import { SentinelError, EmailNotVerifiedError } from "@sentinel/auth-sdk";
import { useAuthStore } from "../store/authStore";
import { refreshTokens } from "./tokenRefresh";
import type { SentinelAuthRedirects } from "../types";

export function createSentinelQueryClient(redirects?: SentinelAuthRedirects): QueryClient {
  const loginPath = redirects?.afterLogout ?? redirects?.login ?? "/login";
  const verifyEmailPath = redirects?.verifyEmail ?? "/verify-email";
  const changePasswordPath = redirects?.changePassword ?? "/change-password";
  const unauthorizedPath = redirects?.unauthorized ?? "/unauthorized";

  const client = new QueryClient({
    defaultOptions: {
      queries: {
        retry: (failureCount, error) => {
          // Never retry 401s — withAuthRetry already attempted one refresh+retry
          // inside the queryFn. Retrying here would re-run with an empty token
          // (after clearTokens()) and produce a cascade of MISSING_TOKEN errors.
          if (error instanceof SentinelError && error.statusCode === 401) return false;
          return failureCount < 1;
        },
        staleTime: 30_000,
      },
    },
    queryCache: new QueryCache({
      onError: (err) => {
        if (!(err instanceof SentinelError)) return;

        if (err.statusCode === 401) {
          void (async () => {
            const refreshed = await refreshTokens();
            if (refreshed) {
              void client.invalidateQueries();
            } else {
              useAuthStore.getState().clearTokens();
              window.location.href = loginPath;
            }
          })();
        } else if (err instanceof EmailNotVerifiedError) {
          window.location.href = verifyEmailPath;
        } else if (err.statusCode === 403 && err.code === "MUST_CHANGE_PASSWORD") {
          window.location.href = changePasswordPath;
        } else if (err.statusCode === 403) {
          window.location.href = unauthorizedPath;
        }
      },
    }),
  });
  return client;
}
