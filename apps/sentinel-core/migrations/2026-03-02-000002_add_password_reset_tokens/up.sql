CREATE TABLE password_reset_tokens (
    reset_token_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    identity_id    UUID NOT NULL REFERENCES user_identities(identity_id) ON DELETE CASCADE,
    token_hash     TEXT NOT NULL UNIQUE,
    expires_at     TIMESTAMPTZ NOT NULL,
    used_at        TIMESTAMPTZ,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ,
    created_by     UUID REFERENCES users(user_id),
    updated_by     UUID REFERENCES users(user_id)
);

CREATE INDEX idx_password_reset_tokens_identity ON password_reset_tokens(identity_id);
CREATE INDEX idx_password_reset_tokens_hash     ON password_reset_tokens(token_hash);
