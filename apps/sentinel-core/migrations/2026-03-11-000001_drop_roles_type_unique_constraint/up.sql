-- Drop the UNIQUE constraint on roles.type so that multiple named roles
-- can share the same permission level (e.g., "Super Admin" and "Admin" both
-- with type = 'admin').  The seeded system roles remain; admins can now create
-- additional named roles without hitting a duplicate-key 500 error.

ALTER TABLE roles DROP CONSTRAINT IF EXISTS roles_type_key;
