import type { SentinelAuthClient } from "@sentinel/auth-sdk";
export declare function registerTokenRefreshClient(client: SentinelAuthClient): void;
/**
 * Deduped token refresh — concurrent callers share the same in-flight promise.
 * Returns true if the refresh succeeded, false otherwise.
 */
export declare function refreshTokens(): Promise<boolean>;
//# sourceMappingURL=tokenRefresh.d.ts.map