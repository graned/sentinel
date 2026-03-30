/**
 * Zustand auth store for the Sentinel Auth React package.
 *
 * Persists to `localStorage` under the key `"sentinel-auth"` so sessions
 * survive page reloads.
 *
 * # Usage
 *
 * ```tsx
 * const { isAuthenticated, accessToken, userId } = useAuthStore();
 * ```
 *
 * # State shape
 *
 * | Field | Description |
 * |-------|-------------|
 * | `userId` | UUID of the authenticated user, or `null` |
 * | `accessToken` | Current PASETO access token, or `null` |
 * | `refreshToken` | Current opaque refresh token, or `null` |
 * | `isAuthenticated` | True when a valid session is present |
 * | `emailVerified` | Baked-in from the login token — re-login required after verification |
 * | `isAdmin` | True when the user has the `admin` role |
 * | `mustChangePassword` | True when the server requires a password change before other access |
 * | `mfaSetupRequired` | True when the admin has mandated MFA but the user hasn't enrolled yet |
 * | `userEmail` | User's email address (populated by `setUserProfile`) |
 * | `firstName` / `lastName` | Display name fields |
 */
import { create } from "zustand";
import { persist } from "zustand/middleware";

interface AuthState {
  userId: string | null;
  accessToken: string | null;
  refreshToken: string | null;
  isAuthenticated: boolean;
  /** Whether the user's email has been verified (baked into the token at login). */
  emailVerified: boolean;
  isAdmin: boolean;
  /** When true, the user must change their password before accessing other resources. */
  mustChangePassword: boolean;
  /** When true, the admin requires MFA but the user hasn't enrolled an authenticator yet. */
  mfaSetupRequired: boolean;
  userEmail: string | null;
  firstName: string | null;
  lastName: string | null;
  setSession: (
    userId: string,
    access: string,
    refresh: string,
    emailVerified: boolean,
    mustChangePassword?: boolean,
  ) => void;
  setIsAdmin: (isAdmin: boolean) => void;
  setUserProfile: (email: string, firstName: string | null, lastName: string | null) => void;
  clearMustChangePassword: () => void;
  setMfaSetupRequired: (val: boolean) => void;
  clearMfaSetupRequired: () => void;
  clearTokens: () => void;
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set) => ({
      userId: null,
      accessToken: null,
      refreshToken: null,
      isAuthenticated: false,
      emailVerified: false,
      isAdmin: false,
      mustChangePassword: false,
      mfaSetupRequired: false,
      userEmail: null,
      firstName: null,
      lastName: null,
      setSession: (userId, access, refresh, emailVerified, mustChangePassword = false) =>
        set({
          userId,
          accessToken: access,
          refreshToken: refresh,
          isAuthenticated: true,
          emailVerified,
          mustChangePassword,
        }),
      setIsAdmin: (isAdmin) => set({ isAdmin }),
      setUserProfile: (email, firstName, lastName) => set({ userEmail: email, firstName, lastName }),
      clearMustChangePassword: () => set({ mustChangePassword: false }),
      setMfaSetupRequired: (val) => set({ mfaSetupRequired: val }),
      clearMfaSetupRequired: () => set({ mfaSetupRequired: false }),
      clearTokens: () =>
        set({
          userId: null,
          accessToken: null,
          refreshToken: null,
          isAuthenticated: false,
          emailVerified: false,
          isAdmin: false,
          mustChangePassword: false,
          mfaSetupRequired: false,
          userEmail: null,
          firstName: null,
          lastName: null,
        }),
    }),
    { name: "sentinel-auth" },
  ),
);
