use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;

use crate::app::state::AppState;
use crate::common::auth::Principal;
use crate::common::error::AppError;
use crate::common::role::Role;
use crate::incidents::model::{
    CreatedKeyResponse, IncidentListQuery, IncidentSaveRequest, IncidentTimelineCreateRequest,
    IncidentTimelineItem, IncidentTimelineUpdateRequest, IncidentTranslationsUpdateRequest,
};
use crate::incidents::service;

#[utoipa::path(
    get,
    path = "/incidents",
    tag = "incidents",
    security((),),
    params(IncidentListQuery),
    responses(
        (status = 200, description = "List of incidents", body = Vec<crate::incidents::model::IncidentListItem>),
        (status = 403, description = "Forbidden — requires ADMIN, CO_OWNER, or CO_OWNERSHIP_BOARD"),
    ),
    description = "List incidents. Requires ADMIN, CO_OWNER, or CO_OWNERSHIP_BOARD."
)]
pub async fn list(
    principal: Principal,
    State(state): State<AppState>,
    Query(query): Query<IncidentListQuery>,
) -> Result<Json<Vec<crate::incidents::model::IncidentListItem>>, AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwner, Role::CoOwnershipBoard, Role::CoOwnershipBoardOps])?;
    let values = service::list(&state.db, query.locale.as_deref(), query.q.as_deref()).await?;
    Ok(Json(values))
}

#[utoipa::path(
    get,
    path = "/incidents/{id}",
    tag = "incidents",
    security((),),
    params(
        ("id" = String, Path, description = "Incident ID"),
        IncidentListQuery,
    ),
    responses(
        (status = 200, description = "Incident detail", body = crate::incidents::model::IncidentDetail),
        (status = 404, description = "Incident not found"),
    ),
    description = "Get incident detail. Requires ADMIN, CO_OWNER, or CO_OWNERSHIP_BOARD."
)]
pub async fn detail(
    principal: Principal,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<IncidentListQuery>,
) -> Result<Json<crate::incidents::model::IncidentDetail>, AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwner, Role::CoOwnershipBoard, Role::CoOwnershipBoardOps])?;
    let Some(value) = service::by_id(&state.db, &id, query.locale.as_deref()).await? else {
        return Err(AppError::not_found("incident not found"));
    };
    Ok(Json(value))
}

#[utoipa::path(
    get,
    path = "/incidents/{id}/edit",
    tag = "incidents",
    security((),),
    params(
        ("id" = String, Path, description = "Incident ID"),
        IncidentListQuery,
    ),
    responses(
        (status = 200, description = "Incident edit data", body = crate::incidents::model::IncidentEditData),
        (status = 404, description = "Incident not found"),
    ),
    description = "Get incident edit data. Requires ADMIN or CO_OWNERSHIP_BOARD."
)]
pub async fn edit(
    principal: Principal,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<IncidentListQuery>,
) -> Result<Json<crate::incidents::model::IncidentEditData>, AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwnershipBoard, Role::CoOwnershipBoardOps])?;
    let Some(value) = service::edit_data(&state.db, &id, query.locale.as_deref()).await? else {
        return Err(AppError::not_found("incident not found"));
    };
    Ok(Json(value))
}

#[utoipa::path(
    get,
    path = "/incidents/{id}/translations",
    tag = "incidents",
    security((),),
    params(
        ("id" = String, Path, description = "Incident ID"),
    ),
    responses(
        (status = 200, description = "Incident translations matrix", body = Vec<crate::incidents::model::IncidentTranslationMatrixRow>),
        (status = 403, description = "Forbidden — requires ADMIN"),
    ),
    description = "List incident translations. Requires ADMIN."
)]
pub async fn translations(
    principal: Principal,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<crate::incidents::model::IncidentTranslationMatrixRow>>, AppError> {
    principal.ensure_role(Role::Admin)?;
    let values = service::list_translations(&state.db, &id).await?;
    Ok(Json(values))
}

#[utoipa::path(
    post,
    path = "/incidents/{id}/translations/replace",
    tag = "incidents",
    security((),),
    params(
        ("id" = String, Path, description = "Incident ID"),
    ),
    request_body = IncidentTranslationsUpdateRequest,
    responses(
        (status = 204, description = "Translations replaced"),
        (status = 403, description = "Forbidden — requires ADMIN"),
    ),
    description = "Replace all incident translations. Requires ADMIN."
)]
pub async fn replace_translations(
    principal: Principal,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<IncidentTranslationsUpdateRequest>,
) -> Result<StatusCode, AppError> {
    principal.ensure_role(Role::Admin)?;
    service::replace_translations(&state.db, &id, &payload.values).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/incidents",
    tag = "incidents",
    security((),),
    request_body = IncidentSaveRequest,
    responses(
        (status = 201, description = "Created incident", body = CreatedKeyResponse),
        (status = 403, description = "Forbidden — requires ADMIN or CO_OWNERSHIP_BOARD"),
    ),
    description = "Create an incident. Requires ADMIN or CO_OWNERSHIP_BOARD."
)]
pub async fn create(
    principal: Principal,
    State(state): State<AppState>,
    Json(payload): Json<IncidentSaveRequest>,
) -> Result<(StatusCode, Json<CreatedKeyResponse>), AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwnershipBoard, Role::CoOwnershipBoardOps])?;
    let key = service::save_partial(&state.db, &payload, principal.user_id).await?;
    Ok((StatusCode::CREATED, Json(CreatedKeyResponse { key })))
}

#[utoipa::path(
    put,
    path = "/incidents/{id}",
    tag = "incidents",
    security((),),
    params(
        ("id" = String, Path, description = "Incident ID"),
    ),
    request_body = IncidentSaveRequest,
    responses(
        (status = 204, description = "Incident updated"),
        (status = 403, description = "Forbidden — requires ADMIN or CO_OWNERSHIP_BOARD"),
    ),
    description = "Update an incident. Requires ADMIN or CO_OWNERSHIP_BOARD."
)]
pub async fn update(
    principal: Principal,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(mut payload): Json<IncidentSaveRequest>,
) -> Result<StatusCode, AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwnershipBoard, Role::CoOwnershipBoardOps])?;
    payload.key = Some(id);
    service::save_partial(&state.db, &payload, principal.user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/incidents/{id}",
    tag = "incidents",
    security((),),
    params(
        ("id" = String, Path, description = "Incident ID"),
    ),
    responses(
        (status = 204, description = "Incident deleted"),
        (status = 403, description = "Forbidden — requires ADMIN"),
        (status = 404, description = "Incident not found"),
    ),
    description = "Delete an incident. Requires ADMIN."
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
        Err(AppError::not_found("incident not found"))
    }
}

pub async fn create_timeline(
    principal: Principal,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<IncidentListQuery>,
    Json(payload): Json<IncidentTimelineCreateRequest>,
) -> Result<(StatusCode, Json<IncidentTimelineItem>), AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwnershipBoard, Role::CoOwnershipBoardOps])?;
    let locale = query.locale.unwrap_or_else(|| "en".to_string());
    let entry = service::create_timeline_entry(&state.db, &id, &payload, &locale).await?;
    Ok((StatusCode::CREATED, Json(entry)))
}

pub async fn update_timeline(
    principal: Principal,
    State(state): State<AppState>,
    Path((id, entry_id)): Path<(String, String)>,
    Query(query): Query<IncidentListQuery>,
    Json(payload): Json<IncidentTimelineUpdateRequest>,
) -> Result<Json<IncidentTimelineItem>, AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwnershipBoard, Role::CoOwnershipBoardOps])?;
    let locale = query.locale.unwrap_or_else(|| "en".to_string());
    let entry = service::update_timeline_entry(&state.db, &id, &entry_id, &payload, &locale).await?;
    Ok(Json(entry))
}

pub async fn delete_timeline(
    principal: Principal,
    State(state): State<AppState>,
    Path((id, entry_id)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwnershipBoard, Role::CoOwnershipBoardOps])?;
    service::delete_timeline_entry(&state.db, &id, &entry_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
