use axum::extract::{FromRef, FromRequestParts};
use axum::http::{HeaderMap, request::Parts};
use chrono::Utc;
use uuid::Uuid;

use crate::app::state::AppState;
use crate::auth::repository;
use crate::auth::service;
use crate::common::error::AppError;

#[derive(Clone)]
pub struct Principal {
    pub email: String,
    pub roles: Vec<String>,
}

impl Principal {
    pub fn ensure_role(&self, role: &str) -> Result<(), AppError> {
        if self.roles.iter().any(|value| value == role) {
            Ok(())
        } else {
            Err(AppError::forbidden("missing required role"))
        }
    }
}

impl<S> FromRequestParts<S> for Principal
where
    AppState: axum::extract::FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let token = bearer_from_headers(&parts.headers)?;
        let claims = service::decode_access_token(&app_state, &token)?;
        let session_id =
            Uuid::parse_str(&claims.sid).map_err(|_| AppError::unauthorized("invalid session claim"))?;

        let session = repository::find_session_by_id(&app_state.db, session_id)
            .await?
            .ok_or_else(|| AppError::unauthorized("invalid session"))?;
        if session.revoked_at.is_some() || session.compromised_at.is_some() || session.expires_at <= Utc::now() {
            return Err(AppError::unauthorized("invalid session"));
        }

        let _user = repository::get_user_by_id(&app_state.db, session.user_id)
            .await?
            .ok_or_else(|| AppError::unauthorized("invalid user"))?;

        Ok(Self {
            email: claims.email,
            roles: claims.roles,
        })
    }
}

fn bearer_from_headers(headers: &HeaderMap) -> Result<String, AppError> {
    let auth = headers
        .get(axum::http::header::AUTHORIZATION)
        .ok_or_else(|| AppError::unauthorized("missing authorization header"))?
        .to_str()
        .map_err(|_| AppError::unauthorized("invalid authorization header"))?;

    let token = auth
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::unauthorized("authorization must use Bearer token"))?;

    if token.is_empty() {
        return Err(AppError::unauthorized("empty bearer token"));
    }

    Ok(token.to_string())
}
