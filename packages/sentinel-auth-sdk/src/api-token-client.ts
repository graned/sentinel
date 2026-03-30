import type {
  ApiTokenData,
  CreateApiTokenData,
  CreateApiTokenRequest,
  RequestFn,
} from './types';

/**
 * Long-lived API token management. All methods require a Bearer token from an
 * account that holds the **admin** role.
 *
 * ```ts
 * const { token } = await client.apiTokens.create(accessToken, {
 *   name: 'CI/CD pipeline',
 * });
 * // Store `token` (sat_<hex>) — it is returned exactly once.
 * ```
 */
export class ApiTokenClient {
  constructor(private readonly req: RequestFn) {}

  /**
   * Create a new API token. The raw `token` value in the response is returned
   * exactly once — it cannot be retrieved again after creation.
   *
   * `POST /v1/api/auth/api-tokens`  (admin)
   */
  async create(accessToken: string, body: CreateApiTokenRequest): Promise<CreateApiTokenData> {
    const { data } = await this.req<CreateApiTokenData>('/v1/api/auth/api-tokens', {
      method: 'POST',
      body: JSON.stringify(body),
      headers: { Authorization: `Bearer ${accessToken}` },
    });
    return data;
  }

  /**
   * List all API tokens (active and revoked) for the authenticated user's account.
   *
   * `GET /v1/api/auth/api-tokens`  (admin)
   */
  async list(accessToken: string): Promise<ApiTokenData[]> {
    const { data } = await this.req<ApiTokenData[]>('/v1/api/auth/api-tokens', {
      headers: { Authorization: `Bearer ${accessToken}` },
    });
    return data;
  }

  /**
   * Soft-revoke a specific token by ID. The token remains in the DB with
   * `revoked_at` set.
   *
   * `DELETE /v1/api/auth/api-tokens/{tokenId}`  (admin)
   *
   * @throws {ApiTokenNotFoundError} Token ID not found or already revoked.
   */
  async revoke(accessToken: string, tokenId: string): Promise<void> {
    await this.req(`/v1/api/auth/api-tokens/${tokenId}`, {
      method: 'DELETE',
      headers: { Authorization: `Bearer ${accessToken}` },
    });
  }

  /**
   * Revoke all API tokens for the authenticated user's account.
   *
   * `DELETE /v1/api/auth/api-tokens`  (admin)
   */
  async revokeAll(accessToken: string): Promise<void> {
    await this.req('/v1/api/auth/api-tokens', {
      method: 'DELETE',
      headers: { Authorization: `Bearer ${accessToken}` },
    });
  }
}
