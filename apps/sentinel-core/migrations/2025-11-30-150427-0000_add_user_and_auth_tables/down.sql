-- ======================
-- DOWN MIGRATION
-- Reverses the entire schema in correct dependency order
-- ======================

-- First, disable all triggers to avoid conflicts
ALTER TABLE users DISABLE TRIGGER ALL;
ALTER TABLE user_identities DISABLE TRIGGER ALL;
ALTER TABLE sessions DISABLE TRIGGER ALL;
ALTER TABLE email_verifications DISABLE TRIGGER ALL;
ALTER TABLE auth_configs DISABLE TRIGGER ALL;

-- Drop triggers (in any order since disabled)
DROP TRIGGER IF EXISTS update_users_updated_at ON users;
DROP TRIGGER IF EXISTS revoke_sessions_on_status_change_trigger ON users;
DROP TRIGGER IF EXISTS update_user_identities_updated_at ON user_identities;
DROP TRIGGER IF EXISTS ensure_single_primary_identity_trigger ON user_identities;
DROP TRIGGER IF EXISTS revoke_sessions_on_password_change_trigger ON user_identities;
DROP TRIGGER IF EXISTS increment_token_version_trigger ON user_identities;
DROP TRIGGER IF EXISTS update_sessions_updated_at ON sessions;
DROP TRIGGER IF EXISTS update_sessions_last_used ON sessions;
DROP TRIGGER IF EXISTS update_email_verifications_updated_at ON email_verifications;
DROP TRIGGER IF EXISTS update_auth_configs_updated_at ON auth_configs;

-- Drop functions
DROP FUNCTION IF EXISTS update_updated_at_column() CASCADE;
DROP FUNCTION IF EXISTS update_session_last_used() CASCADE;
DROP FUNCTION IF EXISTS ensure_single_primary_identity() CASCADE;
DROP FUNCTION IF EXISTS revoke_sessions_on_password_change() CASCADE;
DROP FUNCTION IF EXISTS increment_token_version() CASCADE;
DROP FUNCTION IF EXISTS revoke_sessions_on_status_change() CASCADE;

-- Drop indexes (in any order)
DROP INDEX IF EXISTS idx_users_status;
DROP INDEX IF EXISTS idx_users_token_version;
DROP INDEX IF EXISTS idx_user_identities_user_id;
DROP INDEX IF EXISTS idx_user_identities_email;
DROP INDEX IF EXISTS idx_user_identities_provider;
DROP INDEX IF EXISTS idx_user_identities_primary;
DROP INDEX IF EXISTS idx_sessions_user_id;
DROP INDEX IF EXISTS idx_sessions_identity_id;
DROP INDEX IF EXISTS idx_sessions_refresh_token;
DROP INDEX IF EXISTS idx_sessions_refresh_expires;
DROP INDEX IF EXISTS idx_sessions_active;
DROP INDEX IF EXISTS idx_sessions_device;
DROP INDEX IF EXISTS idx_sessions_family;
DROP INDEX IF EXISTS idx_email_verifications_token;
DROP INDEX IF EXISTS idx_email_verifications_expires;
DROP INDEX IF EXISTS idx_auth_configs_user;
DROP INDEX IF EXISTS idx_auth_configs_active;

-- Drop tables in reverse dependency order (children first)
DROP TABLE IF EXISTS auth_configs CASCADE;
DROP TABLE IF EXISTS email_verifications CASCADE;
DROP TABLE IF EXISTS sessions CASCADE;
DROP TABLE IF EXISTS user_identities CASCADE;
DROP TABLE IF EXISTS users CASCADE;

-- Drop enums (order doesn't matter)
DROP TYPE IF EXISTS revocation_reason CASCADE;
DROP TYPE IF EXISTS identity_provider CASCADE;
DROP TYPE IF EXISTS user_status CASCADE;

