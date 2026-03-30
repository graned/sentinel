/**
 * `SentinelAuthClient` — main entry point for the Sentinel Auth TypeScript SDK.
 *
 * Instantiate once and reuse across your application:
 * ```ts
 * const client = new SentinelAuthClient({ baseUrl: 'http://localhost:8080' });
 * const result = await client.login({ email, password });
 * ```
 *
 * Domain sub-clients are exposed as properties:
 * - `client.user`      — profile, sessions, password change
 * - `client.mfa`       — TOTP enrollment and MFA login verification
 * - `client.apiTokens` — long-lived API token management (admin)
 * - `client.admin`     — roles, policies, email templates (admin)
 * - `client.system`    — health check, SMTP provider config (admin)
 */
import { AdminClient } from './admin-client';
import { ApiTokenClient } from './api-token-client';
import {
  ForbiddenError,
  NetworkError,
  SessionNotFoundError,
  SentinelError,
  createErrorFromCode,
} from './errors';
import { MfaClient } from './mfa-client';
import { SessionCache } from './session';
import { SystemClient } from './system-client';
import { UserClient } from './user-client';
import type {
  ApiEnvelope,
  AuthContextData,
  AuthMethodsData,
  AuthenticateAndAuthorizeData,
  AuthenticateAndAuthorizeRequest,
  AuthenticateRequest,
  BatchCheckData,
  BatchCheckRequest,
  BasicLoginData,
  CheckAuthorizationData,
  CheckAuthorizationRequest,
  ForgotPasswordRequest,
  LoginData,
  LoginRequest,
  LoginResult,
  MfaChallengeData,
  RefreshTokenRequest,
  RegisterData,
  RegisterRequest,
  ResendVerificationRequest,
  ResetPasswordRequest,
  SentinelConfig,
  Session,
} from './types';

const DEFAULT_REFRESH_BUFFER_MS = 5 * 60 * 1000; // 5 minutes

function isMfaChallenge(data: LoginData): data is MfaChallengeData {
  return (data as MfaChallengeData).mfa_required === true;
}

/**
 * Main entry point for the Sentinel Auth SDK.
 *
 * ```ts
 * const client = new SentinelAuthClient({ baseUrl: 'https://auth.example.com' });
 *
 * const result = await client.login({ email: 'alice@example.com', password: 'secret' });
 * if (result.type === 'session') {
 *   const profile = await client.user.getMe(result.session.accessToken);
 * }
 * ```
 *
 * Sub-client namespaces:
 * - `client.user`      — profile, sessions, permissions, password change
 * - `client.mfa`       — TOTP enrollment and MFA login verification
 * - `client.apiTokens` — long-lived API token management (admin)
 * - `client.admin`     — roles, policies, email templates (admin)
 * - `client.system`    — health check
 */
export class SentinelAuthClient {
  private readonly cache: SessionCache;
  private readonly refreshBufferMs: number;
  private readonly baseUrl: string;
  private readonly config: SentinelConfig;

  /** User profile, sessions, permissions, and password management. */
  readonly user: UserClient;
  /** TOTP enrollment and MFA-login verification. */
  readonly mfa: MfaClient;
  /** Long-lived API token management (admin role required). */
  readonly apiTokens: ApiTokenClient;
  /** Role, policy, and email template administration (admin role required). */
  readonly admin: AdminClient;
  /** System-level endpoints (health check). */
  readonly system: SystemClient;

  constructor(config: SentinelConfig) {
    this.config = config;
    this.baseUrl = config.baseUrl.replace(/\/$/, '');
    this.refreshBufferMs = config.refreshBufferMs ?? DEFAULT_REFRESH_BUFFER_MS;
    this.cache = new SessionCache();

    const req = this.request.bind(this);
    this.user = new UserClient(req);
    this.mfa = new MfaClient(req, this.toSession.bind(this), (s) => this.cache.set(s.userId, s));
    this.apiTokens = new ApiTokenClient(req);
    this.admin = new AdminClient(req);
    this.system = new SystemClient(req);
  }

  // ---------------------------------------------------------------------------
  // Auth — public endpoints
  // ---------------------------------------------------------------------------

  /**
   * Register a new user account.
   * The returned user status will be `PendingVerification` until the email
   * is confirmed (if email verification is required).
   *
   * `POST /v1/api/auth/register`
   *
   * @throws {ValidationError}  Weak password or invalid email.
   * @throws {RateLimitError}   10 requests / 15 min per IP.
   */
  async register(body: RegisterRequest): Promise<RegisterData> {
    const { data } = await this.request<RegisterData>('/v1/api/auth/register', {
      method: 'POST',
      body: JSON.stringify(body),
    });
    return data;
  }

  /**
   * Log in with email and password.
   *
   * - **Non-MFA users:** returns `{ type: 'session', session }` and stores the
   *   session in the cache.
   * - **MFA users:** returns `{ type: 'mfa_challenge', mfaSessionToken }`.
   *   Pass the token to `client.mfa.verify()` to complete the login.
   *
   * `POST /v1/api/auth/login`
   *
   * @throws {AuthenticationError}   Wrong credentials.
   * @throws {ValidationError}       Malformed request body.
   * @throws {EmailNotVerifiedError} Email not yet verified.
   * @throws {RateLimitError}        5 requests / 15 min per IP.
   * @throws {NetworkError}          Server unreachable.
   */
  async login(credentials: LoginRequest): Promise<LoginResult> {
    const { data } = await this.request<LoginData>('/v1/api/auth/login', {
      method: 'POST',
      body: JSON.stringify(credentials),
    });

    if (isMfaChallenge(data)) {
      return {
        type: 'mfa_challenge',
        userId: data.user_id,
        mfaSessionToken: data.mfa_session_token,
      };
    }

    const session = this.toSession(data);
    this.cache.set(session.userId, session);
    return { type: 'session', session, mustChangePassword: data.must_change_password, mfaSetupRequired: data.mfa_setup_required ?? false };
  }

  /**
   * Log out the user, revoking only the current session.
   * The cached session is removed regardless of whether the server call
   * succeeds. No-op if no session is cached for `userId`.
   *
   * `POST /v1/api/auth/logout`
   */
  async logout(userId: string): Promise<void> {
    const session = this.cache.get(userId);
    this.cache.delete(userId);
    if (!session) return;

    await this.request('/v1/api/auth/logout', {
      method: 'POST',
      headers: { Authorization: `Bearer ${session.accessToken}` },
    });
  }

  /**
   * Revoke **all** sessions for the authenticated user (sign out everywhere).
   * The local cached session is also removed.
   *
   * `POST /v1/api/auth/logout-all`
   */
  async logoutAll(userId: string): Promise<void> {
    const session = this.cache.get(userId);
    this.cache.delete(userId);
    if (!session) return;

    await this.request('/v1/api/auth/logout-all', {
      method: 'POST',
      headers: { Authorization: `Bearer ${session.accessToken}` },
    });
  }

  /**
   * Exchange the stored refresh token for fresh access + refresh tokens,
   * updating the cache.
   *
   * @throws {SessionNotFoundError} No cached session for `userId`.
   * @throws {InvalidTokenError}    Refresh token is malformed.
   * @throws {ExpiredTokenError}    Refresh token has expired.
   */
  async refreshSession(userId: string, refreshToken?: string): Promise<Session> {
    const tokenToUse = refreshToken ?? this.cache.get(userId)?.refreshToken;
    if (!tokenToUse) throw new SessionNotFoundError(userId);

    const body: RefreshTokenRequest = { refresh_token: tokenToUse };
    const { data } = await this.request<BasicLoginData>('/v1/api/auth/token/refresh', {
      method: 'POST',
      body: JSON.stringify(body),
    });

    const session = this.toSession(data);
    this.cache.set(userId, session);
    return session;
  }

  /**
   * Validate a PASETO access token and return the embedded auth context
   * (user ID, session ID, roles, email verification status).
   * Useful for services that need to verify a token passed from a client.
   *
   * `POST /v1/api/auth/authenticate`
   */
  async authenticate(body: AuthenticateRequest): Promise<AuthContextData> {
    // The backend handler uses ValidatedBearer — it extracts the token from the
    // Authorization header and has no request body. Do NOT send a JSON body.
    const { data } = await this.request<AuthContextData>('/v1/api/auth/authenticate', {
      method: 'POST',
      headers: { Authorization: `Bearer ${body.access_token}` },
    });
    return data;
  }

  /**
   * Check whether a given method + path is allowed for a set of roles
   * under the specified policy.
   *
   * `POST /v1/api/auth/token/authorize`
   */
  async checkAuthorization(body: CheckAuthorizationRequest): Promise<CheckAuthorizationData> {
    const { data } = await this.request<CheckAuthorizationData>('/v1/api/auth/token/authorize', {
      method: 'POST',
      body: JSON.stringify(body),
    });
    return data;
  }

  /**
   * Evaluate multiple method+path checks against a policy in a single request.
   * Uses the compiled in-memory trie engine (~200 ns per check). No customer app needed.
   *
   * `POST /v1/api/auth/token/authorize/batch`
   */
  async checkAuthorizationBatch(body: BatchCheckRequest): Promise<BatchCheckData> {
    const { data } = await this.request<BatchCheckData>('/v1/api/auth/token/authorize/batch', {
      method: 'POST',
      body: JSON.stringify(body),
    });
    return data;
  }

  /**
   * Validate a PASETO access token **and** check whether the caller is
   * authorized to perform a given action — in a single logical operation.
   *
   * This is the recommended entry-point for middleware or API gateway code
   * that needs to gate access to a downstream resource:
   *
   * ```ts
   * const { auth, authorization } = await client.authenticateAndAuthorize({
   *   access_token: bearerToken,
   *   method: 'GET',
   *   path: '/v1/api/user/me',
   * });
   * // auth.user_id, auth.roles, authorization.active_version are available here
   * ```
   *
   * **Flow:**
   * 1. Calls `POST /v1/api/auth/authenticate` to validate the token and
   *    resolve the user's roles.
   * 2. Calls `POST /v1/api/auth/token/authorize` with those roles to evaluate
   *    the policy.
   * 3. If the policy denies the action, throws `ForbiddenError`.
   *
   * @throws {InvalidTokenError}     The token is malformed or cannot be decrypted.
   * @throws {ExpiredTokenError}     The token has passed its TTL.
   * @throws {EmailNotVerifiedError} The user has not verified their email address.
   * @throws {ForbiddenError}        The token is valid but the action is not permitted by policy.
   * @throws {NetworkError}          The Sentinel server could not be reached.
   */
  async authenticateAndAuthorize(
    body: AuthenticateAndAuthorizeRequest,
  ): Promise<AuthenticateAndAuthorizeData> {
    const auth = await this.authenticate({ access_token: body.access_token });
    const authorization = await this.checkAuthorization({
      policy_id: body.policy_id,
      method: body.method,
      path: body.path,
      roles: auth.roles,
    });

    if (!authorization.allowed) {
      throw new ForbiddenError(
        `Access denied: ${body.method} ${body.path} is not permitted for the current roles.`,
      );
    }

    return { auth, authorization };
  }

  /**
   * Verify an email address using the raw token from the verification link.
   *
   * `GET /v1/api/auth/verify-email?token=<raw>`
   */
  async verifyEmail(token: string): Promise<void> {
    await this.request(`/v1/api/auth/verify-email?token=${encodeURIComponent(token)}`);
  }

  /**
   * Trigger a new verification email for the given address.
   * Rate-limited to 10 requests / 15 min per IP.
   *
   * `POST /v1/api/auth/resend-verification`
   */
  async resendVerification(body: ResendVerificationRequest): Promise<void> {
    await this.request('/v1/api/auth/resend-verification', {
      method: 'POST',
      body: JSON.stringify(body),
    });
  }

  /**
   * Retrieve the authentication methods configured on this Sentinel instance
   * (password auth, MFA, API tokens, OIDC clients, etc.).
   *
   * `GET /v1/api/auth/auth-methods`
   */
  async getAuthMethods(): Promise<AuthMethodsData> {
    const { data } = await this.request<AuthMethodsData>('/v1/api/auth/auth-methods');
    return data;
  }

  /**
   * Initiate a password reset flow by sending a reset link to the given email.
   * Always returns successfully — the server silently no-ops for unknown emails
   * to prevent user enumeration.
   *
   * `POST /v1/api/auth/password/forgot`
   */
  async forgotPassword(body: ForgotPasswordRequest): Promise<void> {
    await this.request('/v1/api/auth/password/forgot', {
      method: 'POST',
      body: JSON.stringify(body),
    });
  }

  /**
   * Reset a password using the token from the reset email.
   * All existing sessions are revoked on success.
   *
   * `POST /v1/api/auth/password/reset`
   */
  async resetPassword(body: ResetPasswordRequest): Promise<void> {
    await this.request('/v1/api/auth/password/reset', {
      method: 'POST',
      body: JSON.stringify(body),
    });
  }

  // ---------------------------------------------------------------------------
  // Session cache helpers
  // ---------------------------------------------------------------------------

  /**
   * Return a valid (non-expired) session for `userId`, auto-refreshing it when
   * it is close to expiry (within `refreshBufferMs`). Returns `null` when no
   * session is cached or the session has fully expired.
   */
  async getValidSession(userId: string): Promise<Session | null> {
    const session = this.cache.get(userId);
    if (!session) return null;

    if (this.cache.isExpired(session)) {
      this.cache.delete(userId);
      return null;
    }

    if (this.cache.isExpiringSoon(session, this.refreshBufferMs)) {
      return this.refreshSession(userId);
    }

    return session;
  }

  /** Synchronously retrieve the raw cached session without validity checks. */
  getSession(userId: string): Session | undefined {
    return this.cache.get(userId);
  }

  /** Remove the cached session for `userId` without calling the server. */
  clearSession(userId: string): void {
    this.cache.delete(userId);
  }

  /** Remove all cached sessions without calling the server. */
  clearAllSessions(): void {
    this.cache.clear();
  }

  /**
   * Evict every expired session from the cache.
   * Returns the number of sessions removed.
   */
  evictExpiredSessions(): number {
    return this.cache.evictExpired();
  }

  // ---------------------------------------------------------------------------
  // Internal helpers
  // ---------------------------------------------------------------------------

  private async request<T>(
    path: string,
    options: RequestInit = {},
  ): Promise<{ data: T; requestId: string }> {
    let response: Response;

    try {
      response = await fetch(`${this.baseUrl}${path}`, {
        ...options,
        headers: {
          'Content-Type': 'application/json',
          ...(this.config.headers ?? {}),
          ...(options.headers ?? {}),
        },
      });
    } catch (cause) {
      throw new NetworkError(
        'Unable to reach the Sentinel Auth server. Check your network connection.',
        cause,
      );
    }

    let envelope: ApiEnvelope<T>;
    try {
      envelope = (await response.json()) as ApiEnvelope<T>;
    } catch {
      throw new SentinelError(
        'INVALID_RESPONSE',
        `Server returned non-JSON response (HTTP ${response.status})`,
        response.status,
      );
    }

    const requestId = envelope.request_id;

    if (!envelope.success || envelope.error) {
      const err = envelope.error;
      throw createErrorFromCode(
        err?.code ?? 'INTERNAL_ERROR',
        err?.message ?? 'An unexpected error occurred',
        response.status,
        requestId,
        err?.details,
      );
    }

    if (envelope.data === undefined) {
      throw new SentinelError(
        'EMPTY_RESPONSE',
        'Server returned a successful response with no data',
        response.status,
        requestId,
      );
    }

    return { data: envelope.data as T, requestId };
  }

  private toSession(data: BasicLoginData): Session {
    return {
      userId: data.user_id,
      accessToken: data.access_token,
      refreshToken: data.refresh_token,
      expiresAt: new Date(data.expires_at),
    };
  }
}
