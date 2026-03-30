// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

export interface LoginRequest {
  email: string;
  password: string;
}

export interface RefreshTokenRequest {
  refresh_token: string;
}

export interface RegisterRequest {
  first_name: string;
  last_name: string;
  email: string;
  avatar_url?: string | null;
  /** Must be ≥12 chars with upper, lower, digit, and special character. */
  password: string;
}

export interface AuthenticateRequest {
  access_token: string;
}

export interface ResendVerificationRequest {
  email: string;
}

export interface ForgotPasswordRequest {
  email: string;
}

export interface ResetPasswordRequest {
  token: string;
  /** Must be ≥12 chars with upper, lower, digit, and special character. */
  new_password: string;
}

export interface ChangePasswordRequest {
  current_password: string;
  /** Must be ≥12 chars with upper, lower, digit, and special character. */
  new_password: string;
}

export interface CheckAuthorizationRequest {
  policy_id?: string;
  method: string;
  path: string;
  roles: string[];
}

export interface AuthenticateAndAuthorizeRequest {
  /** PASETO access token to validate. */
  access_token: string;
  /** HTTP method to check (e.g. `"GET"`, `"POST"`). */
  method: string;
  /** Resource path to check (e.g. `"/v1/api/user/me"`). */
  path: string;
  /** Optional policy ID. Omit to use the server's default active policy. */
  policy_id?: string;
}

export interface MfaTotpConfirmRequest {
  /** Exactly 6 digits from the authenticator app. */
  code: string;
}

export interface MfaVerifyRequest {
  /** Short-lived PASETO token returned by the login MFA challenge. */
  mfa_session_token: string;
  /** 6-digit TOTP code or an 8-char recovery code. */
  code: string;
}

export interface CreateApiTokenRequest {
  name: string;
  description?: string;
  /** ISO 8601 UTC expiry. Omit for a non-expiring token. */
  expires_at?: string;
}

export interface CreateRoleRequest {
  /** Must be one of: 'user', 'admin', 'support'. */
  role_type: string;
  name: string;
  description: string;
}

export interface UpdateRoleRequest {
  name?: string;
  description?: string;
}

export interface AssignRoleRequest {
  role_id: string;
}

export interface CreatePolicyRequest {
  name: string;
  environment: string;
  description?: string;
  tenant_id?: string;
  rules: PolicyRule[];
}

export interface PolicyRule {
  method: string;
  path: string;
  roles: string[];
}

export interface UpdatePolicyRulesRequest {
  rules: PolicyRule[];
}

export interface CreateEmailTemplateRequest {
  template_type: EmailTemplateType;
  subject: string;
  body_text: string;
  body_html?: string;
}

export interface UpdateEmailTemplateRequest {
  subject?: string;
  body_text?: string;
  body_html?: string;
  is_active?: boolean;
}

export interface CreateProviderConfigRequest {
  provider: string;
  config: Record<string, unknown>;
  is_active: boolean;
  tenant_id?: string;
}

export interface UpdateProviderConfigRequest {
  config: Record<string, unknown>;
  is_active: boolean;
}

export interface SendTestEmailRequest {
  to_email: string;
}

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

export type EmailTemplateType = 'EmailVerification' | 'PasswordReset' | 'PasswordChanged';

// ---------------------------------------------------------------------------
// Raw API response data shapes (mirror the Rust DTOs)
// ---------------------------------------------------------------------------

/** Returned when login succeeds without MFA. */
export interface BasicLoginData {
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
export interface MfaChallengeData {
  user_id: string;
  /** Always `true` — use this field to discriminate the union. */
  mfa_required: true;
  /** Short-lived PASETO token (5-min TTL). Pass it to the MFA verify endpoint. */
  mfa_session_token: string;
}

export type LoginData = BasicLoginData | MfaChallengeData;

export interface RegisterData {
  user_id: string;
  first_name: string;
  last_name: string;
  avatar_url: string | null;
  /** UserStatus enum value from the server (e.g. 'Active', 'PendingVerification'). */
  status: string;
}

export interface AuthContextData {
  user_id: string;
  session_id: string;
  roles: string[];
  email_verified: boolean;
  /** Present only for policy-test tokens (`scope: "policy_test"`). */
  scope?: 'policy_test';
  /** Policy ID embedded in a test token. Present when `scope === "policy_test"`. */
  policy_test_id?: string;
}

// ---------------------------------------------------------------------------
// Policy testing — batch check
// ---------------------------------------------------------------------------

export interface BatchCheckItem {
  method: string;
  path: string;
}

export interface BatchCheckRequest {
  /** Optional. Omit to use the server's default active policy. */
  policy_id?: string;
  roles: string[];
  /** Max 500 items. */
  checks: BatchCheckItem[];
}

export interface BatchCheckResult {
  method: string;
  path: string;
  allowed: boolean;
}

export interface BatchCheckData {
  policy_id?: string;
  evaluated_version: number;
  results: BatchCheckResult[];
}

// ---------------------------------------------------------------------------
// Policy testing — live probe
// ---------------------------------------------------------------------------

export interface RunProbeRequest {
  /** Base URL of the customer app (e.g. `https://api.myapp.com`). */
  base_url: string;
  roles: string[];
}

export interface ProbeRuleResult {
  method: string;
  path: string;
  allowed: boolean;
  status_code?: number;
  /** Set when the customer app is unreachable (connection failure, timeout). */
  error?: string;
}

export interface RunProbeData {
  policy_id: string;
  evaluated_version: number;
  roles_tested: string[];
  base_url: string;
  results: ProbeRuleResult[];
}

export interface OidcClientInfo {
  client_id: string;
  name: string;
  allowed_scopes: string[];
  pkce_required: boolean;
}

export interface AuthMethodsData {
  password_enabled: boolean;
  mfa_totp_available: boolean;
  api_tokens_available: boolean;
  email_verification_required: boolean;
  email_provider_active: boolean;
  oidc_enabled: boolean;
  oidc_clients: OidcClientInfo[];
}

export interface CheckAuthorizationData {
  allowed: boolean;
  method: string;
  path: string;
  roles: string[];
  active_version: number;
}

/** Combined result returned by `SentinelAuthClient.authenticateAndAuthorize()`. */
export interface AuthenticateAndAuthorizeData {
  /** Validated token context: user ID, session ID, roles, and email verification status. */
  auth: AuthContextData;
  /** Policy evaluation result confirming the action was permitted. */
  authorization: CheckAuthorizationData;
}

export interface UserProfileData {
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

export interface UserSessionData {
  session_id: string;
  user_agent: string | null;
  ip_address: string | null;
  device_type: string | null;
  last_used_at: string | null;
  created_at: string | null;
  expires_at: string;
  is_current: boolean;
}

export interface UserSessionDetailData extends UserSessionData {
  revoked_at: string | null;
  is_active: boolean;
}

export interface AdminSessionData {
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

export interface BulkRevokeSessionsRequest {
  session_ids: string[];
}

export interface BulkRevokeSessionsResponse {
  revoked_count: number;
}

export interface RoleData {
  role_id: string;
  name: string;
  role_type: string;
  description: string;
}

export interface UserPermissionsData {
  user_id: string;
  roles: RoleData[];
}

export interface MfaTotpStartData {
  /** `otpauth://` URI — encode as a QR code for the authenticator app. */
  otpauth_uri: string;
}

export interface MfaTotpConfirmData {
  /** One-time recovery codes — store securely, shown only once. */
  recovery_codes: string[];
}

export interface CreateApiTokenData {
  api_token_id: string;
  /** Raw token in `sat_<hex>` format — returned exactly once. Store securely. */
  token: string;
  name: string;
  expires_at: string | null;
  created_at: string;
}

export interface ApiTokenData {
  api_token_id: string;
  name: string;
  description: string | null;
  expires_at: string | null;
  last_used_at: string | null;
  revoked_at: string | null;
  created_at: string;
}

export interface UserAuthInfoData extends UserProfileData {
  roles: RoleData[];
}

export interface CreatePolicyData {
  policy_id: string;
  name: string;
  environment: string;
  description: string | null;
  active_version: number;
  created_at: string;
}

export interface UpdatePolicyRulesData {
  policy_id: string;
  activated_version: number;
}

export interface PolicyData {
  policy_id: string;
  tenant_id: string | null;
  name: string;
  environment: string;
  description: string | null;
  active_version: number;
  created_at: string;
  updated_at: string;
}

export interface PolicyRulesData {
  policy_id: string;
  version: number;
  rules: PolicyRule[];
}

export interface EmailTemplateData {
  template_id: string;
  template_type: EmailTemplateType;
  subject: string;
  body_text: string;
  body_html: string | null;
  is_active: boolean;
  created_at: string;
  updated_at: string | null;
}

export interface HealthData {
  status: string;
}

export interface ProviderConfigData {
  configuration_id: string;
  tenant_id: string | null;
  provider: string;
  config_redacted: Record<string, unknown>;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface DecryptedProviderConfigData {
  configuration_id: string;
  provider: string;
  config: Record<string, unknown>;
  is_active: boolean;
}

export interface TestProviderConfigData {
  success: boolean;
  message: string;
}

// ---------------------------------------------------------------------------
// API envelope
// ---------------------------------------------------------------------------

export interface ApiErrorBody {
  code: string;
  message: string;
  details?: unknown;
}

export interface ApiEnvelope<T> {
  success: boolean;
  data: T | null;
  error: ApiErrorBody | null;
  /** ISO 8601 UTC timestamp of when the response was generated. */
  timestamp: string;
  request_id: string;
}

// ---------------------------------------------------------------------------
// SDK-level types
// ---------------------------------------------------------------------------

/** Normalized session stored in the in-memory cache. */
export interface Session {
  userId: string;
  accessToken: string;
  refreshToken: string;
  expiresAt: Date;
}

/** Discriminated union returned by `SentinelAuthClient.login()`. */
export type LoginResult =
  | { type: 'session'; session: Session; mustChangePassword: boolean; mfaSetupRequired: boolean }
  | { type: 'mfa_challenge'; userId: string; mfaSessionToken: string };

export interface SentinelConfig {
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
export type RequestFn = <T>(
  path: string,
  options?: RequestInit,
) => Promise<{ data: T; requestId: string }>;

// ---------------------------------------------------------------------------
// Admin user management
// ---------------------------------------------------------------------------

export interface AdminCreateUserRequest {
  email: string;
  first_name: string;
  last_name: string;
  /** Must be ≥12 chars with upper, lower, digit, and special character. */
  password: string;
  /** When true the server sends a verification/invite email immediately. */
  send_invite_email?: boolean;
}

/** Returned by GET /v1/api/admin/users/{userId}/invite-link */
export interface InviteLinkData {
  /** Full URL the invited user must visit to verify their email. */
  invite_url: string;
}

export interface UpdateUserStatusRequest {
  /** Accepted values: "active", "suspended", "inactive" */
  status: 'active' | 'suspended' | 'inactive';
}

export interface AdminUserData {
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

export interface AdminSetMfaRequiredRequest {
  required: boolean;
}

export interface UserMfaStatusData {
  mfa_required: boolean;
  mfa_enabled: boolean;
}

export interface PaginatedAdminUsersResponse {
  items: AdminUserData[];
  total: number;
  page: number;
  page_size: number;
}

// ---------------------------------------------------------------------------
// Insights / analytics
// ---------------------------------------------------------------------------

/** Platform-wide KPI snapshot. `GET /v1/api/system/stats` */
export interface InsightsSummaryData {
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
export interface UserGrowthPoint {
  /** ISO date string, e.g. `"2026-03-01"`. */
  date: string;
  total_users: number;
  new_users: number;
}

/** One data point in a daily session-activity time series. */
export interface SessionActivityPoint {
  /** ISO date string, e.g. `"2026-03-01"`. */
  date: string;
  sessions_created: number;
  unique_users: number;
}
