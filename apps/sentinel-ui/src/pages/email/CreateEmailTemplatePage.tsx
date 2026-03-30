import { useState } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { adminApi } from "../../api/admin";
import { Card } from "../../components/ui/Card";
import { Button } from "@sentinel/auth-react";
import { PageHeader } from "../../components/ui/PageHeader";
import type { CreateEmailTemplateRequest, EmailTemplateType } from "../../types";
import styles from "./CreateEmailTemplatePage.module.css";

const TEMPLATE_TYPES: { value: EmailTemplateType; label: string }[] = [
  { value: "EmailVerification", label: "Email Verification" },
  { value: "PasswordReset",     label: "Password Reset" },
  { value: "PasswordChanged",   label: "Password Changed" },
];

const PLACEHOLDERS: Record<EmailTemplateType, string[]> = {
  EmailVerification: ["{{first_name}}", "{{verification_link}}", "{{email}}"],
  PasswordReset:     ["{{first_name}}", "{{reset_link}}", "{{email}}"],
  PasswordChanged:   ["{{first_name}}", "{{email}}"],
};

function isValidTemplateType(value: string | null): value is EmailTemplateType {
  return value === "EmailVerification" || value === "PasswordReset" || value === "PasswordChanged";
}

export function CreateEmailTemplatePage() {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const qc = useQueryClient();

  const typeParam = searchParams.get("type");
  const initialType: EmailTemplateType = isValidTemplateType(typeParam)
    ? typeParam
    : "EmailVerification";

  const [form, setForm] = useState<CreateEmailTemplateRequest>({
    template_type: initialType,
    subject: "",
    body_text: "",
    body_html: "",
  });

  const createMutation = useMutation({
    mutationFn: (data: CreateEmailTemplateRequest) => adminApi.createEmailTemplate(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["email-templates"] });
      navigate("/email-templates");
    },
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const payload: CreateEmailTemplateRequest = {
      ...form,
      body_html: form.body_html?.trim() || undefined,
    };
    createMutation.mutate(payload);
  };

  return (
    <div className={styles.page}>
      <PageHeader
        title="New Email Template"
        subtitle="Create a custom template to override the system default for this email type."
        action={
          <Button variant="ghost" onClick={() => navigate("/email-templates")}>
            ← Back
          </Button>
        }
      />

      <Card title="Template Details">
        <form className={styles.form} onSubmit={handleSubmit}>
          <label className={styles.label}>
            Template Type
            <select
              value={form.template_type}
              onChange={(e) =>
                setForm({ ...form, template_type: e.target.value as EmailTemplateType })
              }
            >
              {TEMPLATE_TYPES.map((t) => (
                <option key={t.value} value={t.value}>
                  {t.label}
                </option>
              ))}
            </select>
          </label>

          <p className={styles.hint}>
            Available placeholders:{" "}
            {PLACEHOLDERS[form.template_type].map((p) => (
              <code key={p} className={styles.placeholder}>{p}</code>
            ))}
          </p>

          <label className={styles.label}>
            Subject
            <input
              value={form.subject}
              onChange={(e) => setForm({ ...form, subject: e.target.value })}
              required
              placeholder="e.g. Verify your email address"
            />
          </label>

          <label className={styles.label}>
            Body (plain text)
            <textarea
              value={form.body_text}
              onChange={(e) => setForm({ ...form, body_text: e.target.value })}
              required
              rows={8}
              className={styles.textarea}
              placeholder={`Hi {{first_name}},\n\nYour message here...`}
            />
          </label>

          <label className={styles.label}>
            Body (HTML, optional)
            <textarea
              value={form.body_html ?? ""}
              onChange={(e) => setForm({ ...form, body_html: e.target.value })}
              rows={5}
              className={styles.textarea}
              placeholder="<p>Optional HTML version of the email</p>"
            />
          </label>

          {createMutation.error && (
            <span className={styles.error}>
              {createMutation.error instanceof Error
                ? createMutation.error.message
                : "Failed to create template"}
            </span>
          )}

          <div className={styles.actions}>
            <Button type="submit" loading={createMutation.isPending}>
              Create Template
            </Button>
            <Button variant="ghost" type="button" onClick={() => navigate("/email-templates")}>
              Cancel
            </Button>
          </div>
        </form>
      </Card>
    </div>
  );
}
