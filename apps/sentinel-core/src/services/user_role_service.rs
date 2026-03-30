//! User role service — role management and role-to-user assignment.
//!
//! Roles in Sentinel are flat labels (e.g. `admin`, `user`, `support`) stored in the
//! `roles` table.  User-to-role assignments are stored in the `user_roles` join table.
//! The service combines operations on both tables behind a single interface.
//!
//! # Role types
//!
//! The `role_type` column is a Diesel enum (`RoleType`).  Built-in types (`user`, `admin`,
//! `support`) are seeded at migration time.  Admins can also create custom roles.
//!
//! # Role names vs. role types
//!
//! The policy engine evaluates roles by **name** (a free-form string).  The `role_type`
//! enum is used internally for privilege escalation checks (e.g. ensuring at least one
//! `admin` role exists at startup and for the `is_admin` flag in PASETO claims).

use crate::schema::roles::{role_id, type_};
use crate::schema::user_roles::{role_id as user_role_role_id, user_id};
use crate::{
    DbConnection, Role, RoleRepository, RoleType, ServiceError, User, UserRole, UserRoleRepository,
};

use chrono::Utc;
use diesel::BoolExpressionMethods;
use diesel::ExpressionMethods;
use std::sync::Arc;
use uuid::Uuid;

#[derive(diesel::AsChangeset)]
#[diesel(table_name = crate::schema::roles)]
struct RoleUpdateChangeset {
    name: Option<String>,
    description: Option<String>,
    updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Manages roles and user–role assignments.
pub struct UserRoleService {
    role_repository: Arc<RoleRepository>,
    user_role_repository: Arc<UserRoleRepository>,
}

impl UserRoleService {
    pub fn new(
        role_repository: Arc<RoleRepository>,
        user_role_repository: Arc<UserRoleRepository>,
    ) -> Self {
        Self {
            role_repository,
            user_role_repository,
        }
    }

    pub async fn add_role_to_user(
        &self,
        conn: &mut DbConnection<'_>,
        user_role: &UserRole,
    ) -> Result<UserRole, ServiceError> {
        self.user_role_repository
            .create(conn, user_role)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    /// Look up a role by its `RoleType` enum value.
    /// Returns `InternalError` if the seed role is missing (should never happen in production).
    pub async fn get_role_by_type(
        &self,
        conn: &mut DbConnection<'_>,
        role_type: RoleType,
    ) -> Result<Role, ServiceError> {
        let found_role = self
            .role_repository
            .find_where(conn, type_.eq(role_type.clone()))
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        if found_role.len() == 0 {
            return Err(ServiceError::InternalError(format!(
                "Missing role {:#?}",
                role_type
            )));
        }
        Ok(found_role[0].clone())
    }

    /// Fetch all `Role` records assigned to a user (two-step: user_roles → roles join).
    pub async fn get_user_roles(
        &self,
        conn: &mut DbConnection<'_>,
        user: &User,
    ) -> Result<Vec<Role>, ServiceError> {
        let user_roles = self
            .user_role_repository
            .find_where(conn, user_id.eq(user.user_id.clone()))
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let role_ids: Vec<Uuid> = user_roles.iter().map(|ur| ur.role_id).collect();

        if role_ids.is_empty() {
            return Ok(vec![]);
        }

        let roles = self
            .role_repository
            .find_where(conn, role_id.eq_any(role_ids))
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()));
        Ok(roles?)
    }

    pub async fn get_roles_by_user_id(
        &self,
        conn: &mut DbConnection<'_>,
        uid: Uuid,
    ) -> Result<Vec<Role>, ServiceError> {
        let user_roles = self
            .user_role_repository
            .find_where(conn, user_id.eq(uid))
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let role_ids: Vec<Uuid> = user_roles.iter().map(|ur| ur.role_id).collect();
        if role_ids.is_empty() {
            return Ok(vec![]);
        }

        self.role_repository
            .find_where(conn, role_id.eq_any(role_ids))
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    pub async fn find_role_by_id(
        &self,
        conn: &mut DbConnection<'_>,
        rid: Uuid,
    ) -> Result<Option<Role>, ServiceError> {
        self.role_repository
            .find_by_id(conn, rid)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    pub async fn list_roles(
        &self,
        conn: &mut DbConnection<'_>,
    ) -> Result<Vec<Role>, ServiceError> {
        self.role_repository
            .list_all(conn)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    pub async fn create_role(
        &self,
        conn: &mut DbConnection<'_>,
        role: &Role,
    ) -> Result<Role, ServiceError> {
        self.role_repository
            .create(conn, role)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    pub async fn update_role(
        &self,
        conn: &mut DbConnection<'_>,
        rid: Uuid,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<Role, ServiceError> {
        let changeset = RoleUpdateChangeset {
            name,
            description,
            updated_at: Some(Utc::now()),
        };
        self.role_repository
            .update(conn, rid, changeset)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    pub async fn delete_role(
        &self,
        conn: &mut DbConnection<'_>,
        rid: Uuid,
    ) -> Result<(), ServiceError> {
        self.role_repository
            .delete(conn, rid)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// Remove a role from a user by role name.  Returns `NotFoundError` if the role
    /// or the user–role assignment doesn't exist.
    pub async fn remove_role_from_user<'a>(
        &self,
        conn: &mut DbConnection<'a>,
        uid: Uuid,
        role_name: &'a str,
    ) -> Result<(), ServiceError> {
        // Find the role by name
        use crate::schema::roles::name as role_name_col;
        let roles_found = self
            .role_repository
            .find_where(conn, role_name_col.eq(role_name))
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        let role = roles_found
            .into_iter()
            .next()
            .ok_or_else(|| ServiceError::NotFoundError(format!("Role '{}' not found", role_name)))?;

        // Find the user_role entry
        let user_roles_found = self
            .user_role_repository
            .find_where(conn, user_id.eq(uid).and(user_role_role_id.eq(role.role_id)))
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        let ur = user_roles_found
            .into_iter()
            .next()
            .ok_or_else(|| ServiceError::NotFoundError("Role not assigned to user".to_string()))?;

        self.user_role_repository
            .delete(conn, ur.user_role_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn find_user_role(
        &self,
        conn: &mut DbConnection<'_>,
        uid: Uuid,
        rid: Uuid,
    ) -> Result<Option<UserRole>, ServiceError> {
        let found = self
            .user_role_repository
            .find_where(conn, user_id.eq(uid).and(user_role_role_id.eq(rid)))
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        Ok(found.into_iter().next())
    }
}
