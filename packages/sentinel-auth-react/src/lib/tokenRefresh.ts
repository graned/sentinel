import type { SentinelAuthClient } from "@sentinel/auth-sdk";
import { useAuthStore } from "../store/authStore";

/**
 * Module-level client reference set by SentinelAuthProvider.
 * Allows refreshTokens() to be called outside of React context
 * (e.g. from a QueryCache.onError handler or createSentinelQueryClient).
 */
let _client: SentinelAuthClient | null = null;

export function registerTokenRefreshClient(client: SentinelAuthClient): void {
  _client = client;
}

let refreshPromise: Promise<boolean> | null = null;

/**
 * Deduped token refresh — concurrent callers share the same in-flight promise.
 * Returns true if the refresh succeeded, false otherwise.
 */
export async function refreshTokens(): Promise<boolean> {
  if (!_client) return false;
  if (refreshPromise) return refreshPromise;

  const p = (async () => {
    try {
      const { refreshToken, emailVerified, mustChangePassword } = useAuthStore.getState();
      const session = await _client!.refreshSession(refreshToken!);
      useAuthStore.getState().setSession(
        session.userId,
        session.accessToken,
        session.refreshToken,
        emailVerified,
        mustChangePassword,
      );
      return true;
    } catch {
      return false;
    }
  })();

  refreshPromise = p;
  // Clear the promise reference AFTER all callers have received the result —
  // using void+finally so the null assignment runs after the microtask queue.
  void p.finally(() => {
    refreshPromise = null;
  });

  return p;
}
