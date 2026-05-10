// src/admin-client.ts
var AdminClient = class {
  constructor(req) {
    this.req = req;
  }
  // ---------------------------------------------------------------------------
  // Session management
  // ---------------------------------------------------------------------------
  /**
   * List all active (non-revoked, non-expired) sessions across all users.
   *
   * `GET /v1/api/admin/sessions`
   */
  async listActiveSessions(accessToken) {
    const { data } = await this.req("/v1/api/admin/sessions", {
      headers: { Authorization: `Bearer ${accessToken}` }
    });
    return data;
  }
  /**
   * Invalidate a single session by ID.
   *
   * `DELETE /v1/api/admin/sessions/{sessionId}`
   */
  async revokeSession(accessToken, sessionId) {
    await this.req(`/v1/api/admin/sessions/${sessionId}`, {
      method: "DELETE",
      headers: { Authorization: `Bearer ${accessToken}` }
    });
  }
  /**
   * Bulk-invalidate sessions by their IDs.
   *
   * `POST /v1/api/admin/sessions/revoke`
   */
  async revokeSessionsBulk(accessToken, body) {
    const { data } = await this.req(
      "/v1/api/admin/sessions/revoke",
      {
        method: "POST",
        body: JSON.stringify(body),
        headers: { Authorization: `Bearer ${accessToken}` }
      }
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
  async createRole(accessToken, body) {
    const { data } = await this.req("/v1/api/admin/roles", {
      method: "POST",
      body: JSON.stringify(body),
      headers: { Authorization: `Bearer ${accessToken}` }
    });
    return data;
  }
  /**
   * List all roles defined in the system.
   *
   * `GET /v1/api/admin/roles`
   */
  async listRoles(accessToken) {
    const { data } = await this.req("/v1/api/admin/roles", {
      headers: { Authorization: `Bearer ${accessToken}` }
    });
    return data;
  }
  /**
   * Update a role's name or description.
   *
   * `PUT /v1/api/admin/roles/{roleId}`
   */
  async updateRole(accessToken, roleId, body) {
    const { data } = await this.req(`/v1/api/admin/roles/${roleId}`, {
      method: "PUT",
      body: JSON.stringify(body),
      headers: { Authorization: `Bearer ${accessToken}` }
    });
    return data;
  }
  /**
   * Delete a role.
   *
   * `DELETE /v1/api/admin/roles/{roleId}`
   */
  async deleteRole(accessToken, roleId) {
    await this.req(`/v1/api/admin/roles/${roleId}`, {
      method: "DELETE",
      headers: { Authorization: `Bearer ${accessToken}` }
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
  async assignRole(accessToken, userId, body) {
    await this.req(`/v1/api/admin/users/${userId}/roles`, {
      method: "POST",
      body: JSON.stringify(body),
      headers: { Authorization: `Bearer ${accessToken}` }
    });
  }
  /**
   * Remove a named role from a user.
   *
   * `DELETE /v1/api/admin/users/{userId}/roles/{roleName}`
   */
  async removeRole(accessToken, userId, roleName) {
    await this.req(`/v1/api/admin/users/${userId}/roles/${roleName}`, {
      method: "DELETE",
      headers: { Authorization: `Bearer ${accessToken}` }
    });
  }
  /**
   * Get roles and permissions for any user by ID.
   *
   * `GET /v1/api/admin/users/{userId}/permissions`
   */
  async getUserPermissions(accessToken, userId) {
    const { data } = await this.req(
      `/v1/api/admin/users/${userId}/permissions`,
      { headers: { Authorization: `Bearer ${accessToken}` } }
    );
    return data;
  }
  /**
   * Get full profile + roles for any user by ID.
   *
   * `GET /v1/api/admin/users/{userId}/auth-info`
   */
  async getUserAuthInfo(accessToken, userId) {
    const { data } = await this.req(
      `/v1/api/admin/users/${userId}/auth-info`,
      { headers: { Authorization: `Bearer ${accessToken}` } }
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
  async createPolicy(accessToken, body) {
    const { data } = await this.req("/v1/api/admin/policies", {
      method: "POST",
      body: JSON.stringify(body),
      headers: { Authorization: `Bearer ${accessToken}` }
    });
    return data;
  }
  /**
   * Replace the rules for an existing policy. Creates a new compiled version
   * and activates it atomically.
   *
   * `PUT /v1/api/admin/policies/{policyId}/rules`
   */
  async updatePolicyRules(accessToken, policyId, body) {
    const { data } = await this.req(
      `/v1/api/admin/policies/${policyId}/rules`,
      {
        method: "PUT",
        body: JSON.stringify(body),
        headers: { Authorization: `Bearer ${accessToken}` }
      }
    );
    return data;
  }
  /**
   * Delete a policy and all its versions.
   *
   * `DELETE /v1/api/admin/policies/{policyId}`
   */
  async deletePolicy(accessToken, policyId) {
    await this.req(`/v1/api/admin/policies/${policyId}`, {
      method: "DELETE",
      headers: { Authorization: `Bearer ${accessToken}` }
    });
  }
  /**
   * List all RBAC policies.
   *
   * `GET /v1/api/admin/policies`
   */
  async listPolicies(accessToken) {
    const { data } = await this.req("/v1/api/admin/policies", {
      headers: { Authorization: `Bearer ${accessToken}` }
    });
    return data;
  }
  /**
   * Get the active rules for a policy (from the active policy_version).
   *
   * `GET /v1/api/admin/policies/{policyId}/rules`
   */
  async getPolicyRules(accessToken, policyId) {
    const { data } = await this.req(
      `/v1/api/admin/policies/${policyId}/rules`,
      { headers: { Authorization: `Bearer ${accessToken}` } }
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
  async runPolicyProbe(accessToken, policyId, body) {
    const { data } = await this.req(
      `/v1/api/admin/policies/${policyId}/probe`,
      {
        method: "POST",
        body: JSON.stringify(body),
        headers: { Authorization: `Bearer ${accessToken}` }
      }
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
  async listEmailTemplates(accessToken) {
    const { data } = await this.req("/v1/api/admin/email-templates", {
      headers: { Authorization: `Bearer ${accessToken}` }
    });
    return data;
  }
  /**
   * Create a new email template for the given type.
   * Deactivates any previously active template of the same type.
   *
   * `POST /v1/api/admin/email-templates`
   */
  async createEmailTemplate(accessToken, body) {
    const { data } = await this.req("/v1/api/admin/email-templates", {
      method: "POST",
      body: JSON.stringify(body),
      headers: { Authorization: `Bearer ${accessToken}` }
    });
    return data;
  }
  /**
   * Update an existing email template.
   *
   * `PUT /v1/api/admin/email-templates/{templateId}`
   */
  async updateEmailTemplate(accessToken, templateId, body) {
    const { data } = await this.req(
      `/v1/api/admin/email-templates/${templateId}`,
      {
        method: "PUT",
        body: JSON.stringify(body),
        headers: { Authorization: `Bearer ${accessToken}` }
      }
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
  async listUsers(accessToken, params) {
    const qs = params ? "?" + new URLSearchParams(
      Object.entries(params).filter(([, v]) => v !== void 0).map(([k, v]) => [k, String(v)])
    ).toString() : "";
    const { data } = await this.req(
      `/v1/api/admin/users${qs}`,
      { headers: { Authorization: `Bearer ${accessToken}` } }
    );
    return data;
  }
  /**
   * Create (invite) a new user as admin. The user is immediately active with
   * a pre-verified email and the default "user" role.
   *
   * `POST /v1/api/admin/users`
   */
  async createUser(accessToken, body) {
    const { data } = await this.req("/v1/api/admin/users", {
      method: "POST",
      body: JSON.stringify(body),
      headers: { Authorization: `Bearer ${accessToken}` }
    });
    return data;
  }
  /**
   * Delete a user by ID.
   *
   * `DELETE /v1/api/admin/users/{userId}`
   */
  async deleteUser(accessToken, userId) {
    await this.req(`/v1/api/admin/users/${userId}`, {
      method: "DELETE",
      headers: { Authorization: `Bearer ${accessToken}` }
    });
  }
  /**
   * Update a user's status (active / suspended / inactive).
   *
   * `PUT /v1/api/admin/users/{userId}/status`
   */
  async updateUserStatus(accessToken, userId, body) {
    const { data } = await this.req(`/v1/api/admin/users/${userId}/status`, {
      method: "PUT",
      body: JSON.stringify(body),
      headers: { Authorization: `Bearer ${accessToken}` }
    });
    return data;
  }
  /**
   * Send a verification/invite email to an admin-created user.
   * Returns an error if the user's email is already verified.
   *
   * `POST /v1/api/admin/users/{userId}/send-invite`
   */
  async sendUserInvite(accessToken, userId) {
    await this.req(`/v1/api/admin/users/${userId}/send-invite`, {
      method: "POST",
      headers: { Authorization: `Bearer ${accessToken}` }
    });
  }
  /**
   * Generate an invite link for an admin-created user without sending an email.
   * The returned URL contains the raw verification token the user must visit.
   * Returns an error if the user's email is already verified.
   *
   * `GET /v1/api/admin/users/{userId}/invite-link`
   */
  async getUserInviteLink(accessToken, userId) {
    const { data } = await this.req(
      `/v1/api/admin/users/${userId}/invite-link`,
      { headers: { Authorization: `Bearer ${accessToken}` } }
    );
    return data;
  }
  /**
   * Set or clear the admin-mandated MFA requirement for a user.
   * When `required` is `true`, all existing sessions for the user are revoked immediately.
   *
   * `PUT /v1/api/admin/users/{userId}/mfa`
   */
  async setMfaRequired(accessToken, userId, body) {
    const { data } = await this.req(`/v1/api/admin/users/${userId}/mfa`, {
      method: "PUT",
      body: JSON.stringify(body),
      headers: { Authorization: `Bearer ${accessToken}` }
    });
    return data;
  }
};

// src/api-token-client.ts
var ApiTokenClient = class {
  constructor(req) {
    this.req = req;
  }
  /**
   * Create a new API token. The raw `token` value in the response is returned
   * exactly once — it cannot be retrieved again after creation.
   *
   * `POST /v1/api/auth/api-tokens`  (admin)
   */
  async create(accessToken, body) {
    const { data } = await this.req("/v1/api/auth/api-tokens", {
      method: "POST",
      body: JSON.stringify(body),
      headers: { Authorization: `Bearer ${accessToken}` }
    });
    return data;
  }
  /**
   * List all API tokens (active and revoked) for the authenticated user's account.
   *
   * `GET /v1/api/auth/api-tokens`  (admin)
   */
  async list(accessToken) {
    const { data } = await this.req("/v1/api/auth/api-tokens", {
      headers: { Authorization: `Bearer ${accessToken}` }
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
  async revoke(accessToken, tokenId) {
    await this.req(`/v1/api/auth/api-tokens/${tokenId}`, {
      method: "DELETE",
      headers: { Authorization: `Bearer ${accessToken}` }
    });
  }
  /**
   * Revoke all API tokens for the authenticated user's account.
   *
   * `DELETE /v1/api/auth/api-tokens`  (admin)
   */
  async revokeAll(accessToken) {
    await this.req("/v1/api/auth/api-tokens", {
      method: "DELETE",
      headers: { Authorization: `Bearer ${accessToken}` }
    });
  }
};

// src/errors.ts
var SentinelError = class extends Error {
  constructor(code, message, statusCode, requestId, details) {
    super(message);
    this.name = "SentinelError";
    this.code = code;
    this.statusCode = statusCode;
    this.requestId = requestId;
    this.details = details;
    Object.setPrototypeOf(this, new.target.prototype);
  }
};
var AuthenticationError = class extends SentinelError {
  constructor(message, requestId) {
    super("AUTH_ERROR", message, 401, requestId);
    this.name = "AuthenticationError";
  }
};
var ValidationError = class extends SentinelError {
  constructor(message, requestId, details) {
    super("VALIDATION_ERROR", message, 400, requestId, details);
    this.name = "ValidationError";
  }
};
var InvalidTokenError = class extends SentinelError {
  constructor(message, requestId) {
    super("INVALID_TOKEN", message, 401, requestId);
    this.name = "InvalidTokenError";
  }
};
var ExpiredTokenError = class extends SentinelError {
  constructor(message, requestId) {
    super("EXPIRED_TOKEN", message, 401, requestId);
    this.name = "ExpiredTokenError";
  }
};
var MissingTokenError = class extends SentinelError {
  constructor(message, requestId) {
    super("MISSING_TOKEN", message, 401, requestId);
    this.name = "MissingTokenError";
  }
};
var RateLimitError = class extends SentinelError {
  constructor(message, requestId) {
    super("RATE_LIMIT_EXCEEDED", message, 429, requestId);
    this.name = "RateLimitError";
  }
};
var EmailNotVerifiedError = class extends SentinelError {
  constructor(message, requestId) {
    super("EMAIL_NOT_VERIFIED", message, 403, requestId);
    this.name = "EmailNotVerifiedError";
  }
};
var InternalServerError = class extends SentinelError {
  constructor(message, requestId) {
    super("INTERNAL_ERROR", message, 500, requestId);
    this.name = "InternalServerError";
  }
};
var NetworkError = class extends SentinelError {
  constructor(message, cause) {
    super("NETWORK_ERROR", message, 0, void 0, cause);
    this.name = "NetworkError";
  }
};
var SessionNotFoundError = class extends SentinelError {
  constructor(userId) {
    super("SESSION_NOT_FOUND", `No cached session for user "${userId}"`, 0);
    this.name = "SessionNotFoundError";
  }
};
var MfaInvalidCodeError = class extends SentinelError {
  constructor(message, requestId) {
    super("INVALID_MFA_CODE", message, 401, requestId);
    this.name = "MfaInvalidCodeError";
  }
};
var MfaAttemptLimitError = class extends SentinelError {
  constructor(message, requestId) {
    super("MFA_ATTEMPT_LIMIT_EXCEEDED", message, 429, requestId);
    this.name = "MfaAttemptLimitError";
  }
};
var ApiTokenNotFoundError = class extends SentinelError {
  constructor(message, requestId) {
    super("API_TOKEN_NOT_FOUND", message, 404, requestId);
    this.name = "ApiTokenNotFoundError";
  }
};
var ForbiddenError = class extends SentinelError {
  constructor(message, requestId) {
    super("FORBIDDEN", message, 403, requestId);
    this.name = "ForbiddenError";
  }
};
function createErrorFromCode(code, message, statusCode, requestId, details) {
  switch (code) {
    case "AUTH_ERROR":
      return new AuthenticationError(message, requestId);
    case "VALIDATION_ERROR":
      return new ValidationError(message, requestId, details);
    case "INVALID_TOKEN":
      return new InvalidTokenError(message, requestId);
    case "EXPIRED_TOKEN":
      return new ExpiredTokenError(message, requestId);
    case "MISSING_TOKEN":
      return new MissingTokenError(message, requestId);
    case "RATE_LIMIT_EXCEEDED":
      return new RateLimitError(message, requestId);
    case "EMAIL_NOT_VERIFIED":
      return new EmailNotVerifiedError(message, requestId);
    case "INTERNAL_ERROR":
      return new InternalServerError(message, requestId);
    case "INVALID_MFA_CODE":
      return new MfaInvalidCodeError(message, requestId);
    case "MFA_ATTEMPT_LIMIT_EXCEEDED":
      return new MfaAttemptLimitError(message, requestId);
    case "API_TOKEN_NOT_FOUND":
      return new ApiTokenNotFoundError(message, requestId);
    case "FORBIDDEN":
      return new ForbiddenError(message, requestId);
    default:
      return new SentinelError(code, message, statusCode, requestId, details);
  }
}

// src/mfa-client.ts
var MfaClient = class {
  constructor(req, toSession, cacheSession) {
    this.req = req;
    this.toSession = toSession;
    this.cacheSession = cacheSession;
  }
  /**
   * Begin TOTP enrollment — returns an `otpauth://` URI to display as a QR code.
   * Bearer token required (user must be logged in).
   *
   * `POST /v1/api/auth/mfa/totp/start`
   */
  async totpStart(accessToken) {
    const { data } = await this.req("/v1/api/auth/mfa/totp/start", {
      method: "POST",
      headers: { Authorization: `Bearer ${accessToken}` }
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
  async totpConfirm(accessToken, body) {
    const { data } = await this.req("/v1/api/auth/mfa/totp/confirm", {
      method: "POST",
      body: JSON.stringify(body),
      headers: { Authorization: `Bearer ${accessToken}` }
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
  async verify(body) {
    const { data } = await this.req("/v1/api/auth/mfa/verify", {
      method: "POST",
      body: JSON.stringify(body)
    });
    const session = this.toSession(data);
    this.cacheSession(session);
    return session;
  }
};

// src/session.ts
var DEFAULT_REFRESH_BUFFER_MS = 5 * 60 * 1e3;
var SessionCache = class {
  constructor() {
    this.sessions = /* @__PURE__ */ new Map();
  }
  /** Store (or overwrite) a session for the given user. */
  set(userId, session) {
    this.sessions.set(userId, session);
  }
  /** Retrieve the stored session, or `undefined` if none exists. */
  get(userId) {
    return this.sessions.get(userId);
  }
  /** Remove the session for the given user. No-op if not present. */
  delete(userId) {
    this.sessions.delete(userId);
  }
  /** Returns `true` if a session (expired or not) is stored for the user. */
  has(userId) {
    return this.sessions.has(userId);
  }
  /** Returns `true` if the session's access token has already expired. */
  isExpired(session) {
    return Date.now() >= session.expiresAt.getTime();
  }
  /**
   * Returns `true` if the session will expire within `bufferMs` milliseconds.
   * Use this to decide whether a proactive refresh is worthwhile before
   * returning the session to a caller.
   *
   * @param bufferMs - Look-ahead window in ms. Defaults to 5 minutes.
   */
  isExpiringSoon(session, bufferMs = DEFAULT_REFRESH_BUFFER_MS) {
    return Date.now() >= session.expiresAt.getTime() - bufferMs;
  }
  /** Number of sessions currently cached (including any that may be expired). */
  get size() {
    return this.sessions.size;
  }
  /** Remove all cached sessions. */
  clear() {
    this.sessions.clear();
  }
  /**
   * Evict every session whose access token has already expired.
   * Returns the number of sessions removed.
   */
  evictExpired() {
    let removed = 0;
    for (const [userId, session] of this.sessions) {
      if (this.isExpired(session)) {
        this.sessions.delete(userId);
        removed++;
      }
    }
    return removed;
  }
};

// src/system-client.ts
var SystemClient = class {
  constructor(req) {
    this.req = req;
  }
  /**
   * Check server health.
   *
   * `GET /v1/api/system/health`
   */
  async health() {
    const { data } = await this.req("/v1/api/system/health");
    return data;
  }
  /**
   * List all provider configurations (redacted). Admin only.
   *
   * `GET /v1/api/system/config/email`
   */
  async listProviderConfigs(accessToken) {
    const { data } = await this.req("/v1/api/system/config/email", {
      headers: { Authorization: `Bearer ${accessToken}` }
    });
    return data;
  }
  /**
   * Create a new provider configuration. Bearer token required.
   *
   * `POST /v1/api/system/config/email`
   */
  async createProviderConfig(accessToken, body) {
    const { data } = await this.req("/v1/api/system/config/email", {
      method: "POST",
      headers: {
        Authorization: `Bearer ${accessToken}`,
        "Content-Type": "application/json"
      },
      body: JSON.stringify(body)
    });
    return data;
  }
  /**
   * Update an existing provider configuration. Admin only.
   *
   * `PUT /v1/api/system/config/email/:configId`
   */
  async updateProviderConfig(accessToken, configId, body) {
    const { data } = await this.req(
      `/v1/api/system/config/email/${configId}`,
      {
        method: "PUT",
        headers: {
          Authorization: `Bearer ${accessToken}`,
          "Content-Type": "application/json"
        },
        body: JSON.stringify(body)
      }
    );
    return data;
  }
  /**
   * Delete a provider configuration. Admin only.
   *
   * `DELETE /v1/api/system/config/email/:configId`
   */
  async deleteProviderConfig(accessToken, configId) {
    await this.req(`/v1/api/system/config/email/${configId}`, {
      method: "DELETE",
      headers: { Authorization: `Bearer ${accessToken}` }
    });
  }
  /**
   * Reveal the decrypted configuration for a provider. Admin only.
   *
   * `GET /v1/api/system/config/email/:configId/reveal`
   */
  async revealProviderConfig(accessToken, configId) {
    const { data } = await this.req(
      `/v1/api/system/config/email/${configId}/reveal`,
      { headers: { Authorization: `Bearer ${accessToken}` } }
    );
    return data;
  }
  /**
   * Test the SMTP connection for a provider configuration. Admin only.
   *
   * `POST /v1/api/system/config/email/:configId/test`
   */
  async testProviderConfig(accessToken, configId) {
    const { data } = await this.req(
      `/v1/api/system/config/email/${configId}/test`,
      {
        method: "POST",
        headers: { Authorization: `Bearer ${accessToken}` }
      }
    );
    return data;
  }
  /**
   * Send a test email through a provider configuration. Admin only.
   *
   * `POST /v1/api/system/config/email/:configId/send-test`
   */
  async sendTestEmail(accessToken, configId, toEmail) {
    const { data } = await this.req(
      `/v1/api/system/config/email/${configId}/send-test`,
      {
        method: "POST",
        headers: { Authorization: `Bearer ${accessToken}`, "Content-Type": "application/json" },
        body: JSON.stringify({ to_email: toEmail })
      }
    );
    return data;
  }
  // ── Insights / analytics ────────────────────────────────────────────────
  /**
   * Platform-wide KPI snapshot. Admin only.
   *
   * `GET /v1/api/system/stats`
   */
  async getInsightsSummary(accessToken) {
    const { data } = await this.req("/v1/api/system/stats", {
      headers: { Authorization: `Bearer ${accessToken}` }
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
  async getUserGrowth(accessToken, days = 30) {
    const { data } = await this.req(
      `/v1/api/system/analytics/user-growth?days=${days}`,
      { headers: { Authorization: `Bearer ${accessToken}` } }
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
  async getSessionActivity(accessToken, days = 30) {
    const { data } = await this.req(
      `/v1/api/system/analytics/sessions?days=${days}`,
      { headers: { Authorization: `Bearer ${accessToken}` } }
    );
    return data;
  }
};

// src/user-client.ts
var UserClient = class {
  constructor(req) {
    this.req = req;
  }
  /**
   * Fetch the profile of the currently authenticated user.
   *
   * `GET /v1/api/user/me`
   */
  async getMe(accessToken) {
    const { data } = await this.req("/v1/api/user/me", {
      headers: { Authorization: `Bearer ${accessToken}` }
    });
    return data;
  }
  /**
   * Update the authenticated user's profile fields.
   * Only the provided fields are changed; omitted fields are left untouched.
   *
   * `PATCH /v1/api/user/me`
   */
  async updateProfile(accessToken, body) {
    const { data } = await this.req("/v1/api/user/me", {
      method: "PATCH",
      body: JSON.stringify(body),
      headers: { Authorization: `Bearer ${accessToken}` }
    });
    return data;
  }
  /**
   * Change the authenticated user's password.
   * All existing sessions are revoked on success.
   *
   * `POST /v1/api/user/password/change`
   */
  async changePassword(accessToken, body) {
    await this.req("/v1/api/user/password/change", {
      method: "POST",
      body: JSON.stringify(body),
      headers: { Authorization: `Bearer ${accessToken}` }
    });
  }
  /**
   * List all sessions for the authenticated user.
   *
   * `GET /v1/api/user/sessions`
   */
  async getSessions(accessToken) {
    const { data } = await this.req("/v1/api/user/sessions", {
      headers: { Authorization: `Bearer ${accessToken}` }
    });
    return data;
  }
  /**
   * Get details of a specific session.
   *
   * `GET /v1/api/user/sessions/{sessionId}`
   */
  async getSession(accessToken, sessionId) {
    const { data } = await this.req(
      `/v1/api/user/sessions/${sessionId}`,
      { headers: { Authorization: `Bearer ${accessToken}` } }
    );
    return data;
  }
  /**
   * Get the roles and permissions assigned to the authenticated user.
   *
   * `GET /v1/api/user/permissions`
   */
  async getPermissions(accessToken) {
    const { data } = await this.req("/v1/api/user/permissions", {
      headers: { Authorization: `Bearer ${accessToken}` }
    });
    return data;
  }
};

// src/client.ts
var DEFAULT_REFRESH_BUFFER_MS2 = 5 * 60 * 1e3;
function isMfaChallenge(data) {
  return data.mfa_required === true;
}
var SentinelAuthClient = class {
  constructor(config) {
    this.config = config;
    this.baseUrl = config.baseUrl.replace(/\/$/, "");
    this.refreshBufferMs = config.refreshBufferMs ?? DEFAULT_REFRESH_BUFFER_MS2;
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
  async register(body) {
    const { data } = await this.request("/v1/api/auth/register", {
      method: "POST",
      body: JSON.stringify(body)
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
  async login(credentials) {
    const { data } = await this.request("/v1/api/auth/login", {
      method: "POST",
      body: JSON.stringify(credentials)
    });
    if (isMfaChallenge(data)) {
      return {
        type: "mfa_challenge",
        userId: data.user_id,
        mfaSessionToken: data.mfa_session_token
      };
    }
    const session = this.toSession(data);
    this.cache.set(session.userId, session);
    return { type: "session", session, mustChangePassword: data.must_change_password, mfaSetupRequired: data.mfa_setup_required ?? false };
  }
  /**
   * Log out the user, revoking only the current session.
   * The cached session is removed regardless of whether the server call
   * succeeds. No-op if no session is cached for `userId`.
   *
   * `POST /v1/api/auth/logout`
   */
  async logout(userId) {
    const session = this.cache.get(userId);
    this.cache.delete(userId);
    if (!session) return;
    await this.request("/v1/api/auth/logout", {
      method: "POST",
      headers: { Authorization: `Bearer ${session.accessToken}` }
    });
  }
  /**
   * Revoke **all** sessions for the authenticated user (sign out everywhere).
   * The local cached session is also removed.
   *
   * `POST /v1/api/auth/logout-all`
   */
  async logoutAll(userId) {
    const session = this.cache.get(userId);
    this.cache.delete(userId);
    if (!session) return;
    await this.request("/v1/api/auth/logout-all", {
      method: "POST",
      headers: { Authorization: `Bearer ${session.accessToken}` }
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
  async refreshSession(userId, refreshToken) {
    const tokenToUse = refreshToken ?? this.cache.get(userId)?.refreshToken;
    if (!tokenToUse) throw new SessionNotFoundError(userId);
    const body = { refresh_token: tokenToUse };
    const { data } = await this.request("/v1/api/auth/token/refresh", {
      method: "POST",
      body: JSON.stringify(body)
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
  async authenticate(body) {
    const { data } = await this.request("/v1/api/auth/authenticate", {
      method: "POST",
      headers: { Authorization: `Bearer ${body.access_token}` }
    });
    return data;
  }
  /**
   * Check whether a given method + path is allowed for a set of roles
   * under the specified policy.
   *
   * `POST /v1/api/auth/token/authorize`
   */
  async checkAuthorization(body) {
    const { data } = await this.request("/v1/api/auth/token/authorize", {
      method: "POST",
      body: JSON.stringify(body)
    });
    return data;
  }
  /**
   * Evaluate multiple method+path checks against a policy in a single request.
   * Uses the compiled in-memory trie engine (~200 ns per check). No customer app needed.
   *
   * `POST /v1/api/auth/token/authorize/batch`
   */
  async checkAuthorizationBatch(body) {
    const { data } = await this.request("/v1/api/auth/token/authorize/batch", {
      method: "POST",
      body: JSON.stringify(body)
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
  async authenticateAndAuthorize(body) {
    const auth = await this.authenticate({ access_token: body.access_token });
    const authorization = await this.checkAuthorization({
      policy_id: body.policy_id,
      method: body.method,
      path: body.path,
      roles: auth.roles
    });
    if (!authorization.allowed) {
      throw new ForbiddenError(
        `Access denied: ${body.method} ${body.path} is not permitted for the current roles.`
      );
    }
    return { auth, authorization };
  }
  /**
   * Verify an email address using the raw token from the verification link.
   *
   * `GET /v1/api/auth/verify-email?token=<raw>`
   */
  async verifyEmail(token) {
    await this.request(`/v1/api/auth/verify-email?token=${encodeURIComponent(token)}`);
  }
  /**
   * Trigger a new verification email for the given address.
   * Rate-limited to 10 requests / 15 min per IP.
   *
   * `POST /v1/api/auth/resend-verification`
   */
  async resendVerification(body) {
    await this.request("/v1/api/auth/resend-verification", {
      method: "POST",
      body: JSON.stringify(body)
    });
  }
  /**
   * Retrieve the authentication methods configured on this Sentinel instance
   * (password auth, MFA, API tokens, OIDC clients, etc.).
   *
   * `GET /v1/api/auth/auth-methods`
   */
  async getAuthMethods() {
    const { data } = await this.request("/v1/api/auth/auth-methods");
    return data;
  }
  /**
   * Initiate a password reset flow by sending a reset link to the given email.
   * Always returns successfully — the server silently no-ops for unknown emails
   * to prevent user enumeration.
   *
   * `POST /v1/api/auth/password/forgot`
   */
  async forgotPassword(body) {
    await this.request("/v1/api/auth/password/forgot", {
      method: "POST",
      body: JSON.stringify(body)
    });
  }
  /**
   * Reset a password using the token from the reset email.
   * All existing sessions are revoked on success.
   *
   * `POST /v1/api/auth/password/reset`
   */
  async resetPassword(body) {
    await this.request("/v1/api/auth/password/reset", {
      method: "POST",
      body: JSON.stringify(body)
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
  async getValidSession(userId) {
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
  getSession(userId) {
    return this.cache.get(userId);
  }
  /** Remove the cached session for `userId` without calling the server. */
  clearSession(userId) {
    this.cache.delete(userId);
  }
  /** Remove all cached sessions without calling the server. */
  clearAllSessions() {
    this.cache.clear();
  }
  /**
   * Evict every expired session from the cache.
   * Returns the number of sessions removed.
   */
  evictExpiredSessions() {
    return this.cache.evictExpired();
  }
  // ---------------------------------------------------------------------------
  // Internal helpers
  // ---------------------------------------------------------------------------
  async request(path, options = {}) {
    let response;
    try {
      response = await fetch(`${this.baseUrl}${path}`, {
        ...options,
        headers: {
          "Content-Type": "application/json",
          ...this.config.headers ?? {},
          ...options.headers ?? {}
        }
      });
    } catch (cause) {
      throw new NetworkError(
        "Unable to reach the Sentinel Auth server. Check your network connection.",
        cause
      );
    }
    let envelope;
    try {
      envelope = await response.json();
    } catch {
      throw new SentinelError(
        "INVALID_RESPONSE",
        `Server returned non-JSON response (HTTP ${response.status})`,
        response.status
      );
    }
    const requestId = envelope.request_id;
    if (!envelope.success || envelope.error) {
      const err = envelope.error;
      throw createErrorFromCode(
        err?.code ?? "INTERNAL_ERROR",
        err?.message ?? "An unexpected error occurred",
        response.status,
        requestId,
        err?.details
      );
    }
    if (envelope.data === void 0) {
      throw new SentinelError(
        "EMPTY_RESPONSE",
        "Server returned a successful response with no data",
        response.status,
        requestId
      );
    }
    return { data: envelope.data, requestId };
  }
  toSession(data) {
    return {
      userId: data.user_id,
      accessToken: data.access_token,
      refreshToken: data.refresh_token,
      expiresAt: new Date(data.expires_at)
    };
  }
};

// src/middleware/express.ts
function extractBearer(req) {
  const header = req.headers["authorization"];
  const raw = Array.isArray(header) ? header[0] : header;
  if (!raw || !raw.startsWith("Bearer ")) return null;
  return raw.slice(7);
}
function sentinelExpressMiddleware(options) {
  return async (req, res, next) => {
    const token = extractBearer(req);
    if (!token) {
      next();
      return;
    }
    let auth;
    try {
      auth = await options.client.authenticate({ access_token: token });
    } catch {
      res.status(401).json({ error: "UNAUTHORIZED" });
      return;
    }
    const policyId = auth.scope === "policy_test" ? auth.policy_test_id : options.policyId ?? void 0;
    let authz;
    try {
      authz = await options.client.checkAuthorization({
        method: req.method,
        path: req.path,
        policy_id: policyId,
        roles: auth.roles
      });
    } catch {
      res.status(403).json({ error: "FORBIDDEN" });
      return;
    }
    if (!authz.allowed) {
      if (auth.scope === "policy_test") {
        res.status(403).json({
          sentinel_probe: true,
          allowed: false,
          method: req.method,
          path: req.path,
          roles: auth.roles,
          policy_id: policyId,
          active_version: authz.active_version
        });
      } else {
        res.status(403).json({ error: "FORBIDDEN" });
      }
      return;
    }
    if (auth.scope === "policy_test") {
      res.status(200).json({
        sentinel_probe: true,
        allowed: true,
        method: req.method,
        path: req.path,
        roles: auth.roles,
        policy_id: policyId,
        active_version: authz.active_version
      });
      return;
    }
    req.sentinelAuth = auth;
    next();
  };
}
export {
  AdminClient,
  ApiTokenClient,
  ApiTokenNotFoundError,
  AuthenticationError,
  EmailNotVerifiedError,
  ExpiredTokenError,
  ForbiddenError,
  InternalServerError,
  InvalidTokenError,
  MfaAttemptLimitError,
  MfaClient,
  MfaInvalidCodeError,
  MissingTokenError,
  NetworkError,
  RateLimitError,
  SentinelAuthClient,
  SentinelError,
  SessionCache,
  SessionNotFoundError,
  SystemClient,
  UserClient,
  ValidationError,
  createErrorFromCode,
  sentinelExpressMiddleware
};
//# sourceMappingURL=index.mjs.map