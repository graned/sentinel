import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { adminApi } from "../../api/admin";
import { Card } from "../../components/ui/Card";
import { Button } from "@sentinel/auth-react";
import { Badge } from "../../components/ui/Badge";
import { PageHeader } from "../../components/ui/PageHeader";
import { EmptyState } from "../../components/ui/EmptyState";
import type {
  CreateProviderConfigRequest,
  DecryptedProviderConfigData,
  ProviderConfigData,
  UpdateProviderConfigRequest,
} from "../../types";
import styles from "./ProvidersPage.module.css";

type AuthType = "credentials" | "api_key";

type ProviderPreset = {
  host: string;
  port: number;
  username: string;
  use_tls: boolean;
  default_auth_type: AuthType;
};

const PRESETS: Record<string, ProviderPreset> = {
  resend:  { host: "smtp.resend.com",   port: 587, username: "resend", use_tls: false, default_auth_type: "api_key" },
  mailjet: { host: "in-v3.mailjet.com", port: 587, username: "",       use_tls: false, default_auth_type: "credentials" },
  smtp:    { host: "",                  port: 587, username: "",        use_tls: false, default_auth_type: "credentials" },
};

const PROVIDER_LABELS: Record<string, string> = {
  resend:  "Resend",
  mailjet: "Mailjet",
  smtp:    "Custom SMTP",
};

export function ProvidersPage() {
  const qc = useQueryClient();
  const [showForm, setShowForm] = useState(false);

  // Create form state
  const [provider, setProvider]     = useState("resend");
  const [host, setHost]             = useState(PRESETS.resend.host);
  const [port, setPort]             = useState(PRESETS.resend.port);
  const [username, setUsername]     = useState(PRESETS.resend.username);
  const [password, setPassword]     = useState("");
  const [apiKey, setApiKey]         = useState("");
  const [authType, setAuthType]     = useState<AuthType>(PRESETS.resend.default_auth_type);
  const [fromEmail, setFromEmail]   = useState("");
  const [useTls, setUseTls]         = useState(false);
  const [isActive, setIsActive]     = useState(true);

  // Edit form state
  const [editId, setEditId]               = useState<string | null>(null);
  const [editHost, setEditHost]           = useState("");
  const [editPort, setEditPort]           = useState(587);
  const [editUsername, setEditUsername]   = useState("");
  const [editPassword, setEditPassword]   = useState("");
  const [editApiKey, setEditApiKey]       = useState("");
  const [editAuthType, setEditAuthType]   = useState<AuthType>("credentials");
  const [editFromEmail, setEditFromEmail] = useState("");
  const [editUseTls, setEditUseTls]       = useState(false);
  const [editIsActive, setEditIsActive]   = useState(true);

  const [revealData, setRevealData]       = useState<DecryptedProviderConfigData | null>(null);
  const [revealLoading, setRevealLoading] = useState<string | null>(null);
  const [editLoading, setEditLoading]     = useState<string | null>(null);

  // Test state: keyed by configuration_id
  const [testLoading, setTestLoading] = useState<string | null>(null);
  const [testResults, setTestResults] = useState<Record<string, { success: boolean; message: string }>>({});

  // Send test email modal state
  const [testEmailId, setTestEmailId]           = useState<string | null>(null);
  const [testEmailAddr, setTestEmailAddr]       = useState("");
  const [testEmailLoading, setTestEmailLoading] = useState(false);
  const [testEmailResult, setTestEmailResult]   = useState<{ success: boolean; message: string } | null>(null);

  const { data: configs, isLoading } = useQuery({
    queryKey: ["provider-configs"],
    queryFn: () => adminApi.listProviderConfigs(),
  });

  const createMutation = useMutation({
    mutationFn: (data: CreateProviderConfigRequest) => adminApi.createProviderConfig(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["provider-configs"] });
      setShowForm(false);
      setProvider("resend");
      setHost(PRESETS.resend.host);
      setPort(PRESETS.resend.port);
      setUsername(PRESETS.resend.username);
      setPassword("");
      setApiKey("");
      setAuthType(PRESETS.resend.default_auth_type);
      setFromEmail("");
      setUseTls(false);
      setIsActive(true);
    },
  });

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: UpdateProviderConfigRequest }) =>
      adminApi.updateProviderConfig(id, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["provider-configs"] });
      setEditId(null);
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => adminApi.deleteProviderConfig(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["provider-configs"] });
    },
  });

  const handleProviderChange = (p: string) => {
    setProvider(p);
    const preset = PRESETS[p] ?? PRESETS.smtp;
    setHost(preset.host);
    setPort(preset.port);
    setUsername(preset.username);
    setUseTls(preset.use_tls);
    setAuthType(preset.default_auth_type);
    setPassword("");
    setApiKey("");
  };

  const buildConfig = () =>
    authType === "api_key"
      ? { host, port, username, api_key: apiKey, from_email: fromEmail, use_tls: useTls }
      : { host, port, username, password, from_email: fromEmail, use_tls: useTls };

  const handleCreate = (e: React.FormEvent) => {
    e.preventDefault();
    createMutation.mutate({ provider, config: buildConfig(), is_active: isActive });
  };

  const buildEditConfig = () =>
    editAuthType === "api_key"
      ? { host: editHost, port: editPort, username: editUsername, api_key: editApiKey, from_email: editFromEmail, use_tls: editUseTls }
      : { host: editHost, port: editPort, username: editUsername, password: editPassword, from_email: editFromEmail, use_tls: editUseTls };

  const handleUpdate = (e: React.FormEvent, id: string) => {
    e.preventDefault();
    updateMutation.mutate({ id, data: { config: buildEditConfig(), is_active: editIsActive } });
  };

  const openEdit = async (c: ProviderConfigData) => {
    setEditLoading(c.configuration_id);
    try {
      const revealed = await adminApi.revealProviderConfig(c.configuration_id);
      const cfg = revealed.config;
      setEditId(c.configuration_id);
      setEditHost(String(cfg.host ?? ""));
      setEditPort(Number(cfg.port ?? 587));
      setEditUsername(String(cfg.username ?? ""));
      setEditPassword("");
      setEditApiKey("");
      const storedType: AuthType = "api_key" in cfg ? "api_key" : "credentials";
      setEditAuthType(storedType);
      setEditFromEmail(String(cfg.from_email ?? ""));
      setEditUseTls(Boolean(cfg.use_tls));
      setEditIsActive(c.is_active);
      setTestResults((prev) => { const next = { ...prev }; delete next[c.configuration_id]; return next; });
    } finally {
      setEditLoading(null);
    }
  };

  const handleTest = async (id: string) => {
    setTestLoading(id);
    try {
      const result = await adminApi.testProviderConfig(id);
      setTestResults((prev) => ({ ...prev, [id]: result }));
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : "Unknown error";
      setTestResults((prev) => ({ ...prev, [id]: { success: false, message: msg } }));
    } finally {
      setTestLoading(null);
    }
  };

  const handleSendTestEmail = async () => {
    if (!testEmailId) return;
    setTestEmailLoading(true);
    try {
      const result = await adminApi.sendTestEmail(testEmailId, testEmailAddr);
      setTestEmailResult(result);
    } catch (err: unknown) {
      setTestEmailResult({ success: false, message: err instanceof Error ? err.message : "Unknown error" });
    } finally {
      setTestEmailLoading(false);
    }
  };

  const handleReveal = async (id: string) => {
    setRevealLoading(id);
    try {
      const data = await adminApi.revealProviderConfig(id);
      setRevealData(data);
    } finally {
      setRevealLoading(null);
    }
  };

  return (
    <div className={styles.page}>
      {showForm && (
        <div className={styles.overlay} onClick={() => setShowForm(false)}>
          <div className={styles.modal} onClick={(e) => e.stopPropagation()}>
            <div className={styles.modalHeader}>
              <h2 className={styles.modalTitle}>Add Provider Configuration</h2>
              <button className={styles.modalClose} onClick={() => setShowForm(false)}>✕</button>
            </div>
            <form className={styles.form} onSubmit={handleCreate}>
              <label className={styles.label}>
                Provider
                <select
                  value={provider}
                  onChange={(e) => handleProviderChange(e.target.value)}
                >
                  {Object.entries(PROVIDER_LABELS).map(([value, label]) => (
                    <option key={value} value={value}>{label}</option>
                  ))}
                </select>
              </label>
              <label className={styles.label}>
                Host
                <input
                  type="text"
                  value={host}
                  onChange={(e) => setHost(e.target.value)}
                  placeholder="smtp.example.com"
                  required
                />
              </label>
              <label className={styles.label}>
                Port
                <input
                  type="number"
                  min={1}
                  max={65535}
                  value={port}
                  onChange={(e) => setPort(Number(e.target.value))}
                  required
                />
              </label>
              <label className={styles.label}>
                Authentication
                <select
                  value={authType}
                  onChange={(e) => { setAuthType(e.target.value as AuthType); setPassword(""); setApiKey(""); }}
                >
                  <option value="credentials">Username & Password</option>
                  <option value="api_key">API Key</option>
                </select>
              </label>
              {authType === "credentials" ? (
                <>
                  <label className={styles.label}>
                    Username
                    <input
                      type="text"
                      value={username}
                      onChange={(e) => setUsername(e.target.value)}
                      placeholder="username"
                    />
                  </label>
                  <label className={styles.label}>
                    Password
                    <input
                      type="password"
                      value={password}
                      onChange={(e) => setPassword(e.target.value)}
                      required
                    />
                  </label>
                </>
              ) : (
                <label className={styles.label}>
                  API Key
                  <input
                    type="password"
                    value={apiKey}
                    onChange={(e) => setApiKey(e.target.value)}
                    placeholder="sk_..."
                    required
                  />
                </label>
              )}
              <label className={styles.label}>
                From Email
                <input
                  type="email"
                  value={fromEmail}
                  onChange={(e) => setFromEmail(e.target.value)}
                  placeholder="no-reply@yourdomain.com"
                  required
                />
              </label>
              <label className={styles.checkboxLabel}>
                <input
                  type="checkbox"
                  checked={useTls}
                  onChange={(e) => setUseTls(e.target.checked)}
                />
                Use TLS (port 465) — uncheck for STARTTLS (port 587)
              </label>
              <label className={styles.checkboxLabel}>
                <input
                  type="checkbox"
                  checked={isActive}
                  onChange={(e) => setIsActive(e.target.checked)}
                />
                Active
              </label>
              {createMutation.error && (
                <span className={styles.error}>
                  {createMutation.error instanceof Error
                    ? createMutation.error.message
                    : "Save failed"}
                </span>
              )}
              <div className={styles.modalActions}>
                <Button type="submit" loading={createMutation.isPending}>
                  Save Provider
                </Button>
                <Button variant="ghost" onClick={() => setShowForm(false)}>Cancel</Button>
              </div>
            </form>
          </div>
        </div>
      )}

      {revealData && (
        <div className={styles.overlay} onClick={() => setRevealData(null)}>
          <div className={styles.modal} onClick={(e) => e.stopPropagation()}>
            <div className={styles.modalHeader}>
              <h2 className={styles.modalTitle}>Revealed: {revealData.provider}</h2>
              <button className={styles.modalClose} onClick={() => setRevealData(null)}>
                ✕
              </button>
            </div>
            <p className={styles.modalWarning}>
              This contains plaintext secrets. Close this dialog when done.
            </p>
            <pre className={styles.revealJson}>{JSON.stringify(revealData.config, null, 2)}</pre>
          </div>
        </div>
      )}

      {testEmailId && (
        <div className={styles.overlay} onClick={() => setTestEmailId(null)}>
          <div className={styles.modal} onClick={(e) => e.stopPropagation()}>
            <div className={styles.modalHeader}>
              <h2 className={styles.modalTitle}>Send Test Email</h2>
              <button className={styles.modalClose} onClick={() => setTestEmailId(null)}>✕</button>
            </div>
            <p className={styles.modalWarning}>
              A test email will be sent using the stored SMTP configuration.
            </p>
            <label className={styles.modalLabel}>
              Recipient email
              <input
                type="email"
                value={testEmailAddr}
                onChange={(e) => setTestEmailAddr(e.target.value)}
                placeholder="you@example.com"
                required
              />
            </label>
            {testEmailResult && (
              <p className={`${styles.modalResult} ${testEmailResult.success ? styles.testResultSuccess : styles.testResultError}`}>
                {testEmailResult.success ? "✓ " : "✗ "}{testEmailResult.message}
              </p>
            )}
            <div className={styles.modalActions}>
              <Button
                size="sm"
                loading={testEmailLoading}
                onClick={handleSendTestEmail}
                disabled={!testEmailAddr}
              >
                Send
              </Button>
              <Button size="sm" variant="ghost" onClick={() => setTestEmailId(null)}>Cancel</Button>
            </div>
          </div>
        </div>
      )}

      <PageHeader
        title="Provider Configurations"
        subtitle="Configure SMTP email providers for transactional emails."
        action={
          <Button onClick={() => setShowForm(true)}>
            Add Provider
          </Button>
        }
      />

      {isLoading ? (
        <EmptyState message="Loading configurations…" />
      ) : (
        <div className={styles.grid}>
          {configs?.map((c) => (
            <Card key={c.configuration_id} title={`${PROVIDER_LABELS[c.provider] ?? c.provider}`}>
              {editId === c.configuration_id ? (
                <form className={styles.form} onSubmit={(e) => handleUpdate(e, c.configuration_id)}>
                  <label className={styles.label}>
                    Host
                    <input
                      type="text"
                      value={editHost}
                      onChange={(e) => setEditHost(e.target.value)}
                      required
                    />
                  </label>
                  <label className={styles.label}>
                    Port
                    <input
                      type="number"
                      min={1}
                      max={65535}
                      value={editPort}
                      onChange={(e) => setEditPort(Number(e.target.value))}
                      required
                    />
                  </label>
                  <label className={styles.label}>
                    Authentication
                    <select
                      value={editAuthType}
                      onChange={(e) => { setEditAuthType(e.target.value as AuthType); setEditPassword(""); setEditApiKey(""); }}
                    >
                      <option value="credentials">Username & Password</option>
                      <option value="api_key">API Key</option>
                    </select>
                  </label>
                  {editAuthType === "credentials" ? (
                    <>
                      <label className={styles.label}>
                        Username
                        <input
                          type="text"
                          value={editUsername}
                          onChange={(e) => setEditUsername(e.target.value)}
                        />
                      </label>
                      <label className={styles.label}>
                        Password
                        <input
                          type="password"
                          value={editPassword}
                          onChange={(e) => setEditPassword(e.target.value)}
                          required
                        />
                        <span className={styles.warning}>
                          Password is required — enter the current or a new value.
                        </span>
                      </label>
                    </>
                  ) : (
                    <label className={styles.label}>
                      API Key
                      <input
                        type="password"
                        value={editApiKey}
                        onChange={(e) => setEditApiKey(e.target.value)}
                        placeholder="sk_..."
                        required
                      />
                      <span className={styles.warning}>
                        API key is required — enter the current or a new value.
                      </span>
                    </label>
                  )}
                  <label className={styles.label}>
                    From Email
                    <input
                      type="email"
                      value={editFromEmail}
                      onChange={(e) => setEditFromEmail(e.target.value)}
                      required
                    />
                  </label>
                  <label className={styles.checkboxLabel}>
                    <input
                      type="checkbox"
                      checked={editUseTls}
                      onChange={(e) => setEditUseTls(e.target.checked)}
                    />
                    Use TLS (port 465) — uncheck for STARTTLS (port 587)
                  </label>
                  <label className={styles.checkboxLabel}>
                    <input
                      type="checkbox"
                      checked={editIsActive}
                      onChange={(e) => setEditIsActive(e.target.checked)}
                    />
                    Active
                  </label>
                  {updateMutation.error && (
                    <span className={styles.error}>
                      {updateMutation.error instanceof Error
                        ? updateMutation.error.message
                        : "Update failed"}
                    </span>
                  )}
                  <div className={styles.actions}>
                    <Button type="submit" size="sm" loading={updateMutation.isPending}>
                      Save
                    </Button>
                    <Button size="sm" variant="ghost" onClick={() => setEditId(null)}>
                      Cancel
                    </Button>
                  </div>
                </form>
              ) : (
                <>
                  <pre className={styles.preview}>
                    {JSON.stringify(c.config_redacted, null, 2)}
                  </pre>
                  {testResults[c.configuration_id] && (
                    <p className={testResults[c.configuration_id].success ? styles.testResultSuccess : styles.testResultError}>
                      {testResults[c.configuration_id].success ? "✓ " : "✗ "}
                      {testResults[c.configuration_id].message}
                    </p>
                  )}
                  <div className={styles.actions}>
                    <Badge variant={c.is_active ? "active" : "inactive"}>
                      {c.is_active ? "Active" : "Inactive"}
                    </Badge>
                    <Button
                      size="sm"
                      variant="ghost"
                      loading={editLoading === c.configuration_id}
                      onClick={() => openEdit(c)}
                    >
                      Edit
                    </Button>
                    <Button
                      size="sm"
                      variant="ghost"
                      loading={testLoading === c.configuration_id}
                      onClick={() => handleTest(c.configuration_id)}
                    >
                      Check Connection
                    </Button>
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={() => {
                        setTestEmailId(c.configuration_id);
                        setTestEmailAddr("");
                        setTestEmailResult(null);
                      }}
                    >
                      Test Email
                    </Button>
                    <Button
                      size="sm"
                      variant="ghost"
                      loading={revealLoading === c.configuration_id}
                      onClick={() => handleReveal(c.configuration_id)}
                    >
                      Reveal
                    </Button>
                    <Button
                      size="sm"
                      variant="danger"
                      loading={deleteMutation.isPending}
                      onClick={() => {
                        if (confirm("Delete this provider configuration?")) {
                          deleteMutation.mutate(c.configuration_id);
                        }
                      }}
                    >
                      Delete
                    </Button>
                  </div>
                </>
              )}
            </Card>
          ))}
          {!configs?.length && (
            <EmptyState message="No provider configurations. Add one to enable email sending." />
          )}
        </div>
      )}
    </div>
  );
}
