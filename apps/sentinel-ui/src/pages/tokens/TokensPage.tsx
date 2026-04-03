import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { adminApi } from "../../api/admin";
import { Card } from "../../components/ui/Card";
import { Button } from "@sentinel/auth-react";
import { Badge } from "../../components/ui/Badge";
import { PageHeader } from "../../components/ui/PageHeader";
import { EmptyState } from "../../components/ui/EmptyState";
import type { ApiToken as ApiTokenData, CreateApiTokenRequest } from "../../types";
import type { UserProfileData } from "@sentinel/auth-sdk";
import styles from "./TokensPage.module.css";

// ── Create modal ──────────────────────────────────────────────────────────────
function CreateTokenModal({ onClose }: { onClose: () => void }) {
  const qc = useQueryClient();
  const [form, setForm] = useState<CreateApiTokenRequest>({ name: "" });
  const [createdToken, setCreatedToken] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  const createMutation = useMutation({
    mutationFn: (data: CreateApiTokenRequest) => adminApi.createApiToken(data),
    onSuccess: (res) => {
      qc.invalidateQueries({ queryKey: ["api-tokens"] });
      setCreatedToken(res.token);
    },
  });

  function copy() {
    if (!createdToken) return;
    navigator.clipboard.writeText(createdToken).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    });
  }

  return (
    <div className={styles.overlay} onClick={onClose}>
      <div className={styles.modal} onClick={(e) => e.stopPropagation()}>
        <div className={styles.modalHeader}>
          <h2 className={styles.modalTitle}>
            {createdToken ? "Token Created" : "New API Token"}
          </h2>
          <button className={styles.modalClose} onClick={onClose} aria-label="Close">
            ✕
          </button>
        </div>

        {createdToken ? (
          <div className={styles.tokenCreatedBody}>
            <div className={styles.tokenWarning}>
              Copy this token now — it will not be shown again.
            </div>
            <code className={styles.tokenValue}>{createdToken}</code>
            <div className={styles.modalActions}>
              <Button variant="ghost" onClick={copy}>
                {copied ? "Copied!" : "Copy Token"}
              </Button>
              <Button variant="primary" onClick={onClose}>
                Done
              </Button>
            </div>
          </div>
        ) : (
          <form
            className={styles.form}
            onSubmit={(e) => {
              e.preventDefault();
              createMutation.mutate(form);
            }}
          >
            <label className={styles.label}>
              Name <span className={styles.required}>*</span>
              <input
                value={form.name}
                onChange={(e) => setForm({ ...form, name: e.target.value })}
                required
                placeholder="ci-deploy-token"
              />
            </label>
            <label className={styles.label}>
              Description
              <input
                value={form.description ?? ""}
                onChange={(e) =>
                  setForm({ ...form, description: e.target.value || undefined })
                }
                placeholder="Optional description"
              />
            </label>
            <label className={styles.label}>
              Expires at
              <input
                type="datetime-local"
                value={form.expires_at?.slice(0, 16) ?? ""}
                onChange={(e) =>
                  setForm({
                    ...form,
                    expires_at: e.target.value
                      ? new Date(e.target.value).toISOString()
                      : undefined,
                  })
                }
              />
              <span className={styles.hint}>Leave blank for a non-expiring token.</span>
            </label>
            {createMutation.error && (
              <p className={styles.error}>
                {(createMutation.error as Error).message}
              </p>
            )}
            <div className={styles.modalActions}>
              <Button variant="ghost" type="button" onClick={onClose}>
                Cancel
              </Button>
              <Button type="submit" loading={createMutation.isPending}>
                Create Token
              </Button>
            </div>
          </form>
        )}
      </div>
    </div>
  );
}

// ── Token test tab ────────────────────────────────────────────────────────────
function TestTab() {
  const [rawToken, setRawToken] = useState("");
  const [result, setResult] = useState<UserProfileData | null>(null);
  const [loading, setLoading] = useState(false);
  const [testError, setTestError] = useState<string | null>(null);

  async function verify() {
    const token = rawToken.trim();
    if (!token) return;
    setLoading(true);
    setResult(null);
    setTestError(null);
    try {
      const profile = await adminApi.verifyApiToken(token);
      setResult(profile);
    } catch (e) {
      setTestError((e as Error)?.message ?? "Token verification failed.");
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className={styles.testSection}>
      <p className={styles.testHint}>
        Paste a raw <code>sat_*</code> token to verify it is valid and see the identity
        it resolves to.
      </p>
      <label className={styles.label}>
        Raw API token
        <textarea
          className={styles.tokenInput}
          value={rawToken}
          onChange={(e) => setRawToken(e.target.value)}
          placeholder="sat_…"
          rows={3}
          spellCheck={false}
        />
      </label>
      <div className={styles.modalActions}>
        <Button
          loading={loading}
          disabled={!rawToken.trim()}
          onClick={verify}
        >
          Verify Token
        </Button>
      </div>

      {testError && <p className={styles.error}>{testError}</p>}

      {result && (
        <div className={styles.results}>
          <div className={styles.resultsHeader}>Token resolved successfully</div>
          <div className={styles.resultsList}>
            <div className={styles.resultField}>
              <span className={styles.fieldLabel}>User ID</span>
              <code className={styles.fieldValue}>{result.user_id}</code>
            </div>
            <div className={styles.resultField}>
              <span className={styles.fieldLabel}>Email</span>
              <span className={styles.fieldValue}>{result.email}</span>
            </div>
            <div className={styles.resultField}>
              <span className={styles.fieldLabel}>Name</span>
              <span className={styles.fieldValue}>
                {[result.first_name, result.last_name].filter(Boolean).join(" ") || "—"}
              </span>
            </div>
            <div className={styles.resultField}>
              <span className={styles.fieldLabel}>Status</span>
              <Badge variant={result.status === "Active" ? "active" : "muted"}>
                {result.status}
              </Badge>
            </div>
            <div className={styles.resultField}>
              <span className={styles.fieldLabel}>Email verified</span>
              <Badge variant={result.email_verified ? "active" : "warning"}>
                {result.email_verified ? "Yes" : "No"}
              </Badge>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

// ── Details tab ───────────────────────────────────────────────────────────────
function DetailsTab({ token }: { token: ApiTokenData }) {
  const fmt = (d: string | null) => (d ? new Date(d).toLocaleString() : "—");

  return (
    <div className={styles.detailsSection}>
      <div className={styles.results}>
        <div className={styles.resultsList}>
          <div className={styles.resultField}>
            <span className={styles.fieldLabel}>Token ID</span>
            <code className={styles.fieldValue}>{token.api_token_id}</code>
          </div>
          <div className={styles.resultField}>
            <span className={styles.fieldLabel}>Name</span>
            <span className={styles.fieldValue}>{token.name}</span>
          </div>
          {token.description && (
            <div className={styles.resultField}>
              <span className={styles.fieldLabel}>Description</span>
              <span className={styles.fieldValue}>{token.description}</span>
            </div>
          )}
          <div className={styles.resultField}>
            <span className={styles.fieldLabel}>Status</span>
            <Badge variant={token.revoked_at ? "danger" : "active"}>
              {token.revoked_at ? "Revoked" : "Active"}
            </Badge>
          </div>
          <div className={styles.resultField}>
            <span className={styles.fieldLabel}>Created</span>
            <span className={styles.fieldValue}>{fmt(token.created_at)}</span>
          </div>
          <div className={styles.resultField}>
            <span className={styles.fieldLabel}>Expires</span>
            <span className={styles.fieldValue}>{fmt(token.expires_at)}</span>
          </div>
          <div className={styles.resultField}>
            <span className={styles.fieldLabel}>Last used</span>
            <span className={styles.fieldValue}>{fmt(token.last_used_at)}</span>
          </div>
          {token.revoked_at && (
            <div className={styles.resultField}>
              <span className={styles.fieldLabel}>Revoked at</span>
              <span className={styles.fieldValue}>{fmt(token.revoked_at)}</span>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

// ── Inspect drawer ────────────────────────────────────────────────────────────
function InspectDrawer({
  token,
  onClose,
  onRevoke,
  revoking,
}: {
  token: ApiTokenData;
  onClose: () => void;
  onRevoke: (id: string) => void;
  revoking: boolean;
}) {
  const [activeTab, setActiveTab] = useState<"details" | "test">("details");

  return (
    <div className={styles.drawerOverlay} onClick={onClose}>
      <div className={styles.drawer} onClick={(e) => e.stopPropagation()}>
        <div className={styles.drawerHeader}>
          <div>
            <h2 className={styles.modalTitle}>{token.name}</h2>
            <span className={styles.muted}>
              {token.revoked_at ? "Revoked" : "Active"}
            </span>
          </div>
          <button className={styles.modalClose} onClick={onClose} aria-label="Close">
            ✕
          </button>
        </div>

        <div className={styles.tabs}>
          <button
            type="button"
            className={`${styles.tab} ${activeTab === "details" ? styles.tabActive : ""}`}
            onClick={() => setActiveTab("details")}
          >
            Details
          </button>
          <button
            type="button"
            className={`${styles.tab} ${activeTab === "test" ? styles.tabActive : ""}`}
            onClick={() => setActiveTab("test")}
          >
            Test Access
          </button>
        </div>

        <div className={styles.drawerBody}>
          {activeTab === "details" ? (
            <>
              <DetailsTab token={token} />
              {!token.revoked_at && (
                <div className={styles.drawerFooter}>
                  <Button
                    variant="danger"
                    size="sm"
                    loading={revoking}
                    onClick={() => onRevoke(token.api_token_id)}
                  >
                    Revoke Token
                  </Button>
                </div>
              )}
            </>
          ) : (
            <TestTab />
          )}
        </div>
      </div>
    </div>
  );
}

// ── Main page ─────────────────────────────────────────────────────────────────
export function TokensPage() {
  const qc = useQueryClient();
  const [showCreate, setShowCreate] = useState(false);
  const [inspecting, setInspecting] = useState<ApiTokenData | null>(null);

  const { data: tokens, isLoading } = useQuery({
    queryKey: ["api-tokens"],
    queryFn: () => adminApi.listApiTokens(),
  });

  const revokeMutation = useMutation({
    mutationFn: (id: string) => adminApi.revokeApiToken(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["api-tokens"] });
      setInspecting(null);
    },
  });

  const fmt = (d: string | null) => (d ? new Date(d).toLocaleDateString() : "—");

  return (
    <div className={styles.page}>
      <PageHeader
        title="API Tokens"
        subtitle="Long-lived tokens for programmatic access. Shown only once at creation."
        action={
          <Button variant="primary" onClick={() => setShowCreate(true)}>
            New Token
          </Button>
        }
      />

      <Card>
        {isLoading ? (
          <EmptyState message="Loading tokens…" />
        ) : tokens?.length ? (
          <div className={styles.tableWrap}>
            <table className={styles.table}>
              <thead>
                <tr>
                  <th>Name</th>
                  <th>Created</th>
                  <th>Expires</th>
                  <th>Status</th>
                  <th></th>
                </tr>
              </thead>
              <tbody>
                {tokens.map((t) => (
                  <tr key={t.api_token_id}>
                    <td className={styles.bold}>{t.name}</td>
                    <td className={styles.muted}>{fmt(t.created_at)}</td>
                    <td className={styles.muted}>{fmt(t.expires_at)}</td>
                    <td>
                      <Badge variant={t.revoked_at ? "danger" : "active"}>
                        {t.revoked_at ? "Revoked" : "Active"}
                      </Badge>
                    </td>
                    <td>
                      <div className={styles.rowActions}>
                        <Button
                          size="sm"
                          variant="ghost"
                          onClick={() => setInspecting(t)}
                        >
                          Inspect / Test
                        </Button>
                        {!t.revoked_at && (
                          <Button
                            variant="danger"
                            size="sm"
                            loading={revokeMutation.isPending && inspecting?.api_token_id === t.api_token_id}
                            onClick={() => revokeMutation.mutate(t.api_token_id)}
                          >
                            Revoke
                          </Button>
                        )}
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <EmptyState message="No tokens yet. Create one to get started." />
        )}
      </Card>

      {showCreate && (
        <CreateTokenModal onClose={() => setShowCreate(false)} />
      )}

      {inspecting && (
        <InspectDrawer
          token={inspecting}
          onClose={() => setInspecting(null)}
          onRevoke={(id) => revokeMutation.mutate(id)}
          revoking={revokeMutation.isPending}
        />
      )}
    </div>
  );
}
