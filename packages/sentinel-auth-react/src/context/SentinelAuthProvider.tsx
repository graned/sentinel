import { useEffect, type ReactNode } from "react";
import type { SentinelAuthClient } from "@sentinel/auth-sdk";
import { SentinelAuthContext } from "./SentinelAuthContext";
import { registerTokenRefreshClient } from "../lib/tokenRefresh";
import type { SentinelAuthRedirects, SentinelAuthContextValue, SentinelTheme } from "../types";
import "../styles/defaults.css";

export interface SentinelAuthProviderProps {
  /** The configured SentinelAuthClient instance for this app. */
  client: SentinelAuthClient;
  /** Redirect path overrides. All paths default to their conventional values. */
  redirects?: SentinelAuthRedirects;
  /** Theme overrides (colours, branding text). */
  theme?: SentinelTheme;
  children: ReactNode;
}

/**
 * Root provider for @sentinel/auth-react. Place this near the top of your
 * component tree, wrapping your router and QueryClientProvider.
 *
 * @example
 * <SentinelAuthProvider client={sentinelClient} redirects={{ afterLogin: "/dashboard" }}>
 *   <BrowserRouter>
 *     <Routes>
 *       <Route path="/*" element={<SentinelAuthRoutes />} />
 *       ...
 *     </Routes>
 *   </BrowserRouter>
 * </SentinelAuthProvider>
 */
export function SentinelAuthProvider({
  client,
  redirects = {},
  theme = {},
  children,
}: SentinelAuthProviderProps) {
  // Register the client so the module-level refreshTokens() helper can use it
  // without React context (needed inside QueryCache.onError callbacks).
  useEffect(() => {
    registerTokenRefreshClient(client);
  }, [client]);

  const cssOverrides = buildThemeCss(theme);

  const value: SentinelAuthContextValue = { client, redirects, theme };

  return (
    <SentinelAuthContext.Provider value={value}>
      {cssOverrides ? <style>{`:root { ${cssOverrides} }`}</style> : null}
      {children}
    </SentinelAuthContext.Provider>
  );
}

function buildThemeCss(theme: SentinelTheme): string {
  const parts: string[] = [];
  if (theme.primaryColor) {
    parts.push(`--accent-primary: ${theme.primaryColor};`);
    parts.push(`--accent-primary-hover: ${theme.primaryColor};`);
    parts.push(`--border-active: ${theme.primaryColor};`);
  }
  if (theme.secondaryColor) {
    parts.push(`--accent-blue: ${theme.secondaryColor};`);
    parts.push(`--accent-blue-hover: ${theme.secondaryColor};`);
  }
  return parts.join(" ");
}
