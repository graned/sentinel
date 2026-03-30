/**
 * Registration form.  On success the server creates the user and sends a
 * verification email; the user is redirected to `afterRegister` (default: `/verify-email`).
 * The page must be rendered inside `<SentinelAuthProvider>`.
 */
import { useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { SentinelError } from "@sentinel/auth-sdk";
import { useSentinelAuth } from "../../context/SentinelAuthContext";
import { Button } from "../ui/Button";
import { BrandPanel } from "../ui/BrandPanel";
import styles from "./RegisterPage.module.css";

const PASSWORD_RULES = [
  { key: "length",  label: "At least 12 characters",         test: (p: string) => p.length >= 12 },
  { key: "upper",   label: "At least one uppercase letter",  test: (p: string) => /[A-Z]/.test(p) },
  { key: "lower",   label: "At least one lowercase letter",  test: (p: string) => /[a-z]/.test(p) },
  { key: "digit",   label: "At least one number",            test: (p: string) => /[0-9]/.test(p) },
  { key: "special", label: "At least one special character", test: (p: string) => /[^A-Za-z0-9]/.test(p) },
] as const;

export function RegisterPage() {
  const { client, redirects, theme } = useSentinelAuth();
  const [firstName, setFirstName] = useState("");
  const [lastName, setLastName] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [passwordTouched, setPasswordTouched] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const navigate = useNavigate();

  const passwordRuleResults = PASSWORD_RULES.map((rule) => ({
    ...rule,
    met: rule.test(password),
  }));
  const allRulesMet = passwordRuleResults.every((r) => r.met);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    if (!allRulesMet) {
      setPasswordTouched(true);
      return;
    }
    if (password !== confirmPassword) {
      setError("Passwords do not match.");
      return;
    }

    setIsLoading(true);
    try {
      await client.register({ first_name: firstName, last_name: lastName, email, password });
      navigate(redirects.verifyEmail ?? "/verify-email", { state: { email, fromRegistration: true } });
    } catch (err: unknown) {
      setError(err instanceof SentinelError ? err.message : "Registration failed. Please try again.");
    } finally {
      setIsLoading(false);
    }
  };

  const appName = theme.appName ?? "Sentinel";
  const copyright = theme.copyright ?? "© 2026 Sentinel Auth. All rights reserved.";

  return (
    <div className={styles.page}>
      <BrandPanel tagline="Secure. Fast. Reliable." />

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
          <div className={styles.formHeader}>
            <h1 className={styles.formTitle}>Create your account</h1>
            <p className={styles.formSubtitle}>Start your free trial. No credit card required.</p>
          </div>

          <form onSubmit={handleSubmit} className={styles.form}>
            <div className={styles.nameRow}>
              <div className={styles.fieldWrap}>
                <span className={styles.fieldIcon} aria-hidden="true">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
                    <path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2" />
                    <circle cx="12" cy="7" r="4" />
                  </svg>
                </span>
                <input
                  className={styles.fieldInput}
                  type="text"
                  value={firstName}
                  onChange={(e) => setFirstName(e.target.value)}
                  required
                  autoComplete="given-name"
                  placeholder="First Name"
                />
              </div>

              <div className={styles.fieldWrap}>
                <span className={styles.fieldIcon} aria-hidden="true">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
                    <path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2" />
                    <circle cx="12" cy="7" r="4" />
                  </svg>
                </span>
                <input
                  className={styles.fieldInput}
                  type="text"
                  value={lastName}
                  onChange={(e) => setLastName(e.target.value)}
                  required
                  autoComplete="family-name"
                  placeholder="Last Name"
                />
              </div>
            </div>

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
                placeholder="Email"
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
                onChange={(e) => { setPassword(e.target.value); setPasswordTouched(true); }}
                autoComplete="new-password"
                placeholder="Password"
              />
            </div>

            {passwordTouched && (
              <ul className={styles.pwdChecklist} aria-label="Password requirements">
                {passwordRuleResults.map((rule) => (
                  <li key={rule.key} className={`${styles.pwdRule} ${rule.met ? styles.pwdRuleMet : ""}`}>
                    <span className={styles.pwdRuleIcon} aria-hidden="true">
                      {rule.met ? (
                        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                          <polyline points="20 6 9 17 4 12" />
                        </svg>
                      ) : (
                        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                          <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
                        </svg>
                      )}
                    </span>
                    {rule.label}
                  </li>
                ))}
              </ul>
            )}

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
                value={confirmPassword}
                onChange={(e) => setConfirmPassword(e.target.value)}
                autoComplete="new-password"
                placeholder="Confirm Password"
              />
            </div>

            {error && <p className={styles.error}>{error}</p>}

            <Button type="submit" loading={isLoading} className={styles.submitBtn}>
              Create Account
            </Button>
          </form>

          <p className={styles.signinLine}>
            Already have an account?{" "}
            <Link to={redirects.login ?? "/login"} className={styles.signinLink}>Sign in</Link>
          </p>

          <p className={styles.termsNote}>
            Your data is secured using {appName} Auth. By signing up,
            you agree to the{" "}
            <a href="#" className={styles.termsLink}>Terms of Service</a>.
          </p>
        </div>

        <p className={styles.copyright}>{copyright}</p>
      </div>
    </div>
  );
}
