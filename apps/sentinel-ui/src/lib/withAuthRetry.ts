import { SentinelError } from "@sentinel/auth-sdk";
import { refreshTokens, useAuthStore } from "@sentinel/auth-react";

/**
 * Wraps an async API call so that a single 401 triggers a token refresh and
 * one automatic retry. Because getToken() is re-evaluated on every call in
 * admin.ts, the retry automatically picks up the freshly stored token.
 *
 * If the refresh itself fails the session is cleared and the user is sent to
 * the login page, matching the behaviour of QueryCache.onError for queries.
 */
export async function withAuthRetry<T>(fn: () => Promise<T>): Promise<T> {
  try {
    return await fn();
  } catch (err) {
    if (err instanceof SentinelError && err.statusCode === 401) {
      const refreshed = await refreshTokens();
      if (refreshed) {
        return fn();
      }
      useAuthStore.getState().clearTokens();
      window.location.href = "/login";
    }
    throw err;
  }
}
