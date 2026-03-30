import { Badge } from "../../components/ui/Badge";
import { Button } from "@sentinel/auth-react";
import type { PolicyRule } from "../../types";
import styles from "./PoliciesPage.module.css";

export const EMPTY_RULE: PolicyRule = { path: "", method: "GET", roles: [] };

export const METHOD_VARIANTS: Record<string, "active" | "blue" | "warning" | "danger" | "muted"> = {
  GET: "active",
  POST: "blue",
  PUT: "warning",
  PATCH: "warning",
  DELETE: "danger",
  "*": "muted",
};

export function parseRules(rules: PolicyRule[], rolesInput: string[]): PolicyRule[] {
  return rules.map((r, i) => ({
    ...r,
    roles: rolesInput[i]
      .split(",")
      .map((s) => s.trim())
      .filter(Boolean),
  }));
}

export function RulesBuilder({
  rules,
  rolesInput,
  onAddRule,
  onRemoveRule,
  onSetRule,
  onSetRolesInput,
}: {
  rules: PolicyRule[];
  rolesInput: string[];
  onAddRule: () => void;
  onRemoveRule: (i: number) => void;
  onSetRule: (i: number, patch: Partial<PolicyRule>) => void;
  onSetRolesInput: (i: number, val: string) => void;
}) {
  return (
    <div className={styles.rulesSection}>
      <div className={styles.rulesHeader}>
        <span className={styles.rulesTitle}>
          Access Rules
          <span className={styles.ruleCount}>{rules.length}</span>
        </span>
        <Button type="button" size="sm" variant="ghost" onClick={onAddRule}>
          + Add Rule
        </Button>
      </div>
      <div className={styles.rulesList}>
        {rules.map((rule, i) => (
          <div key={i} className={styles.ruleRow}>
            <div className={styles.ruleRowInner}>
              <label className={styles.ruleLabel}>
                <span className={styles.ruleLabelText}>Method</span>
                <div className={styles.methodWrapper}>
                  <select
                    value={rule.method}
                    onChange={(e) => onSetRule(i, { method: e.target.value })}
                    className={styles.methodSelect}
                  >
                    {["GET", "POST", "PUT", "PATCH", "DELETE", "*"].map((m) => (
                      <option key={m}>{m}</option>
                    ))}
                  </select>
                  <Badge variant={METHOD_VARIANTS[rule.method] ?? "muted"}>
                    {rule.method}
                  </Badge>
                </div>
              </label>
              <label className={`${styles.ruleLabel} ${styles.grow}`}>
                <span className={styles.ruleLabelText}>Path pattern</span>
                <input
                  value={rule.path}
                  onChange={(e) => onSetRule(i, { path: e.target.value })}
                  placeholder="/v1/api/user/*"
                  className={styles.pathInput}
                />
              </label>
              <label className={`${styles.ruleLabel} ${styles.grow}`}>
                <span className={styles.ruleLabelText}>Roles (comma-separated)</span>
                <input
                  value={rolesInput[i]}
                  onChange={(e) => onSetRolesInput(i, e.target.value)}
                  placeholder="admin, user"
                />
              </label>
            </div>
            <button
              type="button"
              className={styles.removeBtn}
              onClick={() => onRemoveRule(i)}
              aria-label="Remove rule"
            >
              ✕
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}
