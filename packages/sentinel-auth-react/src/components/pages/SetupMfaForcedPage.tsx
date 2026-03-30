/**
 * Forced MFA setup page.  Shown when `mfaSetupRequired` is `true` in the auth store,
 * which means the admin has mandated MFA but the user hasn't enrolled yet.
 *
 * Flow:
 * 1. Calls `client.mfa.totpStart` to get the `otpauth://` URI → renders as QR code.
 * 2. User scans QR code with their authenticator app.
 * 3. User submits first TOTP code to `client.mfa.totpConfirm`.
 * 4. On success, displays recovery codes and clears `mfaSetupRequired` in the store.
 * 5. Redirects to `afterLogin`.
 */
import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { renderSVG } from "uqr";
import { SentinelError } from "@sentinel/auth-sdk";
import { useSentinelAuth } from "../../context/SentinelAuthContext";
import { useAuthStore } from "../../store/authStore";
import { Button } from "../ui/Button";
import { BrandPanel } from "../ui/BrandPanel";
import styles from "./ResetPasswordPage.module.css";

function parseTotpSecret(uri: string): string | null {
  try {
    const url = new URL(uri);
    return url.searchParams.get("secret");
  } catch {
    return null;
  }
}

export function SetupMfaForcedPage() {
  const { client, redirects, theme } = useSentinelAuth();
  const navigate = useNavigate();
  const { accessToken, clearMfaSetupRequired } = useAuthStore();

  const [step, setStep] = useState<"intro" | "confirm" | "done">("intro");
  const [otpauthUri, setOtpauthUri] = useState<string | null>(null);
  const [recoveryCodes, setRecoveryCodes] = useState<string[]>([]);
  const [code, setCode] = useState("");
  const [error, setError] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [copiedCodes, setCopiedCodes] = useState(false);
  const [copiedSecret, setCopiedSecret] = useState(false);

  const afterLoginPath = redirects.afterLogin ?? "/dashboard";
  const copyright = theme.copyright ?? "© 2026 Sentinel Auth. All rights reserved.";

  const handleStart = async () => {
    setError("");
    setIsLoading(true);
    try {
      const { otpauth_uri } = await client.mfa.totpStart(accessToken!);
      setOtpauthUri(otpauth_uri);
      setStep("confirm");
    } catch (err: unknown) {
      setError(err instanceof SentinelError ? err.message : "Failed to start MFA setup.");
    } finally {
      setIsLoading(false);
    }
  };

  const handleConfirm = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    setIsLoading(true);
    try {
      const { recovery_codes } = await client.mfa.totpConfirm(accessToken!, { code });
      setRecoveryCodes(recovery_codes);
      setStep("done");
    } catch (err: unknown) {
      setError(err instanceof SentinelError ? err.message : "Invalid code. Please try again.");
    } finally {
      setIsLoading(false);
    }
  };

  const handleFinish = () => {
    clearMfaSetupRequired();
    navigate(afterLoginPath);
  };

  const copyRecoveryCodes = () => {
    navigator.clipboard.writeText(recoveryCodes.join("\n")).then(() => {
      setCopiedCodes(true);
      setTimeout(() => setCopiedCodes(false), 2000);
    });
  };

  return (
    <div className={styles.page}>
      <BrandPanel tagline="Set up two-factor authentication" taglineSubtext="MFA is required for your account." />

      <div className={styles.formPanel}>
        <div className={styles.formCard}>
          {step === "intro" && (
            <>
              <div className={styles.formHeader}>
                <div className={styles.lockIcon} aria-hidden="true">
                  <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
                    <rect x="3" y="11" width="18" height="11" rx="2" />
                    <path d="M7 11V7a5 5 0 0110 0v4" />
                  </svg>
                </div>
                <h1 className={styles.formTitle}>Set up MFA</h1>
                <p className={styles.formSubtitle}>
                  Your administrator requires multi-factor authentication. Link an authenticator app to continue.
                </p>
              </div>
              {error && <p className={styles.errorText}>{error}</p>}
              <Button className={styles.submitBtn} onClick={handleStart} loading={isLoading}>
                Set up authenticator app
              </Button>
            </>
          )}

          {step === "confirm" && (
            <>
              <div className={styles.formHeader}>
                <h1 className={styles.formTitle}>Scan QR code</h1>
                <p className={styles.formSubtitle}>
                  Scan this code with your authenticator app, then enter the 6-digit code to verify.
                </p>
              </div>
              {otpauthUri && (
                <div
                  className={styles.qrWrap}
                  // renderSVG output is generated entirely by our own code — safe
                  // eslint-disable-next-line react/no-danger
                  dangerouslySetInnerHTML={{ __html: renderSVG(otpauthUri) }}
                  aria-label="TOTP QR code"
                />
              )}
              {otpauthUri && (() => {
                const secret = parseTotpSecret(otpauthUri);
                return secret ? (
                  <details style={{ marginBottom: "1rem" }}>
                    <summary style={{ cursor: "pointer", fontSize: "0.85rem", color: "var(--color-text-muted, #888)" }}>
                      Can't scan? Enter the key manually
                    </summary>
                    <div style={{ display: "flex", gap: "0.5rem", alignItems: "center", marginTop: "0.5rem" }}>
                      <code style={{ fontSize: "0.8rem", wordBreak: "break-all" }}>{secret}</code>
                      <button
                        type="button"
                        style={{ fontSize: "0.75rem", cursor: "pointer" }}
                        onClick={() => {
                          navigator.clipboard.writeText(secret).then(() => {
                            setCopiedSecret(true);
                            setTimeout(() => setCopiedSecret(false), 2000);
                          });
                        }}
                      >
                        {copiedSecret ? "Copied!" : "Copy key"}
                      </button>
                    </div>
                  </details>
                ) : null;
              })()}
              <form onSubmit={handleConfirm} className={styles.form}>
                <div className={styles.fieldWrap}>
                  <input
                    className={styles.fieldInput}
                    type="text"
                    inputMode="numeric"
                    pattern="[0-9]{6}"
                    maxLength={6}
                    value={code}
                    onChange={(e) => setCode(e.target.value.replace(/\D/g, ""))}
                    placeholder="6-digit code"
                    required
                    autoFocus
                    autoComplete="one-time-code"
                  />
                </div>
                {error && <p className={styles.errorText}>{error}</p>}
                <Button type="submit" loading={isLoading} className={styles.submitBtn} disabled={code.length !== 6}>
                  Verify and enable MFA
                </Button>
              </form>
            </>
          )}

          {step === "done" && (
            <div className={styles.statusCenter}>
              <div className={styles.successIcon} aria-hidden="true">
                <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
                  <circle cx="12" cy="12" r="10" />
                  <polyline points="9 12 11 14 15 10" />
                </svg>
              </div>
              <h1 className={styles.formTitle}>MFA enabled!</h1>
              <p className={styles.formSubtitle}>
                Save these recovery codes somewhere safe. Each code can only be used once.
              </p>
              <ul style={{ listStyle: "none", padding: 0, margin: "1rem 0", display: "grid", gridTemplateColumns: "1fr 1fr", gap: "0.4rem" }}>
                {recoveryCodes.map((c) => (
                  <li key={c}><code style={{ fontSize: "0.85rem" }}>{c}</code></li>
                ))}
              </ul>
              <Button className={styles.actionBtn} onClick={copyRecoveryCodes} style={{ marginBottom: "0.75rem" }}>
                {copiedCodes ? "Copied!" : "Copy all codes"}
              </Button>
              <Button className={styles.actionBtn} onClick={handleFinish}>
                Go to dashboard
              </Button>
            </div>
          )}
        </div>

        <p className={styles.copyright}>{copyright}</p>
      </div>
    </div>
  );
}
