/**
 * Core authentication hook for Sentinel apps.
 *
 * Wraps `SentinelAuthClient` login/logout/MFA flows and keeps the Zustand
 * `authStore` in sync.  Must be used inside `<SentinelAuthProvider>`.
 *
 * # Returned values
 *
 * | Name | Description |
 * |------|-------------|
 * | `login(creds)` | Submit email + password; returns a tagged union: `{ mfa: false }` for direct session, `{ mfa: true, mfaToken }` for MFA challenge |
 * | `verifyMfa(token, code)` | Submit TOTP code against an MFA session token |
 * | `logout()` | Revoke the server session, clear local state, redirect to `afterLogout` |
 * | `isLoading` | True while any async operation is in flight |
 * | `error` | Last error message, or `null` |
 *
 * # Admin detection
 *
 * After a successful direct login or MFA verification, `detectAndSetAdmin` fetches
 * `/v1/api/user/permissions` and sets `isAdmin = true` in the store if the user has
 * the `admin` role.  Errors are swallowed — `isAdmin` defaults to `false`.
 */
import { useState, useCallback } from "react";
import type { LoginRequest } from "@sentinel/auth-sdk";
import { useSentinelAuth } from "../context/SentinelAuthContext";
import { useAuthStore } from "../store/authStore";

/**
 * Fetch the user's permissions and set `isAdmin` in the store.
 * Errors are non-fatal — the user simply remains non-admin.
 */
async function detectAndSetAdmin(client: ReturnType<typeof useSentinelAuth>["client"], accessToken: string) {
  try {
    const perms = await client.user.getPermissions(accessToken);
    const isAdmin = perms.roles.some((r) => r.role_type === "admin");
    useAuthStore.getState().setIsAdmin(isAdmin);
  } catch {
    // Non-fatal — leave isAdmin as false
  }
}

export function useAuth() {
  const { client, redirects } = useSentinelAuth();
  const { isAuthenticated, setSession, clearTokens, setUserProfile, setIsAdmin, setMfaSetupRequired } =
    useAuthStore();
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const login = useCallback(
    async (data: LoginRequest) => {
      setIsLoading(true);
      setError(null);
      try {
        const result = await client.login(data);
        if (result.type === "session") {
          const profile = await client.user.getMe(result.session.accessToken);
          setUserProfile(profile.email, profile.first_name, profile.last_name);
          if (!profile.email_verified) {
            setSession(
              result.session.userId,
              result.session.accessToken,
              result.session.refreshToken,
              false,
            );
            return { success: true as const, mfa: false as const, emailUnverified: true, email: data.email };
          }
          setSession(
            result.session.userId,
            result.session.accessToken,
            result.session.refreshToken,
            true,
            result.mustChangePassword,
          );
          if (result.mustChangePassword) {
            return { success: true as const, mfa: false as const, mustChangePassword: true };
          }
          if (result.mfaSetupRequired) {
            setMfaSetupRequired(true);
            return { success: true as const, mfa: false as const, mfaSetupRequired: true };
          }
          await detectAndSetAdmin(client, result.session.accessToken);
          return { success: true as const, mfa: false as const };
        }
        return {
          success: true as const,
          mfa: true as const,
          mfaToken: result.mfaSessionToken,
        };
      } catch (err: unknown) {
        const msg = (err as { message?: string })?.message ?? "Login failed";
        setError(msg);
        return { success: false as const, mfa: false as const };
      } finally {
        setIsLoading(false);
      }
    },
    [client, setSession, setUserProfile, setIsAdmin, setMfaSetupRequired],
  );

  const verifyMfa = useCallback(
    async (mfaSessionToken: string, code: string) => {
      setIsLoading(true);
      setError(null);
      try {
        const session = await client.mfa.verify({ mfa_session_token: mfaSessionToken, code });
        const profile = await client.user.getMe(session.accessToken);
        setUserProfile(profile.email, profile.first_name, profile.last_name);
        setSession(session.userId, session.accessToken, session.refreshToken, true);
        await detectAndSetAdmin(client, session.accessToken);
        return { success: true as const };
      } catch (err: unknown) {
        const msg = (err as { message?: string })?.message ?? "Verification failed";
        setError(msg);
        return { success: false as const };
      } finally {
        setIsLoading(false);
      }
    },
    [client, setSession, setUserProfile, setIsAdmin],
  );

  const logout = useCallback(async () => {
    const { userId } = useAuthStore.getState();
    try {
      if (userId) await client.logout(userId);
    } finally {
      clearTokens();
      window.location.href = redirects.afterLogout ?? "/login";
    }
  }, [client, clearTokens, redirects.afterLogout]);

  return { isAuthenticated, isLoading, error, login, verifyMfa, logout };
}
