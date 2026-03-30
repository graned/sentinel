CREATE TABLE api_tokens (
    api_token_id  UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id       UUID        NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    name          TEXT        NOT NULL,
    description   TEXT,
    token_hash    TEXT        NOT NULL UNIQUE,
    expires_at    TIMESTAMPTZ,
    last_used_at  TIMESTAMPTZ,
    revoked_at    TIMESTAMPTZ,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ,
    created_by    UUID,
    updated_by    UUID
);

CREATE INDEX idx_api_tokens_user_id    ON api_tokens(user_id);
CREATE INDEX idx_api_tokens_token_hash ON api_tokens(token_hash);
