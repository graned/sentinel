-- Drop trigger first
DROP FUNCTION IF EXISTS update_provider_configurations_updated_at;

-- Drop indexes
DROP INDEX IF EXISTS ux_provider_active_per_tenant;
DROP INDEX IF EXISTS ix_provider_tenant;
DROP INDEX IF EXISTS ix_provider_name;

-- Drop table
DROP TABLE IF EXISTS provider_configurations;

