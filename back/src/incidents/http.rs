use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;

use crate::app::state::AppState;
use crate::common::auth::Principal;
use crate::common::error::AppError;
use crate::incidents::model::{
    IncidentListQuery, IncidentTranslationsUpdateRequest, IncidentUpsertRequest,
};
use crate::incidents::service;

pub async fn list(
    principal: Principal,
    State(state): State<AppState>,
    Query(query): Query<IncidentListQuery>,
) -> Result<Json<Vec<crate::incidents::model::IncidentListItem>>, AppError> {
    let _ = principal.email.as_str();
    let values = service::list(&state.db, query.locale.as_deref(), query.q.as_deref()).await?;
    Ok(Json(values))
}

pub async fn detail(
    principal: Principal,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<IncidentListQuery>,
) -> Result<Json<crate::incidents::model::IncidentDetail>, AppError> {
    let _ = principal.email.as_str();
    let Some(value) = service::by_id(&state.db, &id, query.locale.as_deref()).await? else {
        return Err(AppError::not_found("incident not found"));
    };
    Ok(Json(value))
}

pub async fn translations(
    principal: Principal,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<crate::incidents::model::IncidentTranslationMatrixRow>>, AppError> {
    principal.ensure_role("ADMIN")?;
    let values = service::list_translations(&state.db, &id).await?;
    Ok(Json(values))
}

pub async fn replace_translations(
    principal: Principal,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<IncidentTranslationsUpdateRequest>,
) -> Result<StatusCode, AppError> {
    principal.ensure_role("ADMIN")?;
    service::replace_translations(&state.db, &id, &payload.values).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn create(
    principal: Principal,
    State(state): State<AppState>,
    Json(payload): Json<IncidentUpsertRequest>,
) -> Result<StatusCode, AppError> {
    principal.ensure_role("ADMIN")?;
    service::upsert(&state.db, &payload).await?;
    Ok(StatusCode::CREATED)
}

pub async fn update(
    principal: Principal,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(mut payload): Json<IncidentUpsertRequest>,
) -> Result<StatusCode, AppError> {
    principal.ensure_role("ADMIN")?;
    payload.id = id;
    service::upsert(&state.db, &payload).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete(
    principal: Principal,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    principal.ensure_role("ADMIN")?;
    let deleted = service::delete(&state.db, &id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found("incident not found"))
    }
}
