/**
 * Shared type definitions for `@sentinel/auth-react`.
 *
 * These interfaces are exported as part of the public API so consuming apps can
 * type-check their `<SentinelAuthProvider>` props without importing from the SDK.
 */
import type { ReactNode } from "react";
import type { SentinelAuthClient } from "@sentinel/auth-sdk";

export interface SentinelAuthRedirects {
  /** Path to navigate to after successful login. Default: "/dashboard" */
  afterLogin?: string;
  /** Path to navigate to after logout. Default: "/login" */
  afterLogout?: string;
  /** Path to navigate to after successful registration. Default: "/verify-email" */
  afterRegister?: string;
  /** Path to the forgot-password page. Default: "/forgot-password" */
  forgotPassword?: string;
  /** Path to the verify-email page. Default: "/verify-email" */
  verifyEmail?: string;
  /** Path to the forced password-change page. Default: "/change-password" */
  changePassword?: string;
  /** Path to the forced MFA-setup page. Default: "/setup-mfa" */
  setupMfa?: string;
  /** Path to the unauthorized page. Default: "/unauthorized" */
  unauthorized?: string;
  /** Path to the login page (used by route guards). Default: "/login" */
  login?: string;
  /** Path to the register page. Default: "/register" */
  register?: string;
}

export interface SentinelTheme {
  /** Primary accent colour (cyan). Overrides --accent-primary CSS var. */
  primaryColor?: string;
  /** Secondary accent colour (blue). Overrides --accent-blue CSS var. */
  secondaryColor?: string;
  /** App name displayed in the branding panel wordmark. Default: "Sentinel" */
  appName?: string;
  /** Auth-half tagline. Only applies to Login / Register panels. */
  tagline?: string;
  /** Copyright text in page footer. Default: "© 2026 Sentinel Auth. All rights reserved." */
  copyright?: string;
  /**
   * Custom logo rendered in the brand panel on all auth pages, replacing the
   * default animated Sentinel shield.
   *
   * - Pass a URL string to render an `<img>` (recommended size: 80–120 px).
   * - Pass a ReactNode to render arbitrary JSX (e.g. an inline SVG component).
   */
  logo?: string | ReactNode;
}

export interface SentinelAuthContextValue {
  /** Configured SentinelAuthClient instance. */
  client: SentinelAuthClient;
  /** Resolved redirect paths (may be empty object — use defaults in pages). */
  redirects: SentinelAuthRedirects;
  /** Theme overrides. */
  theme: SentinelTheme;
}
