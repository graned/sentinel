import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import { QueryClientProvider } from "@tanstack/react-query";
import {
  SentinelAuthProvider,
  SentinelAuthRoutes,
  AuthorizedRoute,
  ProtectedRoute,
  createSentinelQueryClient,
} from "@sentinel/auth-react";
import "@sentinel/auth-react/dist/style.css";
import { AppShell } from "./components/layout/AppShell";
import { DashboardPage } from "./pages/dashboard/DashboardPage";
import { RolesPage } from "./pages/roles/RolesPage";
import { PoliciesPage } from "./pages/policies/PoliciesPage";
import { EditRulesPage } from "./pages/policies/EditRulesPage";
import { CreatePolicyPage } from "./pages/policies/CreatePolicyPage";
import { SessionsPage } from "./pages/sessions/SessionsPage";
import { TokensPage } from "./pages/tokens/TokensPage";
import { EmailTemplatesPage } from "./pages/email/EmailTemplatesPage";
import { CreateEmailTemplatePage } from "./pages/email/CreateEmailTemplatePage";
import { ProvidersPage } from "./pages/providers/ProvidersPage";
import { UsersPage } from "./pages/users/UsersPage";
import { sentinelClient } from "./lib/sdkClient";
import { SentinelLogo } from "./components/SentinelLogo";

const theme = {
  appName: "Sentinel",
  logo: <SentinelLogo />,
};

const redirects = {
  afterLogin: "/dashboard",
  afterLogout: "/login",
  afterRegister: "/verify-email",
  login: "/login",
  register: "/register",
  verifyEmail: "/verify-email",
  forgotPassword: "/forgot-password",
  changePassword: "/change-password",
  setupMfa: "/setup-mfa",
  unauthorized: "/unauthorized",
};

const qc = createSentinelQueryClient(redirects);

export default function App() {
  return (
    <SentinelAuthProvider client={sentinelClient} redirects={redirects} theme={theme}>
      <QueryClientProvider client={qc}>
        <BrowserRouter>
          <Routes>
            {/* Auth routes (login, register, verify-email, forgot/reset password, change-password, setup-mfa, unauthorized) */}
            <Route path="/*" element={<SentinelAuthRoutes />} />

            {/* Protected: redirect to /login when not authenticated */}
            <Route element={<ProtectedRoute />}>
              {/* Authorized: check policy endpoint before rendering any admin route */}
              <Route element={<AuthorizedRoute />}>
                <Route element={<AppShell />}>
                  <Route index element={<Navigate to="/dashboard" replace />} />
                  <Route path="/dashboard" element={<DashboardPage />} />
                  <Route path="/roles" element={<RolesPage />} />
                  <Route path="/policies" element={<PoliciesPage />} />
                  <Route path="/policies/new" element={<CreatePolicyPage />} />
                  <Route path="/policies/:policyId/rules" element={<EditRulesPage />} />
                  <Route path="/sessions" element={<SessionsPage />} />
                  <Route path="/tokens" element={<TokensPage />} />
                  <Route path="/email-templates" element={<EmailTemplatesPage />} />
                  <Route path="/email-templates/new" element={<CreateEmailTemplatePage />} />
                  <Route path="/users" element={<UsersPage />} />
                  <Route path="/providers" element={<ProvidersPage />} />
                  <Route path="*" element={<Navigate to="/dashboard" replace />} />
                </Route>
              </Route>
            </Route>
          </Routes>
        </BrowserRouter>
      </QueryClientProvider>
    </SentinelAuthProvider>
  );
}
