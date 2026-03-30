/**
 * Password-reset form.  Reads the `?token=<pr_*>` query param from the reset link
 * and submits it along with a new password to `POST /auth/password/reset`.
 * On success, all sessions are revoked server-side and the user is redirected to login.
 */
import { useState } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { SentinelError } from "@sentinel/auth-sdk";
import { useSentinelAuth } from "../../context/SentinelAuthContext";
import { Button } from "../ui/Button";
import { BrandPanel } from "../ui/BrandPanel";
import styles from "./ResetPasswordPage.module.css";

const RULES = [
  { label: "At least 12 characters", test: (p: string) => p.length >= 12 },
  { label: "One uppercase letter",   test: (p: string) => /[A-Z]/.test(p) },
  { label: "One lowercase letter",   test: (p: string) => /[a-z]/.test(p) },
  { label: "One number",             test: (p: string) => /[0-9]/.test(p) },
  { label: "One special character",  test: (p: string) => /[^A-Za-z0-9]/.test(p) },
];

function PasswordChecklist({ password }: { password: string }) {
  return (
    <ul className={styles.checklist} aria-label="Password requirements">
      {RULES.map(({ label, test }) => {
        const met = test(password);
        return (
          <li key={label} className={styles.checkItem}>
            <svg
              className={`${styles.checkIcon}${met ? ` ${styles.met}` : ""}`}
              width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor"
              strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"
            >
              {met ? <polyline points="20 6 9 17 4 12" /> : <circle cx="12" cy="12" r="9" />}
            </svg>
            <span className={`${styles.checkLabel}${met ? ` ${styles.met}` : ""}`}>{label}</span>
          </li>
        );
      })}
    </ul>
  );
}

function validatePassword(password: string): string | null {
  for (const { label, test } of RULES) {
    if (!test(password)) return `Password must include: ${label.toLowerCase()}.`;
  }
  return null;
}

type Mode = "form" | "success" | "error" | "no-token";

export function ResetPasswordPage() {
  const { client, redirects, theme } = useSentinelAuth();
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const token = searchParams.get("token");

  const [mode, setMode] = useState<Mode>(token ? "form" : "no-token");
  const [newPassword, setNewPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [validationError, setValidationError] = useState("");
  const [apiError, setApiError] = useState("");
  const [isLoading, setIsLoading] = useState(false);

  const loginPath = redirects.login ?? "/login";
  const forgotPath = redirects.forgotPassword ?? "/forgot-password";
  const copyright = theme.copyright ?? "© 2026 Sentinel Auth. All rights reserved.";

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setValidationError("");
    setApiError("");

    const pwError = validatePassword(newPassword);
    if (pwError) { setValidationError(pwError); return; }
    if (newPassword !== confirmPassword) { setValidationError("Passwords do not match."); return; }

    setIsLoading(true);
    try {
      await client.resetPassword({ token: token!, new_password: newPassword });
      setMode("success");
    } catch (err: unknown) {
      const msg = err instanceof SentinelError
        ? err.message
        : "Failed to reset password. The link may have expired or already been used.";
      setApiError(msg);
      setMode("error");
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className={styles.page}>
      <BrandPanel tagline="Set a new password" taglineSubtext="Choose a strong password to secure your account." />

      <div className={styles.formPanel}>
        <div className={styles.topControls} aria-hidden="true">
          <span className={styles.topControlBtn}>
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="10" /><line x1="2" y1="12" x2="22" y2="12" /><path d="M12 2a15.3 15.3 0 010 20M12 2a15.3 15.3 0 000 20" /></svg>
          </span>
          <span className={styles.topControlBtn}>
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round"><line x1="3" y1="6" x2="21" y2="6" /><line x1="3" y1="12" x2="21" y2="12" /><line x1="3" y1="18" x2="21" y2="18" /></svg>
          </span>
          <span className={styles.topControlChevron}>
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polyline points="6 9 12 15 18 9" /></svg>
          </span>
        </div>

        <div className={styles.formCard}>
          {mode === "form" && (
            <>
              <div className={styles.formHeader}>
                <div className={styles.lockIcon} aria-hidden="true">
                  <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
                    <rect x="3" y="11" width="18" height="11" rx="2" /><path d="M7 11V7a5 5 0 0110 0v4" />
                  </svg>
                </div>
                <h1 className={styles.formTitle}>Reset your password</h1>
                <p className={styles.formSubtitle}>
                  Enter a new password for your account. All active sessions will be signed out.
                </p>
              </div>

              <form onSubmit={handleSubmit} className={styles.form}>
                <div>
                  <div className={styles.fieldWrap}>
                    <span className={styles.fieldIcon} aria-hidden="true">
                      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
                        <rect x="3" y="11" width="18" height="11" rx="2" /><path d="M7 11V7a5 5 0 0110 0v4" />
                      </svg>
                    </span>
                    <input className={styles.fieldInput} type="password" value={newPassword} onChange={(e) => setNewPassword(e.target.value)} required autoComplete="new-password" placeholder="New password" autoFocus />
                  </div>
                  <PasswordChecklist password={newPassword} />
                </div>

                <div className={styles.fieldWrap}>
                  <span className={styles.fieldIcon} aria-hidden="true">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
                      <rect x="3" y="11" width="18" height="11" rx="2" /><path d="M7 11V7a5 5 0 0110 0v4" />
                    </svg>
                  </span>
                  <input className={styles.fieldInput} type="password" value={confirmPassword} onChange={(e) => setConfirmPassword(e.target.value)} required autoComplete="new-password" placeholder="Confirm new password" />
                </div>

                {validationError && <p className={styles.errorText}>{validationError}</p>}
                <Button type="submit" loading={isLoading} className={styles.submitBtn}>Reset password</Button>
              </form>
            </>
          )}

          {mode === "success" && (
            <div className={styles.statusCenter}>
              <div className={styles.successIcon} aria-hidden="true">
                <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
                  <circle cx="12" cy="12" r="10" /><polyline points="9 12 11 14 15 10" />
                </svg>
              </div>
              <h1 className={styles.formTitle}>Password reset!</h1>
              <p className={styles.formSubtitle}>Your password has been updated. All previous sessions have been signed out. Sign in with your new password.</p>
              <Button className={styles.actionBtn} onClick={() => navigate(loginPath)}>Sign in</Button>
            </div>
          )}

          {mode === "error" && (
            <div className={styles.statusCenter}>
              <div className={styles.errorIcon} aria-hidden="true">
                <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
                  <circle cx="12" cy="12" r="10" /><line x1="15" y1="9" x2="9" y2="15" /><line x1="9" y1="9" x2="15" y2="15" />
                </svg>
              </div>
              <h1 className={styles.formTitle}>Reset failed</h1>
              <p className={styles.errorTextBlock}>{apiError}</p>
              <Button className={styles.actionBtn} onClick={() => navigate(forgotPath)}>Request a new link</Button>
              <p className={styles.backLine}>
                <button type="button" className={styles.backLink} onClick={() => navigate(loginPath)}>Back to sign in</button>
              </p>
            </div>
          )}

          {mode === "no-token" && (
            <div className={styles.statusCenter}>
              <div className={styles.errorIcon} aria-hidden="true">
                <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
                  <circle cx="12" cy="12" r="10" /><line x1="15" y1="9" x2="9" y2="15" /><line x1="9" y1="9" x2="15" y2="15" />
                </svg>
              </div>
              <h1 className={styles.formTitle}>Invalid reset link</h1>
              <p className={styles.formSubtitle}>This link is invalid or has expired. Request a new password reset link from the login page.</p>
              <Button className={styles.actionBtn} onClick={() => navigate(forgotPath)}>Request a new link</Button>
              <p className={styles.backLine}>
                <button type="button" className={styles.backLink} onClick={() => navigate(loginPath)}>Back to sign in</button>
              </p>
            </div>
          )}
        </div>

        <p className={styles.copyright}>{copyright}</p>
      </div>
    </div>
  );
}
