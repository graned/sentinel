/** Sentinel shield logo — used as theme.logo in SentinelAuthProvider. */
export function SentinelLogo() {
  return (
    <svg
      viewBox="0 0 120 140"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      width="108"
      height="126"
      aria-label="Sentinel"
    >
      <defs>
        <linearGradient id="uiLogoGrad" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stopColor="#06b6d4" />
          <stop offset="100%" stopColor="#3b82f6" />
        </linearGradient>
        <linearGradient id="uiLogoInner" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stopColor="rgba(6,182,212,0.15)" />
          <stop offset="100%" stopColor="rgba(59,130,246,0.08)" />
        </linearGradient>
      </defs>
      <path
        d="M60 4L8 26v42c0 31.4 22.1 60.8 52 68 29.9-7.2 52-36.6 52-68V26L60 4z"
        fill="url(#uiLogoInner)"
        stroke="url(#uiLogoGrad)"
        strokeWidth="2"
      />
      <path
        d="M60 18L22 36v32c0 22.8 16.2 44.1 38 49.4C81.8 112.1 98 90.8 98 68V36L60 18z"
        fill="url(#uiLogoInner)"
        stroke="url(#uiLogoGrad)"
        strokeWidth="1"
        strokeOpacity="0.5"
      />
      <rect x="46" y="66" width="28" height="22" rx="4" fill="url(#uiLogoGrad)" />
      <path
        d="M50 66v-6a10 10 0 0120 0v6"
        stroke="url(#uiLogoGrad)"
        strokeWidth="3.5"
        strokeLinecap="round"
        fill="none"
      />
      <circle cx="60" cy="75" r="3.5" fill="#070d1a" />
      <rect x="58.5" y="75" width="3" height="6" rx="1.5" fill="#070d1a" />
    </svg>
  );
}
