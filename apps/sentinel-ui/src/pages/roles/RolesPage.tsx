import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { adminApi } from "../../api/admin";
import { Card } from "../../components/ui/Card";
import { Button } from "@sentinel/auth-react";
import { Badge } from "../../components/ui/Badge";
import { PageHeader } from "../../components/ui/PageHeader";
import { EmptyState } from "../../components/ui/EmptyState";
import type { CreateRoleRequest } from "../../types";
import styles from "./RolesPage.module.css";

const ROLE_TYPES = ["user", "admin", "support"] as const;

type RoleType = (typeof ROLE_TYPES)[number];

const ROLE_BADGE: Record<RoleType, "blue" | "muted" | "warning"> = {
  admin: "blue",
  user: "muted",
  support: "warning",
};

export function RolesPage() {
  const qc = useQueryClient();
  const [showModal, setShowModal] = useState(false);
  const [form, setForm] = useState<CreateRoleRequest>({
    role_type: "user",
    name: "",
    description: "",
  });

  const { data: roles, isLoading } = useQuery({
    queryKey: ["roles"],
    queryFn: () => adminApi.listRoles(),
  });

  const createMutation = useMutation({
    mutationFn: (data: CreateRoleRequest) => adminApi.createRole(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["roles"] });
      setShowModal(false);
      setForm({ role_type: "user", name: "", description: "" });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (roleId: string) => adminApi.deleteRole(roleId),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["roles"] }),
  });

  function closeModal() {
    setShowModal(false);
    setForm({ role_type: "user", name: "", description: "" });
  }

  return (
    <div className={styles.page}>
      <PageHeader
        title="Roles"
        subtitle="Manage access roles and their types."
        action={
          <Button onClick={() => setShowModal(true)} variant="primary">
            New Role
          </Button>
        }
      />

      <Card>
        {isLoading ? (
          <EmptyState message="Loading roles…" />
        ) : roles?.length ? (
          <div className={styles.tableWrap}>
            <table className={styles.table}>
              <thead>
                <tr>
                  <th>Name</th>
                  <th>Type</th>
                  <th>Description</th>
                  <th></th>
                </tr>
              </thead>
              <tbody>
                {roles.map((r) => (
                  <tr key={r.role_id}>
                    <td className={styles.bold}>{r.name}</td>
                    <td>
                      <Badge variant={ROLE_BADGE[r.role_type as RoleType] ?? "muted"}>
                        {r.role_type}
                      </Badge>
                    </td>
                    <td className={styles.muted}>{r.description ?? "—"}</td>
                    <td>
                      <Button
                        variant="danger"
                        size="sm"
                        loading={deleteMutation.isPending}
                        onClick={() => deleteMutation.mutate(r.role_id)}
                      >
                        Delete
                      </Button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <EmptyState message="No roles yet. Create one to get started." />
        )}
      </Card>

      {showModal && (
        <div className={styles.overlay} onClick={closeModal}>
          <div className={styles.modal} onClick={(e) => e.stopPropagation()}>
            <div className={styles.modalHeader}>
              <h2 className={styles.modalTitle}>Create Role</h2>
              <button className={styles.modalClose} onClick={closeModal}>
                ✕
              </button>
            </div>
            <form
              className={styles.form}
              onSubmit={(e) => {
                e.preventDefault();
                createMutation.mutate(form);
              }}
            >
              <label className={styles.label}>
                Type
                <select
                  value={form.role_type}
                  onChange={(e) => setForm({ ...form, role_type: e.target.value })}
                >
                  {ROLE_TYPES.map((t) => (
                    <option key={t} value={t}>
                      {t}
                    </option>
                  ))}
                </select>
              </label>
              <label className={styles.label}>
                Name
                <input
                  value={form.name}
                  onChange={(e) => setForm({ ...form, name: e.target.value })}
                  required
                  placeholder="e.g. super-admin"
                />
              </label>
              <label className={styles.label}>
                Description
                <input
                  value={form.description ?? ""}
                  onChange={(e) => setForm({ ...form, description: e.target.value })}
                  placeholder="Optional description"
                />
              </label>
              {createMutation.isError && (
                <p className={styles.error}>
                  {(createMutation.error as Error)?.message ?? "Failed to create role."}
                </p>
              )}
              <div className={styles.modalActions}>
                <Button type="submit" loading={createMutation.isPending}>
                  Create Role
                </Button>
                <Button type="button" variant="ghost" onClick={closeModal}>
                  Cancel
                </Button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
