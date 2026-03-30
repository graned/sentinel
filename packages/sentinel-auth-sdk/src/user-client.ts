import type {
  ChangePasswordRequest,
  RequestFn,
  UserPermissionsData,
  UserProfileData,
  UserSessionData,
  UserSessionDetailData,
} from './types';

/**
 * Methods scoped to the authenticated user.
 * Every method requires a valid Bearer `accessToken` from a live session.
 *
 * ```ts
 * const session = await client.getValidSession(userId);
 * const profile  = await client.user.getMe(session.accessToken);
 * ```
 */
export class UserClient {
  constructor(private readonly req: RequestFn) {}

  /**
   * Fetch the profile of the currently authenticated user.
   *
   * `GET /v1/api/user/me`
   */
  async getMe(accessToken: string): Promise<UserProfileData> {
    const { data } = await this.req<UserProfileData>('/v1/api/user/me', {
      headers: { Authorization: `Bearer ${accessToken}` },
    });
    return data;
  }

  /**
   * Change the authenticated user's password.
   * All existing sessions are revoked on success.
   *
   * `POST /v1/api/user/password/change`
   */
  async changePassword(accessToken: string, body: ChangePasswordRequest): Promise<void> {
    await this.req('/v1/api/user/password/change', {
      method: 'POST',
      body: JSON.stringify(body),
      headers: { Authorization: `Bearer ${accessToken}` },
    });
  }

  /**
   * List all sessions for the authenticated user.
   *
   * `GET /v1/api/user/sessions`
   */
  async getSessions(accessToken: string): Promise<UserSessionData[]> {
    const { data } = await this.req<UserSessionData[]>('/v1/api/user/sessions', {
      headers: { Authorization: `Bearer ${accessToken}` },
    });
    return data;
  }

  /**
   * Get details of a specific session.
   *
   * `GET /v1/api/user/sessions/{sessionId}`
   */
  async getSession(accessToken: string, sessionId: string): Promise<UserSessionDetailData> {
    const { data } = await this.req<UserSessionDetailData>(
      `/v1/api/user/sessions/${sessionId}`,
      { headers: { Authorization: `Bearer ${accessToken}` } },
    );
    return data;
  }

  /**
   * Get the roles and permissions assigned to the authenticated user.
   *
   * `GET /v1/api/user/permissions`
   */
  async getPermissions(accessToken: string): Promise<UserPermissionsData> {
    const { data } = await this.req<UserPermissionsData>('/v1/api/user/permissions', {
      headers: { Authorization: `Bearer ${accessToken}` },
    });
    return data;
  }
}
