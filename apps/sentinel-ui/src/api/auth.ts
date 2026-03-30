import { sentinelClient } from "../lib/sdkClient";
import { useAuthStore } from "@sentinel/auth-react";
import type { LoginRequest } from "@sentinel/auth-sdk";

const getToken = () => useAuthStore.getState().accessToken ?? "";

export const authApi = {
  login: (data: LoginRequest) => sentinelClient.login(data),
  logout: (userId: string) => sentinelClient.logout(userId),
  logoutAll: (userId: string) => sentinelClient.logoutAll(userId),
  getMe: () => sentinelClient.user.getMe(getToken()),
  refreshSession: () => {
    const { userId, refreshToken } = useAuthStore.getState();
    if (!userId || !refreshToken) return Promise.reject(new Error("No session to refresh"));
    return sentinelClient.refreshSession(userId, refreshToken);
  },
  forgotPassword: (email: string) => sentinelClient.forgotPassword({ email }),
  resetPassword: (token: string, newPassword: string) =>
    sentinelClient.resetPassword({ token, new_password: newPassword }),
};
