use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::IntoResponse;
use axum::Json;
use axum_extra::extract::PrivateCookieJar;

use crate::app::state::AppState;
use crate::auth::model::{
    ExchangeRequest, LogoutRequest, OAuthCallbackQuery, OAuthStartQuery, Provider, RefreshRequest,
};
use crate::auth::service;
use crate::common::error::AppError;

pub async fn start_oauth(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Path(provider_raw): Path<String>,
    Query(query): Query<OAuthStartQuery>,
) -> Result<(PrivateCookieJar, axum::response::Redirect), AppError> {
    let provider = Provider::parse(&provider_raw).ok_or_else(|| AppError::bad_request("unsupported provider"))?;
    service::start_oauth(&state, jar, provider, query).await
}

pub async fn oauth_callback(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Path(provider_raw): Path<String>,
    Query(query): Query<OAuthCallbackQuery>,
) -> Result<(PrivateCookieJar, axum::response::Redirect), AppError> {
    let provider = Provider::parse(&provider_raw).ok_or_else(|| AppError::bad_request("unsupported provider"))?;
    service::oauth_callback(&state, jar, provider, query).await
}

pub async fn exchange_session(
    State(state): State<AppState>,
    Json(payload): Json<ExchangeRequest>,
) -> Result<impl IntoResponse, AppError> {
    let response = service::exchange_session(&state, &payload.code).await?;
    Ok(([(header::CACHE_CONTROL, "no-store")], Json(response)))
}

pub async fn refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<RefreshRequest>,
) -> Result<Json<crate::auth::model::RefreshResponse>, AppError> {
    let response = service::refresh(&state, &headers, &payload).await?;
    Ok(Json(response))
}

pub async fn me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<crate::auth::model::MeResponse>, AppError> {
    let response = service::me(&state, &headers).await?;
    Ok(Json(response))
}

pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<LogoutRequest>,
) -> Result<StatusCode, AppError> {
    service::logout(&state, &headers, payload.refresh_token.as_deref()).await?;
    Ok(StatusCode::NO_CONTENT)
}
