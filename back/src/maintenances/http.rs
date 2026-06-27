use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;

use crate::app::state::AppState;
use crate::common::auth::Principal;
use crate::common::error::AppError;
use crate::common::role::Role;
use crate::maintenances::model::{
    CreatedKeyResponse, MaintenanceListQuery, MaintenanceSaveRequest, MaintenanceTimelineCreateRequest,
    MaintenanceTimelineItem, MaintenanceTimelineUpdateRequest, MaintenanceTranslationsUpdateRequest,
};
use crate::maintenances::service;

#[utoipa::path(
    get,
    path = "/maintenances",
    tag = "maintenances",
    security((),),
    params(MaintenanceListQuery),
    responses(
        (status = 200, description = "List of maintenances", body = Vec<crate::maintenances::model::MaintenanceListItem>),
        (status = 403, description = "Forbidden — requires ADMIN, CO_OWNER, or CO_OWNERSHIP_BOARD"),
    ),
    description = "List maintenances. Requires ADMIN, CO_OWNER, or CO_OWNERSHIP_BOARD."
)]
pub async fn list(
    principal: Principal,
    State(state): State<AppState>,
    Query(query): Query<MaintenanceListQuery>,
) -> Result<Json<Vec<crate::maintenances::model::MaintenanceListItem>>, AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwner, Role::CoOwnershipBoard, Role::CoOwnershipBoardOps])?;
    let values = service::list(&state.db, query.locale.as_deref(), query.q.as_deref()).await?;
    Ok(Json(values))
}

#[utoipa::path(
    get,
    path = "/maintenances/{id}",
    tag = "maintenances",
    security((),),
    params(
        ("id" = String, Path, description = "Maintenance ID"),
        MaintenanceListQuery,
    ),
    responses(
        (status = 200, description = "Maintenance detail", body = crate::maintenances::model::MaintenanceDetail),
        (status = 404, description = "Maintenance not found"),
    ),
    description = "Get maintenance detail. Requires ADMIN, CO_OWNER, or CO_OWNERSHIP_BOARD."
)]
pub async fn detail(
    principal: Principal,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<MaintenanceListQuery>,
) -> Result<Json<crate::maintenances::model::MaintenanceDetail>, AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwner, Role::CoOwnershipBoard, Role::CoOwnershipBoardOps])?;
    let Some(value) = service::by_id(&state.db, &id, query.locale.as_deref()).await? else {
        return Err(AppError::not_found("maintenance not found"));
    };
    Ok(Json(value))
}

#[utoipa::path(
    get,
    path = "/maintenances/{id}/edit",
    tag = "maintenances",
    security((),),
    params(
        ("id" = String, Path, description = "Maintenance ID"),
        MaintenanceListQuery,
    ),
    responses(
        (status = 200, description = "Maintenance edit data", body = crate::maintenances::model::MaintenanceEditData),
        (status = 404, description = "Maintenance not found"),
    ),
    description = "Get maintenance edit data. Requires ADMIN or CO_OWNERSHIP_BOARD."
)]
pub async fn edit(
    principal: Principal,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<MaintenanceListQuery>,
) -> Result<Json<crate::maintenances::model::MaintenanceEditData>, AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwnershipBoard, Role::CoOwnershipBoardOps])?;
    let Some(value) = service::edit_data(&state.db, &id, query.locale.as_deref()).await? else {
        return Err(AppError::not_found("maintenance not found"));
    };
    Ok(Json(value))
}

#[utoipa::path(
    get,
    path = "/maintenances/{id}/translations",
    tag = "maintenances",
    security((),),
    params(
        ("id" = String, Path, description = "Maintenance ID"),
    ),
    responses(
        (status = 200, description = "Maintenance translations matrix", body = Vec<crate::maintenances::model::MaintenanceTranslationMatrixRow>),
        (status = 403, description = "Forbidden — requires ADMIN"),
    ),
    description = "List maintenance translations. Requires ADMIN."
)]
pub async fn translations(
    principal: Principal,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<crate::maintenances::model::MaintenanceTranslationMatrixRow>>, AppError> {
    principal.ensure_role(Role::Admin)?;
    let values = service::list_translations(&state.db, &id).await?;
    Ok(Json(values))
}

#[utoipa::path(
    post,
    path = "/maintenances/{id}/translations/replace",
    tag = "maintenances",
    security((),),
    params(
        ("id" = String, Path, description = "Maintenance ID"),
    ),
    request_body = MaintenanceTranslationsUpdateRequest,
    responses(
        (status = 204, description = "Translations replaced"),
        (status = 403, description = "Forbidden — requires ADMIN"),
    ),
    description = "Replace all maintenance translations. Requires ADMIN."
)]
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

#[utoipa::path(
    post,
    path = "/maintenances",
    tag = "maintenances",
    security((),),
    request_body = MaintenanceSaveRequest,
    responses(
        (status = 201, description = "Created maintenance", body = CreatedKeyResponse),
        (status = 403, description = "Forbidden — requires ADMIN or CO_OWNERSHIP_BOARD"),
    ),
    description = "Create a maintenance. Requires ADMIN or CO_OWNERSHIP_BOARD."
)]
pub async fn create(
    principal: Principal,
    State(state): State<AppState>,
    Json(payload): Json<MaintenanceSaveRequest>,
) -> Result<(StatusCode, Json<CreatedKeyResponse>), AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwnershipBoard, Role::CoOwnershipBoardOps])?;
    let key = service::save_partial(&state.db, &payload, principal.user_id).await?;
    Ok((StatusCode::CREATED, Json(CreatedKeyResponse { key })))
}

#[utoipa::path(
    put,
    path = "/maintenances/{id}",
    tag = "maintenances",
    security((),),
    params(
        ("id" = String, Path, description = "Maintenance ID"),
    ),
    request_body = MaintenanceSaveRequest,
    responses(
        (status = 204, description = "Maintenance updated"),
        (status = 403, description = "Forbidden — requires ADMIN or CO_OWNERSHIP_BOARD"),
    ),
    description = "Update a maintenance. Requires ADMIN or CO_OWNERSHIP_BOARD."
)]
pub async fn update(
    principal: Principal,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(mut payload): Json<MaintenanceSaveRequest>,
) -> Result<StatusCode, AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwnershipBoard, Role::CoOwnershipBoardOps])?;
    payload.key = Some(id);
    service::save_partial(&state.db, &payload, principal.user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/maintenances/{id}",
    tag = "maintenances",
    security((),),
    params(
        ("id" = String, Path, description = "Maintenance ID"),
    ),
    responses(
        (status = 204, description = "Maintenance deleted"),
        (status = 403, description = "Forbidden — requires ADMIN"),
        (status = 404, description = "Maintenance not found"),
    ),
    description = "Delete a maintenance. Requires ADMIN."
)]
pub async fn delete(
    principal: Principal,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwnershipBoardOps])?;
    let deleted = service::delete(&state.db, &id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found("maintenance not found"))
    }
}

pub async fn create_timeline(
    principal: Principal,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<MaintenanceListQuery>,
    Json(payload): Json<MaintenanceTimelineCreateRequest>,
) -> Result<(StatusCode, Json<MaintenanceTimelineItem>), AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwnershipBoard, Role::CoOwnershipBoardOps])?;
    let locale = query.locale.unwrap_or_else(|| "en".to_string());
    let entry = service::create_timeline_entry(&state.db, &id, &payload, &locale, principal.user_id).await?;
    Ok((StatusCode::CREATED, Json(entry)))
}

pub async fn update_timeline(
    principal: Principal,
    State(state): State<AppState>,
    Path((id, entry_id)): Path<(String, String)>,
    Query(query): Query<MaintenanceListQuery>,
    Json(payload): Json<MaintenanceTimelineUpdateRequest>,
) -> Result<Json<MaintenanceTimelineItem>, AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwnershipBoard, Role::CoOwnershipBoardOps])?;
    let locale = query.locale.unwrap_or_else(|| "en".to_string());
    let entry = service::update_timeline_entry(&state.db, &id, &entry_id, &payload, &locale, &principal).await?;
    Ok(Json(entry))
}

pub async fn delete_timeline(
    principal: Principal,
    State(state): State<AppState>,
    Path((id, entry_id)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwnershipBoard, Role::CoOwnershipBoardOps])?;
    service::delete_timeline_entry(&state.db, &id, &entry_id, &principal).await?;
    Ok(StatusCode::NO_CONTENT)
}
