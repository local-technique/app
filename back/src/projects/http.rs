use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;

use crate::app::state::AppState;
use crate::common::auth::Principal;
use crate::common::error::AppError;
use crate::common::role::Role;
use crate::projects::model::{CreatedKeyResponse, ProjectListQuery, ProjectSaveRequest, ProjectTranslationsUpdateRequest};
use crate::projects::service;

pub async fn list(
    principal: Principal,
    State(state): State<AppState>,
    Query(query): Query<ProjectListQuery>,
) -> Result<Json<Vec<crate::projects::model::ProjectListItem>>, AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwner, Role::CoOwnershipBoard])?;
    let values = service::list(&state.db, query.locale.as_deref(), query.q.as_deref()).await?;
    Ok(Json(values))
}

pub async fn detail(
    principal: Principal,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<ProjectListQuery>,
) -> Result<Json<crate::projects::model::ProjectDetail>, AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwner, Role::CoOwnershipBoard])?;
    let Some(value) = service::by_id(&state.db, &id, query.locale.as_deref()).await? else {
        return Err(AppError::not_found("project not found"));
    };
    Ok(Json(value))
}

pub async fn edit(
    principal: Principal,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<ProjectListQuery>,
) -> Result<Json<crate::projects::model::ProjectEditData>, AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwnershipBoard])?;
    let Some(value) = service::edit_data(&state.db, &id, query.locale.as_deref()).await? else {
        return Err(AppError::not_found("project not found"));
    };
    Ok(Json(value))
}

pub async fn translations(
    principal: Principal,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<crate::projects::model::ProjectTranslationMatrixRow>>, AppError> {
    principal.ensure_role(Role::Admin)?;
    let values = service::list_translations(&state.db, &id).await?;
    Ok(Json(values))
}

pub async fn replace_translations(
    principal: Principal,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<ProjectTranslationsUpdateRequest>,
) -> Result<StatusCode, AppError> {
    principal.ensure_role(Role::Admin)?;
    service::replace_translations(&state.db, &id, &payload.values).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn create(
    principal: Principal,
    State(state): State<AppState>,
    Json(payload): Json<ProjectSaveRequest>,
) -> Result<(StatusCode, Json<CreatedKeyResponse>), AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwnershipBoard])?;
    let key = service::save_partial(&state.db, &payload, principal.user_id).await?;
    Ok((StatusCode::CREATED, Json(CreatedKeyResponse { key })))
}

pub async fn update(
    principal: Principal,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(mut payload): Json<ProjectSaveRequest>,
) -> Result<StatusCode, AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwnershipBoard])?;
    payload.key = Some(id);
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
        Err(AppError::not_found("project not found"))
    }
}
