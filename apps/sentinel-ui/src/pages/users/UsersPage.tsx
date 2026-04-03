import { useState, useMemo, useEffect, useRef } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useMutation } from "@tanstack/react-query";
import { adminApi } from "../../api/admin";
import { Card } from "../../components/ui/Card";
import { Button } from "@sentinel/auth-react";
import { Badge } from "../../components/ui/Badge";
import { PageHeader } from "../../components/ui/PageHeader";
import { EmptyState } from "../../components/ui/EmptyState";
import type { AdminUser, AdminCreateUserRequest, PaginatedAdminUsersResponse } from "../../types";
import type { RoleData } from "@sentinel/auth-sdk";
import styles from "./UsersPage.module.css";

// ── Helpers ──────────────────────────────────────────────────────────────────

function getInitials(user: AdminUser): string {
  const f = user.first_name?.charAt(0) ?? "";
  const l = user.last_name?.charAt(0) ?? "";
  return (f + l).toUpperCase() || user.email.charAt(0).toUpperCase();
}

function formatDate(iso: string | null): string {
  if (!iso) return "—";
  return new Date(iso).toLocaleDateString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}

function generateSecurePassword(): string {
  const upper = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
  const lower = "abcdefghijklmnopqrstuvwxyz";
  const digits = "0123456789";
  const special = "!@#$%^&*";
  const all = upper + lower + digits + special;
  const arr = new Uint8Array(16);
  crypto.getRandomValues(arr);
  let pwd =
    upper[arr[0] % upper.length] +
    lower[arr[1] % lower.length] +
    digits[arr[2] % digits.length] +
    special[arr[3] % special.length];
  for (let i = 4; i < 16; i++) pwd += all[arr[i] % all.length];
  return pwd;
}

async function copyToClipboard(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    const ta = document.createElement("textarea");
    ta.value = text;
    ta.style.cssText = "position:fixed;opacity:0;top:0;left:0";
    document.body.appendChild(ta);
    ta.select();
    const ok = document.execCommand("copy");
    document.body.removeChild(ta);
    return ok;
  }
}


const STATUS_BADGE: Record<AdminUser["status"], "active" | "warning" | "inactive" | "blue"> = {
  Active: "active",
  Suspended: "warning",
  Inactive: "inactive",
  PendingVerification: "blue",
};

const STATUS_LABEL: Record<AdminUser["status"], string> = {
  Active: "Active",
  Suspended: "Suspended",
  Inactive: "Inactive",
  PendingVerification: "Pending",
};

// ── SVG icons ────────────────────────────────────────────────────────────────

function IconPerson() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="12" cy="8" r="4" />
      <path d="M4 20c0-4 3.6-7 8-7s8 3 8 7" />
    </svg>
  );
}

function IconMail() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <rect x="2" y="4" width="20" height="16" rx="2" />
      <path d="m22 7-8.97 5.7a1.94 1.94 0 0 1-2.06 0L2 7" />
    </svg>
  );
}

function IconShield() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
    </svg>
  );
}

function IconChevronDown() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="m6 9 6 6 6-6" />
    </svg>
  );
}

// ── Page ─────────────────────────────────────────────────────────────────────

const PAGE_SIZE = 20;

export function UsersPage() {
  const qc = useQueryClient();

  // Invite modal state
  const [showInviteModal, setShowInviteModal] = useState(false);
  const [firstName, setFirstName] = useState("");
  const [lastName, setLastName] = useState("");
  const [inviteEmail, setInviteEmail] = useState("");
  const [inviteRoleId, setInviteRoleId] = useState<string | null>(null);
  const [sendInviteEmail, setSendInviteEmail] = useState(true);
  const [dropdownOpen, setDropdownOpen] = useState(false);
  const [formError, setFormError] = useState<string | null>(null);
  const [createdUserId, setCreatedUserId] = useState<string | null>(null);
  const [invitePassword, setInvitePassword] = useState<string | null>(null);
  const [copiedLink, setCopiedLink] = useState(false);
  const [inviteUrl, setInviteUrl] = useState<string | null>(null);
  const [inviteLinkError, setInviteLinkError] = useState<string | null>(null);
  const [inviteLinkLoading, setInviteLinkLoading] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  // Table state
  const [tableCopyingUserId, setTableCopyingUserId] = useState<string | null>(null);
  const [tableCopiedUserId, setTableCopiedUserId] = useState<string | null>(null);
  const [tableInviteUrls, setTableInviteUrls] = useState<Record<string, string>>({});
  const [tableExpandedUserId, setTableExpandedUserId] = useState<string | null>(null);
  const [search, setSearch] = useState("");
  const [page, setPage] = useState(1);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [deleteModalIds, setDeleteModalIds] = useState<string[]>([]);
  const [isDeleting, setIsDeleting] = useState(false);
  const [deleteError, setDeleteError] = useState<string | null>(null);

  useEffect(() => {
    setPage(1);
  }, [search]);

  // Close dropdown on outside click
  useEffect(() => {
    if (!dropdownOpen) return;
    const handler = (e: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(e.target as Node)) {
        setDropdownOpen(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [dropdownOpen]);

  const { data, isLoading } = useQuery<PaginatedAdminUsersResponse>({
    queryKey: ["admin-users", page],
    queryFn: () => adminApi.listUsers({ page, page_size: PAGE_SIZE }),
  });

  const { data: roles } = useQuery<RoleData[]>({
    queryKey: ["roles"],
    queryFn: () => adminApi.listRoles(),
  });

  // Pick a sensible default role when roles load
  useEffect(() => {
    if (roles && roles.length > 0 && inviteRoleId === null) {
      const defaultRole = roles.find((r) => r.role_type === "user") ?? roles[0];
      setInviteRoleId(defaultRole.role_id);
    }
  }, [roles, inviteRoleId]);

  const selectedRole = roles?.find((r) => r.role_id === inviteRoleId) ?? null;

  const createMutation = useMutation({
    mutationFn: (data: AdminCreateUserRequest) => adminApi.createUser(data),
    onSuccess: async (created) => {
      // Assign non-default role if selected
      if (inviteRoleId) {
        try {
          await adminApi.assignRole(created.user_id, { role_id: inviteRoleId });
        } catch {
          // Role assignment failure is non-fatal; user is already created
        }
      }
      qc.invalidateQueries({ queryKey: ["admin-users"] });
      setCreatedUserId(created.user_id);
    },
    onError: (err: unknown) => {
      setFormError((err as { message?: string })?.message ?? "Failed to create user");
    },
  });

  const statusMutation = useMutation({
    mutationFn: ({ userId, status }: { userId: string; status: "active" | "suspended" | "inactive" }) =>
      adminApi.updateUserStatus(userId, { status }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["admin-users"] }),
  });

  const mfaMutation = useMutation({
    mutationFn: ({ userId, required }: { userId: string; required: boolean }) =>
      adminApi.setMfaRequired(userId, { required }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["admin-users"] }),
  });

  const filtered = useMemo(() => {
    if (!data) return [];
    const q = search.trim().toLowerCase();
    if (!q) return data.items;
    return data.items.filter(
      (u) =>
        u.email.toLowerCase().includes(q) ||
        (u.first_name ?? "").toLowerCase().includes(q) ||
        (u.last_name ?? "").toLowerCase().includes(q),
    );
  }, [data, search]);

  const totalCount = data?.total ?? 0;
  const totalPages = Math.max(1, Math.ceil(totalCount / PAGE_SIZE));

  const allSelected = filtered.length > 0 && filtered.every((u) => selectedIds.has(u.user_id));
  const someSelected = filtered.some((u) => selectedIds.has(u.user_id));

  const toggleSelect = (id: string) => {
    setSelectedIds((prev) => {
      const next = new Set(prev);
      next.has(id) ? next.delete(id) : next.add(id);
      return next;
    });
  };

  const toggleAll = () => {
    if (allSelected) {
      setSelectedIds((prev) => {
        const next = new Set(prev);
        filtered.forEach((u) => next.delete(u.user_id));
        return next;
      });
    } else {
      setSelectedIds((prev) => {
        const next = new Set(prev);
        filtered.forEach((u) => next.add(u.user_id));
        return next;
      });
    }
  };

  const handleDelete = (user: AdminUser) => setDeleteModalIds([user.user_id]);
  const handleBulkDelete = () => setDeleteModalIds([...selectedIds]);

  const closeModal = () => {
    setDeleteModalIds([]);
    setDeleteError(null);
  };

  const closeInviteModal = () => {
    setShowInviteModal(false);
    setFirstName("");
    setLastName("");
    setInviteEmail("");
    setInviteRoleId(null);
    setSendInviteEmail(true);
    setFormError(null);
    setDropdownOpen(false);
    setCreatedUserId(null);
    setInvitePassword(null);
    setCopiedLink(false);
    setInviteUrl(null);
    setInviteLinkError(null);
  };

  const handleCopyInviteLink = async () => {
    if (!createdUserId) return;
    // If already fetched, just re-copy
    const cached = inviteUrl;
    if (cached) {
      const ok = await copyToClipboard(cached);
      if (ok) { setCopiedLink(true); setTimeout(() => setCopiedLink(false), 3000); }
      return;
    }
    setInviteLinkLoading(true);
    setInviteLinkError(null);
    try {
      const { invite_url } = await adminApi.getUserInviteLink(createdUserId);
      setInviteUrl(invite_url);
      const ok = await copyToClipboard(invite_url);
      if (ok) { setCopiedLink(true); setTimeout(() => setCopiedLink(false), 3000); }
    } catch (err: unknown) {
      setInviteLinkError((err as { message?: string })?.message ?? "Failed to get invite link");
    } finally {
      setInviteLinkLoading(false);
    }
  };

  const handleTableInviteLink = async (userId: string) => {
    // Toggle collapse if already expanded for this user
    if (tableExpandedUserId === userId) {
      setTableExpandedUserId(null);
      return;
    }
    // Use cached URL if available
    const cached = tableInviteUrls[userId];
    if (cached) {
      const ok = await copyToClipboard(cached);
      if (ok) {
        setTableCopiedUserId(userId);
        setTimeout(() => setTableCopiedUserId(null), 3000);
      } else {
        setTableExpandedUserId(userId);
      }
      return;
    }
    setTableCopyingUserId(userId);
    try {
      const { invite_url } = await adminApi.getUserInviteLink(userId);
      setTableInviteUrls((prev) => ({ ...prev, [userId]: invite_url }));
      const ok = await copyToClipboard(invite_url);
      if (ok) {
        setTableCopiedUserId(userId);
        setTimeout(() => setTableCopiedUserId(null), 3000);
      } else {
        setTableExpandedUserId(userId);
      }
    } catch {
      // Silently fail; user can retry
    } finally {
      setTableCopyingUserId(null);
    }
  };

  const confirmDelete = async () => {
    setIsDeleting(true);
    setDeleteError(null);
    try {
      for (const id of deleteModalIds) {
        await adminApi.deleteUser(id);
      }
      setSelectedIds((prev) => {
        const next = new Set(prev);
        deleteModalIds.forEach((id) => next.delete(id));
        return next;
      });
      qc.invalidateQueries({ queryKey: ["admin-users"] });
      closeModal();
    } catch (err: unknown) {
      setDeleteError((err as { message?: string })?.message ?? "Failed to delete user(s)");
    } finally {
      setIsDeleting(false);
    }
  };

  const handleToggleFreeze = (user: AdminUser) => {
    if (user.status === "Active") {
      statusMutation.mutate({ userId: user.user_id, status: "suspended" });
    } else if (user.status === "Suspended") {
      statusMutation.mutate({ userId: user.user_id, status: "active" });
    }
  };

  const handleInviteSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setFormError(null);
    const password = generateSecurePassword();
    setInvitePassword(password);
    createMutation.mutate({
      email: inviteEmail,
      first_name: firstName,
      last_name: lastName,
      password,
      send_invite_email: sendInviteEmail,
    });
  };

  const allUsers = data?.items ?? [];
  const modalUsers = deleteModalIds
    .map((id) => allUsers.find((u) => u.user_id === id))
    .filter(Boolean) as AdminUser[];

  return (
    <div className={styles.page}>
      <PageHeader
        title="Users"
        subtitle="Manage registered user accounts."
        action={
          <Button onClick={() => setShowInviteModal(true)} variant="primary">
            + Invite User
          </Button>
        }
      />

      {/* Stats row */}
      <div className={styles.statsRow}>
        <div className={styles.statCard}>
          <span className={styles.statLabel}>Total Users</span>
          <span className={styles.statValue}>{totalCount}</span>
        </div>
      </div>

      {/* Search */}
      <div className={styles.searchRow}>
        <input
          className={styles.searchInput}
          placeholder="Search users by name or email…"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
      </div>

      {/* Bulk selection toolbar */}
      {someSelected && (
        <div className={styles.selectionBar}>
          <span className={styles.selectionCount}>
            {selectedIds.size} user{selectedIds.size !== 1 ? "s" : ""} selected
          </span>
          <Button variant="danger" size="sm" onClick={handleBulkDelete}>
            Delete selected
          </Button>
          <Button variant="ghost" size="sm" onClick={() => setSelectedIds(new Set())}>
            Clear
          </Button>
        </div>
      )}

      {/* Table */}
      <Card>
        {isLoading ? (
          <EmptyState message="Loading users…" />
        ) : filtered.length === 0 ? (
          <EmptyState message={search ? "No users match your search." : "No users yet."} />
        ) : (
          <>
            <div className={styles.tableWrap}>
              <table className={styles.table}>
                <thead>
                  <tr>
                    <th className={styles.checkboxCol}>
                      <input
                        type="checkbox"
                        checked={allSelected}
                        onChange={toggleAll}
                        aria-label="Select all users"
                      />
                    </th>
                    <th>User</th>
                    <th>Status</th>
                    <th>MFA</th>
                    <th>Roles</th>
                    <th>Created</th>
                    <th>Actions</th>
                  </tr>
                </thead>
                <tbody>
                  {filtered.map((user) => (
                    <tr key={user.user_id} className={selectedIds.has(user.user_id) ? styles.rowSelected : undefined}>
                      <td className={styles.checkboxCol}>
                        <input
                          type="checkbox"
                          checked={selectedIds.has(user.user_id)}
                          onChange={() => toggleSelect(user.user_id)}
                          aria-label={`Select ${user.email}`}
                        />
                      </td>
                      <td>
                        <div className={styles.userCell}>
                          <div className={styles.avatar}>{getInitials(user)}</div>
                          <div className={styles.userInfo}>
                            <span className={styles.userName}>
                              {user.first_name || user.last_name
                                ? `${user.first_name ?? ""} ${user.last_name ?? ""}`.trim()
                                : "—"}
                            </span>
                            <span className={styles.userEmail}>{user.email}</span>
                          </div>
                        </div>
                      </td>
                      <td>
                        <Badge variant={STATUS_BADGE[user.status]}>
                          {STATUS_LABEL[user.status]}
                        </Badge>
                      </td>
                      <td>
                        <div className={styles.roleBadges}>
                          {user.mfa_enabled ? (
                            <Badge variant="active">Active</Badge>
                          ) : user.mfa_required ? (
                            <Badge variant="warning">Required</Badge>
                          ) : (
                            <span className={styles.muted}>—</span>
                          )}
                        </div>
                      </td>
                      <td>
                        <div className={styles.roleBadges}>
                          {user.roles.length > 0 ? (
                            user.roles.map((r) => (
                              <Badge key={r.role_id} variant="muted">
                                {r.role_type}
                              </Badge>
                            ))
                          ) : (
                            <span className={styles.muted}>—</span>
                          )}
                        </div>
                      </td>
                      <td className={styles.muted}>{formatDate(user.created_at)}</td>
                      <td>
                        <div className={styles.actions} style={{ flexDirection: "column", alignItems: "flex-start", gap: "0.4rem" }}>
                          <div style={{ display: "flex", gap: "0.4rem", flexWrap: "wrap" }}>
                            {(user.status === "Active" || user.status === "Suspended") && (
                              <Button
                                variant="ghost"
                                size="sm"
                                loading={statusMutation.isPending}
                                onClick={() => handleToggleFreeze(user)}
                              >
                                {user.status === "Active" ? "Freeze" : "Unfreeze"}
                              </Button>
                            )}
                            {!user.mfa_enabled && (
                              <Button
                                variant={user.mfa_required ? "primary" : "ghost"}
                                size="sm"
                                loading={mfaMutation.isPending && mfaMutation.variables?.userId === user.user_id}
                                onClick={() => mfaMutation.mutate({ userId: user.user_id, required: !user.mfa_required })}
                              >
                                {user.mfa_required ? "Disable Requirement" : "Require MFA"}
                              </Button>
                            )}
                            {user.status === "PendingVerification" && (
                              <Button
                                variant="ghost"
                                size="sm"
                                loading={tableCopyingUserId === user.user_id}
                                onClick={() => handleTableInviteLink(user.user_id)}
                              >
                                {tableCopiedUserId === user.user_id
                                  ? "Copied!"
                                  : tableExpandedUserId === user.user_id
                                  ? "Hide link"
                                  : "Invite link"}
                              </Button>
                            )}
                            <Button
                              variant="danger"
                              size="sm"
                              onClick={() => handleDelete(user)}
                            >
                              Delete
                            </Button>
                          </div>
                          {tableExpandedUserId === user.user_id && tableInviteUrls[user.user_id] && (
                            <input
                              readOnly
                              value={tableInviteUrls[user.user_id]}
                              className={styles.searchInput}
                              style={{ fontSize: "0.72rem", fontFamily: "monospace", width: "100%", maxWidth: "320px" }}
                              onFocus={(e) => e.target.select()}
                            />
                          )}
                        </div>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
            <div className={styles.pagination}>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setPage((p) => p - 1)}
                disabled={page === 1}
              >
                ← Previous
              </Button>
              <span className={styles.pageInfo}>
                Page {page} of {totalPages} &middot; {totalCount} total
              </span>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setPage((p) => p + 1)}
                disabled={page >= totalPages}
              >
                Next →
              </Button>
            </div>
          </>
        )}
      </Card>

      {/* Invite User modal */}
      {showInviteModal && (
        <div className={styles.overlay} role="dialog" aria-modal="true" aria-labelledby="invite-modal-title">
          <div className={styles.inviteModal}>
            <div className={styles.inviteModalHeader}>
              <h3 id="invite-modal-title" className={styles.inviteModalTitle}>Create User</h3>
              <button
                type="button"
                className={styles.inviteModalClose}
                onClick={closeInviteModal}
                aria-label="Close"
              >
                ×
              </button>
            </div>

            {createdUserId ? (
              <div style={{ display: "flex", flexDirection: "column", gap: "1rem", padding: "0.5rem 0" }}>
                <p style={{ margin: 0, color: "var(--success, #10b981)", fontWeight: 600 }}>
                  User created successfully.
                </p>
                {/* Temporary password */}
                {invitePassword && (
                  <div>
                    <p style={{ margin: "0 0 0.4rem", fontSize: "0.8rem", color: "var(--text-secondary)", fontWeight: 600, textTransform: "uppercase", letterSpacing: "0.05em" }}>
                      Temporary password — share this with the user
                    </p>
                    <div style={{ display: "flex", gap: "0.5rem", alignItems: "center" }}>
                      <input
                        readOnly
                        value={invitePassword}
                        className={styles.inviteInput}
                        style={{ fontFamily: "monospace", fontSize: "0.9rem", flex: 1 }}
                        onFocus={(e) => e.target.select()}
                      />
                      <Button
                        type="button"
                        variant="ghost"
                        size="sm"
                        onClick={async () => { await copyToClipboard(invitePassword); }}
                      >
                        Copy
                      </Button>
                    </div>
                  </div>
                )}
                {/* Invite link */}
                <div>
                  <p style={{ margin: "0 0 0.4rem", fontSize: "0.8rem", color: "var(--text-secondary)", fontWeight: 600, textTransform: "uppercase", letterSpacing: "0.05em" }}>
                    {sendInviteEmail ? "Invite email sent · also copy link" : "Invite link — share with user to verify email"}
                  </p>
                  <Button
                    onClick={handleCopyInviteLink}
                    variant="ghost"
                    loading={inviteLinkLoading}
                  >
                    {copiedLink ? "Copied!" : "Copy invite link"}
                  </Button>
                  {inviteUrl && (
                    <input
                      readOnly
                      value={inviteUrl}
                      className={styles.inviteInput}
                      style={{ marginTop: "0.5rem", fontSize: "0.78rem", fontFamily: "monospace" }}
                      onFocus={(e) => e.target.select()}
                    />
                  )}
                  {inviteLinkError && (
                    <p style={{ margin: "0.4rem 0 0", fontSize: "0.8rem", color: "var(--danger)" }}>
                      {inviteLinkError}
                    </p>
                  )}
                </div>
                <div className={styles.inviteFooter}>
                  <Button type="button" onClick={closeInviteModal}>
                    Done
                  </Button>
                </div>
              </div>
            ) : (
            <form onSubmit={handleInviteSubmit}>
              {/* First Name / Last Name */}
              <div className={styles.inviteNameRow}>
                <div className={styles.inviteField}>
                  <label className={styles.inviteFieldLabel} htmlFor="invite-firstname">
                    <span className={styles.inviteFieldIcon}><IconPerson /></span>
                    First Name
                  </label>
                  <input
                    id="invite-firstname"
                    className={styles.inviteInput}
                    value={firstName}
                    onChange={(e) => setFirstName(e.target.value)}
                    placeholder="Jane"
                    required
                  />
                </div>
                <div className={styles.inviteField}>
                  <label className={styles.inviteFieldLabel} htmlFor="invite-lastname">
                    <span className={styles.inviteFieldIcon}><IconPerson /></span>
                    Last Name
                  </label>
                  <input
                    id="invite-lastname"
                    className={styles.inviteInput}
                    value={lastName}
                    onChange={(e) => setLastName(e.target.value)}
                    placeholder="Smith"
                    required
                  />
                </div>
              </div>

              {/* Email Address */}
              <div className={styles.inviteField}>
                <label className={styles.inviteFieldLabel} htmlFor="invite-email">
                  <span className={styles.inviteFieldIcon}><IconMail /></span>
                  Email Address
                </label>
                <input
                  id="invite-email"
                  type="email"
                  className={styles.inviteInput}
                  value={inviteEmail}
                  onChange={(e) => setInviteEmail(e.target.value)}
                  placeholder="Email Address"
                  required
                />
              </div>

              {/* Role */}
              <div className={styles.inviteField}>
                <label className={styles.inviteFieldLabel}>
                  <span className={styles.inviteFieldIcon}><IconShield /></span>
                  Role
                </label>
                <div className={styles.inviteDropdown} ref={dropdownRef}>
                  <button
                    type="button"
                    className={styles.inviteDropdownTrigger}
                    onClick={() => setDropdownOpen((o) => !o)}
                    aria-haspopup="listbox"
                    aria-expanded={dropdownOpen}
                  >
                    <span className={styles.inviteDropdownSelected}>
                      <span className={styles.inviteDropdownIcon}><IconShield /></span>
                      {selectedRole ? selectedRole.name : "Select a role"}
                    </span>
                    <span className={dropdownOpen ? styles.chevronUp : styles.chevronDown}>
                      <IconChevronDown />
                    </span>
                  </button>
                  {dropdownOpen && (
                    <ul className={styles.inviteDropdownMenu} role="listbox">
                      {(roles ?? []).map((role) => (
                        <li
                          key={role.role_id}
                          role="option"
                          aria-selected={role.role_id === inviteRoleId}
                          className={
                            role.role_id === inviteRoleId
                              ? `${styles.inviteDropdownOption} ${styles.inviteDropdownOptionActive}`
                              : styles.inviteDropdownOption
                          }
                          onClick={() => {
                            setInviteRoleId(role.role_id);
                            setDropdownOpen(false);
                          }}
                        >
                          <span className={styles.inviteDropdownIcon}><IconShield /></span>
                          {role.name}
                        </li>
                      ))}
                    </ul>
                  )}
                </div>
              </div>

              {/* Send invite email */}
              <label className={styles.inviteCheckRow}>
                <input
                  type="checkbox"
                  checked={sendInviteEmail}
                  onChange={(e) => setSendInviteEmail(e.target.checked)}
                />
                Send an invite email
              </label>

              {formError && <p className={styles.formError}>{formError}</p>}

              <div className={styles.inviteFooter}>
                <Button type="button" variant="ghost" onClick={closeInviteModal}>
                  Cancel
                </Button>
                <Button type="submit" loading={createMutation.isPending}>
                  Create User
                </Button>
              </div>
            </form>
            )}
          </div>
        </div>
      )}

      {/* Delete confirmation modal */}
      {deleteModalIds.length > 0 && (
        <div className={styles.overlay} role="dialog" aria-modal="true" aria-labelledby="delete-modal-title">
          <div className={styles.modal}>
            <div className={styles.modalHeader}>
              <h3 id="delete-modal-title" className={styles.modalTitle}>
                {deleteModalIds.length === 1 ? "Delete User" : `Delete ${deleteModalIds.length} Users`}
              </h3>
            </div>
            {deleteModalIds.length === 1 ? (
              <p className={styles.modalBody}>
                Are you sure you want to delete{" "}
                <strong>{modalUsers[0]?.email ?? deleteModalIds[0]}</strong>? This action cannot be undone.
              </p>
            ) : (
              <div className={styles.modalBody}>
                <p style={{ margin: "0 0 0.5rem" }}>
                  Are you sure you want to delete these <strong>{deleteModalIds.length} users</strong>? This action cannot be undone.
                </p>
                <ul className={styles.deleteList}>
                  {modalUsers.map((u) => (
                    <li key={u.user_id}>{u.email}</li>
                  ))}
                </ul>
              </div>
            )}
            {deleteError && <p className={styles.formError}>{deleteError}</p>}
            <div className={styles.modalActions}>
              <Button variant="ghost" onClick={closeModal} disabled={isDeleting}>
                Cancel
              </Button>
              <Button variant="danger" loading={isDeleting} onClick={confirmDelete}>
                {deleteModalIds.length === 1 ? "Delete" : `Delete ${deleteModalIds.length} Users`}
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
