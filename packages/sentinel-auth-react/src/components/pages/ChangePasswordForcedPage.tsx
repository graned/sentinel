/**
 * Forced password-change page.  Shown when `mustChangePassword` is `true` in the
 * auth store (the server set this flag on the user's account, e.g. after an admin
 * reset or first login with a temporary password).
 *
 * On success the `mustChangePassword` flag is cleared in the store and the user
 * is redirected to `afterLogin`.  All sessions are revoked server-side.
 */
import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { SentinelError } from "@sentinel/auth-sdk";
import { useSentinelAuth } from "../../context/SentinelAuthContext";
import { useAuthStore } from "../../store/authStore";
import { Button } from "../ui/Button";
import { BrandPanel } from "../ui/BrandPanel";
// Reuses the same layout/form styles as ResetPasswordPage
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

export function ChangePasswordForcedPage() {
  const { client, redirects, theme } = useSentinelAuth();
  const navigate = useNavigate();
  const { accessToken, clearMustChangePassword } = useAuthStore();

  const [currentPassword, setCurrentPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [validationError, setValidationError] = useState("");
  const [apiError, setApiError] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [success, setSuccess] = useState(false);

  const afterLoginPath = redirects.afterLogin ?? "/dashboard";
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
      await client.user.changePassword(accessToken!, {
        current_password: currentPassword,
        new_password: newPassword,
      });
      clearMustChangePassword();
      setSuccess(true);
    } catch (err: unknown) {
      const msg = err instanceof SentinelError
        ? err.message
        : "Failed to change password. Please check your current password and try again.";
      setApiError(msg);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className={styles.page}>
      <BrandPanel tagline="Set a new password" taglineSubtext="A new password is required to continue." />

      <div className={styles.formPanel}>
        <div className={styles.formCard}>
          {!success ? (
            <>
              <div className={styles.formHeader}>
                <div className={styles.lockIcon} aria-hidden="true">
                  <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
                    <rect x="3" y="11" width="18" height="11" rx="2" /><path d="M7 11V7a5 5 0 0110 0v4" />
                  </svg>
                </div>
                <h1 className={styles.formTitle}>Change your password</h1>
                <p className={styles.formSubtitle}>A temporary password was set for your account. Choose a new password to get started.</p>
              </div>

              <form onSubmit={handleSubmit} className={styles.form}>
                <div className={styles.fieldWrap}>
                  <span className={styles.fieldIcon} aria-hidden="true">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
                      <rect x="3" y="11" width="18" height="11" rx="2" /><path d="M7 11V7a5 5 0 0110 0v4" />
                    </svg>
                  </span>
                  <input className={styles.fieldInput} type="password" value={currentPassword} onChange={(e) => setCurrentPassword(e.target.value)} required autoComplete="current-password" placeholder="Temporary password" autoFocus />
                </div>

                <div>
                  <div className={styles.fieldWrap}>
                    <span className={styles.fieldIcon} aria-hidden="true">
                      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
                        <rect x="3" y="11" width="18" height="11" rx="2" /><path d="M7 11V7a5 5 0 0110 0v4" />
                      </svg>
                    </span>
                    <input className={styles.fieldInput} type="password" value={newPassword} onChange={(e) => setNewPassword(e.target.value)} required autoComplete="new-password" placeholder="New password" />
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
                {apiError && <p className={styles.errorText}>{apiError}</p>}

                <Button type="submit" loading={isLoading} className={styles.submitBtn}>Set new password</Button>
              </form>
            </>
          ) : (
            <div className={styles.statusCenter}>
              <div className={styles.successIcon} aria-hidden="true">
                <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
                  <circle cx="12" cy="12" r="10" /><polyline points="9 12 11 14 15 10" />
                </svg>
              </div>
              <h1 className={styles.formTitle}>Password updated!</h1>
              <p className={styles.formSubtitle}>Your password has been changed. All previous sessions have been signed out.</p>
              <Button className={styles.actionBtn} onClick={() => navigate(afterLoginPath)}>Go to dashboard</Button>
            </div>
          )}
        </div>

        <p className={styles.copyright}>{copyright}</p>
      </div>
    </div>
  );
}
