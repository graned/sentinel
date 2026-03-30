CREATE TYPE email_template_type AS ENUM (
    'email_verification',
    'password_reset',
    'password_changed'
);

CREATE TABLE email_templates (
    template_id   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_type email_template_type NOT NULL,
    subject       TEXT NOT NULL,
    body_text     TEXT NOT NULL,
    body_html     TEXT,
    is_active     BOOLEAN NOT NULL DEFAULT TRUE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ,
    created_by    UUID REFERENCES users(user_id),
    updated_by    UUID REFERENCES users(user_id)
);

-- Only one active template per type at a time
CREATE UNIQUE INDEX idx_email_templates_type_active
    ON email_templates(template_type) WHERE is_active = TRUE;
