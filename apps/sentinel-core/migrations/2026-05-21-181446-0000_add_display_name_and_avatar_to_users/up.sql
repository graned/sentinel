-- Add display_name column to users table
ALTER TABLE users
ADD COLUMN display_name VARCHAR(255);

-- Add index on display_name for efficient lookups
CREATE INDEX idx_users_display_name ON users(display_name);

-- Add token_federation to identity_provider enum
ALTER TYPE identity_provider ADD VALUE 'token_federation';
