import type {
  CreateProviderConfigRequest,
  DecryptedProviderConfigData,
  HealthData,
  InsightsSummaryData,
  ProviderConfigData,
  RequestFn,
  SessionActivityPoint,
  TestProviderConfigData,
  UpdateProviderConfigRequest,
  UserGrowthPoint,
} from './types';

/**
 * System-level endpoints.
 *
 * ```ts
 * const { status } = await client.system.health();
 * ```
 */
export class SystemClient {
  constructor(private readonly req: RequestFn) {}

  /**
   * Check server health.
   *
   * `GET /v1/api/system/health`
   */
  async health(): Promise<HealthData> {
    const { data } = await this.req<HealthData>('/v1/api/system/health');
    return data;
  }

  /**
   * List all provider configurations (redacted). Admin only.
   *
   * `GET /v1/api/system/config/email`
   */
  async listProviderConfigs(accessToken: string): Promise<ProviderConfigData[]> {
    const { data } = await this.req<ProviderConfigData[]>('/v1/api/system/config/email', {
      headers: { Authorization: `Bearer ${accessToken}` },
    });
    return data;
  }

  /**
   * Create a new provider configuration. Bearer token required.
   *
   * `POST /v1/api/system/config/email`
   */
  async createProviderConfig(
    accessToken: string,
    body: CreateProviderConfigRequest,
  ): Promise<ProviderConfigData> {
    const { data } = await this.req<ProviderConfigData>('/v1/api/system/config/email', {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${accessToken}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(body),
    });
    return data;
  }

  /**
   * Update an existing provider configuration. Admin only.
   *
   * `PUT /v1/api/system/config/email/:configId`
   */
  async updateProviderConfig(
    accessToken: string,
    configId: string,
    body: UpdateProviderConfigRequest,
  ): Promise<ProviderConfigData> {
    const { data } = await this.req<ProviderConfigData>(`/v1/api/system/config/email/${configId}`, {
      method: 'PUT',
      headers: {
        Authorization: `Bearer ${accessToken}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(body),
    });
    return data;
  }

  /**
   * Delete a provider configuration. Admin only.
   *
   * `DELETE /v1/api/system/config/email/:configId`
   */
  async deleteProviderConfig(accessToken: string, configId: string): Promise<void> {
    await this.req<void>(`/v1/api/system/config/email/${configId}`, {
      method: 'DELETE',
      headers: { Authorization: `Bearer ${accessToken}` },
    });
  }

  /**
   * Reveal the decrypted configuration for a provider. Admin only.
   *
   * `GET /v1/api/system/config/email/:configId/reveal`
   */
  async revealProviderConfig(
    accessToken: string,
    configId: string,
  ): Promise<DecryptedProviderConfigData> {
    const { data } = await this.req<DecryptedProviderConfigData>(
      `/v1/api/system/config/email/${configId}/reveal`,
      { headers: { Authorization: `Bearer ${accessToken}` } },
    );
    return data;
  }

  /**
   * Test the SMTP connection for a provider configuration. Admin only.
   *
   * `POST /v1/api/system/config/email/:configId/test`
   */
  async testProviderConfig(accessToken: string, configId: string): Promise<TestProviderConfigData> {
    const { data } = await this.req<TestProviderConfigData>(
      `/v1/api/system/config/email/${configId}/test`,
      {
        method: 'POST',
        headers: { Authorization: `Bearer ${accessToken}` },
      },
    );
    return data;
  }

  /**
   * Send a test email through a provider configuration. Admin only.
   *
   * `POST /v1/api/system/config/email/:configId/send-test`
   */
  async sendTestEmail(
    accessToken: string,
    configId: string,
    toEmail: string,
  ): Promise<TestProviderConfigData> {
    const { data } = await this.req<TestProviderConfigData>(
      `/v1/api/system/config/email/${configId}/send-test`,
      {
        method: 'POST',
        headers: { Authorization: `Bearer ${accessToken}`, 'Content-Type': 'application/json' },
        body: JSON.stringify({ to_email: toEmail }),
      },
    );
    return data;
  }

  // ── Insights / analytics ────────────────────────────────────────────────

  /**
   * Platform-wide KPI snapshot. Admin only.
   *
   * `GET /v1/api/system/stats`
   */
  async getInsightsSummary(accessToken: string): Promise<InsightsSummaryData> {
    const { data } = await this.req<InsightsSummaryData>('/v1/api/system/stats', {
      headers: { Authorization: `Bearer ${accessToken}` },
    });
    return data;
  }

  /**
   * Cumulative user-growth time series. Admin only.
   *
   * `GET /v1/api/system/analytics/user-growth?days=N`
   *
   * @param days Number of days to look back (1–365, default 30).
   */
  async getUserGrowth(accessToken: string, days = 30): Promise<UserGrowthPoint[]> {
    const { data } = await this.req<UserGrowthPoint[]>(
      `/v1/api/system/analytics/user-growth?days=${days}`,
      { headers: { Authorization: `Bearer ${accessToken}` } },
    );
    return data;
  }

  /**
   * Daily session-activity time series. Admin only.
   *
   * `GET /v1/api/system/analytics/sessions?days=N`
   *
   * @param days Number of days to look back (1–365, default 30).
   */
  async getSessionActivity(accessToken: string, days = 30): Promise<SessionActivityPoint[]> {
    const { data } = await this.req<SessionActivityPoint[]>(
      `/v1/api/system/analytics/sessions?days=${days}`,
      { headers: { Authorization: `Bearer ${accessToken}` } },
    );
    return data;
  }
}
