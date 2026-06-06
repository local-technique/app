use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;

use crate::app::state::AppState;
use crate::common::auth::Principal;
use crate::common::error::AppError;
use crate::common::role::Role;
use crate::maintenances::model::{
    MaintenanceListQuery, MaintenanceSaveRequest, MaintenanceTranslationsUpdateRequest,
};
use crate::maintenances::service;

pub async fn list(
    principal: Principal,
    State(state): State<AppState>,
    Query(query): Query<MaintenanceListQuery>,
) -> Result<Json<Vec<crate::maintenances::model::MaintenanceListItem>>, AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwner, Role::CoOwnershipBoard])?;
    let values = service::list(&state.db, query.locale.as_deref(), query.q.as_deref()).await?;
    Ok(Json(values))
}

pub async fn detail(
    principal: Principal,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<MaintenanceListQuery>,
) -> Result<Json<crate::maintenances::model::MaintenanceDetail>, AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwner, Role::CoOwnershipBoard])?;
    let Some(value) = service::by_id(&state.db, &id, query.locale.as_deref()).await? else {
        return Err(AppError::not_found("maintenance not found"));
    };
    Ok(Json(value))
}

pub async fn edit(
    principal: Principal,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<MaintenanceListQuery>,
) -> Result<Json<crate::maintenances::model::MaintenanceEditData>, AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwnershipBoard])?;
    let Some(value) = service::edit_data(&state.db, &id, query.locale.as_deref()).await? else {
        return Err(AppError::not_found("maintenance not found"));
    };
    Ok(Json(value))
}

pub async fn translations(
    principal: Principal,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<crate::maintenances::model::MaintenanceTranslationMatrixRow>>, AppError> {
    principal.ensure_role(Role::Admin)?;
    let values = service::list_translations(&state.db, &id).await?;
    Ok(Json(values))
}

pub async fn replace_translations(
    principal: Principal,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<MaintenanceTranslationsUpdateRequest>,
) -> Result<StatusCode, AppError> {
    principal.ensure_role(Role::Admin)?;
    service::replace_translations(&state.db, &id, &payload.values).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn create(
    principal: Principal,
    State(state): State<AppState>,
    Json(payload): Json<MaintenanceSaveRequest>,
) -> Result<StatusCode, AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwnershipBoard])?;
    service::save_partial(&state.db, &payload, principal.user_id).await?;
    Ok(StatusCode::CREATED)
}

pub async fn update(
    principal: Principal,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(mut payload): Json<MaintenanceSaveRequest>,
) -> Result<StatusCode, AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwnershipBoard])?;
    payload.id = id;
    service::save_partial(&state.db, &payload, principal.user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete(
    principal: Principal,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    principal.ensure_role(Role::Admin)?;
    let deleted = service::delete(&state.db, &id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found("maintenance not found"))
    }
}
