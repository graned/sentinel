import type {
  AdminCreateUserRequest,
  AdminSessionData,
  AdminUserData,
  BulkRevokeSessionsRequest,
  BulkRevokeSessionsResponse,
  InviteLinkData,
  PaginatedAdminUsersResponse,
  AssignRoleRequest,
  CreateEmailTemplateRequest,
  CreatePolicyData,
  CreatePolicyRequest,
  PolicyData,
  PolicyRulesData,
  CreateRoleRequest,
  EmailTemplateData,
  RequestFn,
  RoleData,
  RunProbeData,
  RunProbeRequest,
  UpdateEmailTemplateRequest,
  UpdateUserStatusRequest,
  UpdatePolicyRulesData,
  UpdatePolicyRulesRequest,
  UpdateRoleRequest,
  AdminSetMfaRequiredRequest,
  UserMfaStatusData,
  UserAuthInfoData,
  UserPermissionsData,
} from './types';

/**
 * Admin-only operations. Every method requires a Bearer token from an account
 * that holds the **admin** role.
 *
 * ```ts
 * const role = await client.admin.createRole(accessToken, {
 *   role_type: 'support',
 *   name: 'Support Agent',
 *   description: 'Customer support access',
 * });
 * ```
 */
export class AdminClient {
  constructor(private readonly req: RequestFn) {}

  // ---------------------------------------------------------------------------
  // Session management
  // ---------------------------------------------------------------------------

  /**
   * List all active (non-revoked, non-expired) sessions across all users.
   *
   * `GET /v1/api/admin/sessions`
   */
  async listActiveSessions(accessToken: string): Promise<AdminSessionData[]> {
    const { data } = await this.req<AdminSessionData[]>('/v1/api/admin/sessions', {
      headers: { Authorization: `Bearer ${accessToken}` },
    });
    return data;
  }

  /**
   * Invalidate a single session by ID.
   *
   * `DELETE /v1/api/admin/sessions/{sessionId}`
   */
  async revokeSession(accessToken: string, sessionId: string): Promise<void> {
    await this.req(`/v1/api/admin/sessions/${sessionId}`, {
      method: 'DELETE',
      headers: { Authorization: `Bearer ${accessToken}` },
    });
  }

  /**
   * Bulk-invalidate sessions by their IDs.
   *
   * `POST /v1/api/admin/sessions/revoke`
   */
  async revokeSessionsBulk(
    accessToken: string,
    body: BulkRevokeSessionsRequest,
  ): Promise<BulkRevokeSessionsResponse> {
    const { data } = await this.req<BulkRevokeSessionsResponse>(
      '/v1/api/admin/sessions/revoke',
      {
        method: 'POST',
        body: JSON.stringify(body),
        headers: { Authorization: `Bearer ${accessToken}` },
      },
    );
    return data;
  }

  // ---------------------------------------------------------------------------
  // Roles
  // ---------------------------------------------------------------------------

  /**
   * Create a new role.
   * `role_type` must be one of: `'user'`, `'admin'`, `'support'`.
   *
   * `POST /v1/api/admin/roles`
   */
  async createRole(accessToken: string, body: CreateRoleRequest): Promise<RoleData> {
    const { data } = await this.req<RoleData>('/v1/api/admin/roles', {
      method: 'POST',
      body: JSON.stringify(body),
      headers: { Authorization: `Bearer ${accessToken}` },
    });
    return data;
  }

  /**
   * List all roles defined in the system.
   *
   * `GET /v1/api/admin/roles`
   */
  async listRoles(accessToken: string): Promise<RoleData[]> {
    const { data } = await this.req<RoleData[]>('/v1/api/admin/roles', {
      headers: { Authorization: `Bearer ${accessToken}` },
    });
    return data;
  }

  /**
   * Update a role's name or description.
   *
   * `PUT /v1/api/admin/roles/{roleId}`
   */
  async updateRole(accessToken: string, roleId: string, body: UpdateRoleRequest): Promise<RoleData> {
    const { data } = await this.req<RoleData>(`/v1/api/admin/roles/${roleId}`, {
      method: 'PUT',
      body: JSON.stringify(body),
      headers: { Authorization: `Bearer ${accessToken}` },
    });
    return data;
  }

  /**
   * Delete a role.
   *
   * `DELETE /v1/api/admin/roles/{roleId}`
   */
  async deleteRole(accessToken: string, roleId: string): Promise<void> {
    await this.req(`/v1/api/admin/roles/${roleId}`, {
      method: 'DELETE',
      headers: { Authorization: `Bearer ${accessToken}` },
    });
  }

  // ---------------------------------------------------------------------------
  // User role management
  // ---------------------------------------------------------------------------

  /**
   * Assign a role to a user.
   *
   * `POST /v1/api/admin/users/{userId}/roles`
   */
  async assignRole(accessToken: string, userId: string, body: AssignRoleRequest): Promise<void> {
    await this.req(`/v1/api/admin/users/${userId}/roles`, {
      method: 'POST',
      body: JSON.stringify(body),
      headers: { Authorization: `Bearer ${accessToken}` },
    });
  }

  /**
   * Remove a named role from a user.
   *
   * `DELETE /v1/api/admin/users/{userId}/roles/{roleName}`
   */
  async removeRole(accessToken: string, userId: string, roleName: string): Promise<void> {
    await this.req(`/v1/api/admin/users/${userId}/roles/${roleName}`, {
      method: 'DELETE',
      headers: { Authorization: `Bearer ${accessToken}` },
    });
  }

  /**
   * Get roles and permissions for any user by ID.
   *
   * `GET /v1/api/admin/users/{userId}/permissions`
   */
  async getUserPermissions(accessToken: string, userId: string): Promise<UserPermissionsData> {
    const { data } = await this.req<UserPermissionsData>(
      `/v1/api/admin/users/${userId}/permissions`,
      { headers: { Authorization: `Bearer ${accessToken}` } },
    );
    return data;
  }

  /**
   * Get full profile + roles for any user by ID.
   *
   * `GET /v1/api/admin/users/{userId}/auth-info`
   */
  async getUserAuthInfo(accessToken: string, userId: string): Promise<UserAuthInfoData> {
    const { data } = await this.req<UserAuthInfoData>(
      `/v1/api/admin/users/${userId}/auth-info`,
      { headers: { Authorization: `Bearer ${accessToken}` } },
    );
    return data;
  }

  // ---------------------------------------------------------------------------
  // Policy management
  // ---------------------------------------------------------------------------

  /**
   * Create a new RBAC policy with an initial set of rules.
   *
   * `POST /v1/api/admin/policies`
   */
  async createPolicy(accessToken: string, body: CreatePolicyRequest): Promise<CreatePolicyData> {
    const { data } = await this.req<CreatePolicyData>('/v1/api/admin/policies', {
      method: 'POST',
      body: JSON.stringify(body),
      headers: { Authorization: `Bearer ${accessToken}` },
    });
    return data;
  }

  /**
   * Replace the rules for an existing policy. Creates a new compiled version
   * and activates it atomically.
   *
   * `PUT /v1/api/admin/policies/{policyId}/rules`
   */
  async updatePolicyRules(
    accessToken: string,
    policyId: string,
    body: UpdatePolicyRulesRequest,
  ): Promise<UpdatePolicyRulesData> {
    const { data } = await this.req<UpdatePolicyRulesData>(
      `/v1/api/admin/policies/${policyId}/rules`,
      {
        method: 'PUT',
        body: JSON.stringify(body),
        headers: { Authorization: `Bearer ${accessToken}` },
      },
    );
    return data;
  }

  /**
   * Delete a policy and all its versions.
   *
   * `DELETE /v1/api/admin/policies/{policyId}`
   */
  async deletePolicy(accessToken: string, policyId: string): Promise<void> {
    await this.req(`/v1/api/admin/policies/${policyId}`, {
      method: 'DELETE',
      headers: { Authorization: `Bearer ${accessToken}` },
    });
  }

  /**
   * List all RBAC policies.
   *
   * `GET /v1/api/admin/policies`
   */
  async listPolicies(accessToken: string): Promise<PolicyData[]> {
    const { data } = await this.req<PolicyData[]>('/v1/api/admin/policies', {
      headers: { Authorization: `Bearer ${accessToken}` },
    });
    return data;
  }

  /**
   * Get the active rules for a policy (from the active policy_version).
   *
   * `GET /v1/api/admin/policies/{policyId}/rules`
   */
  async getPolicyRules(accessToken: string, policyId: string): Promise<PolicyRulesData> {
    const { data } = await this.req<PolicyRulesData>(
      `/v1/api/admin/policies/${policyId}/rules`,
      { headers: { Authorization: `Bearer ${accessToken}` } },
    );
    return data;
  }

  /**
   * Run a live endpoint probe against the customer app.
   * Sentinel issues an internal test token, fans out HTTP calls for every rule
   * in the policy, and returns per-rule allow/deny results with HTTP status codes.
   *
   * Requires admin role. The customer app must be publicly accessible from the
   * Sentinel server. Use `checkAuthorizationBatch` for offline/localhost testing.
   *
   * `POST /v1/api/admin/policies/{policyId}/probe`
   */
  async runPolicyProbe(
    accessToken: string,
    policyId: string,
    body: RunProbeRequest,
  ): Promise<RunProbeData> {
    const { data } = await this.req<RunProbeData>(
      `/v1/api/admin/policies/${policyId}/probe`,
      {
        method: 'POST',
        body: JSON.stringify(body),
        headers: { Authorization: `Bearer ${accessToken}` },
      },
    );
    return data;
  }

  // ---------------------------------------------------------------------------
  // Email templates
  // ---------------------------------------------------------------------------

  /**
   * List all email templates (active and inactive).
   *
   * `GET /v1/api/admin/email-templates`
   */
  async listEmailTemplates(accessToken: string): Promise<EmailTemplateData[]> {
    const { data } = await this.req<EmailTemplateData[]>('/v1/api/admin/email-templates', {
      headers: { Authorization: `Bearer ${accessToken}` },
    });
    return data;
  }

  /**
   * Create a new email template for the given type.
   * Deactivates any previously active template of the same type.
   *
   * `POST /v1/api/admin/email-templates`
   */
  async createEmailTemplate(
    accessToken: string,
    body: CreateEmailTemplateRequest,
  ): Promise<EmailTemplateData> {
    const { data } = await this.req<EmailTemplateData>('/v1/api/admin/email-templates', {
      method: 'POST',
      body: JSON.stringify(body),
      headers: { Authorization: `Bearer ${accessToken}` },
    });
    return data;
  }

  /**
   * Update an existing email template.
   *
   * `PUT /v1/api/admin/email-templates/{templateId}`
   */
  async updateEmailTemplate(
    accessToken: string,
    templateId: string,
    body: UpdateEmailTemplateRequest,
  ): Promise<EmailTemplateData> {
    const { data } = await this.req<EmailTemplateData>(
      `/v1/api/admin/email-templates/${templateId}`,
      {
        method: 'PUT',
        body: JSON.stringify(body),
        headers: { Authorization: `Bearer ${accessToken}` },
      },
    );
    return data;
  }

  // ---------------------------------------------------------------------------
  // User management
  // ---------------------------------------------------------------------------

  /**
   * List users with server-side pagination.
   *
   * `GET /v1/api/admin/users`
   */
  async listUsers(
    accessToken: string,
    params?: { page?: number; page_size?: number },
  ): Promise<PaginatedAdminUsersResponse> {
    const qs = params
      ? '?' + new URLSearchParams(
          Object.entries(params)
            .filter(([, v]) => v !== undefined)
            .map(([k, v]) => [k, String(v)]),
        ).toString()
      : '';
    const { data } = await this.req<PaginatedAdminUsersResponse>(
      `/v1/api/admin/users${qs}`,
      { headers: { Authorization: `Bearer ${accessToken}` } },
    );
    return data;
  }

  /**
   * Create (invite) a new user as admin. The user is immediately active with
   * a pre-verified email and the default "user" role.
   *
   * `POST /v1/api/admin/users`
   */
  async createUser(accessToken: string, body: AdminCreateUserRequest): Promise<AdminUserData> {
    const { data } = await this.req<AdminUserData>('/v1/api/admin/users', {
      method: 'POST',
      body: JSON.stringify(body),
      headers: { Authorization: `Bearer ${accessToken}` },
    });
    return data;
  }

  /**
   * Delete a user by ID.
   *
   * `DELETE /v1/api/admin/users/{userId}`
   */
  async deleteUser(accessToken: string, userId: string): Promise<void> {
    await this.req(`/v1/api/admin/users/${userId}`, {
      method: 'DELETE',
      headers: { Authorization: `Bearer ${accessToken}` },
    });
  }

  /**
   * Update a user's status (active / suspended / inactive).
   *
   * `PUT /v1/api/admin/users/{userId}/status`
   */
  async updateUserStatus(
    accessToken: string,
    userId: string,
    body: UpdateUserStatusRequest,
  ): Promise<AdminUserData> {
    const { data } = await this.req<AdminUserData>(`/v1/api/admin/users/${userId}/status`, {
      method: 'PUT',
      body: JSON.stringify(body),
      headers: { Authorization: `Bearer ${accessToken}` },
    });
    return data;
  }

  /**
   * Send a verification/invite email to an admin-created user.
   * Returns an error if the user's email is already verified.
   *
   * `POST /v1/api/admin/users/{userId}/send-invite`
   */
  async sendUserInvite(accessToken: string, userId: string): Promise<void> {
    await this.req(`/v1/api/admin/users/${userId}/send-invite`, {
      method: 'POST',
      headers: { Authorization: `Bearer ${accessToken}` },
    });
  }

  /**
   * Generate an invite link for an admin-created user without sending an email.
   * The returned URL contains the raw verification token the user must visit.
   * Returns an error if the user's email is already verified.
   *
   * `GET /v1/api/admin/users/{userId}/invite-link`
   */
  async getUserInviteLink(accessToken: string, userId: string): Promise<InviteLinkData> {
    const { data } = await this.req<InviteLinkData>(
      `/v1/api/admin/users/${userId}/invite-link`,
      { headers: { Authorization: `Bearer ${accessToken}` } },
    );
    return data;
  }

  /**
   * Set or clear the admin-mandated MFA requirement for a user.
   * When `required` is `true`, all existing sessions for the user are revoked immediately.
   *
   * `PUT /v1/api/admin/users/{userId}/mfa`
   */
  async setMfaRequired(
    accessToken: string,
    userId: string,
    body: AdminSetMfaRequiredRequest,
  ): Promise<UserMfaStatusData> {
    const { data } = await this.req<UserMfaStatusData>(`/v1/api/admin/users/${userId}/mfa`, {
      method: 'PUT',
      body: JSON.stringify(body),
      headers: { Authorization: `Bearer ${accessToken}` },
    });
    return data;
  }
}
