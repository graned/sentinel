import type {
  BasicLoginData,
  MfaTotpConfirmData,
  MfaTotpConfirmRequest,
  MfaTotpStartData,
  MfaVerifyRequest,
  RequestFn,
  Session,
} from './types';

/**
 * TOTP-based multi-factor authentication.
 *
 * **Enrollment flow** (requires an active session):
 * ```ts
 * const { otpauth_uri } = await client.mfa.totpStart(accessToken);
 * // Display QR code to user, then:
 * const { recovery_codes } = await client.mfa.totpConfirm(accessToken, { code: '123456' });
 * ```
 *
 * **Login flow** (after login returns `mfa_challenge`):
 * ```ts
 * const session = await client.mfa.verify({ mfa_session_token, code: '123456' });
 * ```
 */
export class MfaClient {
  constructor(
    private readonly req: RequestFn,
    private readonly toSession: (data: BasicLoginData) => Session,
    private readonly cacheSession: (session: Session) => void,
  ) {}

  /**
   * Begin TOTP enrollment — returns an `otpauth://` URI to display as a QR code.
   * Bearer token required (user must be logged in).
   *
   * `POST /v1/api/auth/mfa/totp/start`
   */
  async totpStart(accessToken: string): Promise<MfaTotpStartData> {
    const { data } = await this.req<MfaTotpStartData>('/v1/api/auth/mfa/totp/start', {
      method: 'POST',
      headers: { Authorization: `Bearer ${accessToken}` },
    });
    return data;
  }

  /**
   * Confirm TOTP enrollment by submitting the first code from the app.
   * Returns one-time recovery codes — store them securely.
   * Bearer token required.
   *
   * `POST /v1/api/auth/mfa/totp/confirm`
   */
  async totpConfirm(accessToken: string, body: MfaTotpConfirmRequest): Promise<MfaTotpConfirmData> {
    const { data } = await this.req<MfaTotpConfirmData>('/v1/api/auth/mfa/totp/confirm', {
      method: 'POST',
      body: JSON.stringify(body),
      headers: { Authorization: `Bearer ${accessToken}` },
    });
    return data;
  }

  /**
   * Complete a login that requires MFA.
   * On success, stores the resulting session in the cache and returns it.
   *
   * `POST /v1/api/auth/mfa/verify`
   *
   * @throws {MfaInvalidCodeError}     Wrong TOTP code or recovery code.
   * @throws {MfaAttemptLimitError}    Too many failed attempts (>5 in 15 min).
   * @throws {RateLimitError}          IP-level rate limit hit.
   */
  async verify(body: MfaVerifyRequest): Promise<Session> {
    const { data } = await this.req<BasicLoginData>('/v1/api/auth/mfa/verify', {
      method: 'POST',
      body: JSON.stringify(body),
    });
    const session = this.toSession(data);
    this.cacheSession(session);
    return session;
  }
}
