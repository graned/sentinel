/**
 * Full-page login form with inline MFA challenge screen.
 *
 * Flow:
 * 1. User submits email + password.
 * 2. If the server returns a session directly → redirect to `afterLogin`.
 * 3. If the server returns an MFA challenge → show TOTP / recovery-code input.
 * 4. User submits the code → session established → redirect to `afterLogin`.
 *
 * Special post-login redirects:
 * - `emailUnverified: true` → verify-email page
 * - `mustChangePassword: true` → change-password page
 * - `mfaSetupRequired: true` → setup-mfa page
 */
import { useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { useAuth } from "../../hooks/useAuth";
import { useSentinelAuth } from "../../context/SentinelAuthContext";
import { Button } from "../ui/Button";
import { BrandPanel } from "../ui/BrandPanel";
import styles from "./LoginPage.module.css";

export function LoginPage() {
  const { redirects, theme } = useSentinelAuth();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [mfaToken, setMfaToken] = useState<string | null>(null);
  const [mfaCode, setMfaCode] = useState("");
  const { login, verifyMfa, isLoading, error } = useAuth();
  const navigate = useNavigate();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const result = await login({ email, password });
    if (!result.success) return;
    if (result.mfa) {
      setMfaToken(result.mfaToken);
      return;
    }
    if ("emailUnverified" in result && result.emailUnverified) {
      navigate(redirects.verifyEmail ?? "/verify-email", { state: { email: result.email } });
      return;
    }
    if ("mustChangePassword" in result && result.mustChangePassword) {
      navigate(redirects.changePassword ?? "/change-password");
      return;
    }
    if ("mfaSetupRequired" in result && result.mfaSetupRequired) {
      navigate(redirects.setupMfa ?? "/setup-mfa");
      return;
    }
    navigate(redirects.afterLogin ?? "/dashboard");
  };

  const handleMfaSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const result = await verifyMfa(mfaToken!, mfaCode);
    if (result.success) navigate(redirects.afterLogin ?? "/dashboard");
  };

  const copyright = theme.copyright ?? "© 2026 Sentinel Auth. All rights reserved.";

  return (
    <div className={styles.page}>
      <BrandPanel tagline="Secure. Fast. Reliable." />

      {/* ── Right form panel ── */}
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
          {mfaToken ? (
            <>
              <div className={styles.formHeader}>
                <h1 className={styles.formTitle}>Two-factor authentication</h1>
                <p className={styles.formSubtitle}>Enter the 6-digit code from your authenticator app.</p>
              </div>

              <form onSubmit={handleMfaSubmit} className={styles.form}>
                <div className={styles.fieldWrap}>
                  <span className={styles.fieldIcon} aria-hidden="true">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
                      <rect x="3" y="11" width="18" height="11" rx="2" />
                      <path d="M7 11V7a5 5 0 0110 0v4" />
                    </svg>
                  </span>
                  <input
                    className={`${styles.fieldInput} ${styles.mfaInput}`}
                    type="text"
                    inputMode="numeric"
                    pattern="[0-9]{6}"
                    maxLength={6}
                    value={mfaCode}
                    onChange={(e) => setMfaCode(e.target.value.replace(/\D/g, ""))}
                    placeholder="000000"
                    required
                    autoFocus
                    autoComplete="one-time-code"
                  />
                </div>

                {error && <p className={styles.error}>{error}</p>}

                <Button type="submit" loading={isLoading} disabled={mfaCode.length !== 6} className={styles.submitBtn}>
                  Verify
                </Button>

                <button type="button" className={styles.backLink} onClick={() => { setMfaToken(null); setMfaCode(""); }}>
                  ← Back to sign in
                </button>
              </form>
            </>
          ) : (
            <>
              <div className={styles.formHeader}>
                <h1 className={styles.formTitle}>Sign in to your account</h1>
                <p className={styles.formSubtitle}>Welcome back! Please enter your credentials.</p>
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
                  />
                </div>

                <div className={styles.fieldWrap}>
                  <span className={styles.fieldIcon} aria-hidden="true">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
                      <rect x="3" y="11" width="18" height="11" rx="2" />
                      <path d="M7 11V7a5 5 0 0110 0v4" />
                    </svg>
                  </span>
                  <input
                    className={styles.fieldInput}
                    type="password"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    required
                    autoComplete="current-password"
                    placeholder="••••••••"
                  />
                </div>

                <div className={styles.forgotRow}>
                  <Link to={redirects.forgotPassword ?? "/forgot-password"} className={styles.forgotLink}>
                    Forgot your password?
                  </Link>
                </div>

                {error && <p className={styles.error}>{error}</p>}

                <Button type="submit" loading={isLoading} className={styles.submitBtn}>
                  Sign In
                </Button>
              </form>

              <p className={styles.signupLine}>
                Don&apos;t have an account?{" "}
                <Link to={redirects.register ?? "/register"} className={styles.signupLink}>Sign up</Link>
              </p>
            </>
          )}
        </div>

        <p className={styles.copyright}>{copyright}</p>
      </div>
    </div>
  );
}
