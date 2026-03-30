CREATE TABLE user_recovery_codes (
    user_recovery_code_id UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id               UUID        NOT NULL,
    code_hash             TEXT        NOT NULL,
    used_at               TIMESTAMPTZ NULL,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT fk_user_recovery_codes_user
        FOREIGN KEY (user_id) REFERENCES users (user_id) ON DELETE CASCADE,
    CONSTRAINT uq_user_recovery_code UNIQUE (user_id, code_hash)
);
CREATE INDEX idx_user_recovery_codes_user_id ON user_recovery_codes(user_id);
