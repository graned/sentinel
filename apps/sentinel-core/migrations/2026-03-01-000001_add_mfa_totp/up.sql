CREATE TABLE user_mfa_totp (
    user_mfa_totp_id     UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id              UUID        NOT NULL UNIQUE,
    secret_encrypted     BYTEA       NOT NULL,
    enabled              BOOLEAN     NOT NULL DEFAULT FALSE,
    enrolled_at          TIMESTAMPTZ NULL,
    last_used_at         TIMESTAMPTZ NULL,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT fk_user_mfa_totp_user
        FOREIGN KEY (user_id) REFERENCES users (user_id) ON DELETE CASCADE
);
