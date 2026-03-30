/**
 * 403 Unauthorized landing page.
 * Shown by `AuthorizedRoute` when the policy engine denies access.
 * Provides a "Go back" button and a logout option.
 */
import { useCallback } from "react";
import { useSentinelAuth } from "../../context/SentinelAuthContext";
import { useAuthStore } from "../../store/authStore";
import { Button } from "../ui/Button";
import { BrandPanel } from "../ui/BrandPanel";
import styles from "./UnauthorizedPage.module.css";

function LockIcon() {
  return (
    <svg
      className={styles.lockSvg}
      viewBox="0 0 120 140"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      aria-hidden="true"
    >
      <defs>
        <linearGradient id="unauthShieldGrad" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stopColor="#f87171" />
          <stop offset="100%" stopColor="#ef4444" />
        </linearGradient>
        <linearGradient id="unauthShieldInner" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stopColor="rgba(248,113,113,0.15)" />
          <stop offset="100%" stopColor="rgba(239,68,68,0.08)" />
        </linearGradient>
      </defs>
      <path
        d="M60 4L8 26v42c0 31.4 22.1 60.8 52 68 29.9-7.2 52-36.6 52-68V26L60 4z"
        fill="url(#unauthShieldInner)"
        stroke="url(#unauthShieldGrad)"
        strokeWidth="2"
      />
      <path
        d="M60 18L22 36v32c0 22.8 16.2 44.1 38 49.4C81.8 112.1 98 90.8 98 68V36L60 18z"
        fill="url(#unauthShieldInner)"
        stroke="url(#unauthShieldGrad)"
        strokeWidth="1"
        strokeOpacity="0.5"
      />
      <rect x="46" y="66" width="28" height="22" rx="4" fill="url(#unauthShieldGrad)" />
      <path
        d="M50 66v-6a10 10 0 0120 0v6"
        stroke="url(#unauthShieldGrad)"
        strokeWidth="3.5"
        strokeLinecap="round"
        fill="none"
      />
      <line x1="53" y1="72" x2="67" y2="82" stroke="#070d1a" strokeWidth="2.5" strokeLinecap="round" />
      <line x1="67" y1="72" x2="53" y2="82" stroke="#070d1a" strokeWidth="2.5" strokeLinecap="round" />
    </svg>
  );
}

export function UnauthorizedPage() {
  const { client, redirects, theme } = useSentinelAuth();
  const { userId, clearTokens } = useAuthStore();

  const afterLogoutPath = redirects.afterLogout ?? "/login";
  const copyright = theme.copyright ?? "© 2026 Sentinel Auth. All rights reserved.";

  const handleSignOut = useCallback(async () => {
    try {
      if (userId) await client.logout(userId);
    } finally {
      clearTokens();
      window.location.href = afterLogoutPath;
    }
  }, [userId, clearTokens, client, afterLogoutPath]);

  return (
    <div className={styles.page}>
      <BrandPanel
        tagline="Access control"
        taglineSubtext="Your identity is verified, but access was denied."
        defaultIcon={<LockIcon />}
        showOrbits={false}
      />

      <div className={styles.formPanel}>
        <div className={styles.topControls} aria-hidden="true">
          <span className={styles.topControlBtn}>
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
              <circle cx="12" cy="12" r="10" />
              <line x1="2" y1="12" x2="22" y2="12" />
              <path d="M12 2a15.3 15.3 0 010 20M12 2a15.3 15.3 0 000 20" />
            </svg>
          </span>
          <span className={styles.topControlBtn}>
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
              <line x1="3" y1="6" x2="21" y2="6" />
              <line x1="3" y1="12" x2="21" y2="12" />
              <line x1="3" y1="18" x2="21" y2="18" />
            </svg>
          </span>
          <span className={styles.topControlChevron}>
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <polyline points="6 9 12 15 18 9" />
            </svg>
          </span>
        </div>

        <div className={styles.formCard}>
          <div className={styles.statusCenter}>
            <div className={styles.errorIcon} aria-hidden="true">
              <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
                <circle cx="12" cy="12" r="10" />
                <line x1="15" y1="9" x2="9" y2="15" />
                <line x1="9" y1="9" x2="15" y2="15" />
              </svg>
            </div>
            <h1 className={styles.formTitle}>Access denied</h1>
            <p className={styles.formSubtitle}>
              Your account doesn&apos;t have permission to access this area. Contact your administrator if you believe this is a mistake.
            </p>
            <Button className={styles.actionBtn} onClick={handleSignOut}>
              Sign out
            </Button>
          </div>
        </div>

        <p className={styles.copyright}>{copyright}</p>
      </div>
    </div>
  );
}
