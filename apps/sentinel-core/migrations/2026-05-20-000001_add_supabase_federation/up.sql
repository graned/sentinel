-- Add display_name column to users table
ALTER TABLE users 
    ADD COLUMN display_name VARCHAR(255);

-- Create index on display_name for potential future lookups
CREATE INDEX idx_users_display_name ON users(display_name);

-- Create external_identities table for federated identity mappings
CREATE TABLE external_identities (
    external_identity_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    
    -- Provider identification
    provider TEXT NOT NULL,           -- e.g., 'supabase'
    issuer TEXT NOT NULL,             -- e.g., 'http://localhost:9999/auth/v1'
    subject TEXT NOT NULL,            -- Supabase 'sub' claim (stable identity key)
    
    -- Snapshot data from token (not used for matching)
    email_snapshot TEXT,              -- Email at time of linking
    metadata JSONB,                   -- Additional claims snapshot (user_metadata)
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login_at TIMESTAMPTZ,
    
    -- Optional: track who created/updated
    created_by UUID REFERENCES users(user_id),
    updated_by UUID REFERENCES users(user_id),
    
    -- Constraints
    CONSTRAINT uk_external_identities_provider_issuer_subject 
        UNIQUE (provider, issuer, subject)
);

-- Index for fast lookups by provider+issuer+subject (the lookup key)
CREATE INDEX idx_external_identities_lookup 
    ON external_identities(provider, issuer, subject);

-- Index for fast lookups by user_id (to find all identities for a user)
CREATE INDEX idx_external_identities_user_id 
    ON external_identities(user_id);

-- Trigger to update updated_at
CREATE TRIGGER update_external_identities_updated_at
    BEFORE UPDATE ON external_identities
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
