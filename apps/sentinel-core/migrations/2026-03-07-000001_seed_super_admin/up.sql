-- Seed: super admin user (admin@sentinel.local / admin) + wildcard policy
-- This migration is for initial setup only.
-- The password 'admin' is a plaintext value; the trigger
-- trg_user_identities_password_hash will bcrypt it automatically.

DO $$
DECLARE
    v_user_id           UUID := gen_random_uuid();
    v_identity_id       UUID := gen_random_uuid();
    v_admin_role_id     UUID;
    v_policy_id         UUID := gen_random_uuid();
    v_policy_version_id UUID := gen_random_uuid();
    v_compiled_rules    BYTEA;
    v_rules             JSONB;
BEGIN
    -- Require that role seeds from a previous migration are present
    SELECT role_id INTO v_admin_role_id FROM roles WHERE type = 'admin';
    IF v_admin_role_id IS NULL THEN
        RAISE EXCEPTION 'Admin role not found — run migrations in order.';
    END IF;

    -- Idempotency guard: skip if the super admin already exists
    IF EXISTS (SELECT 1 FROM user_identities WHERE email = 'admin@sentinel.local') THEN
        RAISE NOTICE 'Super admin already exists, skipping seed.';
        RETURN;
    END IF;

    -- ── User ─────────────────────────────────────────────────────────────────
    INSERT INTO users (user_id, first_name, last_name, status, token_version)
    VALUES (v_user_id, 'Super', 'Admin', 'active', 1);

    -- The password trigger hashes any value that doesn't already look like bcrypt.
    INSERT INTO user_identities (
        identity_id, user_id, provider,
        email, password_hash, email_verified, is_primary
    )
    VALUES (
        v_identity_id, v_user_id, 'email_password',
        'admin@sentinel.local', 'admin', true, true
    );

    INSERT INTO user_roles (user_role_id, user_id, role_id)
    VALUES (gen_random_uuid(), v_user_id, v_admin_role_id);

    -- ── Policy ───────────────────────────────────────────────────────────────
    -- compiled_rules is a bincode-serialized TrieNode for the single rule:
    --   { method: "*", path: "/**", roles: ["admin"] }
    --
    -- Layout (66 bytes, all lengths are u64 LE, enum variant is u32 LE):
    --   root.rules len      = 0      → 0000000000000000
    --   root.children len   = 1      → 0100000000000000
    --   Segment::Glob       = var 3  → 03000000
    --   glob.rules len      = 1      → 0100000000000000
    --   key "*" len         = 1      → 0100000000000000
    --   key "*"                      → 2a
    --   value vec len       = 1      → 0100000000000000
    --   "admin" len         = 5      → 0500000000000000
    --   "admin"                      → 61646d696e
    --   glob.children len   = 0      → 0000000000000000
    v_compiled_rules := decode(
        '0000000000000000'   -- root.rules len = 0
        '0100000000000000'   -- root.children len = 1
        '03000000'           -- Segment::Glob (variant 3)
        '0100000000000000'   -- glob.rules len = 1
        '0100000000000000'   -- key "*" len = 1
        '2a'                 -- "*"
        '0100000000000000'   -- value vec len = 1
        '0500000000000000'   -- "admin" len = 5
        '61646d696e'         -- "admin"
        '0000000000000000',  -- glob.children len = 0
        'hex'
    );

    v_rules := '{"rules":[{"method":"*","path":"/**","roles":["admin"]}]}'::jsonb;

    INSERT INTO policies (policy_id, environment, name, description, active_version)
    VALUES (
        v_policy_id,
        'production',
        'Super Admin Policy',
        'Grants the admin role full access to all endpoints. Created by seed migration.',
        1
    );

    INSERT INTO policy_versions (
        policy_version_id, policy_id, version,
        rules, rules_hash,
        compiled_rules, compiled_hash,
        compiler_version
    )
    VALUES (
        v_policy_version_id,
        v_policy_id,
        1,
        v_rules,
        encode(digest(v_rules::text::bytea, 'sha256'), 'hex'),
        v_compiled_rules,
        encode(digest(v_compiled_rules, 'sha256'), 'hex'),
        'seed'
    );
END $$;
