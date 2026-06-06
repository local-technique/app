use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;

use crate::app::state::AppState;
use crate::categories::model::{CategoryCreateRequest, CategoryListQuery, CategoryUpdateRequest};
use crate::categories::service;
use crate::common::auth::Principal;
use crate::common::error::AppError;
use crate::common::role::Role;

pub async fn list(
    principal: Principal,
    State(state): State<AppState>,
    Query(query): Query<CategoryListQuery>,
) -> Result<Json<Vec<crate::categories::model::CategoryItem>>, AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwnershipBoard])?;
    Ok(Json(service::list(&state.db, query.locale.as_deref()).await?))
}

pub async fn admin_list(
    principal: Principal,
    State(state): State<AppState>,
    Query(query): Query<CategoryListQuery>,
) -> Result<Json<Vec<crate::categories::model::CategoryItem>>, AppError> {
    principal.ensure_role(Role::Admin)?;
    Ok(Json(service::list(&state.db, query.locale.as_deref()).await?))
}

pub async fn create(
    principal: Principal,
    State(state): State<AppState>,
    Json(payload): Json<CategoryCreateRequest>,
) -> Result<StatusCode, AppError> {
    principal.ensure_role(Role::Admin)?;
    service::create(&state.db, &payload).await?;
    Ok(StatusCode::CREATED)
}

pub async fn update(
    principal: Principal,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<CategoryUpdateRequest>,
) -> Result<StatusCode, AppError> {
    principal.ensure_role(Role::Admin)?;
    service::update(&state.db, &id, &payload).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete(
    principal: Principal,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    principal.ensure_role(Role::Admin)?;
    service::delete(&state.db, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}
