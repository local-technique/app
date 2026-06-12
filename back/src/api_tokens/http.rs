use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use crate::api_tokens::model::{CreateTokenResponse, TokenInfoResponse};
use crate::api_tokens::service;
use crate::app::state::AppState;
use crate::common::auth::Principal;
use crate::common::error::AppError;

pub async fn create_token(
    State(state): State<AppState>,
    principal: Principal,
) -> Result<(StatusCode, Json<CreateTokenResponse>), AppError> {
    let response = service::create_token(&state.db, principal.user_id).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn get_token(
    State(state): State<AppState>,
    principal: Principal,
) -> Result<Json<TokenInfoResponse>, AppError> {
    let info = service::get_token_info(&state.db, principal.user_id)
        .await?
        .ok_or_else(|| AppError::not_found("no api token found"))?;
    Ok(Json(info))
}

pub async fn revoke_token(
    State(state): State<AppState>,
    principal: Principal,
) -> Result<StatusCode, AppError> {
    service::revoke_token(&state.db, principal.user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
