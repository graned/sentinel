ALTER TABLE user_identities
  ADD COLUMN must_change_password BOOLEAN NOT NULL DEFAULT FALSE;
