CREATE TABLE oidc_signing_keys (
    oidc_signing_key_id UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    kid                 TEXT        NOT NULL UNIQUE,
    alg                 TEXT        NOT NULL,
    public_jwk_json     JSONB       NOT NULL,
    private_key_encrypted BYTEA    NOT NULL,
    status              TEXT        NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);
