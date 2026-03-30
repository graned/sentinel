import { sentinelClient } from "../lib/sdkClient";
import { useAuthStore } from "@sentinel/auth-react";
import { withAuthRetry } from "../lib/withAuthRetry";
import type {
  AdminCreateUserRequest,
  AssignRoleRequest,
  BatchCheckRequest,
  BulkRevokeSessionsRequest,
  CreateApiTokenRequest,
  CreateEmailTemplateRequest,
  CreatePolicyRequest,
  CreateProviderConfigRequest,
  CreateRoleRequest,
  RunProbeRequest,
  UpdateEmailTemplateRequest,
  UpdatePolicyRulesRequest,
  UpdateProviderConfigRequest,
  UpdateRoleRequest,
  UpdateUserStatusRequest,
  AdminSetMfaRequiredRequest,
} from "@sentinel/auth-sdk";

const getToken = () => useAuthStore.getState().accessToken ?? "";

export const adminApi = {
  // Roles
  listRoles: () => withAuthRetry(() => sentinelClient.admin.listRoles(getToken())),
  createRole: (body: CreateRoleRequest) =>
    withAuthRetry(() => sentinelClient.admin.createRole(getToken(), body)),
  updateRole: (id: string, body: UpdateRoleRequest) =>
    withAuthRetry(() => sentinelClient.admin.updateRole(getToken(), id, body)),
  deleteRole: (id: string) =>
    withAuthRetry(() => sentinelClient.admin.deleteRole(getToken(), id)),

  // User roles
  assignRole: (userId: string, body: AssignRoleRequest) =>
    withAuthRetry(() => sentinelClient.admin.assignRole(getToken(), userId, body)),
  removeRole: (userId: string, roleName: string) =>
    withAuthRetry(() => sentinelClient.admin.removeRole(getToken(), userId, roleName)),
  getUserAuthInfo: (userId: string) =>
    withAuthRetry(() => sentinelClient.admin.getUserAuthInfo(getToken(), userId)),
  getUserPermissions: (userId: string) =>
    withAuthRetry(() => sentinelClient.admin.getUserPermissions(getToken(), userId)),

  // Email templates
  listEmailTemplates: () =>
    withAuthRetry(() => sentinelClient.admin.listEmailTemplates(getToken())),
  createEmailTemplate: (body: CreateEmailTemplateRequest) =>
    withAuthRetry(() => sentinelClient.admin.createEmailTemplate(getToken(), body)),
  updateEmailTemplate: (id: string, body: UpdateEmailTemplateRequest) =>
    withAuthRetry(() => sentinelClient.admin.updateEmailTemplate(getToken(), id, body)),

  // Policies
  listPolicies: () => withAuthRetry(() => sentinelClient.admin.listPolicies(getToken())),
  getPolicyRules: (id: string) =>
    withAuthRetry(() => sentinelClient.admin.getPolicyRules(getToken(), id)),
  createPolicy: (body: CreatePolicyRequest) =>
    withAuthRetry(() => sentinelClient.admin.createPolicy(getToken(), body)),
  updatePolicyRules: (id: string, body: UpdatePolicyRulesRequest) =>
    withAuthRetry(() => sentinelClient.admin.updatePolicyRules(getToken(), id, body)),
  deletePolicy: (id: string) =>
    withAuthRetry(() => sentinelClient.admin.deletePolicy(getToken(), id)),
  runPolicyProbe: (id: string, body: RunProbeRequest) =>
    withAuthRetry(() => sentinelClient.admin.runPolicyProbe(getToken(), id, body)),
  checkAuthorizationBatch: (body: BatchCheckRequest) =>
    withAuthRetry(() => sentinelClient.checkAuthorizationBatch(body)),

  // API tokens
  listApiTokens: () => withAuthRetry(() => sentinelClient.apiTokens.list(getToken())),
  createApiToken: (body: CreateApiTokenRequest) =>
    withAuthRetry(() => sentinelClient.apiTokens.create(getToken(), body)),
  revokeApiToken: (id: string) =>
    withAuthRetry(() => sentinelClient.apiTokens.revoke(getToken(), id)),
  revokeAllApiTokens: () => withAuthRetry(() => sentinelClient.apiTokens.revokeAll(getToken())),
  verifyApiToken: (rawToken: string) => sentinelClient.user.getMe(rawToken),

  // Provider configs
  listProviderConfigs: () =>
    withAuthRetry(() => sentinelClient.system.listProviderConfigs(getToken())),
  createProviderConfig: (body: CreateProviderConfigRequest) =>
    withAuthRetry(() => sentinelClient.system.createProviderConfig(getToken(), body)),
  updateProviderConfig: (id: string, body: UpdateProviderConfigRequest) =>
    withAuthRetry(() => sentinelClient.system.updateProviderConfig(getToken(), id, body)),
  deleteProviderConfig: (id: string) =>
    withAuthRetry(() => sentinelClient.system.deleteProviderConfig(getToken(), id)),
  revealProviderConfig: (id: string) =>
    withAuthRetry(() => sentinelClient.system.revealProviderConfig(getToken(), id)),
  testProviderConfig: (id: string) =>
    withAuthRetry(() => sentinelClient.system.testProviderConfig(getToken(), id)),
  sendTestEmail: (id: string, toEmail: string) =>
    withAuthRetry(() => sentinelClient.system.sendTestEmail(getToken(), id, toEmail)),

  // Users
  listUsers: (params?: { page?: number; page_size?: number }) =>
    withAuthRetry(() => sentinelClient.admin.listUsers(getToken(), params)),
  createUser: (body: AdminCreateUserRequest) =>
    withAuthRetry(() => sentinelClient.admin.createUser(getToken(), body)),
  deleteUser: (userId: string) =>
    withAuthRetry(() => sentinelClient.admin.deleteUser(getToken(), userId)),
  updateUserStatus: (userId: string, body: UpdateUserStatusRequest) =>
    withAuthRetry(() => sentinelClient.admin.updateUserStatus(getToken(), userId, body)),
  resendVerification: (email: string) =>
    withAuthRetry(() => sentinelClient.resendVerification({ email })),
  sendUserInvite: (userId: string) =>
    withAuthRetry(() => sentinelClient.admin.sendUserInvite(getToken(), userId)),
  getUserInviteLink: (userId: string) =>
    withAuthRetry(() => sentinelClient.admin.getUserInviteLink(getToken(), userId)),
  setMfaRequired: (userId: string, body: AdminSetMfaRequiredRequest) =>
    withAuthRetry(() => sentinelClient.admin.setMfaRequired(getToken(), userId, body)),

  // Sessions
  listActiveSessions: () =>
    withAuthRetry(() => sentinelClient.admin.listActiveSessions(getToken())),
  revokeSession: (sessionId: string) =>
    withAuthRetry(() => sentinelClient.admin.revokeSession(getToken(), sessionId)),
  revokeSessionsBulk: (body: BulkRevokeSessionsRequest) =>
    withAuthRetry(() => sentinelClient.admin.revokeSessionsBulk(getToken(), body)),

  // Insights / analytics
  getInsightsSummary: () =>
    withAuthRetry(() => sentinelClient.system.getInsightsSummary(getToken())),
  getUserGrowth: (days: number) =>
    withAuthRetry(() => sentinelClient.system.getUserGrowth(getToken(), days)),
  getSessionActivity: (days: number) =>
    withAuthRetry(() => sentinelClient.system.getSessionActivity(getToken(), days)),
};
