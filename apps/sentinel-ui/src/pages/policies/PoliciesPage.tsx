import { useState, useEffect, useRef } from "react";
import { useNavigate } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { adminApi } from "../../api/admin";
import { Card } from "../../components/ui/Card";
import { Button } from "@sentinel/auth-react";
import { PageHeader } from "../../components/ui/PageHeader";
import { Badge } from "../../components/ui/Badge";
import { EmptyState } from "../../components/ui/EmptyState";
import type {
  BatchCheckData,
  PolicyData,
  PolicyRulesData,
  RoleData,
  RunProbeData,
} from "@sentinel/auth-sdk";
import { METHOD_VARIANTS } from "./rulesHelpers";
import styles from "./PoliciesPage.module.css";

// ── Read-only rules table used in the inspect drawer ──────────────────────────
function ReadOnlyRulesTable({ rulesData }: { rulesData: PolicyRulesData }) {
  return (
    <div>
      <div className={styles.inspectMeta}>
        <span className={styles.muted}>Version v{rulesData.version}</span>
        <span className={styles.ruleCount}>{rulesData.rules.length} rules</span>
      </div>
      <div className={styles.rulesList}>
        {rulesData.rules.length === 0 ? (
          <EmptyState message="No rules defined yet." />
        ) : (
          rulesData.rules.map((rule, i) => (
            <div key={i} className={`${styles.ruleRow} ${styles.ruleRowReadOnly}`}>
              <Badge variant={METHOD_VARIANTS[rule.method] ?? "muted"}>{rule.method}</Badge>
              <code className={styles.pathCode}>{rule.path}</code>
              <div className={styles.rolePills}>
                {rule.roles.map((r) => (
                  <span key={r} className={styles.rolePill}>{r}</span>
                ))}
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}

// ── Role chip multiselect ─────────────────────────────────────────────────────
function RoleChipSelect({
  selected,
  available,
  loading,
  onChange,
}: {
  selected: string[];
  available: RoleData[] | undefined;
  loading: boolean;
  onChange: (roles: string[]) => void;
}) {
  const [open, setOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  // Close dropdown when clicking outside
  useEffect(() => {
    if (!open) return;
    function handle(e: MouseEvent) {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    }
    document.addEventListener("mousedown", handle);
    return () => document.removeEventListener("mousedown", handle);
  }, [open]);

  const unselected = (available ?? []).filter((r) => !selected.includes(r.role_type));

  function add(roleType: string) {
    onChange([...selected, roleType]);
  }

  function remove(name: string) {
    onChange(selected.filter((r) => r !== name));
    // keep dropdown open so user can keep removing via dropdown clicks
  }

  return (
    <div className={styles.chipSelectWrapper} ref={containerRef}>
      <span className={styles.labelText}>Roles to simulate</span>
      <div
        className={`${styles.chipSelectField} ${open ? styles.chipSelectFieldOpen : ""}`}
        onClick={() => setOpen((v) => !v)}
      >
        {selected.length === 0 && (
          <span className={styles.chipPlaceholder}>Select roles…</span>
        )}
        {selected.map((name) => (
          <span key={name} className={styles.chip}>
            {name}
            <button
              type="button"
              className={styles.chipRemoveBtn}
              onClick={(e) => { e.stopPropagation(); remove(name); }}
              aria-label={`Remove ${name}`}
            >
              ✕
            </button>
          </span>
        ))}
        <span className={styles.chipDropdownTrigger}>
          {open ? "▲" : "▼"}
        </span>
      </div>

      {selected.length > 0 && (
        <button
          type="button"
          className={styles.chipClear}
          onClick={() => onChange([])}
        >
          Clear all
        </button>
      )}

      {open && (
        <div className={styles.chipDropdown}>
          {loading ? (
            <div className={styles.chipDropdownItem} style={{ color: "var(--text-muted)" }}>
              Loading roles…
            </div>
          ) : unselected.length === 0 ? (
            <div className={styles.chipDropdownItem} style={{ color: "var(--text-muted)" }}>
              {available?.length === 0 ? "No roles defined" : "All roles selected"}
            </div>
          ) : (
            unselected.map((r) => (
              <button
                key={r.role_id}
                type="button"
                className={styles.chipDropdownItem}
                onClick={(e) => { e.stopPropagation(); add(r.role_type); }}
              >
                {r.name}
                <span className={styles.chipDropdownDesc}>{r.role_type}</span>
              </button>
            ))
          )}
        </div>
      )}
    </div>
  );
}

// ── Test tab ──────────────────────────────────────────────────────────────────
function TestTab({
  policy,
  rulesData,
  availableRoles,
  rolesLoading,
}: {
  policy: PolicyData;
  rulesData: PolicyRulesData | undefined;
  availableRoles: RoleData[] | undefined;
  rolesLoading: boolean;
}) {
  const [selectedRoles, setSelectedRoles] = useState<string[]>([]);
  const [liveMode, setLiveMode] = useState(false);
  const [baseUrl, setBaseUrl] = useState("https://");
  const [batchResult, setBatchResult] = useState<BatchCheckData | null>(null);
  const [probeResult, setProbeResult] = useState<RunProbeData | null>(null);
  const [loading, setLoading] = useState(false);
  const [testError, setTestError] = useState<string | null>(null);

  async function runOfflineTest() {
    if (!rulesData) return;
    setLoading(true);
    setTestError(null);
    setBatchResult(null);
    setProbeResult(null);
    try {
      const checks = rulesData.rules.map((r) => ({ method: r.method, path: r.path }));
      const result = await adminApi.checkAuthorizationBatch({
        policy_id: policy.policy_id,
        roles: selectedRoles,
        checks,
      });
      setBatchResult(result);
    } catch (e) {
      setTestError((e as Error)?.message ?? "Batch check failed.");
    } finally {
      setLoading(false);
    }
  }

  async function runLiveProbe() {
    setLoading(true);
    setTestError(null);
    setBatchResult(null);
    setProbeResult(null);
    try {
      const result = await adminApi.runPolicyProbe(policy.policy_id, {
        base_url: baseUrl,
        roles: selectedRoles,
      });
      setProbeResult(result);
    } catch (e) {
      setTestError((e as Error)?.message ?? "Probe failed.");
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className={styles.testSection}>
      <RoleChipSelect
        selected={selectedRoles}
        available={availableRoles}
        loading={rolesLoading}
        onChange={setSelectedRoles}
      />

      <div className={styles.liveModeToggle}>
        <label className={styles.toggleLabel}>
          <input
            type="checkbox"
            checked={liveMode}
            onChange={(e) => setLiveMode(e.target.checked)}
          />
          Test against real app (live probe)
        </label>
        {liveMode && (
          <label className={styles.label}>
            App base URL
            <input
              value={baseUrl}
              onChange={(e) => setBaseUrl(e.target.value)}
              placeholder="https://api.myapp.com"
              type="url"
            />
            <span className={styles.hint}>
              Your app must use <code>sentinelExpressMiddleware</code> and be reachable from this server.
            </span>
          </label>
        )}
      </div>

      <div className={styles.modalActions}>
        <Button
          type="button"
          loading={loading}
          disabled={!rulesData || selectedRoles.length === 0 || (liveMode && !baseUrl)}
          onClick={liveMode ? runLiveProbe : runOfflineTest}
        >
          {liveMode ? "Run Probe" : "Run Test"}
        </Button>
      </div>

      {testError && <p className={styles.error}>{testError}</p>}

      {/* Offline batch results */}
      {batchResult && (
        <div className={styles.results}>
          <div className={styles.resultsHeader}>
            Results — v{batchResult.evaluated_version}
          </div>
          <div className={styles.resultsList}>
            {batchResult.results.map((r, i) => (
              <div key={i} className={styles.resultRow}>
                <Badge variant={METHOD_VARIANTS[r.method] ?? "muted"}>{r.method}</Badge>
                <code className={styles.pathCode}>{r.path}</code>
                <span className={r.allowed ? styles.allowed : styles.denied}>
                  {r.allowed ? "ALLOWED" : "DENIED"}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Live probe results */}
      {probeResult && (
        <div className={styles.results}>
          <div className={styles.resultsHeader}>
            Probe results — v{probeResult.evaluated_version} — {probeResult.base_url}
          </div>
          <div className={styles.resultsList}>
            {probeResult.results.map((r, i) => (
              <div key={i} className={styles.resultRow}>
                <Badge variant={METHOD_VARIANTS[r.method] ?? "muted"}>{r.method}</Badge>
                <code className={styles.pathCode}>{r.path}</code>
                <span className={r.allowed ? styles.allowed : styles.denied}>
                  {r.allowed ? "ALLOWED" : "DENIED"}
                </span>
                {r.status_code !== undefined && (
                  <span className={styles.muted}>{r.status_code}</span>
                )}
                {r.error && <span className={styles.probeError}>{r.error}</span>}
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

// ── Main page component ───────────────────────────────────────────────────────

export function PoliciesPage() {
  const navigate = useNavigate();
  const qc = useQueryClient();

  const { data: policies, isLoading, isError, error } = useQuery({
    queryKey: ["policies"],
    queryFn: () => adminApi.listPolicies(),
  });

  const { data: availableRoles, isLoading: rolesLoading } = useQuery({
    queryKey: ["roles"],
    queryFn: () => adminApi.listRoles(),
  });

  // ── Delete state ───────────────────────────────────────────────────────────
  const [confirmDeleteId, setConfirmDeleteId] = useState<string | null>(null);

  const deleteMutation = useMutation({
    mutationFn: (id: string) => adminApi.deletePolicy(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["policies"] });
      setConfirmDeleteId(null);
    },
  });

  // ── Inspect drawer state ───────────────────────────────────────────────────
  const [inspectingPolicy, setInspectingPolicy] = useState<PolicyData | null>(null);
  const [activeTab, setActiveTab] = useState<"rules" | "test">("rules");

  // Rules query — used by inspect drawer
  const { data: policyRules, isLoading: rulesLoading } = useQuery({
    queryKey: ["policy-rules", inspectingPolicy?.policy_id],
    queryFn: () => adminApi.getPolicyRules(inspectingPolicy!.policy_id),
    enabled: !!inspectingPolicy,
  });

  // ── Inspect drawer helpers ─────────────────────────────────────────────────
  function openInspect(policy: PolicyData) {
    setActiveTab("rules");
    setInspectingPolicy(policy);
  }

  function closeInspect() {
    setInspectingPolicy(null);
  }

  // ── Render ─────────────────────────────────────────────────────────────────
  return (
    <div className={styles.page}>
      <PageHeader
        title="Policies"
        subtitle="Define access control rules for your API endpoints."
        action={
          <Button variant="primary" onClick={() => navigate("/policies/new")}>
            New Policy
          </Button>
        }
      />

      <Card>
        {isLoading ? (
          <EmptyState message="Loading policies…" />
        ) : isError ? (
          <p className={styles.error}>
            Failed to load policies:{" "}
            {(error as Error)?.message ?? "unexpected error"}
          </p>
        ) : !policies?.length ? (
          <EmptyState message="No policies yet. Create one to get started." />
        ) : (
          <div className={styles.tableWrap}>
            <table className={styles.table}>
              <thead>
                <tr>
                  <th>Name</th>
                  <th>Environment</th>
                  <th>Active version</th>
                  <th>Created</th>
                  <th></th>
                </tr>
              </thead>
              <tbody>
                {policies.map((p) => (
                  <tr key={p.policy_id}>
                    <td className={styles.bold}>{p.name}</td>
                    <td>
                      <Badge variant="muted">{p.environment}</Badge>
                    </td>
                    <td className={styles.muted}>v{p.active_version}</td>
                    <td className={styles.muted}>
                      {new Date(p.created_at).toLocaleDateString()}
                    </td>
                    <td>
                      <div className={styles.rowActions}>
                        {confirmDeleteId === p.policy_id ? (
                          <>
                            <span className={styles.muted}>Are you sure?</span>
                            <Button
                              size="sm"
                              variant="danger"
                              loading={deleteMutation.isPending}
                              onClick={() => deleteMutation.mutate(p.policy_id)}
                            >
                              Yes, delete
                            </Button>
                            <Button
                              size="sm"
                              variant="ghost"
                              onClick={() => setConfirmDeleteId(null)}
                            >
                              Cancel
                            </Button>
                          </>
                        ) : (
                          <>
                            <Button
                              size="sm"
                              variant="ghost"
                              onClick={() => navigate(`/policies/${p.policy_id}/rules`, { state: { policy: p } })}
                            >
                              Edit Rules
                            </Button>
                            <Button size="sm" variant="ghost" onClick={() => openInspect(p)}>
                              Inspect / Test
                            </Button>
                            <Button
                              size="sm"
                              variant="ghost"
                              onClick={() => setConfirmDeleteId(p.policy_id)}
                            >
                              Delete
                            </Button>
                          </>
                        )}
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </Card>

      {/* ── Inspect / Test drawer ── */}
      {inspectingPolicy && (
        <div className={styles.drawerOverlay} onClick={closeInspect}>
          <div className={styles.drawer} onClick={(e) => e.stopPropagation()}>
            <div className={styles.drawerHeader}>
              <div>
                <h2 className={styles.modalTitle}>{inspectingPolicy.name}</h2>
                <span className={styles.muted}>{inspectingPolicy.environment}</span>
              </div>
              <button className={styles.modalClose} onClick={closeInspect} aria-label="Close">
                ✕
              </button>
            </div>

            <div className={styles.tabs}>
              <button
                type="button"
                className={`${styles.tab} ${activeTab === "rules" ? styles.tabActive : ""}`}
                onClick={() => setActiveTab("rules")}
              >
                Rules
              </button>
              <button
                type="button"
                className={`${styles.tab} ${activeTab === "test" ? styles.tabActive : ""}`}
                onClick={() => setActiveTab("test")}
              >
                Test
              </button>
            </div>

            <div className={styles.drawerBody}>
              {rulesLoading ? (
                <EmptyState message="Loading rules…" />
              ) : activeTab === "rules" ? (
                policyRules ? (
                  <ReadOnlyRulesTable rulesData={policyRules} />
                ) : (
                  <EmptyState message="No rules loaded." />
                )
              ) : (
                <TestTab
                  policy={inspectingPolicy}
                  rulesData={policyRules}
                  availableRoles={availableRoles}
                  rolesLoading={rolesLoading}
                />
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
