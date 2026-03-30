-- Reverse the super admin seed migration.
-- Deletes cascade from users → user_identities, user_roles, sessions, etc.

DO $$
DECLARE
    v_user_id UUID;
BEGIN
    SELECT ui.user_id INTO v_user_id
    FROM user_identities ui
    WHERE ui.email = 'admin@sentinel.local';

    IF v_user_id IS NULL THEN
        RAISE NOTICE 'Super admin not found, nothing to remove.';
        RETURN;
    END IF;

    -- Cascade handles: user_identities, user_roles, sessions, api_tokens, etc.
    DELETE FROM users WHERE user_id = v_user_id;
END $$;

-- Remove the seeded policy (any policy named 'Super Admin Policy' in production env)
DELETE FROM policies
WHERE name = 'Super Admin Policy'
  AND environment = 'production'
  AND tenant_id IS NULL;
