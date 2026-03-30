import type { ReactNode } from "react";
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
export declare function BrandPanel({ tagline, taglineSubtext, defaultIcon, showOrbits, }: BrandPanelProps): import("react/jsx-runtime").JSX.Element;
//# sourceMappingURL=BrandPanel.d.ts.map