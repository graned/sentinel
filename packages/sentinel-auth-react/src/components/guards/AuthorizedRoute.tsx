import { Navigate, Outlet } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { ForbiddenError } from "@sentinel/auth-sdk";
import { useSentinelAuth } from "../../context/SentinelAuthContext";
import { useAuthStore } from "../../store/authStore";

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
export function AuthorizedRoute() {
  const { client, redirects } = useSentinelAuth();
  const accessToken = useAuthStore((s) => s.accessToken) ?? "";
  const unauthorizedPath = redirects.unauthorized ?? "/unauthorized";

  const { data, isPending, error } = useQuery({
    queryKey: ["authz-check", accessToken],
    queryFn: () =>
      client.authenticateAndAuthorize({
        access_token: accessToken,
        method: "GET",
        path: "/v1/api/admin/roles",
      }),
    retry: false,
    staleTime: 5 * 60 * 1000,
    enabled: !!accessToken,
  });

  if (isPending) return null;
  if (error instanceof ForbiddenError) return <Navigate to={unauthorizedPath} replace />;
  if (error || !data) return null;

  return <Outlet />;
}
