-- Drop trigger
DROP TRIGGER IF EXISTS update_external_identities_updated_at ON external_identities;

-- Drop indexes
DROP INDEX IF EXISTS idx_external_identities_user_id;
DROP INDEX IF EXISTS idx_external_identities_lookup;
DROP INDEX IF EXISTS idx_users_display_name;

-- Drop unique constraint (via index drop)
DROP INDEX IF EXISTS uk_external_identities_provider_issuer_subject;

-- Drop external_identities table
DROP TABLE IF EXISTS external_identities;

-- Drop display_name column from users
ALTER TABLE users DROP COLUMN IF EXISTS display_name;
