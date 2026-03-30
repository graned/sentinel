import { Navigate, Outlet, useLocation } from "react-router-dom";
import { useAuthStore } from "../../store/authStore";
import { useSentinelAuth } from "../../context/SentinelAuthContext";

/**
 * Wraps protected routes. Redirects unauthenticated users to the login page.
 * Redirects authenticated-but-unverified users to the verify-email page.
 * Redirects users with mustChangePassword=true to the change-password page.
 */
export function ProtectedRoute() {
  const { redirects } = useSentinelAuth();
  const isAuthenticated = useAuthStore((s) => s.isAuthenticated);
  const emailVerified = useAuthStore((s) => s.emailVerified);
  const mustChangePassword = useAuthStore((s) => s.mustChangePassword);
  const { pathname } = useLocation();

  const loginPath = redirects.login ?? "/login";
  const verifyEmailPath = redirects.verifyEmail ?? "/verify-email";
  const changePasswordPath = redirects.changePassword ?? "/change-password";

  if (!isAuthenticated) return <Navigate to={loginPath} replace />;
  if (!emailVerified) return <Navigate to={verifyEmailPath} replace />;
  if (mustChangePassword && pathname !== changePasswordPath)
    return <Navigate to={changePasswordPath} replace />;
  return <Outlet />;
}

/**
 * Wraps public-only routes (e.g. /login, /register).
 * Redirects fully authenticated + verified users to the after-login path.
 * Redirects authenticated-but-unverified users to the verify-email page.
 */
export function PublicRoute() {
  const { redirects } = useSentinelAuth();
  const isAuthenticated = useAuthStore((s) => s.isAuthenticated);
  const emailVerified = useAuthStore((s) => s.emailVerified);
  const mustChangePassword = useAuthStore((s) => s.mustChangePassword);

  const afterLoginPath = redirects.afterLogin ?? "/dashboard";
  const verifyEmailPath = redirects.verifyEmail ?? "/verify-email";
  const changePasswordPath = redirects.changePassword ?? "/change-password";

  if (isAuthenticated && emailVerified && mustChangePassword) return <Navigate to={changePasswordPath} replace />;
  if (isAuthenticated && emailVerified) return <Navigate to={afterLoginPath} replace />;
  if (isAuthenticated && !emailVerified) return <Navigate to={verifyEmailPath} replace />;
  return <Outlet />;
}
