import type { ReactNode } from "react";
import { useSentinelAuth } from "../../context/SentinelAuthContext";
import styles from "./BrandPanel.module.css";

function DefaultShieldIcon() {
  return (
    <svg
      className={styles.iconSvg}
      viewBox="0 0 120 140"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      aria-hidden="true"
    >
      <defs>
        <linearGradient id="brandShieldGrad" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stopColor="#06b6d4" />
          <stop offset="100%" stopColor="#3b82f6" />
        </linearGradient>
        <linearGradient id="brandShieldInner" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stopColor="rgba(6,182,212,0.15)" />
          <stop offset="100%" stopColor="rgba(59,130,246,0.08)" />
        </linearGradient>
      </defs>
      <path
        d="M60 4L8 26v42c0 31.4 22.1 60.8 52 68 29.9-7.2 52-36.6 52-68V26L60 4z"
        fill="url(#brandShieldInner)"
        stroke="url(#brandShieldGrad)"
        strokeWidth="2"
      />
      <path
        d="M60 18L22 36v32c0 22.8 16.2 44.1 38 49.4C81.8 112.1 98 90.8 98 68V36L60 18z"
        fill="url(#brandShieldInner)"
        stroke="url(#brandShieldGrad)"
        strokeWidth="1"
        strokeOpacity="0.5"
      />
      <rect x="46" y="66" width="28" height="22" rx="4" fill="url(#brandShieldGrad)" />
      <path
        d="M50 66v-6a10 10 0 0120 0v6"
        stroke="url(#brandShieldGrad)"
        strokeWidth="3.5"
        strokeLinecap="round"
        fill="none"
      />
      <circle cx="60" cy="75" r="3.5" fill="#070d1a" />
      <rect x="58.5" y="75" width="3" height="6" rx="1.5" fill="#070d1a" />
    </svg>
  );
}

export interface BrandPanelProps {
  /** Page-specific tagline shown below the wordmark. Overridden by theme.tagline if set. */
  tagline: string;
  /** Optional secondary line below the tagline. */
  taglineSubtext?: string;
  /**
   * Fallback icon rendered when theme.logo is not provided.
   * Defaults to the Sentinel shield SVG.
   */
  defaultIcon?: ReactNode;
  /**
   * When false, the orbiting dot decorations are hidden.
   * Useful for pages with a custom icon that looks better without orbits.
   * Default: true.
   */
  showOrbits?: boolean;
}

/**
 * Shared left-column brand panel used by all auth pages.
 *
 * Logo priority (highest → lowest):
 *   1. theme.logo  (string URL or ReactNode set in SentinelAuthProvider)
 *   2. defaultIcon (per-page override prop)
 *   3. Built-in Sentinel shield SVG
 */
export function BrandPanel({
  tagline,
  taglineSubtext,
  defaultIcon,
  showOrbits = true,
}: BrandPanelProps) {
  const { theme } = useSentinelAuth();
  const appName = theme.appName ?? "Sentinel";
  // theme.tagline overrides the per-page tagline
  const resolvedTagline = theme.tagline ?? tagline;

  let logoNode: ReactNode;
  if (theme.logo == null) {
    logoNode = defaultIcon ?? <DefaultShieldIcon />;
  } else if (typeof theme.logo === "string") {
    logoNode = <img src={theme.logo} alt={appName} className={styles.logoImg} />;
  } else {
    logoNode = theme.logo;
  }

  return (
    <div className={styles.brandPanel}>
      <div className={styles.gridDots} aria-hidden="true" />

      <div className={styles.animationStage} aria-hidden="true">
        <div className={`${styles.ring} ${styles.ring1}`} />
        <div className={`${styles.ring} ${styles.ring2}`} />
        <div className={`${styles.ring} ${styles.ring3}`} />
        {showOrbits && (
          <>
            <div className={`${styles.orbitTrack} ${styles.orbitOuter}`}>
              <span className={styles.orbitDot} />
            </div>
            <div className={`${styles.orbitTrack} ${styles.orbitOuter} ${styles.orbitPhase}`}>
              <span className={styles.orbitDot} />
            </div>
            <div className={`${styles.orbitTrack} ${styles.orbitMid}`}>
              <span className={styles.orbitDot} />
            </div>
            <div className={`${styles.orbitTrack} ${styles.orbitMid} ${styles.orbitPhase2}`}>
              <span className={styles.orbitDot} />
            </div>
          </>
        )}
        <div className={styles.iconWrap}>
          {logoNode}
        </div>
      </div>

      <div className={styles.wordmark}>
        <span className={styles.wordmarkName}>{appName}</span>
        <span className={styles.wordmarkAuth}>&nbsp;Auth</span>
      </div>
      <p className={styles.tagline}>{resolvedTagline}</p>
      {taglineSubtext && <p className={styles.taglineSubtext}>{taglineSubtext}</p>}
    </div>
  );
}
