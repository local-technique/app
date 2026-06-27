use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;

use crate::app::state::AppState;
use crate::common::auth::Principal;
use crate::common::error::AppError;
use crate::common::role::Role;
use crate::projects::model::{
    CreatedKeyResponse, ProjectListQuery, ProjectSaveRequest, ProjectTimelineCreateRequest,
    ProjectTimelineItem, ProjectTimelineUpdateRequest, ProjectTranslationsUpdateRequest,
};
use crate::projects::service;

#[utoipa::path(
    get,
    path = "/projects",
    tag = "projects",
    security((),),
    params(ProjectListQuery),
    responses(
        (status = 200, description = "List of projects", body = Vec<crate::projects::model::ProjectListItem>),
        (status = 403, description = "Forbidden — requires ADMIN, CO_OWNER, or CO_OWNERSHIP_BOARD"),
    ),
    description = "List projects. Requires ADMIN, CO_OWNER, or CO_OWNERSHIP_BOARD."
)]
pub async fn list(
    principal: Principal,
    State(state): State<AppState>,
    Query(query): Query<ProjectListQuery>,
) -> Result<Json<Vec<crate::projects::model::ProjectListItem>>, AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwner, Role::CoOwnershipBoard, Role::CoOwnershipBoardOps])?;
    let values = service::list(&state.db, query.locale.as_deref(), query.q.as_deref()).await?;
    Ok(Json(values))
}

#[utoipa::path(
    get,
    path = "/projects/{id}",
    tag = "projects",
    security((),),
    params(
        ("id" = String, Path, description = "Project ID"),
        ProjectListQuery,
    ),
    responses(
        (status = 200, description = "Project detail", body = crate::projects::model::ProjectDetail),
        (status = 404, description = "Project not found"),
    ),
    description = "Get project detail. Requires ADMIN, CO_OWNER, or CO_OWNERSHIP_BOARD."
)]
pub async fn detail(
    principal: Principal,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<ProjectListQuery>,
) -> Result<Json<crate::projects::model::ProjectDetail>, AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwner, Role::CoOwnershipBoard, Role::CoOwnershipBoardOps])?;
    let Some(value) = service::by_id(&state.db, &id, query.locale.as_deref()).await? else {
        return Err(AppError::not_found("project not found"));
    };
    Ok(Json(value))
}

#[utoipa::path(
    get,
    path = "/projects/{id}/edit",
    tag = "projects",
    security((),),
    params(
        ("id" = String, Path, description = "Project ID"),
        ProjectListQuery,
    ),
    responses(
        (status = 200, description = "Project edit data", body = crate::projects::model::ProjectEditData),
        (status = 404, description = "Project not found"),
    ),
    description = "Get project edit data. Requires ADMIN or CO_OWNERSHIP_BOARD."
)]
pub async fn edit(
    principal: Principal,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<ProjectListQuery>,
) -> Result<Json<crate::projects::model::ProjectEditData>, AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwnershipBoard, Role::CoOwnershipBoardOps])?;
    let Some(value) = service::edit_data(&state.db, &id, query.locale.as_deref()).await? else {
        return Err(AppError::not_found("project not found"));
    };
    Ok(Json(value))
}

#[utoipa::path(
    get,
    path = "/projects/{id}/translations",
    tag = "projects",
    security((),),
    params(
        ("id" = String, Path, description = "Project ID"),
    ),
    responses(
        (status = 200, description = "Project translations matrix", body = Vec<crate::projects::model::ProjectTranslationMatrixRow>),
        (status = 403, description = "Forbidden — requires ADMIN"),
    ),
    description = "List project translations. Requires ADMIN."
)]
pub async fn translations(
    principal: Principal,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<crate::projects::model::ProjectTranslationMatrixRow>>, AppError> {
    principal.ensure_role(Role::Admin)?;
    let values = service::list_translations(&state.db, &id).await?;
    Ok(Json(values))
}

#[utoipa::path(
    post,
    path = "/projects/{id}/translations/replace",
    tag = "projects",
    security((),),
    params(
        ("id" = String, Path, description = "Project ID"),
    ),
    request_body = ProjectTranslationsUpdateRequest,
    responses(
        (status = 204, description = "Translations replaced"),
        (status = 403, description = "Forbidden — requires ADMIN"),
    ),
    description = "Replace all project translations. Requires ADMIN."
)]
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

#[utoipa::path(
    post,
    path = "/projects",
    tag = "projects",
    security((),),
    request_body = ProjectSaveRequest,
    responses(
        (status = 201, description = "Created project", body = CreatedKeyResponse),
        (status = 403, description = "Forbidden — requires ADMIN or CO_OWNERSHIP_BOARD"),
    ),
    description = "Create a project. Requires ADMIN or CO_OWNERSHIP_BOARD."
)]
pub async fn create(
    principal: Principal,
    State(state): State<AppState>,
    Json(payload): Json<ProjectSaveRequest>,
) -> Result<(StatusCode, Json<CreatedKeyResponse>), AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwnershipBoard, Role::CoOwnershipBoardOps])?;
    let key = service::save_partial(&state.db, &payload, principal.user_id).await?;
    Ok((StatusCode::CREATED, Json(CreatedKeyResponse { key })))
}

#[utoipa::path(
    put,
    path = "/projects/{id}",
    tag = "projects",
    security((),),
    params(
        ("id" = String, Path, description = "Project ID"),
    ),
    request_body = ProjectSaveRequest,
    responses(
        (status = 204, description = "Project updated"),
        (status = 403, description = "Forbidden — requires ADMIN or CO_OWNERSHIP_BOARD"),
    ),
    description = "Update a project. Requires ADMIN or CO_OWNERSHIP_BOARD."
)]
pub async fn update(
    principal: Principal,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(mut payload): Json<ProjectSaveRequest>,
) -> Result<StatusCode, AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwnershipBoard, Role::CoOwnershipBoardOps])?;
    payload.key = Some(id);
    service::save_partial(&state.db, &payload, principal.user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/projects/{id}",
    tag = "projects",
    security((),),
    params(
        ("id" = String, Path, description = "Project ID"),
    ),
    responses(
        (status = 204, description = "Project deleted"),
        (status = 403, description = "Forbidden — requires ADMIN"),
        (status = 404, description = "Project not found"),
    ),
    description = "Delete a project. Requires ADMIN."
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
        Err(AppError::not_found("project not found"))
    }
}

pub async fn create_timeline(
    principal: Principal,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<ProjectListQuery>,
    Json(payload): Json<ProjectTimelineCreateRequest>,
) -> Result<(StatusCode, Json<ProjectTimelineItem>), AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwnershipBoard, Role::CoOwnershipBoardOps])?;
    let locale = query.locale.unwrap_or_else(|| "en".to_string());
    let entry = service::create_timeline_entry(&state.db, &id, &payload, &locale, principal.user_id).await?;
    Ok((StatusCode::CREATED, Json(entry)))
}

pub async fn update_timeline(
    principal: Principal,
    State(state): State<AppState>,
    Path((id, entry_id)): Path<(String, String)>,
    Query(query): Query<ProjectListQuery>,
    Json(payload): Json<ProjectTimelineUpdateRequest>,
) -> Result<Json<ProjectTimelineItem>, AppError> {
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
