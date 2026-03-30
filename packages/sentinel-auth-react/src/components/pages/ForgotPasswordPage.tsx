/**
 * Forgot-password form.  Submits the user's email to `POST /auth/password/forgot`.
 * The server always returns 200 (anti-enumeration) so the page shows a success
 * message regardless of whether the email is registered.
 */
import { useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { useSentinelAuth } from "../../context/SentinelAuthContext";
import { Button } from "../ui/Button";
import { BrandPanel } from "../ui/BrandPanel";
import styles from "./ForgotPasswordPage.module.css";

type Mode = "form" | "sent";

export function ForgotPasswordPage() {
  const { client, redirects, theme } = useSentinelAuth();
  const navigate = useNavigate();
  const [mode, setMode] = useState<Mode>("form");
  const [email, setEmail] = useState("");
  const [isLoading, setIsLoading] = useState(false);

  const loginPath = redirects.login ?? "/login";
  const copyright = theme.copyright ?? "© 2026 Sentinel Auth. All rights reserved.";

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsLoading(true);
    try {
      await client.forgotPassword({ email });
    } catch {
      // Intentionally swallow errors — always show "sent" to prevent enumeration
    } finally {
      setIsLoading(false);
      setMode("sent");
    }
  };

  return (
    <div className={styles.page}>
      <BrandPanel tagline="Password recovery" taglineSubtext="We'll send you a secure reset link." />

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
          {mode === "form" && (
            <>
              <div className={styles.formHeader}>
                <div className={styles.lockIcon} aria-hidden="true">
                  <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
                    <rect x="3" y="11" width="18" height="11" rx="2" />
                    <path d="M7 11V7a5 5 0 0110 0v4" />
                  </svg>
                </div>
                <h1 className={styles.formTitle}>Forgot your password?</h1>
                <p className={styles.formSubtitle}>
                  Enter your email address and we'll send you a link to reset your password.
                </p>
              </div>

              <form onSubmit={handleSubmit} className={styles.form}>
                <div className={styles.fieldWrap}>
                  <span className={styles.fieldIcon} aria-hidden="true">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
                      <rect x="2" y="4" width="20" height="16" rx="2" />
                      <polyline points="2,4 12,13 22,4" />
                    </svg>
                  </span>
                  <input
                    className={styles.fieldInput}
                    type="email"
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    required
                    autoComplete="email"
                    placeholder="user@example.com"
                    autoFocus
                  />
                </div>

                <Button type="submit" loading={isLoading} className={styles.submitBtn}>
                  Send reset link
                </Button>
              </form>

              <p className={styles.backLine}>
                <button type="button" className={styles.backLink} onClick={() => navigate(loginPath)}>
                  Back to sign in
                </button>
              </p>
            </>
          )}

          {mode === "sent" && (
            <div className={styles.statusCenter}>
              <div className={styles.envelopeIcon} aria-hidden="true">
                <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
                  <rect x="2" y="4" width="20" height="16" rx="2" />
                  <polyline points="2,4 12,13 22,4" />
                </svg>
              </div>
              <h1 className={styles.formTitle}>Check your email</h1>
              <p className={styles.formSubtitle}>
                If an account exists for that email address, we've sent a password reset link. Check your inbox (and spam folder).
              </p>
              <Link to={loginPath} className={styles.actionBtnLink}>
                <Button className={styles.actionBtn}>Back to sign in</Button>
              </Link>
              <p className={styles.altAction}>
                <button type="button" className={styles.altLink} onClick={() => { setEmail(""); setMode("form"); }}>
                  Try a different email
                </button>
              </p>
            </div>
          )}
        </div>

        <p className={styles.copyright}>{copyright}</p>
      </div>
    </div>
  );
}
