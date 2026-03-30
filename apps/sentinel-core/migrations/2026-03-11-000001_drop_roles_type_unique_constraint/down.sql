-- Re-add the UNIQUE constraint on roles.type (only safe if each type has at
-- most one row at this point).
ALTER TABLE roles ADD CONSTRAINT roles_type_key UNIQUE (type);
