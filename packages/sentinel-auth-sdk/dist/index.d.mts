interface LoginRequest {
    email: string;
    password: string;
}
interface RefreshTokenRequest {
    refresh_token: string;
}
interface RegisterRequest {
    first_name: string;
    last_name: string;
    email: string;
    avatar_url?: string | null;
    /** Must be ≥12 chars with upper, lower, digit, and special character. */
    password: string;
}
interface AuthenticateRequest {
    access_token: string;
}
interface ResendVerificationRequest {
    email: string;
}
interface ForgotPasswordRequest {
    email: string;
}
interface ResetPasswordRequest {
    token: string;
    /** Must be ≥12 chars with upper, lower, digit, and special character. */
    new_password: string;
}
interface ChangePasswordRequest {
    current_password: string;
    /** Must be ≥12 chars with upper, lower, digit, and special character. */
    new_password: string;
}
interface UpdateProfileRequest {
    /** New first name (1–100 chars when provided). */
    first_name?: string;
    /** New last name (1–100 chars when provided). */
    last_name?: string;
    /** New avatar URL. */
    avatar_url?: string;
}
interface CheckAuthorizationRequest {
    policy_id?: string;
    method: string;
    path: string;
    roles: string[];
}
interface AuthenticateAndAuthorizeRequest {
    /** PASETO access token to validate. */
    access_token: string;
    /** HTTP method to check (e.g. `"GET"`, `"POST"`). */
    method: string;
    /** Resource path to check (e.g. `"/v1/api/user/me"`). */
    path: string;
    /** Optional policy ID. Omit to use the server's default active policy. */
    policy_id?: string;
}
interface MfaTotpConfirmRequest {
    /** Exactly 6 digits from the authenticator app. */
    code: string;
}
interface MfaVerifyRequest {
    /** Short-lived PASETO token returned by the login MFA challenge. */
    mfa_session_token: string;
    /** 6-digit TOTP code or an 8-char recovery code. */
    code: string;
}
interface CreateApiTokenRequest {
    name: string;
    description?: string;
    /** ISO 8601 UTC expiry. Omit for a non-expiring token. */
    expires_at?: string;
}
interface CreateRoleRequest {
    /** Must be one of: 'user', 'admin', 'support'. */
    role_type: string;
    name: string;
    description: string;
}
interface UpdateRoleRequest {
    name?: string;
    description?: string;
}
interface AssignRoleRequest {
    role_id: string;
}
interface CreatePolicyRequest {
    name: string;
    environment: string;
    description?: string;
    tenant_id?: string;
    rules: PolicyRule[];
}
interface PolicyRule {
    method: string;
    path: string;
    roles: string[];
}
interface UpdatePolicyRulesRequest {
    rules: PolicyRule[];
}
interface CreateEmailTemplateRequest {
    template_type: EmailTemplateType;
    subject: string;
    body_text: string;
    body_html?: string;
}
interface UpdateEmailTemplateRequest {
    subject?: string;
    body_text?: string;
    body_html?: string;
    is_active?: boolean;
}
interface CreateProviderConfigRequest {
    provider: string;
    config: Record<string, unknown>;
    is_active: boolean;
    tenant_id?: string;
}
interface UpdateProviderConfigRequest {
    config: Record<string, unknown>;
    is_active: boolean;
}
interface SendTestEmailRequest {
    to_email: string;
}
type EmailTemplateType = 'EmailVerification' | 'PasswordReset' | 'PasswordChanged';
/** Returned when login succeeds without MFA. */
interface BasicLoginData {
    user_id: string;
    access_token: string;
    refresh_token: string;
    /** ISO 8601 UTC timestamp — expiry of the access token. */
    expires_at: string;
    /**
     * True when the account was created by an admin with a temporary password.
     * The user must change their password before accessing other endpoints.
     */
    must_change_password: boolean;
    /**
     * True when an admin has mandated MFA for the user but the user has not yet
     * enrolled a TOTP authenticator. The session is fully valid — the user
     * should be redirected to complete MFA setup before accessing other areas.
     */
    mfa_setup_required: boolean;
}
/** Returned when the user has MFA enabled; a second step is required. */
interface MfaChallengeData {
    user_id: string;
    /** Always `true` — use this field to discriminate the union. */
    mfa_required: true;
    /** Short-lived PASETO token (5-min TTL). Pass it to the MFA verify endpoint. */
    mfa_session_token: string;
}
type LoginData = BasicLoginData | MfaChallengeData;
interface RegisterData {
    user_id: string;
    first_name: string;
    last_name: string;
    avatar_url: string | null;
    /** UserStatus enum value from the server (e.g. 'Active', 'PendingVerification'). */
    status: string;
}
interface AuthContextData {
    user_id: string;
    session_id: string;
    roles: string[];
    email_verified: boolean;
    /** Present only for policy-test tokens (`scope: "policy_test"`). */
    scope?: 'policy_test';
    /** Policy ID embedded in a test token. Present when `scope === "policy_test"`. */
    policy_test_id?: string;
}
interface BatchCheckItem {
    method: string;
    path: string;
}
interface BatchCheckRequest {
    /** Optional. Omit to use the server's default active policy. */
    policy_id?: string;
    roles: string[];
    /** Max 500 items. */
    checks: BatchCheckItem[];
}
interface BatchCheckResult {
    method: string;
    path: string;
    allowed: boolean;
}
interface BatchCheckData {
    policy_id?: string;
    evaluated_version: number;
    results: BatchCheckResult[];
}
interface RunProbeRequest {
    /** Base URL of the customer app (e.g. `https://api.myapp.com`). */
    base_url: string;
    roles: string[];
}
interface ProbeRuleResult {
    method: string;
    path: string;
    allowed: boolean;
    status_code?: number;
    /** Set when the customer app is unreachable (connection failure, timeout). */
    error?: string;
}
interface RunProbeData {
    policy_id: string;
    evaluated_version: number;
    roles_tested: string[];
    base_url: string;
    results: ProbeRuleResult[];
}
interface OidcClientInfo {
    client_id: string;
    name: string;
    allowed_scopes: string[];
    pkce_required: boolean;
}
interface AuthMethodsData {
    password_enabled: boolean;
    mfa_totp_available: boolean;
    api_tokens_available: boolean;
    email_verification_required: boolean;
    email_provider_active: boolean;
    oidc_enabled: boolean;
    oidc_clients: OidcClientInfo[];
}
interface CheckAuthorizationData {
    allowed: boolean;
    method: string;
    path: string;
    roles: string[];
    active_version: number;
}
/** Combined result returned by `SentinelAuthClient.authenticateAndAuthorize()`. */
interface AuthenticateAndAuthorizeData {
    /** Validated token context: user ID, session ID, roles, and email verification status. */
    auth: AuthContextData;
    /** Policy evaluation result confirming the action was permitted. */
    authorization: CheckAuthorizationData;
}
interface UserProfileData {
    user_id: string;
    first_name: string | null;
    last_name: string | null;
    avatar_url: string | null;
    status: string;
    email: string;
    email_verified: boolean;
    mfa_enabled: boolean;
    created_at: string | null;
}
interface UserSessionData {
    session_id: string;
    user_agent: string | null;
    ip_address: string | null;
    device_type: string | null;
    last_used_at: string | null;
    created_at: string | null;
    expires_at: string;
    is_current: boolean;
}
interface UserSessionDetailData extends UserSessionData {
    revoked_at: string | null;
    is_active: boolean;
}
interface AdminSessionData {
    session_id: string;
    user_id: string;
    user_email: string;
    user_agent: string | null;
    ip_address: string | null;
    device_type: string | null;
    last_used_at: string | null;
    created_at: string | null;
    expires_at: string;
}
interface BulkRevokeSessionsRequest {
    session_ids: string[];
}
interface BulkRevokeSessionsResponse {
    revoked_count: number;
}
interface RoleData {
    role_id: string;
    name: string;
    role_type: string;
    description: string;
}
interface UserPermissionsData {
    user_id: string;
    roles: RoleData[];
}
interface MfaTotpStartData {
    /** `otpauth://` URI — encode as a QR code for the authenticator app. */
    otpauth_uri: string;
}
interface MfaTotpConfirmData {
    /** One-time recovery codes — store securely, shown only once. */
    recovery_codes: string[];
}
interface CreateApiTokenData {
    api_token_id: string;
    /** Raw token in `sat_<hex>` format — returned exactly once. Store securely. */
    token: string;
    name: string;
    expires_at: string | null;
    created_at: string;
}
interface ApiTokenData {
    api_token_id: string;
    name: string;
    description: string | null;
    expires_at: string | null;
    last_used_at: string | null;
    revoked_at: string | null;
    created_at: string;
}
interface UserAuthInfoData extends UserProfileData {
    roles: RoleData[];
}
interface CreatePolicyData {
    policy_id: string;
    name: string;
    environment: string;
    description: string | null;
    active_version: number;
    created_at: string;
}
interface UpdatePolicyRulesData {
    policy_id: string;
    activated_version: number;
}
interface PolicyData {
    policy_id: string;
    tenant_id: string | null;
    name: string;
    environment: string;
    description: string | null;
    active_version: number;
    created_at: string;
    updated_at: string;
}
interface PolicyRulesData {
    policy_id: string;
    version: number;
    rules: PolicyRule[];
}
interface EmailTemplateData {
    template_id: string;
    template_type: EmailTemplateType;
    subject: string;
    body_text: string;
    body_html: string | null;
    is_active: boolean;
    created_at: string;
    updated_at: string | null;
}
interface HealthData {
    status: string;
}
interface ProviderConfigData {
    configuration_id: string;
    tenant_id: string | null;
    provider: string;
    config_redacted: Record<string, unknown>;
    is_active: boolean;
    created_at: string;
    updated_at: string;
}
interface DecryptedProviderConfigData {
    configuration_id: string;
    provider: string;
    config: Record<string, unknown>;
    is_active: boolean;
}
interface TestProviderConfigData {
    success: boolean;
    message: string;
}
interface ApiErrorBody {
    code: string;
    message: string;
    details?: unknown;
}
interface ApiEnvelope<T> {
    success: boolean;
    data: T | null;
    error: ApiErrorBody | null;
    /** ISO 8601 UTC timestamp of when the response was generated. */
    timestamp: string;
    request_id: string;
}
/** Normalized session stored in the in-memory cache. */
interface Session {
    userId: string;
    accessToken: string;
    refreshToken: string;
    expiresAt: Date;
}
/** Discriminated union returned by `SentinelAuthClient.login()`. */
type LoginResult = {
    type: 'session';
    session: Session;
    mustChangePassword: boolean;
    mfaSetupRequired: boolean;
} | {
    type: 'mfa_challenge';
    userId: string;
    mfaSessionToken: string;
};
interface SentinelConfig {
    /** Base URL of the Sentinel Auth server, e.g. `https://auth.example.com`. */
    baseUrl: string;
    /**
     * How many milliseconds before expiry the SDK will proactively refresh a
     * session when `getValidSession()` is called. Defaults to 5 minutes.
     */
    refreshBufferMs?: number;
    /**
     * Default HTTP headers merged into every request. Useful for injecting
     * tracing headers or, in tests, a unique `X-Forwarded-For` to avoid
     * sharing rate-limit buckets across client instances.
     */
    headers?: Record<string, string>;
}
/**
 * Internal type — the bound `request()` method passed to each sub-client so
 * they share the same base URL, headers, and error-handling logic.
 */
type RequestFn = <T>(path: string, options?: RequestInit) => Promise<{
    data: T;
    requestId: string;
}>;
interface AdminCreateUserRequest {
    email: string;
    first_name: string;
    last_name: string;
    /** Must be ≥12 chars with upper, lower, digit, and special character. */
    password: string;
    /** When true the server sends a verification/invite email immediately. */
    send_invite_email?: boolean;
}
/** Returned by GET /v1/api/admin/users/{userId}/invite-link */
interface InviteLinkData {
    /** Full URL the invited user must visit to verify their email. */
    invite_url: string;
}
interface UpdateUserStatusRequest {
    /** Accepted values: "active", "suspended", "inactive" */
    status: 'active' | 'suspended' | 'inactive';
}
interface AdminUserData {
    user_id: string;
    first_name: string | null;
    last_name: string | null;
    email: string;
    email_verified: boolean;
    status: 'Active' | 'Inactive' | 'Suspended' | 'PendingVerification';
    roles: RoleData[];
    mfa_enabled: boolean;
    mfa_required: boolean;
    created_at: string | null;
}
interface AdminSetMfaRequiredRequest {
    required: boolean;
}
interface UserMfaStatusData {
    mfa_required: boolean;
    mfa_enabled: boolean;
}
interface PaginatedAdminUsersResponse {
    items: AdminUserData[];
    total: number;
    page: number;
    page_size: number;
}
/** Platform-wide KPI snapshot. `GET /v1/api/system/stats` */
interface InsightsSummaryData {
    total_users: number;
    new_users_week: number;
    new_users_month: number;
    active_users_week: number;
    active_users_month: number;
    active_sessions: number;
    /** MFA adoption percentage (0–100). */
    mfa_adoption_pct: number;
    /** Email-verified percentage (0–100). */
    email_verified_pct: number;
}
/** One data point in a cumulative user-growth time series. */
interface UserGrowthPoint {
    /** ISO date string, e.g. `"2026-03-01"`. */
    date: string;
    total_users: number;
    new_users: number;
}
/** One data point in a daily session-activity time series. */
interface SessionActivityPoint {
    /** ISO date string, e.g. `"2026-03-01"`. */
    date: string;
    sessions_created: number;
    unique_users: number;
}

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
declare class AdminClient {
    private readonly req;
    constructor(req: RequestFn);
    /**
     * List all active (non-revoked, non-expired) sessions across all users.
     *
     * `GET /v1/api/admin/sessions`
     */
    listActiveSessions(accessToken: string): Promise<AdminSessionData[]>;
    /**
     * Invalidate a single session by ID.
     *
     * `DELETE /v1/api/admin/sessions/{sessionId}`
     */
    revokeSession(accessToken: string, sessionId: string): Promise<void>;
    /**
     * Bulk-invalidate sessions by their IDs.
     *
     * `POST /v1/api/admin/sessions/revoke`
     */
    revokeSessionsBulk(accessToken: string, body: BulkRevokeSessionsRequest): Promise<BulkRevokeSessionsResponse>;
    /**
     * Create a new role.
     * `role_type` must be one of: `'user'`, `'admin'`, `'support'`.
     *
     * `POST /v1/api/admin/roles`
     */
    createRole(accessToken: string, body: CreateRoleRequest): Promise<RoleData>;
    /**
     * List all roles defined in the system.
     *
     * `GET /v1/api/admin/roles`
     */
    listRoles(accessToken: string): Promise<RoleData[]>;
    /**
     * Update a role's name or description.
     *
     * `PUT /v1/api/admin/roles/{roleId}`
     */
    updateRole(accessToken: string, roleId: string, body: UpdateRoleRequest): Promise<RoleData>;
    /**
     * Delete a role.
     *
     * `DELETE /v1/api/admin/roles/{roleId}`
     */
    deleteRole(accessToken: string, roleId: string): Promise<void>;
    /**
     * Assign a role to a user.
     *
     * `POST /v1/api/admin/users/{userId}/roles`
     */
    assignRole(accessToken: string, userId: string, body: AssignRoleRequest): Promise<void>;
    /**
     * Remove a named role from a user.
     *
     * `DELETE /v1/api/admin/users/{userId}/roles/{roleName}`
     */
    removeRole(accessToken: string, userId: string, roleName: string): Promise<void>;
    /**
     * Get roles and permissions for any user by ID.
     *
     * `GET /v1/api/admin/users/{userId}/permissions`
     */
    getUserPermissions(accessToken: string, userId: string): Promise<UserPermissionsData>;
    /**
     * Get full profile + roles for any user by ID.
     *
     * `GET /v1/api/admin/users/{userId}/auth-info`
     */
    getUserAuthInfo(accessToken: string, userId: string): Promise<UserAuthInfoData>;
    /**
     * Create a new RBAC policy with an initial set of rules.
     *
     * `POST /v1/api/admin/policies`
     */
    createPolicy(accessToken: string, body: CreatePolicyRequest): Promise<CreatePolicyData>;
    /**
     * Replace the rules for an existing policy. Creates a new compiled version
     * and activates it atomically.
     *
     * `PUT /v1/api/admin/policies/{policyId}/rules`
     */
    updatePolicyRules(accessToken: string, policyId: string, body: UpdatePolicyRulesRequest): Promise<UpdatePolicyRulesData>;
    /**
     * Delete a policy and all its versions.
     *
     * `DELETE /v1/api/admin/policies/{policyId}`
     */
    deletePolicy(accessToken: string, policyId: string): Promise<void>;
    /**
     * List all RBAC policies.
     *
     * `GET /v1/api/admin/policies`
     */
    listPolicies(accessToken: string): Promise<PolicyData[]>;
    /**
     * Get the active rules for a policy (from the active policy_version).
     *
     * `GET /v1/api/admin/policies/{policyId}/rules`
     */
    getPolicyRules(accessToken: string, policyId: string): Promise<PolicyRulesData>;
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
    runPolicyProbe(accessToken: string, policyId: string, body: RunProbeRequest): Promise<RunProbeData>;
    /**
     * List all email templates (active and inactive).
     *
     * `GET /v1/api/admin/email-templates`
     */
    listEmailTemplates(accessToken: string): Promise<EmailTemplateData[]>;
    /**
     * Create a new email template for the given type.
     * Deactivates any previously active template of the same type.
     *
     * `POST /v1/api/admin/email-templates`
     */
    createEmailTemplate(accessToken: string, body: CreateEmailTemplateRequest): Promise<EmailTemplateData>;
    /**
     * Update an existing email template.
     *
     * `PUT /v1/api/admin/email-templates/{templateId}`
     */
    updateEmailTemplate(accessToken: string, templateId: string, body: UpdateEmailTemplateRequest): Promise<EmailTemplateData>;
    /**
     * List users with server-side pagination.
     *
     * `GET /v1/api/admin/users`
     */
    listUsers(accessToken: string, params?: {
        page?: number;
        page_size?: number;
    }): Promise<PaginatedAdminUsersResponse>;
    /**
     * Create (invite) a new user as admin. The user is immediately active with
     * a pre-verified email and the default "user" role.
     *
     * `POST /v1/api/admin/users`
     */
    createUser(accessToken: string, body: AdminCreateUserRequest): Promise<AdminUserData>;
    /**
     * Delete a user by ID.
     *
     * `DELETE /v1/api/admin/users/{userId}`
     */
    deleteUser(accessToken: string, userId: string): Promise<void>;
    /**
     * Update a user's status (active / suspended / inactive).
     *
     * `PUT /v1/api/admin/users/{userId}/status`
     */
    updateUserStatus(accessToken: string, userId: string, body: UpdateUserStatusRequest): Promise<AdminUserData>;
    /**
     * Send a verification/invite email to an admin-created user.
     * Returns an error if the user's email is already verified.
     *
     * `POST /v1/api/admin/users/{userId}/send-invite`
     */
    sendUserInvite(accessToken: string, userId: string): Promise<void>;
    /**
     * Generate an invite link for an admin-created user without sending an email.
     * The returned URL contains the raw verification token the user must visit.
     * Returns an error if the user's email is already verified.
     *
     * `GET /v1/api/admin/users/{userId}/invite-link`
     */
    getUserInviteLink(accessToken: string, userId: string): Promise<InviteLinkData>;
    /**
     * Set or clear the admin-mandated MFA requirement for a user.
     * When `required` is `true`, all existing sessions for the user are revoked immediately.
     *
     * `PUT /v1/api/admin/users/{userId}/mfa`
     */
    setMfaRequired(accessToken: string, userId: string, body: AdminSetMfaRequiredRequest): Promise<UserMfaStatusData>;
}

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
declare class ApiTokenClient {
    private readonly req;
    constructor(req: RequestFn);
    /**
     * Create a new API token. The raw `token` value in the response is returned
     * exactly once — it cannot be retrieved again after creation.
     *
     * `POST /v1/api/auth/api-tokens`  (admin)
     */
    create(accessToken: string, body: CreateApiTokenRequest): Promise<CreateApiTokenData>;
    /**
     * List all API tokens (active and revoked) for the authenticated user's account.
     *
     * `GET /v1/api/auth/api-tokens`  (admin)
     */
    list(accessToken: string): Promise<ApiTokenData[]>;
    /**
     * Soft-revoke a specific token by ID. The token remains in the DB with
     * `revoked_at` set.
     *
     * `DELETE /v1/api/auth/api-tokens/{tokenId}`  (admin)
     *
     * @throws {ApiTokenNotFoundError} Token ID not found or already revoked.
     */
    revoke(accessToken: string, tokenId: string): Promise<void>;
    /**
     * Revoke all API tokens for the authenticated user's account.
     *
     * `DELETE /v1/api/auth/api-tokens`  (admin)
     */
    revokeAll(accessToken: string): Promise<void>;
}

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
declare class MfaClient {
    private readonly req;
    private readonly toSession;
    private readonly cacheSession;
    constructor(req: RequestFn, toSession: (data: BasicLoginData) => Session, cacheSession: (session: Session) => void);
    /**
     * Begin TOTP enrollment — returns an `otpauth://` URI to display as a QR code.
     * Bearer token required (user must be logged in).
     *
     * `POST /v1/api/auth/mfa/totp/start`
     */
    totpStart(accessToken: string): Promise<MfaTotpStartData>;
    /**
     * Confirm TOTP enrollment by submitting the first code from the app.
     * Returns one-time recovery codes — store them securely.
     * Bearer token required.
     *
     * `POST /v1/api/auth/mfa/totp/confirm`
     */
    totpConfirm(accessToken: string, body: MfaTotpConfirmRequest): Promise<MfaTotpConfirmData>;
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
    verify(body: MfaVerifyRequest): Promise<Session>;
}

/**
 * System-level endpoints.
 *
 * ```ts
 * const { status } = await client.system.health();
 * ```
 */
declare class SystemClient {
    private readonly req;
    constructor(req: RequestFn);
    /**
     * Check server health.
     *
     * `GET /v1/api/system/health`
     */
    health(): Promise<HealthData>;
    /**
     * List all provider configurations (redacted). Admin only.
     *
     * `GET /v1/api/system/config/email`
     */
    listProviderConfigs(accessToken: string): Promise<ProviderConfigData[]>;
    /**
     * Create a new provider configuration. Bearer token required.
     *
     * `POST /v1/api/system/config/email`
     */
    createProviderConfig(accessToken: string, body: CreateProviderConfigRequest): Promise<ProviderConfigData>;
    /**
     * Update an existing provider configuration. Admin only.
     *
     * `PUT /v1/api/system/config/email/:configId`
     */
    updateProviderConfig(accessToken: string, configId: string, body: UpdateProviderConfigRequest): Promise<ProviderConfigData>;
    /**
     * Delete a provider configuration. Admin only.
     *
     * `DELETE /v1/api/system/config/email/:configId`
     */
    deleteProviderConfig(accessToken: string, configId: string): Promise<void>;
    /**
     * Reveal the decrypted configuration for a provider. Admin only.
     *
     * `GET /v1/api/system/config/email/:configId/reveal`
     */
    revealProviderConfig(accessToken: string, configId: string): Promise<DecryptedProviderConfigData>;
    /**
     * Test the SMTP connection for a provider configuration. Admin only.
     *
     * `POST /v1/api/system/config/email/:configId/test`
     */
    testProviderConfig(accessToken: string, configId: string): Promise<TestProviderConfigData>;
    /**
     * Send a test email through a provider configuration. Admin only.
     *
     * `POST /v1/api/system/config/email/:configId/send-test`
     */
    sendTestEmail(accessToken: string, configId: string, toEmail: string): Promise<TestProviderConfigData>;
    /**
     * Platform-wide KPI snapshot. Admin only.
     *
     * `GET /v1/api/system/stats`
     */
    getInsightsSummary(accessToken: string): Promise<InsightsSummaryData>;
    /**
     * Cumulative user-growth time series. Admin only.
     *
     * `GET /v1/api/system/analytics/user-growth?days=N`
     *
     * @param days Number of days to look back (1–365, default 30).
     */
    getUserGrowth(accessToken: string, days?: number): Promise<UserGrowthPoint[]>;
    /**
     * Daily session-activity time series. Admin only.
     *
     * `GET /v1/api/system/analytics/sessions?days=N`
     *
     * @param days Number of days to look back (1–365, default 30).
     */
    getSessionActivity(accessToken: string, days?: number): Promise<SessionActivityPoint[]>;
}

/**
 * Methods scoped to the authenticated user.
 * Every method requires a valid Bearer `accessToken` from a live session.
 *
 * ```ts
 * const session = await client.getValidSession(userId);
 * const profile  = await client.user.getMe(session.accessToken);
 * ```
 */
declare class UserClient {
    private readonly req;
    constructor(req: RequestFn);
    /**
     * Fetch the profile of the currently authenticated user.
     *
     * `GET /v1/api/user/me`
     */
    getMe(accessToken: string): Promise<UserProfileData>;
    /**
     * Update the authenticated user's profile fields.
     * Only the provided fields are changed; omitted fields are left untouched.
     *
     * `PATCH /v1/api/user/me`
     */
    updateProfile(accessToken: string, body: UpdateProfileRequest): Promise<UserProfileData>;
    /**
     * Change the authenticated user's password.
     * All existing sessions are revoked on success.
     *
     * `POST /v1/api/user/password/change`
     */
    changePassword(accessToken: string, body: ChangePasswordRequest): Promise<void>;
    /**
     * List all sessions for the authenticated user.
     *
     * `GET /v1/api/user/sessions`
     */
    getSessions(accessToken: string): Promise<UserSessionData[]>;
    /**
     * Get details of a specific session.
     *
     * `GET /v1/api/user/sessions/{sessionId}`
     */
    getSession(accessToken: string, sessionId: string): Promise<UserSessionDetailData>;
    /**
     * Get the roles and permissions assigned to the authenticated user.
     *
     * `GET /v1/api/user/permissions`
     */
    getPermissions(accessToken: string): Promise<UserPermissionsData>;
}

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
declare class SentinelAuthClient {
    private readonly cache;
    private readonly refreshBufferMs;
    private readonly baseUrl;
    private readonly config;
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
    constructor(config: SentinelConfig);
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
    register(body: RegisterRequest): Promise<RegisterData>;
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
    login(credentials: LoginRequest): Promise<LoginResult>;
    /**
     * Log out the user, revoking only the current session.
     * The cached session is removed regardless of whether the server call
     * succeeds. No-op if no session is cached for `userId`.
     *
     * `POST /v1/api/auth/logout`
     */
    logout(userId: string): Promise<void>;
    /**
     * Revoke **all** sessions for the authenticated user (sign out everywhere).
     * The local cached session is also removed.
     *
     * `POST /v1/api/auth/logout-all`
     */
    logoutAll(userId: string): Promise<void>;
    /**
     * Exchange the stored refresh token for fresh access + refresh tokens,
     * updating the cache.
     *
     * @throws {SessionNotFoundError} No cached session for `userId`.
     * @throws {InvalidTokenError}    Refresh token is malformed.
     * @throws {ExpiredTokenError}    Refresh token has expired.
     */
    refreshSession(userId: string, refreshToken?: string): Promise<Session>;
    /**
     * Validate a PASETO access token and return the embedded auth context
     * (user ID, session ID, roles, email verification status).
     * Useful for services that need to verify a token passed from a client.
     *
     * `POST /v1/api/auth/authenticate`
     */
    authenticate(body: AuthenticateRequest): Promise<AuthContextData>;
    /**
     * Check whether a given method + path is allowed for a set of roles
     * under the specified policy.
     *
     * `POST /v1/api/auth/token/authorize`
     */
    checkAuthorization(body: CheckAuthorizationRequest): Promise<CheckAuthorizationData>;
    /**
     * Evaluate multiple method+path checks against a policy in a single request.
     * Uses the compiled in-memory trie engine (~200 ns per check). No customer app needed.
     *
     * `POST /v1/api/auth/token/authorize/batch`
     */
    checkAuthorizationBatch(body: BatchCheckRequest): Promise<BatchCheckData>;
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
    authenticateAndAuthorize(body: AuthenticateAndAuthorizeRequest): Promise<AuthenticateAndAuthorizeData>;
    /**
     * Verify an email address using the raw token from the verification link.
     *
     * `GET /v1/api/auth/verify-email?token=<raw>`
     */
    verifyEmail(token: string): Promise<void>;
    /**
     * Trigger a new verification email for the given address.
     * Rate-limited to 10 requests / 15 min per IP.
     *
     * `POST /v1/api/auth/resend-verification`
     */
    resendVerification(body: ResendVerificationRequest): Promise<void>;
    /**
     * Retrieve the authentication methods configured on this Sentinel instance
     * (password auth, MFA, API tokens, OIDC clients, etc.).
     *
     * `GET /v1/api/auth/auth-methods`
     */
    getAuthMethods(): Promise<AuthMethodsData>;
    /**
     * Initiate a password reset flow by sending a reset link to the given email.
     * Always returns successfully — the server silently no-ops for unknown emails
     * to prevent user enumeration.
     *
     * `POST /v1/api/auth/password/forgot`
     */
    forgotPassword(body: ForgotPasswordRequest): Promise<void>;
    /**
     * Reset a password using the token from the reset email.
     * All existing sessions are revoked on success.
     *
     * `POST /v1/api/auth/password/reset`
     */
    resetPassword(body: ResetPasswordRequest): Promise<void>;
    /**
     * Return a valid (non-expired) session for `userId`, auto-refreshing it when
     * it is close to expiry (within `refreshBufferMs`). Returns `null` when no
     * session is cached or the session has fully expired.
     */
    getValidSession(userId: string): Promise<Session | null>;
    /** Synchronously retrieve the raw cached session without validity checks. */
    getSession(userId: string): Session | undefined;
    /** Remove the cached session for `userId` without calling the server. */
    clearSession(userId: string): void;
    /** Remove all cached sessions without calling the server. */
    clearAllSessions(): void;
    /**
     * Evict every expired session from the cache.
     * Returns the number of sessions removed.
     */
    evictExpiredSessions(): number;
    private request;
    private toSession;
}

/**
 * In-memory store for `Session` objects, keyed by `userId`.
 *
 * This is a pure data structure — it has no knowledge of the HTTP layer and
 * never makes network calls. Refresh logic lives in `SentinelAuthClient`.
 */
declare class SessionCache {
    private readonly sessions;
    /** Store (or overwrite) a session for the given user. */
    set(userId: string, session: Session): void;
    /** Retrieve the stored session, or `undefined` if none exists. */
    get(userId: string): Session | undefined;
    /** Remove the session for the given user. No-op if not present. */
    delete(userId: string): void;
    /** Returns `true` if a session (expired or not) is stored for the user. */
    has(userId: string): boolean;
    /** Returns `true` if the session's access token has already expired. */
    isExpired(session: Session): boolean;
    /**
     * Returns `true` if the session will expire within `bufferMs` milliseconds.
     * Use this to decide whether a proactive refresh is worthwhile before
     * returning the session to a caller.
     *
     * @param bufferMs - Look-ahead window in ms. Defaults to 5 minutes.
     */
    isExpiringSoon(session: Session, bufferMs?: number): boolean;
    /** Number of sessions currently cached (including any that may be expired). */
    get size(): number;
    /** Remove all cached sessions. */
    clear(): void;
    /**
     * Evict every session whose access token has already expired.
     * Returns the number of sessions removed.
     */
    evictExpired(): number;
}

/**
 * Base class for all errors thrown by the Sentinel Auth SDK.
 *
 * Every error carries:
 * - `code`       — machine-readable code matching the server's `ApiErrorBody.code`
 * - `statusCode` — HTTP status (0 for network/local errors)
 * - `requestId`  — the `request_id` from the response envelope, useful for tracing
 * - `details`    — optional extra context from the server
 */
declare class SentinelError extends Error {
    readonly code: string;
    readonly statusCode: number;
    readonly requestId: string | undefined;
    readonly details: unknown;
    constructor(code: string, message: string, statusCode: number, requestId?: string, details?: unknown);
}
/** Invalid credentials (wrong email / password). HTTP 401. */
declare class AuthenticationError extends SentinelError {
    constructor(message: string, requestId?: string);
}
/** Request body failed validation (e.g. invalid email format). HTTP 400. */
declare class ValidationError extends SentinelError {
    constructor(message: string, requestId?: string, details?: unknown);
}
/** The supplied token is malformed or cannot be decrypted. HTTP 401. */
declare class InvalidTokenError extends SentinelError {
    constructor(message: string, requestId?: string);
}
/** The supplied token has passed its TTL. HTTP 401. */
declare class ExpiredTokenError extends SentinelError {
    constructor(message: string, requestId?: string);
}
/** No Bearer token was included in the request. HTTP 401. */
declare class MissingTokenError extends SentinelError {
    constructor(message: string, requestId?: string);
}
/** Too many requests — the IP-level or MFA attempt rate limit was hit. HTTP 429. */
declare class RateLimitError extends SentinelError {
    constructor(message: string, requestId?: string);
}
/**
 * The user's email address has not been verified yet.
 * They must click the verification link before accessing protected resources.
 * HTTP 403.
 */
declare class EmailNotVerifiedError extends SentinelError {
    constructor(message: string, requestId?: string);
}
/** An unexpected server-side error occurred. HTTP 500. */
declare class InternalServerError extends SentinelError {
    constructor(message: string, requestId?: string);
}
/**
 * The network request could not be completed (DNS failure, connection refused,
 * etc.). `statusCode` is 0.
 */
declare class NetworkError extends SentinelError {
    constructor(message: string, cause?: unknown);
}
/**
 * No cached session was found for the given `userId`.
 * Thrown by `refreshSession()` when called for an unknown user.
 */
declare class SessionNotFoundError extends SentinelError {
    constructor(userId: string);
}
/** The submitted MFA code (TOTP or recovery) was incorrect. HTTP 401. */
declare class MfaInvalidCodeError extends SentinelError {
    constructor(message: string, requestId?: string);
}
/**
 * Too many failed MFA attempts — the per-token attempt counter was exceeded.
 * Distinct from the IP-level `RateLimitError`. HTTP 429.
 */
declare class MfaAttemptLimitError extends SentinelError {
    constructor(message: string, requestId?: string);
}
/** The requested API token does not exist or has already been revoked. HTTP 404. */
declare class ApiTokenNotFoundError extends SentinelError {
    constructor(message: string, requestId?: string);
}
/** The caller does not have the required role (e.g. admin) to perform this operation. HTTP 403. */
declare class ForbiddenError extends SentinelError {
    constructor(message: string, requestId?: string);
}
/**
 * Creates the most specific `SentinelError` subclass for a given API error
 * code. Falls back to the base `SentinelError` for unknown codes.
 */
declare function createErrorFromCode(code: string, message: string, statusCode: number, requestId?: string, details?: unknown): SentinelError;

/**
 * Express middleware for Sentinel Auth.
 *
 * Validates the Bearer token on every request using Sentinel's `authenticate`
 * and `checkAuthorization` endpoints. When a policy-test token is detected
 * (`scope: "policy_test"`), the middleware short-circuits before the route
 * handler runs and returns a synthetic JSON response — this is how the
 * live probe mode works without executing real business logic.
 *
 * Usage:
 * ```ts
 * import express from 'express';
 * import { SentinelAuthClient } from '@sentinel/auth-sdk';
 * import { sentinelExpressMiddleware } from '@sentinel/auth-sdk/middleware/express';
 *
 * const sentinel = new SentinelAuthClient({ baseUrl: 'https://auth.example.com' });
 * const app = express();
 *
 * app.use(sentinelExpressMiddleware({ client: sentinel }));
 * ```
 */

interface Request {
    method: string;
    path: string;
    headers: Record<string, string | string[] | undefined>;
    sentinelAuth?: AuthContextData;
}
interface Response {
    status(code: number): Response;
    json(body: unknown): void;
}
type NextFunction = () => void;
type RequestHandler = (req: Request, res: Response, next: NextFunction) => void | Promise<void>;
interface SentinelMiddlewareOptions {
    /** Configured `SentinelAuthClient` instance. */
    client: SentinelAuthClient;
    /**
     * Override the policy ID used for authorization checks. Omit to use the
     * server's default active policy. For probe calls the token's embedded
     * `policy_test_id` is always used regardless of this option.
     */
    policyId?: string;
}
/**
 * Returns an Express-compatible middleware that:
 * 1. Extracts the Bearer token from `Authorization`.
 * 2. Calls `sentinel.authenticate()` to validate it.
 * 3. Calls `sentinel.checkAuthorization()` to evaluate the active policy.
 * 4. For policy-test tokens (`scope: "policy_test"`): short-circuits with a
 *    synthetic JSON response so the live probe sees accurate allow/deny results
 *    without executing real route handlers.
 * 5. For normal tokens: attaches `auth` to `req.sentinelAuth` and calls `next()`.
 */
declare function sentinelExpressMiddleware(options: SentinelMiddlewareOptions): RequestHandler;

export { AdminClient, type AdminCreateUserRequest, type AdminSessionData, type AdminSetMfaRequiredRequest, type AdminUserData, type ApiEnvelope, type ApiErrorBody, ApiTokenClient, type ApiTokenData, ApiTokenNotFoundError, type AssignRoleRequest, type AuthContextData, type AuthMethodsData, type AuthenticateAndAuthorizeData, type AuthenticateAndAuthorizeRequest, type AuthenticateRequest, AuthenticationError, type BasicLoginData, type BatchCheckData, type BatchCheckItem, type BatchCheckRequest, type BatchCheckResult, type BulkRevokeSessionsRequest, type BulkRevokeSessionsResponse, type ChangePasswordRequest, type CheckAuthorizationData, type CheckAuthorizationRequest, type CreateApiTokenData, type CreateApiTokenRequest, type CreateEmailTemplateRequest, type CreatePolicyData, type CreatePolicyRequest, type CreateProviderConfigRequest, type CreateRoleRequest, type DecryptedProviderConfigData, EmailNotVerifiedError, type EmailTemplateData, type EmailTemplateType, ExpiredTokenError, ForbiddenError, type ForgotPasswordRequest, type HealthData, type InsightsSummaryData, InternalServerError, InvalidTokenError, type InviteLinkData, type LoginData, type LoginRequest, type LoginResult, MfaAttemptLimitError, type MfaChallengeData, MfaClient, MfaInvalidCodeError, type MfaTotpConfirmData, type MfaTotpConfirmRequest, type MfaTotpStartData, type MfaVerifyRequest, MissingTokenError, NetworkError, type OidcClientInfo, type PaginatedAdminUsersResponse, type PolicyData, type PolicyRule, type PolicyRulesData, type ProbeRuleResult, type ProviderConfigData, RateLimitError, type RefreshTokenRequest, type RegisterData, type RegisterRequest, type RequestFn, type ResendVerificationRequest, type ResetPasswordRequest, type RoleData, type RunProbeData, type RunProbeRequest, type SendTestEmailRequest, SentinelAuthClient, type SentinelConfig, SentinelError, type SentinelMiddlewareOptions, type Session, type SessionActivityPoint, SessionCache, SessionNotFoundError, SystemClient, type TestProviderConfigData, type UpdateEmailTemplateRequest, type UpdatePolicyRulesData, type UpdatePolicyRulesRequest, type UpdateProfileRequest, type UpdateProviderConfigRequest, type UpdateRoleRequest, type UpdateUserStatusRequest, type UserAuthInfoData, UserClient, type UserGrowthPoint, type UserMfaStatusData, type UserPermissionsData, type UserProfileData, type UserSessionData, type UserSessionDetailData, ValidationError, createErrorFromCode, sentinelExpressMiddleware };
