CREATE TABLE provider_configurations (
    configuration_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Multi-tenant support (nullable if single-tenant for now)
    tenant_id UUID NULL,

    -- 'smtp', 'ses', 'mailgun', etc.
    provider TEXT NOT NULL,

    -- The REAL config (encrypted in Rust)
    config_encrypted BYTEA NOT NULL,

    -- Safe-to-display version (masked secrets)
    config_redacted JSONB NOT NULL DEFAULT '{}'::jsonb,

    is_active BOOLEAN NOT NULL DEFAULT true,

    -- Auditing
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by UUID NULL,
    updated_by UUID NULL
);

-- Only ONE active config per provider per tenant
CREATE UNIQUE INDEX ux_provider_active_per_tenant
ON provider_configurations (
    COALESCE(tenant_id, '00000000-0000-0000-0000-000000000000'::uuid),
    provider
)
WHERE is_active = true;

-- Helpful indexes
CREATE INDEX ix_provider_tenant
ON provider_configurations (tenant_id);

CREATE INDEX ix_provider_name
ON provider_configurations (provider);

-- ======================
-- TRIGGERS
-- ======================

-- Users triggers
CREATE TRIGGER update_provider_configurations_updated_at
    BEFORE UPDATE ON provider_configurations
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();


