import { useState, useEffect, useRef } from "react";
import { renderSVG } from "uqr";
import { SentinelError } from "@sentinel/auth-sdk";
import { sentinelClient } from "../../lib/sdkClient";
import { useAuthStore } from "@sentinel/auth-react";
import styles from "./UserProfilePanel.module.css";

type Tab = "profile" | "password" | "mfa";

type MfaStep = "idle" | "active" | "pending" | "confirm" | "done";

const RULES = [
  { label: "At least 12 characters", test: (p: string) => p.length >= 12 },
  { label: "One uppercase letter", test: (p: string) => /[A-Z]/.test(p) },
  { label: "One lowercase letter", test: (p: string) => /[a-z]/.test(p) },
  { label: "One number", test: (p: string) => /[0-9]/.test(p) },
  { label: "One special character", test: (p: string) => /[^A-Za-z0-9]/.test(p) },
];

function PasswordChecklist({ password }: { password: string }) {
  return (
    <ul className={styles.checklist}>
      {RULES.map(({ label, test }) => {
        const met = test(password);
        return (
          <li key={label} className={styles.checkItem}>
            <svg
              className={`${styles.checkIcon} ${met ? styles.met : ""}`}
              width="13"
              height="13"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2.5"
              strokeLinecap="round"
              strokeLinejoin="round"
              aria-hidden="true"
            >
              {met ? <polyline points="20 6 9 17 4 12" /> : <circle cx="12" cy="12" r="9" />}
            </svg>
            <span className={`${styles.checkLabel} ${met ? styles.met : ""}`}>{label}</span>
          </li>
        );
      })}
    </ul>
  );
}

interface Props {
  open: boolean;
  onClose: () => void;
}

export function UserProfilePanel({ open, onClose }: Props) {
  const { accessToken, userEmail, firstName, lastName } = useAuthStore();
  const [tab, setTab] = useState<Tab>("profile");

  // ── Password tab state ──────────────────────────────────────────
  const [currentPw, setCurrentPw] = useState("");
  const [newPw, setNewPw] = useState("");
  const [confirmPw, setConfirmPw] = useState("");
  const [pwError, setPwError] = useState("");
  const [pwSuccess, setPwSuccess] = useState(false);
  const [pwLoading, setPwLoading] = useState(false);

  // ── MFA tab state ───────────────────────────────────────────────
  const [mfaStep, setMfaStep] = useState<MfaStep>("idle");
  const [otpauthUri, setOtpauthUri] = useState("");
  const [mfaCode, setMfaCode] = useState("");
  const [recoveryCodes, setRecoveryCodes] = useState<string[]>([]);
  const [mfaError, setMfaError] = useState("");
  const [mfaLoading, setMfaLoading] = useState(false);
  const [copiedCodes, setCopiedCodes] = useState(false);
  const [copiedSecret, setCopiedSecret] = useState(false);

  const panelRef = useRef<HTMLDivElement>(null);

  function parseTotpSecret(uri: string): string | null {
    try {
      const url = new URL(uri);
      return url.searchParams.get("secret");
    } catch {
      return null;
    }
  }

  // Reset inner state when panel is closed
  useEffect(() => {
    if (!open) {
      setTab("profile");
      setCurrentPw(""); setNewPw(""); setConfirmPw("");
      setPwError(""); setPwSuccess(false);
      setMfaStep("idle"); setOtpauthUri("");
      setMfaCode(""); setRecoveryCodes([]); setMfaError("");
      setCopiedCodes(false); setCopiedSecret(false);
    }
  }, [open]);

  // Fetch MFA status when switching to the MFA tab
  useEffect(() => {
    if (tab !== "mfa" || !accessToken) return;
    sentinelClient.user.getMe(accessToken).then((profile) => {
      if (profile.mfa_enabled) setMfaStep("active");
      else setMfaStep("idle");
    }).catch(() => {/* silently ignore — UI stays on idle */});
  }, [tab, accessToken]);

  // Trap Escape key
  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => { if (e.key === "Escape") onClose(); };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, onClose]);

  // ── Password submit ─────────────────────────────────────────────
  async function handlePasswordSubmit(e: React.FormEvent) {
    e.preventDefault();
    setPwError("");
    for (const { label, test } of RULES) {
      if (!test(newPw)) { setPwError(`Password must include: ${label.toLowerCase()}.`); return; }
    }
    if (newPw !== confirmPw) { setPwError("Passwords do not match."); return; }
    setPwLoading(true);
    try {
      await sentinelClient.user.changePassword(accessToken!, {
        current_password: currentPw,
        new_password: newPw,
      });
      setPwSuccess(true);
      setCurrentPw(""); setNewPw(""); setConfirmPw("");
    } catch (err) {
      setPwError(err instanceof SentinelError ? err.message : "Failed to change password.");
    } finally {
      setPwLoading(false);
    }
  }

  // ── MFA: start enrollment ───────────────────────────────────────
  async function handleMfaStart() {
    setMfaError("");
    setMfaLoading(true);
    try {
      const { otpauth_uri } = await sentinelClient.mfa.totpStart(accessToken!);
      setOtpauthUri(otpauth_uri);
      setMfaStep("confirm");
    } catch (err) {
      setMfaError(err instanceof SentinelError ? err.message : "Failed to start MFA setup.");
    } finally {
      setMfaLoading(false);
    }
  }

  // ── MFA: confirm code ───────────────────────────────────────────
  async function handleMfaConfirm(e: React.FormEvent) {
    e.preventDefault();
    setMfaError("");
    setMfaLoading(true);
    try {
      const { recovery_codes } = await sentinelClient.mfa.totpConfirm(accessToken!, { code: mfaCode });
      setRecoveryCodes(recovery_codes);
      setMfaStep("done");
    } catch (err) {
      setMfaError(err instanceof SentinelError ? err.message : "Invalid code. Please try again.");
    } finally {
      setMfaLoading(false);
    }
  }

  // ── Copy recovery codes ─────────────────────────────────────────
  function copyRecoveryCodes() {
    navigator.clipboard.writeText(recoveryCodes.join("\n")).then(() => {
      setCopiedCodes(true);
      setTimeout(() => setCopiedCodes(false), 2000);
    });
  }

  const displayName = [firstName, lastName].filter(Boolean).join(" ") || userEmail || "";
  const avatarLetter = (firstName?.[0] ?? userEmail?.[0] ?? "?").toUpperCase();

  return (
    <>
      {open && <div className={styles.backdrop} onClick={onClose} aria-hidden="true" />}

      <div
        ref={panelRef}
        className={`${styles.panel} ${open ? styles.panelOpen : ""}`}
        role="dialog"
        aria-modal="true"
        aria-label="User profile"
      >
        {/* Header */}
        <div className={styles.header}>
          <div className={styles.headerAvatar}>{avatarLetter}</div>
          <div className={styles.headerInfo}>
            <span className={styles.headerName}>{displayName}</span>
            {firstName && <span className={styles.headerEmail}>{userEmail}</span>}
          </div>
          <button className={styles.closeBtn} onClick={onClose} aria-label="Close">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>

        {/* Tabs */}
        <div className={styles.tabs} role="tablist">
          {(["profile", "password", "mfa"] as Tab[]).map((t) => (
            <button
              key={t}
              role="tab"
              aria-selected={tab === t}
              className={`${styles.tab} ${tab === t ? styles.tabActive : ""}`}
              onClick={() => { setTab(t); setPwSuccess(false); }}
            >
              {t === "profile" ? "Profile" : t === "password" ? "Password" : "MFA"}
            </button>
          ))}
        </div>

        {/* Body */}
        <div className={styles.body}>

          {/* ── Profile tab ── */}
          {tab === "profile" && (
            <div className={styles.section}>
              <div className={styles.profileAvatarLarge}>{avatarLetter}</div>
              <dl className={styles.dl}>
                {(firstName || lastName) && (
                  <>
                    <dt>Name</dt>
                    <dd>{[firstName, lastName].filter(Boolean).join(" ")}</dd>
                  </>
                )}
                <dt>Email</dt>
                <dd>{userEmail}</dd>
              </dl>
            </div>
          )}

          {/* ── Password tab ── */}
          {tab === "password" && (
            <div className={styles.section}>
              {pwSuccess ? (
                <div className={styles.successBox}>
                  <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
                    <circle cx="12" cy="12" r="10" />
                    <polyline points="9 12 11 14 15 10" />
                  </svg>
                  <p>Password changed successfully. All other sessions have been signed out.</p>
                </div>
              ) : (
                <form onSubmit={handlePasswordSubmit} className={styles.form}>
                  <label className={styles.label}>
                    Current password
                    <input
                      className={styles.input}
                      type="password"
                      value={currentPw}
                      onChange={(e) => setCurrentPw(e.target.value)}
                      required
                      autoComplete="current-password"
                    />
                  </label>
                  <label className={styles.label}>
                    New password
                    <input
                      className={styles.input}
                      type="password"
                      value={newPw}
                      onChange={(e) => setNewPw(e.target.value)}
                      required
                      autoComplete="new-password"
                    />
                  </label>
                  {newPw && <PasswordChecklist password={newPw} />}
                  <label className={styles.label}>
                    Confirm new password
                    <input
                      className={styles.input}
                      type="password"
                      value={confirmPw}
                      onChange={(e) => setConfirmPw(e.target.value)}
                      required
                      autoComplete="new-password"
                    />
                  </label>
                  {pwError && <p className={styles.error}>{pwError}</p>}
                  <button type="submit" className={styles.primaryBtn} disabled={pwLoading}>
                    {pwLoading ? <span className={styles.spinner} /> : null}
                    Change password
                  </button>
                </form>
              )}
            </div>
          )}

          {/* ── MFA tab ── */}
          {tab === "mfa" && (
            <div className={styles.section}>

              {mfaStep === "active" && (
                <>
                  <div className={styles.successBox}>
                    <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
                      <circle cx="12" cy="12" r="10" />
                      <polyline points="9 12 11 14 15 10" />
                    </svg>
                    <p>Two-factor authentication is active on your account.</p>
                  </div>
                  {mfaError && <p className={styles.error}>{mfaError}</p>}
                  <button className={styles.secondaryBtn} onClick={handleMfaStart} disabled={mfaLoading}>
                    {mfaLoading ? <span className={styles.spinner} /> : null}
                    Re-enroll authenticator app
                  </button>
                </>
              )}

              {mfaStep === "idle" && (
                <>
                  <div className={styles.mfaIntro}>
                    <svg width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
                      <rect x="3" y="11" width="18" height="11" rx="2" />
                      <path d="M7 11V7a5 5 0 0110 0v4" />
                    </svg>
                    <p>Add an extra layer of security with a time-based one-time password (TOTP) authenticator app.</p>
                  </div>
                  {mfaError && <p className={styles.error}>{mfaError}</p>}
                  <button className={styles.primaryBtn} onClick={handleMfaStart} disabled={mfaLoading}>
                    {mfaLoading ? <span className={styles.spinner} /> : null}
                    Set up authenticator app
                  </button>
                </>
              )}

              {mfaStep === "confirm" && (
                <>
                  <p className={styles.mfaHint}>
                    Scan this QR code with your authenticator app (Google Authenticator, Authy, 1Password, etc.).
                  </p>
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
                      <details className={styles.manualDetails}>
                        <summary>Can't scan? Enter the key manually</summary>
                        <div className={styles.secretBox}>
                          <code className={styles.secretKey}>{secret}</code>
                          <button
                            type="button"
                            className={styles.copySecretBtn}
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
                  <form onSubmit={handleMfaConfirm} className={styles.form}>
                    <label className={styles.label}>
                      Verification code
                      <input
                        className={`${styles.input} ${styles.codeInput}`}
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
                    </label>
                    {mfaError && <p className={styles.error}>{mfaError}</p>}
                    <button type="submit" className={styles.primaryBtn} disabled={mfaLoading || mfaCode.length !== 6}>
                      {mfaLoading ? <span className={styles.spinner} /> : null}
                      Verify and enable MFA
                    </button>
                  </form>
                </>
              )}

              {mfaStep === "done" && (
                <>
                  <div className={styles.successBox}>
                    <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
                      <circle cx="12" cy="12" r="10" />
                      <polyline points="9 12 11 14 15 10" />
                    </svg>
                    <p>MFA enabled successfully!</p>
                  </div>
                  <div className={styles.recoverySection}>
                    <p className={styles.recoveryHint}>
                      Save these recovery codes somewhere safe. Each code can only be used once.
                    </p>
                    <ul className={styles.recoveryCodes}>
                      {recoveryCodes.map((c) => <li key={c}><code>{c}</code></li>)}
                    </ul>
                    <button className={styles.secondaryBtn} onClick={copyRecoveryCodes}>
                      {copiedCodes ? "Copied!" : "Copy all codes"}
                    </button>
                  </div>
                </>
              )}
            </div>
          )}
        </div>
      </div>
    </>
  );
}
