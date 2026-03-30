//! Repository for the `user_roles` join table.
//!
//! Each row represents one role assignment: `(user_id, role_id)`.  Roles are never
//! implicitly cascaded — they must be explicitly removed via `delete` (by PK) or by
//! the bulk `remove_role_from_user` logic in `UserRoleService`.

use crate::impl_repository;
use crate::UserRole;

use uuid::Uuid;

impl_repository!(
    UserRoleRepository for UserRole,
    crate::schema::user_roles::table,
    crate::schema::user_roles::user_role_id,
    Uuid
);
