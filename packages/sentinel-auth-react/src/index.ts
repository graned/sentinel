// Types
export type { SentinelAuthContextValue, SentinelAuthRedirects, SentinelTheme } from "./types";

// Context
export { SentinelAuthContext, useSentinelAuth } from "./context/SentinelAuthContext";
export { SentinelAuthProvider } from "./context/SentinelAuthProvider";

// Store
export { useAuthStore } from "./store/authStore";

// Hooks
export { useSentinelAuth as useSentinelConfig } from "./context/SentinelAuthContext";
export { useAuth } from "./hooks/useAuth";

// Token refresh (for QueryCache integration)
export { refreshTokens, registerTokenRefreshClient } from "./lib/tokenRefresh";

// QueryClient factory
export { createSentinelQueryClient } from "./lib/createSentinelQueryClient";

// Route guards
export { ProtectedRoute, PublicRoute } from "./components/guards/ProtectedRoute";
export { AuthorizedRoute } from "./components/guards/AuthorizedRoute";

// Routes bundle
export { SentinelAuthRoutes } from "./components/routes/SentinelAuthRoutes";

// Standalone page components
export { LoginPage } from "./components/pages/LoginPage";
export { RegisterPage } from "./components/pages/RegisterPage";
export { ForgotPasswordPage } from "./components/pages/ForgotPasswordPage";
export { ResetPasswordPage } from "./components/pages/ResetPasswordPage";
export { VerifyEmailPage } from "./components/pages/VerifyEmailPage";
export { ChangePasswordForcedPage } from "./components/pages/ChangePasswordForcedPage";
export { SetupMfaForcedPage } from "./components/pages/SetupMfaForcedPage";
export { UnauthorizedPage } from "./components/pages/UnauthorizedPage";

// UI primitives
export { Button } from "./components/ui/Button";
export { BrandPanel } from "./components/ui/BrandPanel";
export type { BrandPanelProps } from "./components/ui/BrandPanel";
