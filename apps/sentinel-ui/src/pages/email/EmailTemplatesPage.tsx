import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "react-router-dom";
import { adminApi } from "../../api/admin";
import { Card } from "../../components/ui/Card";
import { Button } from "@sentinel/auth-react";
import { Badge } from "../../components/ui/Badge";
import { PageHeader } from "../../components/ui/PageHeader";
import { EmptyState } from "../../components/ui/EmptyState";
import type {
  EmailTemplate,
  EmailTemplateType,
  UpdateEmailTemplateRequest,
} from "../../types";
import styles from "./EmailTemplatesPage.module.css";

const TABS: { type: EmailTemplateType; label: string }[] = [
  { type: "EmailVerification", label: "Email Verification" },
  { type: "PasswordReset",     label: "Password Reset" },
  { type: "PasswordChanged",   label: "Password Changed" },
];

const PLACEHOLDERS: Record<EmailTemplateType, string[]> = {
  EmailVerification: ["{{first_name}}", "{{verification_link}}", "{{email}}"],
  PasswordReset:     ["{{first_name}}", "{{reset_link}}", "{{email}}"],
  PasswordChanged:   ["{{first_name}}", "{{email}}"],
};

export function EmailTemplatesPage() {
  const qc = useQueryClient();
  const navigate = useNavigate();
  const [activeTab, setActiveTab] = useState<EmailTemplateType>("EmailVerification");
  const [editId, setEditId] = useState<string | null>(null);
  const [editForm, setEditForm] = useState<UpdateEmailTemplateRequest>({});

  const { data: templates, isLoading } = useQuery({
    queryKey: ["email-templates"],
    queryFn: () => adminApi.listEmailTemplates(),
  });

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: UpdateEmailTemplateRequest }) =>
      adminApi.updateEmailTemplate(id, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["email-templates"] });
      setEditId(null);
    },
  });

  const activeTemplate: EmailTemplate | undefined = templates?.find(
    (t) => t.template_type === activeTab && t.is_active
  );

  return (
    <div className={styles.page}>
      <PageHeader
        title="Email Templates"
        subtitle="Customize transactional email content. System defaults are used when no custom template is active."
        action={
          <Button onClick={() => navigate(`/email-templates/new?type=${activeTab}`)}>
            New Template
          </Button>
        }
      />

      <div className={styles.tabs}>
        {TABS.map((tab) => (
          <button
            key={tab.type}
            className={`${styles.tab} ${activeTab === tab.type ? styles.tabActive : ""}`}
            onClick={() => {
              setActiveTab(tab.type);
              setEditId(null);
            }}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {isLoading ? (
        <EmptyState message="Loading templates…" />
      ) : activeTemplate ? (
        <Card title={TABS.find((t) => t.type === activeTab)!.label}>
          {editId === activeTemplate.template_id ? (
            <form
              className={styles.form}
              onSubmit={(e) => {
                e.preventDefault();
                updateMutation.mutate({ id: activeTemplate.template_id, data: editForm });
              }}
            >
              <p className={styles.hint}>
                Available placeholders:{" "}
                {PLACEHOLDERS[activeTab].map((p) => (
                  <code key={p} className={styles.placeholder}>{p}</code>
                ))}
              </p>
              <label className={styles.label}>
                Subject
                <input
                  defaultValue={activeTemplate.subject}
                  onChange={(e) => setEditForm({ ...editForm, subject: e.target.value })}
                  required
                />
              </label>
              <label className={styles.label}>
                Body (plain text)
                <textarea
                  defaultValue={activeTemplate.body_text}
                  onChange={(e) => setEditForm({ ...editForm, body_text: e.target.value })}
                  rows={8}
                  className={styles.textarea}
                />
              </label>
              <label className={styles.label}>
                Body (HTML, optional)
                <textarea
                  defaultValue={activeTemplate.body_html ?? ""}
                  onChange={(e) =>
                    setEditForm({ ...editForm, body_html: e.target.value || undefined })
                  }
                  rows={5}
                  className={styles.textarea}
                  placeholder="<p>Optional HTML version</p>"
                />
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
              <p className={styles.hint}>
                Available placeholders:{" "}
                {PLACEHOLDERS[activeTab].map((p) => (
                  <code key={p} className={styles.placeholder}>{p}</code>
                ))}
              </p>
              <p className={styles.fieldLabel}>Subject</p>
              <p className={styles.subject}>{activeTemplate.subject}</p>
              <p className={styles.fieldLabel}>Body</p>
              <pre className={styles.preview}>{activeTemplate.body_text}</pre>
              {activeTemplate.body_html && (
                <>
                  <p className={styles.fieldLabel}>HTML Body</p>
                  <pre className={styles.preview}>{activeTemplate.body_html}</pre>
                </>
              )}
              <div className={styles.actions}>
                <Badge variant={activeTemplate.is_active ? "active" : "inactive"}>
                  {activeTemplate.is_active ? "Active" : "Inactive"}
                </Badge>
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={() => {
                    setEditId(activeTemplate.template_id);
                    setEditForm({
                      subject: activeTemplate.subject,
                      body_text: activeTemplate.body_text,
                      body_html: activeTemplate.body_html ?? undefined,
                    });
                  }}
                >
                  Edit
                </Button>
              </div>
            </>
          )}
        </Card>
      ) : (
        <EmptyState message="No active template for this type. Create one to override the system default." />
      )}
    </div>
  );
}
