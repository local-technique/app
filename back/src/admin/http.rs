use axum::extract::{Path, Query, State};
use axum::Json;
use uuid::Uuid;

use crate::admin::model::{AdminUsersQuery, UpdateUserRolesRequest};
use crate::admin::service;
use crate::app::state::AppState;
use crate::common::auth::Principal;
use crate::common::error::AppError;
use crate::common::role::Role;

pub async fn roles(
    principal: Principal,
) -> Result<Json<crate::admin::model::RolesResponse>, AppError> {
    principal.ensure_role(Role::Admin)?;
    Ok(Json(service::available_roles()))
}

pub async fn users(
    principal: Principal,
    State(state): State<AppState>,
    Query(query): Query<AdminUsersQuery>,
) -> Result<Json<crate::admin::model::AdminUsersResponse>, AppError> {
    principal.ensure_role(Role::Admin)?;
    Ok(Json(service::list_users(&state.db, query).await?))
}

pub async fn update_user_roles(
    principal: Principal,
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<UpdateUserRolesRequest>,
) -> Result<Json<crate::admin::model::UpdateUserRolesResponse>, AppError> {
    principal.ensure_role(Role::Admin)?;
    Ok(Json(
        service::replace_non_admin_roles(&state.db, user_id, payload.roles).await?,
    ))
}
