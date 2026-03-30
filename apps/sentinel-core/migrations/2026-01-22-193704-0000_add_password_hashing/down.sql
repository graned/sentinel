DROP TRIGGER IF EXISTS trg_user_identities_password_hash ON user_identities;
DROP FUNCTION IF EXISTS user_identities_password_hash_trigger();
DROP FUNCTION IF EXISTS user_identities_hash_password(TEXT, INT);

-- Usually don't drop pgcrypto in down migrations.
-- DROP EXTENSION IF EXISTS pgcrypto;

