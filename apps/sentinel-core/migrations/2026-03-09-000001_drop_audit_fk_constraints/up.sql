-- Drop FK constraints on audit-only created_by/updated_by columns so that
-- deleting a user does not violate RESTRICT on rows that merely reference
-- the deleted user as their creator/updater.
--
-- These columns are historical metadata, not true relational dependencies.
-- The user's actual data (sessions, identities, MFA, API tokens, etc.)
-- is still fully cascade-deleted via the ON DELETE CASCADE user_id FKs.

ALTER TABLE users
    DROP CONSTRAINT IF EXISTS users_created_by_fkey,
    DROP CONSTRAINT IF EXISTS users_updated_by_fkey;

ALTER TABLE user_identities
    DROP CONSTRAINT IF EXISTS user_identities_created_by_fkey,
    DROP CONSTRAINT IF EXISTS user_identities_updated_by_fkey;

ALTER TABLE sessions
    DROP CONSTRAINT IF EXISTS sessions_created_by_fkey,
    DROP CONSTRAINT IF EXISTS sessions_updated_by_fkey;

ALTER TABLE email_verifications
    DROP CONSTRAINT IF EXISTS email_verifications_created_by_fkey,
    DROP CONSTRAINT IF EXISTS email_verifications_updated_by_fkey;

ALTER TABLE auth_configs
    DROP CONSTRAINT IF EXISTS auth_configs_created_by_fkey,
    DROP CONSTRAINT IF EXISTS auth_configs_updated_by_fkey;

ALTER TABLE email_templates
    DROP CONSTRAINT IF EXISTS email_templates_created_by_fkey,
    DROP CONSTRAINT IF EXISTS email_templates_updated_by_fkey;

ALTER TABLE password_reset_tokens
    DROP CONSTRAINT IF EXISTS password_reset_tokens_created_by_fkey,
    DROP CONSTRAINT IF EXISTS password_reset_tokens_updated_by_fkey;
