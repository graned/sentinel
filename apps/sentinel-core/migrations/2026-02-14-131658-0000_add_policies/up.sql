-- up.sql
-- Policies + Policy Versions (rules + compiled_rules)
-- ---------- tables ----------
CREATE TABLE IF NOT EXISTS policies (
  policy_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

  tenant_id UUID NULL,                 -- NULL = single-tenant mode
  environment TEXT NOT NULL DEFAULT 'prod',

  name TEXT NOT NULL DEFAULT 'default',
  description TEXT NULL,

  -- Points to currently active version for this policy set
  active_version BIGINT NOT NULL,

  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

  CONSTRAINT uq_policy_identity UNIQUE (tenant_id, environment, name),
  CONSTRAINT active_version_positive CHECK (active_version > 0)
);

CREATE TRIGGER trg_policies_updated_at
BEFORE UPDATE ON policies
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE IF NOT EXISTS policy_versions (
  policy_version_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

  policy_id UUID NOT NULL REFERENCES policies(policy_id) ON DELETE CASCADE,

  -- Monotonic per policy_id
  version BIGINT NOT NULL,

  -- Source of truth (editable rules)
  rules JSONB NOT NULL,
  rules_hash TEXT NOT NULL,

  -- Runtime artifact loaded into your trie
  compiled_rules BYTEA NOT NULL,
  compiled_hash TEXT NOT NULL,

  compiler_version TEXT NOT NULL DEFAULT 'v1',
  compiled_at TIMESTAMPTZ NOT NULL DEFAULT now(),

  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

  CONSTRAINT uq_policy_version UNIQUE (policy_id, version),
  CONSTRAINT rules_is_object CHECK (jsonb_typeof(rules) = 'object'),
  CONSTRAINT version_positive CHECK (version > 0),
  CONSTRAINT rules_hash_nonempty CHECK (length(trim(rules_hash)) > 0),
  CONSTRAINT compiled_hash_nonempty CHECK (length(trim(compiled_hash)) > 0)
);

-- ---------- indexes ----------
-- Fast load: policy_id + version, and "latest version" queries
CREATE INDEX IF NOT EXISTS idx_policy_versions_policy_version_desc
  ON policy_versions (policy_id, version DESC);

-- Optional but useful for admin UIs / audits:
CREATE INDEX IF NOT EXISTS idx_policy_versions_compiled_at_desc
  ON policy_versions (policy_id, compiled_at DESC);

-- Optional JSONB index for rules searching (admin tooling; not runtime hot path)
CREATE INDEX IF NOT EXISTS idx_policy_versions_rules_gin
  ON policy_versions USING GIN (rules);

