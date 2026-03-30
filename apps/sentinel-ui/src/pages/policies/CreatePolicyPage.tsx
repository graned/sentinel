import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { adminApi } from "../../api/admin";
import { Card } from "../../components/ui/Card";
import { Button } from "@sentinel/auth-react";
import { PageHeader } from "../../components/ui/PageHeader";
import { RulesBuilder, parseRules, EMPTY_RULE } from "./rulesHelpers";
import { formatApiError } from "../../lib/formatApiError";
import type { PolicyRule } from "../../types";
import styles from "./PoliciesPage.module.css";

export function CreatePolicyPage() {
  const navigate = useNavigate();
  const qc = useQueryClient();

  const [name, setName] = useState("");
  const [rules, setRules] = useState<PolicyRule[]>([{ ...EMPTY_RULE }]);
  const [rolesInput, setRolesInput] = useState<string[]>([""]);

  const createMutation = useMutation({
    mutationFn: ({ name, rules }: { name: string; rules: PolicyRule[] }) =>
      adminApi.createPolicy({ name, environment: "production", rules }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["policies"] });
      navigate("/policies");
    },
  });

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    createMutation.mutate({ name, rules: parseRules(rules, rolesInput) });
  }

  return (
    <div className={styles.page}>
      <PageHeader
        title="New Policy"
        subtitle="Define a name and access control rules for your new policy."
        action={
          <Button variant="ghost" onClick={() => navigate("/policies")}>
            ← Back to Policies
          </Button>
        }
      />

      <Card>
        <form className={styles.form} onSubmit={handleSubmit}>
          <div className={styles.modalActions}>
            <Button type="submit" loading={createMutation.isPending}>
              Create Policy
            </Button>
          </div>

          {createMutation.isError && (
            <p className={styles.error}>
              {formatApiError(createMutation.error)}
            </p>
          )}

          <label className={styles.label}>
            Policy name
            <input
              value={name}
              onChange={(e) => setName(e.target.value)}
              required
              placeholder="my-api-policy"
              autoFocus
            />
          </label>

          <RulesBuilder
            rules={rules}
            rolesInput={rolesInput}
            onAddRule={() => {
              setRules([...rules, { ...EMPTY_RULE }]);
              setRolesInput([...rolesInput, ""]);
            }}
            onRemoveRule={(i) => {
              setRules(rules.filter((_, idx) => idx !== i));
              setRolesInput(rolesInput.filter((_, idx) => idx !== i));
            }}
            onSetRule={(i, patch) =>
              setRules(rules.map((r, idx) => (idx === i ? { ...r, ...patch } : r)))
            }
            onSetRolesInput={(i, val) =>
              setRolesInput(rolesInput.map((r, idx) => (idx === i ? val : r)))
            }
          />
        </form>
      </Card>
    </div>
  );
}
