import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { adminApi } from "../../api/admin";
import { Card } from "../../components/ui/Card";
import { PageHeader } from "../../components/ui/PageHeader";
import { EmptyState } from "../../components/ui/EmptyState";
import styles from "./SessionsPage.module.css";

export function SessionsPage() {
  const qc = useQueryClient();
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [confirmId, setConfirmId] = useState<string | null>(null);
  const [confirmBulk, setConfirmBulk] = useState(false);

  const { data: sessions, isLoading } = useQuery({
    queryKey: ["admin-sessions"],
    queryFn: () => adminApi.listActiveSessions(),
  });

  const revokeMutation = useMutation({
    mutationFn: (sessionId: string) => adminApi.revokeSession(sessionId),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ["admin-sessions"] });
      setConfirmId(null);
      setSelected(new Set());
    },
  });

  const bulkRevokeMutation = useMutation({
    mutationFn: (ids: string[]) =>
      adminApi.revokeSessionsBulk({ session_ids: ids }),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ["admin-sessions"] });
      setConfirmBulk(false);
      setSelected(new Set());
    },
  });

  const fmt = (d: string | null) => (d ? new Date(d).toLocaleString() : "—");

  const allIds = sessions?.map((s) => s.session_id) ?? [];
  const allSelected = allIds.length > 0 && allIds.every((id) => selected.has(id));

  function toggleAll() {
    if (allSelected) {
      setSelected(new Set());
    } else {
      setSelected(new Set(allIds));
    }
  }

  function toggleOne(id: string) {
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }

  return (
    <div className={styles.page}>
      <PageHeader
        title="Sessions"
        subtitle="All active sessions across all users."
      />

      {selected.size > 0 && (
        <div className={styles.bulkBar}>
          <span className={styles.bulkCount}>{selected.size} selected</span>
          {confirmBulk ? (
            <>
              <span className={styles.confirmText}>
                Invalidate {selected.size} session{selected.size > 1 ? "s" : ""}?
              </span>
              <button
                className={styles.dangerBtn}
                onClick={() => bulkRevokeMutation.mutate([...selected])}
                disabled={bulkRevokeMutation.isPending}
              >
                {bulkRevokeMutation.isPending ? "Invalidating…" : "Confirm"}
              </button>
              <button className={styles.cancelBtn} onClick={() => setConfirmBulk(false)}>
                Cancel
              </button>
            </>
          ) : (
            <button className={styles.dangerBtn} onClick={() => setConfirmBulk(true)}>
              Invalidate Selected
            </button>
          )}
        </div>
      )}

      <Card>
        {isLoading ? (
          <EmptyState message="Loading sessions…" />
        ) : sessions?.length ? (
          <div className={styles.tableWrap}>
            <table className={styles.table}>
              <thead>
                <tr>
                  <th className={styles.checkCol}>
                    <input
                      type="checkbox"
                      checked={allSelected}
                      onChange={toggleAll}
                      aria-label="Select all"
                    />
                  </th>
                  <th>Session ID</th>
                  <th>User</th>
                  <th>IP Address</th>
                  <th>User Agent</th>
                  <th>Created</th>
                  <th>Expires</th>
                  <th className={styles.actionsCol}>Actions</th>
                </tr>
              </thead>
              <tbody>
                {sessions.map((s) => (
                  <tr key={s.session_id}>
                    <td className={styles.checkCol}>
                      <input
                        type="checkbox"
                        checked={selected.has(s.session_id)}
                        onChange={() => toggleOne(s.session_id)}
                        aria-label={`Select session ${s.session_id.slice(0, 8)}`}
                      />
                    </td>
                    <td className={styles.mono}>{s.session_id.slice(0, 8)}…</td>
                    <td className={styles.email}>{s.user_email}</td>
                    <td className={styles.ip}>{s.ip_address ?? "—"}</td>
                    <td className={styles.muted}>
                      {s.user_agent
                        ? s.user_agent.slice(0, 40) +
                          (s.user_agent.length > 40 ? "…" : "")
                        : "—"}
                    </td>
                    <td className={styles.muted}>{fmt(s.created_at)}</td>
                    <td className={styles.muted}>{fmt(s.expires_at)}</td>
                    <td className={styles.actionsCol}>
                      {confirmId === s.session_id ? (
                        <span className={styles.inlineConfirm}>
                          <button
                            className={styles.dangerBtnSm}
                            onClick={() => revokeMutation.mutate(s.session_id)}
                            disabled={revokeMutation.isPending}
                          >
                            {revokeMutation.isPending ? "…" : "Confirm"}
                          </button>
                          <button
                            className={styles.cancelBtnSm}
                            onClick={() => setConfirmId(null)}
                          >
                            Cancel
                          </button>
                        </span>
                      ) : (
                        <button
                          className={styles.dangerBtnSm}
                          onClick={() => setConfirmId(s.session_id)}
                        >
                          Invalidate
                        </button>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <EmptyState message="No active sessions." />
        )}
      </Card>
    </div>
  );
}
