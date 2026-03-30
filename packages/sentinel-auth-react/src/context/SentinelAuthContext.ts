/**
 * React context that carries the Sentinel client, redirect paths, and theme.
 * Populated by `<SentinelAuthProvider>`; consumed via `useSentinelAuth()`.
 *
 * Do not access this context directly — use `useSentinelAuth()` instead, which
 * provides a helpful error message when called outside the provider.
 */
import { createContext, useContext } from "react";
import type { SentinelAuthContextValue } from "../types";

export const SentinelAuthContext = createContext<SentinelAuthContextValue | null>(null);

/**
 * Returns the current `SentinelAuthContextValue`.
 * Throws with a descriptive message when called outside `<SentinelAuthProvider>`.
 */
export function useSentinelAuth(): SentinelAuthContextValue {
  const ctx = useContext(SentinelAuthContext);
  if (!ctx) {
    throw new Error("useSentinelAuth must be called inside <SentinelAuthProvider>");
  }
  return ctx;
}
