CREATE TABLE oidc_auth_codes (
    oidc_auth_code_id    UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    code_hash            TEXT        NOT NULL UNIQUE,
    oidc_client_id       UUID        NOT NULL,
    user_id              UUID        NOT NULL,
    redirect_uri         TEXT        NOT NULL,
    scope                TEXT        NOT NULL,
    nonce                TEXT        NULL,
    code_challenge       TEXT        NOT NULL,
    code_challenge_method TEXT       NOT NULL,
    expires_at           TIMESTAMPTZ NOT NULL,
    consumed_at          TIMESTAMPTZ NULL,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT fk_oidc_auth_codes_client
        FOREIGN KEY (oidc_client_id) REFERENCES oidc_clients(oidc_client_id) ON DELETE CASCADE,
    CONSTRAINT fk_oidc_auth_codes_user
        FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE
);
CREATE INDEX idx_oidc_auth_codes_code_hash ON oidc_auth_codes(code_hash);
