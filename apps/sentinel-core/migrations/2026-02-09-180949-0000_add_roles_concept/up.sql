CREATE TYPE role_type AS ENUM (
  'user',
  'admin',
  'support'
);

CREATE TABLE roles (
  role_id uuid PRIMARY KEY,
  type role_type NOT NULL UNIQUE,
  name text NOT NULL,
  description text NOT NULL DEFAULT '',
  -- Auditing
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  created_by UUID NULL,
  updated_by UUID NULL
);

CREATE TABLE user_roles (
  user_role_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id uuid NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
  role_id uuid NOT NULL REFERENCES roles(role_id) ON DELETE CASCADE,
  created_at timestamptz NOT NULL DEFAULT now(),
  created_by UUID NULL
);

CREATE INDEX user_roles_user_id_idx ON user_roles(user_id);
CREATE INDEX user_roles_role_id_idx ON user_roles(role_id);

-- Seeds
INSERT INTO roles (role_id, type, name, description)
VALUES
  (
    gen_random_uuid(),
    'user',
    'User',
    'Default role assigned to all users. Allows basic access to the system.'
  ),
  (
    gen_random_uuid(),
    'admin',
    'Admin',
    'Full administrative access. Can manage users, roles, and system settings.'
  ),
  (
    gen_random_uuid(),
    'support',
    'Support',
    'Support staff role. Can view users and assist with troubleshooting.'
  )
ON CONFLICT (type) DO NOTHING;

