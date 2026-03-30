import { type ReactNode } from "react";
import type { SentinelAuthClient } from "@sentinel/auth-sdk";
import type { SentinelAuthRedirects, SentinelTheme } from "../types";
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
export declare function SentinelAuthProvider({ client, redirects, theme, children, }: SentinelAuthProviderProps): import("react/jsx-runtime").JSX.Element;
//# sourceMappingURL=SentinelAuthProvider.d.ts.map