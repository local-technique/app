use axum::extract::{FromRef, FromRequestParts};
use axum::http::{HeaderMap, request::Parts};
use chrono::Utc;
use uuid::Uuid;

use crate::api_tokens::repository as api_tokens_repository;
use crate::api_tokens::service as api_tokens_service;
use crate::app::state::AppState;
use crate::auth::repository;
use crate::auth::service;
use crate::common::error::AppError;
use crate::common::role::Role;

#[derive(Clone)]
pub struct Principal {
    pub user_id: Uuid,
    pub roles: Vec<String>,
}

impl Principal {
    pub fn ensure_role(&self, role: Role) -> Result<(), AppError> {
        if self.roles.iter().any(|value| value == role.code()) {
            Ok(())
        } else {
            Err(AppError::forbidden("missing required role"))
        }
    }

    pub fn ensure_any_role(&self, roles: &[Role]) -> Result<(), AppError> {
        if roles.iter().any(|role| self.roles.iter().any(|value| value == role.code())) {
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

        if token.starts_with("lc_") {
            return authenticate_via_api_token(&app_state, &token).await;
        }

        let claims = service::decode_access_token(&app_state, &token)?;
        let session_id =
            Uuid::parse_str(&claims.sid).map_err(|_| AppError::unauthorized("invalid session claim"))?;

        let session = repository::find_session_by_id(&app_state.db, session_id)
            .await?
            .ok_or_else(|| AppError::unauthorized("invalid session"))?;
        if session.revoked_at.is_some() || session.compromised_at.is_some() || session.expires_at <= Utc::now() {
            return Err(AppError::unauthorized("invalid session"));
        }

        let user = repository::get_user_by_id(&app_state.db, session.user_id)
            .await?
            .ok_or_else(|| AppError::unauthorized("invalid user"))?;

        Ok(Self {
            user_id: user.id,
            roles: user.roles,
        })
    }
}

async fn authenticate_via_api_token(app_state: &AppState, token: &str) -> Result<Principal, AppError> {
    let lookup_hash = api_tokens_service::hash_for_lookup(token);

    let api_token = api_tokens_repository::find_token_by_hash(&app_state.db, &lookup_hash)
        .await?
        .ok_or_else(|| AppError::unauthorized("invalid token"))?;

    if !api_tokens_service::verify_token(token, &api_token.token_hash)
        .unwrap_or(false)
    {
        return Err(AppError::unauthorized("invalid token"));
    }

    tokio::spawn({
        let db = app_state.db.clone();
        let tid = api_token.id;
        async move {
            let _ = api_tokens_repository::update_last_used(&db, tid).await;
        }
    });

    let user = repository::get_user_by_id(&app_state.db, api_token.user_id)
        .await?
        .ok_or_else(|| AppError::unauthorized("invalid user"))?;

    Ok(Principal {
        user_id: user.id,
        roles: user.roles,
    })
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
