-- Enable crypt() / gen_salt()
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- Hash function: bcrypt with cost 12
CREATE OR REPLACE FUNCTION user_identities_hash_password(p_plain TEXT, p_cost INT DEFAULT 12)
RETURNS TEXT
LANGUAGE sql
IMMUTABLE
AS $$
  SELECT crypt(p_plain, gen_salt('bf', p_cost));
$$;

-- Trigger function:
-- - INSERT: hash password_hash if present (assumed plaintext)
-- - UPDATE: if password_hash changed, hash it (assumed plaintext)
-- - In both cases, skip hashing if value already looks like bcrypt ($2a$/$2b$/$2y$)
CREATE OR REPLACE FUNCTION user_identities_password_hash_trigger()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
  -- INSERT path
  IF TG_OP = 'INSERT' THEN
    IF NEW.password_hash IS NOT NULL THEN
      -- If it doesn't look like bcrypt, treat as plaintext and hash it
      IF NEW.password_hash !~ '^\\$2[aby]\\$' THEN
        NEW.password_hash := user_identities_hash_password(NEW.password_hash, 12);
      END IF;
    END IF;
    RETURN NEW;
  END IF;

  -- UPDATE path
  IF TG_OP = 'UPDATE' THEN
    -- Only touch it if it actually changed
    IF NEW.password_hash IS DISTINCT FROM OLD.password_hash THEN
      IF NEW.password_hash IS NOT NULL THEN
        NEW.password_hash := user_identities_hash_password(NEW.password_hash, 12);
      END IF;
    END IF;

    RETURN NEW;
  END IF;

  RETURN NEW;
END;
$$;

-- Attach trigger
DROP TRIGGER IF EXISTS trg_user_identities_password_hash ON user_identities;

CREATE TRIGGER trg_user_identities_password_hash
BEFORE INSERT OR UPDATE ON user_identities
FOR EACH ROW
EXECUTE FUNCTION user_identities_password_hash_trigger();

