import { useState, useEffect } from "react";
import { useParams, useNavigate, useLocation } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { adminApi } from "../../api/admin";
import { Card } from "../../components/ui/Card";
import { Button } from "@sentinel/auth-react";
import { PageHeader } from "../../components/ui/PageHeader";
import { EmptyState } from "../../components/ui/EmptyState";
import type { PolicyData } from "@sentinel/auth-sdk";
import { RulesBuilder, parseRules, EMPTY_RULE } from "./rulesHelpers";
import { formatApiError } from "../../lib/formatApiError";
import type { PolicyRule } from "../../types";
import styles from "./PoliciesPage.module.css";

export function EditRulesPage() {
  const { policyId } = useParams<{ policyId: string }>();
  const navigate = useNavigate();
  const location = useLocation();
  const qc = useQueryClient();

  const policy = (location.state as { policy?: PolicyData } | null)?.policy;

  const [editRules, setEditRules] = useState<PolicyRule[]>([{ ...EMPTY_RULE }]);
  const [editRolesInput, setEditRolesInput] = useState<string[]>([""]);

  const { data: policyRules, isLoading: rulesLoading } = useQuery({
    queryKey: ["policy-rules", policyId],
    queryFn: () => adminApi.getPolicyRules(policyId!),
    enabled: !!policyId,
  });

  useEffect(() => {
    if (!policyRules) return;
    setEditRules(policyRules.rules.map((r) => ({ ...r })));
    setEditRolesInput(policyRules.rules.map((r) => r.roles.join(", ")));
  }, [policyRules]);

  const updateRulesMutation = useMutation({
    mutationFn: ({ id, rules }: { id: string; rules: PolicyRule[] }) =>
      adminApi.updatePolicyRules(id, { rules }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["policies"] });
      qc.invalidateQueries({ queryKey: ["policy-rules", policyId] });
    },
  });

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!policyId) return;
    updateRulesMutation.mutate({
      id: policyId,
      rules: parseRules(editRules, editRolesInput),
    });
  }

  const policyName = policy?.name ?? "Policy";

  return (
    <div className={styles.page}>
      <PageHeader
        title={`Edit Rules — ${policyName}`}
        subtitle="Modify the access control rules for this policy."
        action={
          <Button variant="ghost" onClick={() => navigate("/policies")}>
            ← Back to Policies
          </Button>
        }
      />

      <Card>
        {updateRulesMutation.isSuccess && (
          <div className={styles.success}>Rules saved successfully.</div>
        )}

        <form className={styles.form} onSubmit={handleSubmit}>
          <div className={styles.modalActions}>
            <Button type="submit" loading={updateRulesMutation.isPending} disabled={rulesLoading}>
              Save Rules
            </Button>
          </div>

          {updateRulesMutation.isError && (
            <p className={styles.error}>
              {formatApiError(updateRulesMutation.error)}
            </p>
          )}

          {rulesLoading ? (
            <EmptyState message="Loading current rules…" />
          ) : (
            <RulesBuilder
              rules={editRules}
              rolesInput={editRolesInput}
              onAddRule={() => {
                setEditRules([...editRules, { ...EMPTY_RULE }]);
                setEditRolesInput([...editRolesInput, ""]);
              }}
              onRemoveRule={(i) => {
                setEditRules(editRules.filter((_, idx) => idx !== i));
                setEditRolesInput(editRolesInput.filter((_, idx) => idx !== i));
              }}
              onSetRule={(i, patch) =>
                setEditRules(editRules.map((r, idx) => (idx === i ? { ...r, ...patch } : r)))
              }
              onSetRolesInput={(i, val) =>
                setEditRolesInput(editRolesInput.map((r, idx) => (idx === i ? val : r)))
              }
            />
          )}
        </form>
      </Card>
    </div>
  );
}
