use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use crate::app::state::AppState;
use crate::common::auth::Principal;
use crate::common::error::AppError;
use crate::common::role::Role;
use crate::translations::model::BulkTranslationsUpdateRequest;
use crate::translations::service;

pub async fn list_matrix(
    principal: Principal,
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::translations::model::TranslationMatrixRow>>, AppError> {
    principal.ensure_role(Role::Admin)?;
    let values = service::list_matrix(&state.db).await?;
    Ok(Json(values))
}

pub async fn upsert_bulk(
    principal: Principal,
    State(state): State<AppState>,
    Json(payload): Json<BulkTranslationsUpdateRequest>,
) -> Result<StatusCode, AppError> {
    principal.ensure_role(Role::Admin)?;
    service::upsert_bulk(&state.db, &payload.values).await?;
    Ok(StatusCode::NO_CONTENT)
}
