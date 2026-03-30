-- Seed default email templates if none exist for each type.
-- WHERE NOT EXISTS ensures idempotency.

INSERT INTO email_templates (template_id, template_type, subject, body_text, is_active, created_at)
SELECT
    gen_random_uuid(),
    'email_verification'::email_template_type,
    'Verify your email address',
    E'Hi {{first_name}},\n\nPlease verify your email: {{verification_link}}',
    TRUE,
    now()
WHERE NOT EXISTS (
    SELECT 1 FROM email_templates WHERE template_type = 'email_verification'
);

INSERT INTO email_templates (template_id, template_type, subject, body_text, is_active, created_at)
SELECT
    gen_random_uuid(),
    'password_reset'::email_template_type,
    'Reset your password',
    E'Hi {{first_name}},\n\nClick here to reset your password: {{reset_link}}\n\nThis link expires in 1 hour. If you did not request this, ignore this email.',
    TRUE,
    now()
WHERE NOT EXISTS (
    SELECT 1 FROM email_templates WHERE template_type = 'password_reset'
);

INSERT INTO email_templates (template_id, template_type, subject, body_text, is_active, created_at)
SELECT
    gen_random_uuid(),
    'password_changed'::email_template_type,
    'Your password was changed',
    E'Hi {{first_name}},\n\nYour password was recently changed. If this wasn''t you, please contact support immediately.',
    TRUE,
    now()
WHERE NOT EXISTS (
    SELECT 1 FROM email_templates WHERE template_type = 'password_changed'
);
