/**
 * Email verification page.
 *
 * Reads the `?token=<ev_*>` query param from the URL and submits it to the server.
 * On success:
 * - If the user is already authenticated (has an `accessToken` in the store), the
 *   `emailVerified` flag is updated in the store so the `ProtectedRoute` guard allows
 *   access immediately.
 * - Redirects to `afterLogin` (default `/dashboard`).
 *
 * Also handles the "resend verification email" flow when the token is missing or expired.
 */
import { useState, useEffect, useCallback, useRef } from "react";
import { Link, useNavigate, useLocation, useSearchParams } from "react-router-dom";
import { SentinelError } from "@sentinel/auth-sdk";
import { useSentinelAuth } from "../../context/SentinelAuthContext";
import { useAuthStore } from "../../store/authStore";
import { Button } from "../ui/Button";
import { BrandPanel } from "../ui/BrandPanel";
import styles from "./VerifyEmailPage.module.css";

const RESEND_COOLDOWN = 30;

function maskEmail(email: string): string {
  const [local, domain] = email.split("@");
  if (!domain) return email;
  if (local.length <= 2) return `${local[0]}*@${domain}`;
  const masked = local[0] + "*".repeat(local.length - 2) + local[local.length - 1];
  return `${masked}@${domain}`;
}

type Mode = "pending" | "verifying" | "success" | "error";

interface LocationState {
  email?: string;
  fromRegistration?: boolean;
}

export function VerifyEmailPage() {
  const { client, redirects, theme } = useSentinelAuth();
  const navigate = useNavigate();
  const location = useLocation();
  const [searchParams] = useSearchParams();
  const clearTokens = useAuthStore((s) => s.clearTokens);

  const state = (location.state ?? {}) as LocationState;
  const tokenFromUrl = searchParams.get("token");

  const [mode, setMode] = useState<Mode>(tokenFromUrl ? "verifying" : "pending");
  const verifyCalledRef = useRef(false);
  const [email, setEmail] = useState(state.email ?? "");
  const [emailInput, setEmailInput] = useState("");
  const [errorMsg, setErrorMsg] = useState("");
  const [resendMsg, setResendMsg] = useState("");
  const [countdown, setCountdown] = useState(RESEND_COOLDOWN);
  const [resending, setResending] = useState(false);

  const loginPath = redirects.login ?? "/login";
  const registerPath = redirects.register ?? "/register";
  const copyright = theme.copyright ?? "© 2026 Sentinel Auth. All rights reserved.";

  useEffect(() => {
    if (countdown <= 0) return;
    const id = setInterval(() => setCountdown((c) => Math.max(0, c - 1)), 1000);
    return () => clearInterval(id);
  }, [countdown]);

  useEffect(() => {
    if (!tokenFromUrl || verifyCalledRef.current) return;
    verifyCalledRef.current = true;
    client.verifyEmail(tokenFromUrl)
      .then(() => {
        clearTokens();
        setMode("success");
      })
      .catch((err: unknown) => {
        const msg = err instanceof SentinelError ? err.message : "Verification failed. The link may have expired or already been used.";
        setErrorMsg(msg);
        setMode("error");
      });
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [tokenFromUrl]);

  const handleResend = useCallback(async () => {
    const targetEmail = email || emailInput.trim();
    if (!targetEmail) return;
    setResending(true);
    setResendMsg("");
    try {
      await client.resendVerification({ email: targetEmail });
      if (!email) setEmail(emailInput.trim());
      setResendMsg("Verification email sent! Check your inbox.");
      setCountdown(RESEND_COOLDOWN);
    } catch (err: unknown) {
      const msg = err instanceof SentinelError ? err.message : "Failed to resend. Please try again.";
      setResendMsg(msg);
    } finally {
      setResending(false);
    }
  }, [client, email, emailInput]);

  const handleRetryPending = () => { setMode("pending"); setErrorMsg(""); };

  return (
    <div className={styles.page}>
      <BrandPanel tagline="Secure account verification" taglineSubtext="We've sent a confirmation link to your email." />

      <div className={styles.formPanel}>
        <div className={styles.topControls} aria-hidden="true">
          <span className={styles.topControlBtn}><svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="10" /><line x1="2" y1="12" x2="22" y2="12" /><path d="M12 2a15.3 15.3 0 010 20M12 2a15.3 15.3 0 000 20" /></svg></span>
          <span className={styles.topControlBtn}><svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round"><line x1="3" y1="6" x2="21" y2="6" /><line x1="3" y1="12" x2="21" y2="12" /><line x1="3" y1="18" x2="21" y2="18" /></svg></span>
          <span className={styles.topControlChevron}><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polyline points="6 9 12 15 18 9" /></svg></span>
        </div>

        <div className={styles.formCard}>
          {mode === "verifying" && (
            <div className={styles.statusCenter}>
              <div className={styles.spinner} aria-label="Verifying" />
              <h1 className={styles.formTitle}>Verifying your email…</h1>
              <p className={styles.formSubtitle}>Please wait a moment.</p>
            </div>
          )}

          {mode === "success" && (
            <div className={styles.statusCenter}>
              <div className={styles.successIcon} aria-hidden="true">
                <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
                  <circle cx="12" cy="12" r="10" /><polyline points="9 12 11 14 15 10" />
                </svg>
              </div>
              <h1 className={styles.formTitle}>Email verified!</h1>
              <p className={styles.formSubtitle}>Your email has been verified. Please sign in to continue.</p>
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
              <h1 className={styles.formTitle}>Verification failed</h1>
              <p className={styles.errorText}>{errorMsg}</p>
              <Button className={styles.actionBtn} onClick={handleRetryPending}>Request a new link</Button>
              <p className={styles.backLine}>
                <button type="button" className={styles.backLink} onClick={() => { clearTokens(); navigate(loginPath); }}>Back to sign in</button>
              </p>
            </div>
          )}

          {mode === "pending" && (
            <>
              {state.fromRegistration && (
                <div className={styles.successBanner} role="status">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
                    <circle cx="12" cy="12" r="10" /><polyline points="9 12 11 14 15 10" />
                  </svg>
                  Account created successfully
                </div>
              )}

              <div className={styles.formHeader}>
                <div className={styles.envelopeIcon} aria-hidden="true">
                  <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
                    <rect x="2" y="4" width="20" height="16" rx="2" /><polyline points="2,4 12,13 22,4" />
                  </svg>
                </div>
                <h1 className={styles.formTitle}>Verify your email</h1>
                {email ? (
                  <p className={styles.formSubtitle}>
                    We sent a verification link to{" "}
                    <strong className={styles.emailHighlight}>{maskEmail(email)}</strong>.{" "}
                    Click it to activate your account.
                  </p>
                ) : (
                  <p className={styles.formSubtitle}>Enter your email address to receive a new verification link.</p>
                )}
              </div>

              {!email && (
                <div className={styles.fieldWrap}>
                  <span className={styles.fieldIcon} aria-hidden="true">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
                      <rect x="2" y="4" width="20" height="16" rx="2" /><polyline points="2,4 12,13 22,4" />
                    </svg>
                  </span>
                  <input className={styles.fieldInput} type="email" value={emailInput} onChange={(e) => setEmailInput(e.target.value)} placeholder="user@example.com" autoComplete="email" />
                </div>
              )}

              {resendMsg && (
                <p className={resendMsg.includes("sent") ? styles.successMsg : styles.errorText}>{resendMsg}</p>
              )}

              <Button className={styles.submitBtn} onClick={handleResend} loading={resending} disabled={countdown > 0 || (!email && !emailInput.trim())}>
                {countdown > 0 ? `Resend in ${countdown}s` : "Resend verification email"}
              </Button>

              <p className={styles.altAction}>
                <Link to={registerPath} className={styles.altLink}>Use a different email</Link>
              </p>

              <p className={styles.spamHint}>Didn&apos;t receive it? Check your spam folder or request a new link.</p>

              <p className={styles.backLine}>
                <button type="button" className={styles.backLink} onClick={() => { clearTokens(); navigate(loginPath); }}>Back to sign in</button>
              </p>
            </>
          )}
        </div>

        <p className={styles.copyright}>{copyright}</p>
      </div>
    </div>
  );
}
