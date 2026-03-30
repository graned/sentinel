import type { Session } from './types';

/** Default window before expiry in which a proactive refresh is triggered. */
const DEFAULT_REFRESH_BUFFER_MS = 5 * 60 * 1000; // 5 minutes

/**
 * In-memory store for `Session` objects, keyed by `userId`.
 *
 * This is a pure data structure — it has no knowledge of the HTTP layer and
 * never makes network calls. Refresh logic lives in `SentinelAuthClient`.
 */
export class SessionCache {
  private readonly sessions = new Map<string, Session>();

  /** Store (or overwrite) a session for the given user. */
  set(userId: string, session: Session): void {
    this.sessions.set(userId, session);
  }

  /** Retrieve the stored session, or `undefined` if none exists. */
  get(userId: string): Session | undefined {
    return this.sessions.get(userId);
  }

  /** Remove the session for the given user. No-op if not present. */
  delete(userId: string): void {
    this.sessions.delete(userId);
  }

  /** Returns `true` if a session (expired or not) is stored for the user. */
  has(userId: string): boolean {
    return this.sessions.has(userId);
  }

  /** Returns `true` if the session's access token has already expired. */
  isExpired(session: Session): boolean {
    return Date.now() >= session.expiresAt.getTime();
  }

  /**
   * Returns `true` if the session will expire within `bufferMs` milliseconds.
   * Use this to decide whether a proactive refresh is worthwhile before
   * returning the session to a caller.
   *
   * @param bufferMs - Look-ahead window in ms. Defaults to 5 minutes.
   */
  isExpiringSoon(session: Session, bufferMs = DEFAULT_REFRESH_BUFFER_MS): boolean {
    return Date.now() >= session.expiresAt.getTime() - bufferMs;
  }

  /** Number of sessions currently cached (including any that may be expired). */
  get size(): number {
    return this.sessions.size;
  }

  /** Remove all cached sessions. */
  clear(): void {
    this.sessions.clear();
  }

  /**
   * Evict every session whose access token has already expired.
   * Returns the number of sessions removed.
   */
  evictExpired(): number {
    let removed = 0;
    for (const [userId, session] of this.sessions) {
      if (this.isExpired(session)) {
        this.sessions.delete(userId);
        removed++;
      }
    }
    return removed;
  }
}
