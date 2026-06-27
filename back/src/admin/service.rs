use std::collections::HashSet;

use uuid::Uuid;

use crate::admin::model::{
    AdminUserItem, AdminUsersQuery, AdminUsersResponse, RoleDescriptor, RolesResponse, UpdateUserRolesResponse,
};
use crate::admin::repository;
use crate::common::error::AppError;
use crate::common::role::Role;

const PAGE_SIZE: i64 = 30;
const NO_ROLE_FILTER: &str = "__NO_ROLE__";

pub fn available_roles() -> RolesResponse {
    RolesResponse {
        roles: Role::ALL
            .into_iter()
            .map(|role| RoleDescriptor {
                code: role.code(),
                label_key: role.label_key(),
            })
            .collect(),
    }
}

pub async fn list_users(db: &sqlx::PgPool, query: AdminUsersQuery) -> Result<AdminUsersResponse, AppError> {
    let offset = query.offset.unwrap_or(0).max(0);
    let limit = query.limit.unwrap_or(PAGE_SIZE);
    if limit != PAGE_SIZE {
        return Err(AppError::bad_request("limit must be 30"));
    }

    let sort = query.sort.as_deref().unwrap_or("id");
    if !matches!(sort, "id" | "email" | "created_at" | "last_login_at") {
        return Err(AppError::bad_request("unsupported sort"));
    }

    let direction = query.direction.as_deref().unwrap_or("asc");
    if !matches!(direction, "asc" | "desc") {
        return Err(AppError::bad_request("unsupported direction"));
    }

    let role = query.role.as_deref();
    let only_without_roles = role == Some(NO_ROLE_FILTER);
    if let Some(role) = role.filter(|value| *value != NO_ROLE_FILTER) {
        validate_known_role(role)?;
    }

    let search = query.search_email.as_deref().map(str::trim).filter(|value| !value.is_empty());
    let (rows, total) = repository::list_users(db, repository::ListUsersParams {
        offset,
        limit: PAGE_SIZE,
        search_email: search,
        role: role.filter(|value| *value != NO_ROLE_FILTER),
        only_without_roles,
        sort,
        direction,
    })
    .await?;

    Ok(AdminUsersResponse {
        items: rows
            .into_iter()
            .map(|row| AdminUserItem {
                id: row.id.to_string(),
                email: row.email,
                first_name: row.first_name,
                last_name: row.last_name,
                created_at: row.created_at,
                last_login_at: row.last_login_at,
                roles: row.roles,
            })
            .collect(),
        total,
        offset,
        limit: PAGE_SIZE,
    })
}

pub async fn replace_non_admin_roles(
    db: &sqlx::PgPool,
    user_id: Uuid,
    requested_roles: Vec<String>,
) -> Result<UpdateUserRolesResponse, AppError> {
    let mut seen = HashSet::new();
    let mut roles = Vec::new();
    for role in requested_roles {
        if Role::parse(&role) == Some(Role::Admin) {
            return Err(AppError::bad_request("ADMIN cannot be updated here"));
        }
        let parsed = validate_assignable_role(&role)?;
        if seen.insert(role.clone()) {
            roles.push(parsed.code().to_string());
        }
    }

    let updated = repository::replace_non_admin_roles(db, user_id, &roles)
        .await?
        .ok_or_else(|| AppError::not_found("user not found"))?;

    Ok(UpdateUserRolesResponse {
        id: user_id.to_string(),
        roles: updated,
    })
}

pub async fn update_user_names(
    db: &sqlx::PgPool,
    user_id: Uuid,
    first_name: Option<String>,
    last_name: Option<String>,
) -> Result<crate::admin::model::UpdateUserNamesResponse, AppError> {
    let first_name = first_name.map(|n| n.trim().to_string()).filter(|n| !n.is_empty());
    let last_name = last_name.map(|n| n.trim().to_string()).filter(|n| !n.is_empty());

    repository::update_user_names(db, user_id, first_name, last_name)
        .await?
        .ok_or_else(|| AppError::not_found("user not found"))
}

fn validate_known_role(role: &str) -> Result<(), AppError> {
    Role::parse(role)
        .map(|_| ())
        .ok_or_else(|| AppError::bad_request("unknown role"))
}

fn validate_assignable_role(role: &str) -> Result<Role, AppError> {
    let parsed = Role::parse(role).ok_or_else(|| AppError::bad_request("unknown role"))?;
    if parsed.is_assignable() {
        Ok(parsed)
    } else {
        Err(AppError::bad_request("role is not assignable"))
    }
}
