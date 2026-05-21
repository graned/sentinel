-- Remove display_name column from users table
ALTER TABLE users
DROP COLUMN IF EXISTS display_name;

-- Note: We cannot remove values from PostgreSQL enums, so token_federation remains
-- This is intentional to avoid breaking existing data
