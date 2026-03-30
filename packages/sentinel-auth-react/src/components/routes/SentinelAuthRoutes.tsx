/**
 * Drop-in `<Routes>` block that mounts all Sentinel auth pages.
 *
 * Mount this as a catch-all route in your app's router:
 *
 * ```tsx
 * <Route path="/*" element={<SentinelAuthRoutes />} />
 * ```
 *
 * # Route breakdown
 *
 * | Guard | Paths |
 * |-------|-------|
 * | `PublicRoute` (redirects authenticated users away) | `/login`, `/register` |
 * | Open (no guard) | `/verify-email`, `/forgot-password`, `/reset-password` |
 * | `ProtectedRoute` (redirects unauthenticated users to login) | `/change-password`, `/setup-mfa`, `/unauthorized` |
 *
 * All paths are configurable via `SentinelAuthProvider`'s `redirects` prop.
 */
import { Routes, Route } from "react-router-dom";
import { useSentinelAuth } from "../../context/SentinelAuthContext";
import { ProtectedRoute, PublicRoute } from "../guards/ProtectedRoute";
import { LoginPage } from "../pages/LoginPage";
import { RegisterPage } from "../pages/RegisterPage";
import { VerifyEmailPage } from "../pages/VerifyEmailPage";
import { ForgotPasswordPage } from "../pages/ForgotPasswordPage";
import { ResetPasswordPage } from "../pages/ResetPasswordPage";
import { ChangePasswordForcedPage } from "../pages/ChangePasswordForcedPage";
import { SetupMfaForcedPage } from "../pages/SetupMfaForcedPage";
import { UnauthorizedPage } from "../pages/UnauthorizedPage";

export function SentinelAuthRoutes() {
  const { redirects } = useSentinelAuth();

  const loginPath = redirects.login ?? "/login";
  const registerPath = redirects.register ?? "/register";
  const verifyEmailPath = redirects.verifyEmail ?? "/verify-email";
  const forgotPasswordPath = redirects.forgotPassword ?? "/forgot-password";
  const resetPasswordPath = "/reset-password";
  const changePasswordPath = redirects.changePassword ?? "/change-password";
  const setupMfaPath = redirects.setupMfa ?? "/setup-mfa";
  const unauthorizedPath = redirects.unauthorized ?? "/unauthorized";

  return (
    <Routes>
      {/* Public-only: redirect to afterLogin when already authenticated */}
      <Route element={<PublicRoute />}>
        <Route path={loginPath} element={<LoginPage />} />
        <Route path={registerPath} element={<RegisterPage />} />
      </Route>

      {/* Semi-public: accessible to all */}
      <Route path={verifyEmailPath} element={<VerifyEmailPage />} />
      <Route path={forgotPasswordPath} element={<ForgotPasswordPage />} />
      <Route path={resetPasswordPath} element={<ResetPasswordPage />} />

      {/* Protected: redirect to login when not authenticated */}
      <Route element={<ProtectedRoute />}>
        <Route path={changePasswordPath} element={<ChangePasswordForcedPage />} />
        <Route path={setupMfaPath} element={<SetupMfaForcedPage />} />
        <Route path={unauthorizedPath} element={<UnauthorizedPage />} />
      </Route>
    </Routes>
  );
}
