CREATE TABLE oidc_clients (
    oidc_client_id    UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id         UUID        NULL,
    client_id         TEXT        NOT NULL UNIQUE,
    client_secret_hash TEXT       NULL,
    name              TEXT        NOT NULL,
    redirect_uris     TEXT[]      NOT NULL,
    allowed_scopes    TEXT[]      NOT NULL DEFAULT ARRAY['openid'],
    pkce_required     BOOLEAN     NOT NULL DEFAULT TRUE,
    is_confidential   BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_oidc_clients_client_id ON oidc_clients(client_id);
