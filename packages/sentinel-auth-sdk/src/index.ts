// Main client
export { SentinelAuthClient } from './client';

// Sub-clients (also accessible as client.user, client.mfa, etc.)
export { UserClient } from './user-client';
export { MfaClient } from './mfa-client';
export { ApiTokenClient } from './api-token-client';
export { AdminClient } from './admin-client';
export { SystemClient } from './system-client';

// Session cache (exported for custom cache management / testing)
export { SessionCache } from './session';

// Error classes
export {
  SentinelError,
  AuthenticationError,
  ValidationError,
  InvalidTokenError,
  ExpiredTokenError,
  MissingTokenError,
  RateLimitError,
  EmailNotVerifiedError,
  InternalServerError,
  NetworkError,
  SessionNotFoundError,
  MfaInvalidCodeError,
  MfaAttemptLimitError,
  ApiTokenNotFoundError,
  ForbiddenError,
  createErrorFromCode,
} from './errors';

// Types — requests
export type {
  AdminCreateUserRequest,
  BatchCheckItem,
  BatchCheckRequest,
  BulkRevokeSessionsRequest,
  UpdateUserStatusRequest,
  LoginRequest,
  RefreshTokenRequest,
  RegisterRequest,
  AuthenticateRequest,
  ResendVerificationRequest,
  ForgotPasswordRequest,
  ResetPasswordRequest,
  ChangePasswordRequest,
  CheckAuthorizationRequest,
  AuthenticateAndAuthorizeRequest,
  MfaTotpConfirmRequest,
  MfaVerifyRequest,
  CreateApiTokenRequest,
  CreateRoleRequest,
  UpdateRoleRequest,
  AssignRoleRequest,
  CreatePolicyRequest,
  PolicyRule,
  UpdatePolicyRulesRequest,
  CreateEmailTemplateRequest,
  UpdateEmailTemplateRequest,
  UpdateProfileRequest,
  CreateProviderConfigRequest,
  UpdateProviderConfigRequest,
  RunProbeRequest,
  SendTestEmailRequest,
} from './types';

// Types — enums
export type { EmailTemplateType } from './types';

// Types — response data
export type {
  BasicLoginData,
  MfaChallengeData,
  LoginData,
  RegisterData,
  AuthContextData,
  OidcClientInfo,
  AuthMethodsData,
  BatchCheckData,
  BatchCheckResult,
  CheckAuthorizationData,
  AuthenticateAndAuthorizeData,
  UserProfileData,
  UserSessionData,
  UserSessionDetailData,
  RoleData,
  UserPermissionsData,
  MfaTotpStartData,
  MfaTotpConfirmData,
  CreateApiTokenData,
  ApiTokenData,
  UserAuthInfoData,
  CreatePolicyData,
  UpdatePolicyRulesData,
  PolicyData,
  PolicyRulesData,
  ProbeRuleResult,
  RunProbeData,
  EmailTemplateData,
  HealthData,
  ProviderConfigData,
  DecryptedProviderConfigData,
  TestProviderConfigData,
  ApiErrorBody,
  ApiEnvelope,
  AdminUserData,
  AdminSetMfaRequiredRequest,
  UserMfaStatusData,
  InviteLinkData,
  PaginatedAdminUsersResponse,
  AdminSessionData,
  BulkRevokeSessionsResponse,
  InsightsSummaryData,
  UserGrowthPoint,
  SessionActivityPoint,
} from './types';

// Middleware
export { sentinelExpressMiddleware } from './middleware/express';
export type { SentinelMiddlewareOptions } from './middleware/express';

// Types — SDK-level
export type {
  Session,
  LoginResult,
  SentinelConfig,
  RequestFn,
} from './types';
