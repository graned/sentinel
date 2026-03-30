-- ======================
-- EXTENSIONS (needed for gen_random_uuid)
-- ======================
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- ======================
-- ENUMS
-- ======================

-- User status
CREATE TYPE user_status AS ENUM (
    'active',
    'inactive',
    'suspended',
    'pending_verification'
);

-- Identity provider (replaces auth_methods table)
CREATE TYPE identity_provider AS ENUM (
    'email_password',
    'google_oauth',
    'github_oauth',
    'microsoft_oauth',
    'apple_oauth',
    'saml',
    'ldap',
    'custom'
);

-- Session revocation reasons (proper enum)
CREATE TYPE revocation_reason AS ENUM (
    'user_logout',
    'password_change',
    'token_compromised',
    'account_suspended',
    'manual_revocation',
    'suspicious_activity',
    'token_rotation',
    'expired'
);

-- ======================
-- TABLES
-- ======================

-- Users table
CREATE TABLE users (
    user_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Core profile
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    avatar_url TEXT,

    -- Status
    status user_status NOT NULL DEFAULT 'pending_verification',

    -- Token version for immediate revocation
    token_version INTEGER NOT NULL DEFAULT 1,

    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    -- Optional: track who created/updated
    created_by UUID REFERENCES users(user_id),
    updated_by UUID REFERENCES users(user_id)
);

-- User identities table (multiple auth methods per user)
CREATE TABLE user_identities (
    identity_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,

    -- Authentication provider
    provider identity_provider NOT NULL DEFAULT 'email_password',

    -- Provider-specific identifier
    provider_user_id VARCHAR(255),

    -- Email (for email_password provider)
    email VARCHAR(255) NOT NULL,

    -- Password (only for email_password provider)
    password_hash TEXT,
    password_changed_at TIMESTAMPTZ,

    -- Email verification
    email_verified BOOLEAN DEFAULT FALSE,

    -- OAuth tokens (for OAuth providers)
    oauth_access_token TEXT,
    oauth_refresh_token TEXT,
    oauth_token_expires_at TIMESTAMPTZ,

    -- Primary identity flag
    is_primary BOOLEAN NOT NULL DEFAULT FALSE,

    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    last_login_at TIMESTAMPTZ,

    -- Optional: track who created/updated
    created_by UUID REFERENCES users(user_id),
    updated_by UUID REFERENCES users(user_id),

    -- Constraints
    CONSTRAINT chk_email_password_provider CHECK (
        provider != 'email_password' OR password_hash IS NOT NULL
    )
);

-- User sessions table (PASETO-based)
CREATE TABLE sessions (
    session_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Foreign keys
    user_id UUID NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    identity_id UUID NOT NULL REFERENCES user_identities(identity_id) ON DELETE CASCADE,

    -- Refresh token (hashed, stored)
    refresh_token_hash TEXT NOT NULL,
    refresh_token_family UUID NOT NULL DEFAULT gen_random_uuid(),

    -- Device info
    user_agent TEXT,
    ip_address TEXT,

    -- Generated device type from user agent
    device_type VARCHAR(20) GENERATED ALWAYS AS (
        CASE
            WHEN user_agent ILIKE '%mobile%' OR
                 user_agent ILIKE '%android%' OR
                 user_agent ILIKE '%iphone%' THEN 'mobile'
            WHEN user_agent ILIKE '%tablet%' OR
                 user_agent ILIKE '%ipad%' THEN 'tablet'
            WHEN user_agent ILIKE '%bot%' OR
                 user_agent ILIKE '%crawler%' OR
                 user_agent ILIKE '%spider%' THEN 'bot'
            ELSE 'desktop'
        END
    ) STORED,

    -- Expiration (refresh token only)
    refresh_token_expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '90 days'),

    -- Revocation info
    revoked_at TIMESTAMPTZ,
    revoked_reason revocation_reason,

    -- Usage tracking
    last_used_at TIMESTAMPTZ DEFAULT NOW(),

    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    -- Optional: track who created/updated
    created_by UUID REFERENCES users(user_id),
    updated_by UUID REFERENCES users(user_id)
);

-- Email verifications table
CREATE TABLE email_verifications (
    verification_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Foreign key (renamed for clarity)
    identity_id UUID NOT NULL REFERENCES user_identities(identity_id) ON DELETE CASCADE,

    -- Verification token (hashed)
    token_hash TEXT NOT NULL,

    -- Expiration
    expires_at TIMESTAMPTZ NOT NULL,

    -- Verification timestamp
    verified_at TIMESTAMPTZ,

    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    -- Optional: track who created/updated
    created_by UUID REFERENCES users(user_id),
    updated_by UUID REFERENCES users(user_id)
);

-- AUTH configurations (for user-customizable auth later)
CREATE TABLE auth_configs (
    config_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Foreign key
    user_id UUID NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,

    -- Key (hashed)
    key_hash TEXT NOT NULL,

    -- Algorithm (v4.local for now)
    algorithm VARCHAR(20) NOT NULL DEFAULT 'v4.local',

    -- Issuer
    issuer VARCHAR(255) NOT NULL DEFAULT 'auth_saas',

    -- Default expiry
    default_expiry_seconds INTEGER NOT NULL DEFAULT 900, -- 15 minutes

    -- Status
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    rotated_at TIMESTAMPTZ,

    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    -- Optional: track who created/updated
    created_by UUID REFERENCES users(user_id),
    updated_by UUID REFERENCES users(user_id)
);

-- ======================
-- INDEXES
-- ======================

-- Users indexes
CREATE INDEX idx_users_status ON users(status);
CREATE INDEX idx_users_token_version ON users(token_version);

-- User identities indexes
CREATE INDEX idx_user_identities_user_id ON user_identities(user_id);
CREATE INDEX idx_user_identities_email ON user_identities(email);
CREATE INDEX idx_user_identities_provider ON user_identities(provider);
CREATE INDEX idx_user_identities_primary ON user_identities(user_id, is_primary) WHERE is_primary = TRUE;

CREATE UNIQUE INDEX ux_user_identities_email_verified_true
ON user_identities (email)
WHERE email_verified = TRUE;

CREATE UNIQUE INDEX ux_user_identities_provider_provider_user_id_not_null
ON user_identities (provider, provider_user_id)
WHERE provider_user_id IS NOT NULL;

-- Sessions indexes
CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_identity_id ON sessions(identity_id);
CREATE INDEX idx_sessions_refresh_token ON sessions(refresh_token_hash);
CREATE INDEX idx_sessions_refresh_expires ON sessions(refresh_token_expires_at) WHERE revoked_at IS NULL;
CREATE INDEX idx_sessions_active ON sessions(user_id, revoked_at) WHERE revoked_at IS NULL;
CREATE INDEX idx_sessions_device ON sessions(device_type);
CREATE INDEX idx_sessions_family ON sessions(refresh_token_family);

-- Email verifications indexes
CREATE INDEX idx_email_verifications_token ON email_verifications(token_hash);
CREATE INDEX idx_email_verifications_expires ON email_verifications(expires_at) WHERE verified_at IS NULL;

-- auth configs indexes
CREATE INDEX idx_auth_configs_user ON auth_configs(user_id);
CREATE INDEX idx_auth_configs_active ON auth_configs(is_active) WHERE is_active = TRUE;

-- ======================
-- FUNCTIONS
-- ======================

-- Update timestamp function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Update session last_used_at
CREATE OR REPLACE FUNCTION update_session_last_used()
RETURNS TRIGGER AS $$
BEGIN
    NEW.last_used_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Ensure only one primary identity per user
CREATE OR REPLACE FUNCTION ensure_single_primary_identity()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.is_primary THEN
        UPDATE user_identities
        SET is_primary = FALSE
        WHERE user_id = NEW.user_id
          AND identity_id <> NEW.identity_id;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Revoke all sessions when password changes
CREATE OR REPLACE FUNCTION revoke_sessions_on_password_change()
RETURNS TRIGGER AS $$
BEGIN
    IF OLD.password_hash IS DISTINCT FROM NEW.password_hash THEN
        UPDATE sessions
        SET revoked_at = NOW(),
            revoked_reason = 'password_change',
            updated_at = NOW()
        WHERE identity_id = NEW.identity_id
          AND revoked_at IS NULL;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Increment token version when sensitive data changes
CREATE OR REPLACE FUNCTION increment_token_version()
RETURNS TRIGGER AS $$
BEGIN
    IF OLD.password_hash IS DISTINCT FROM NEW.password_hash
       OR OLD.email_verified IS DISTINCT FROM NEW.email_verified
       OR (OLD.provider = 'email_password' AND NEW.provider <> 'email_password')
       OR (OLD.provider <> 'email_password' AND NEW.provider = 'email_password') THEN
        UPDATE users
        SET token_version = token_version + 1,
            updated_at = NOW()
        WHERE user_id = NEW.user_id;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Revoke all sessions when user status changes to inactive/suspended
CREATE OR REPLACE FUNCTION revoke_sessions_on_status_change()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.status IN ('inactive', 'suspended')
       AND OLD.status NOT IN ('inactive', 'suspended') THEN
        UPDATE sessions
        SET revoked_at = NOW(),
            revoked_reason = 'account_suspended',
            updated_at = NOW()
        WHERE user_id = NEW.user_id
          AND revoked_at IS NULL;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ======================
-- TRIGGERS
-- ======================

-- Users triggers
CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER revoke_sessions_on_status_change_trigger
    AFTER UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION revoke_sessions_on_status_change();

-- User identities triggers
CREATE TRIGGER update_user_identities_updated_at
    BEFORE UPDATE ON user_identities
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER ensure_single_primary_identity_trigger
    BEFORE INSERT OR UPDATE ON user_identities
    FOR EACH ROW
    EXECUTE FUNCTION ensure_single_primary_identity();

CREATE TRIGGER revoke_sessions_on_password_change_trigger
    AFTER UPDATE ON user_identities
    FOR EACH ROW
    EXECUTE FUNCTION revoke_sessions_on_password_change();

CREATE TRIGGER increment_token_version_trigger
    AFTER UPDATE ON user_identities
    FOR EACH ROW
    EXECUTE FUNCTION increment_token_version();

-- Sessions triggers
CREATE TRIGGER update_sessions_updated_at
    BEFORE UPDATE ON sessions
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_sessions_last_used
    BEFORE UPDATE ON sessions
    FOR EACH ROW
    WHEN (OLD.refresh_token_hash IS DISTINCT FROM NEW.refresh_token_hash)
    EXECUTE FUNCTION update_session_last_used();

-- Email verifications trigger
CREATE TRIGGER update_email_verifications_updated_at
    BEFORE UPDATE ON email_verifications
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Auth configs trigger
CREATE TRIGGER update_auth_configs_updated_at
    BEFORE UPDATE ON auth_configs
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
