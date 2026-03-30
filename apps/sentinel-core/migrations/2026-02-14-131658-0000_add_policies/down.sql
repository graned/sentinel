-- down.sql
-- Rollback: drop in dependency order

DROP INDEX IF EXISTS idx_policy_versions_rules_gin;
DROP INDEX IF EXISTS idx_policy_versions_compiled_at_desc;
DROP INDEX IF EXISTS idx_policy_versions_policy_version_desc;

DROP TABLE IF EXISTS policy_versions;

-- policies trigger first, then table
DROP TRIGGER IF EXISTS trg_policies_updated_at ON policies;
DROP TABLE IF EXISTS policies;

