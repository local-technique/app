use std::time::{Duration, SystemTime};

use axum::http::{HeaderMap, header};
use axum::response::Redirect;
use axum_extra::extract::PrivateCookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{Duration as ChronoDuration, Utc};
use cookie::time::Duration as CookieDuration;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rand::RngCore;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::app::state::{AppState, ExchangeCodeRecord};
use crate::auth::model::{
    AccessTokenClaims, ExchangeResponse, MeResponse, OAuthCallbackQuery, OAuthStartQuery, Provider, RefreshRequest,
    RefreshResponse,
};
use crate::auth::repository;
use crate::common::error::AppError;

const OAUTH_STATE_TTL: Duration = Duration::from_secs(10 * 60);
const ACCESS_TOKEN_TTL: Duration = Duration::from_secs(15 * 60);
const REFRESH_SESSION_TTL: Duration = Duration::from_secs(7 * 24 * 60 * 60);
const EXCHANGE_CODE_TTL: Duration = Duration::from_secs(2 * 60);

#[derive(Debug, Serialize, Deserialize)]
struct OAuthStateCookie {
    provider: String,
    state: String,
    code_verifier: Option<String>,
    redirect_to: String,
    issued_at_unix: u64,
}

impl OAuthStateCookie {
    fn to_cookie_value(&self) -> Result<String, AppError> {
        serde_json::to_string(self)
            .map_err(|e| AppError::internal(format!("failed to serialize oauth state: {e}")))
    }

    fn from_cookie_value(value: &str) -> Result<Self, AppError> {
        serde_json::from_str::<Self>(value).map_err(|_| AppError::bad_request("invalid oauth state cookie"))
    }
}

pub fn decode_access_token(state: &AppState, token: &str) -> Result<AccessTokenClaims, AppError> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_aud = false;
    validation.validate_exp = true;
    let data = decode::<AccessTokenClaims>(
        token,
        &DecodingKey::from_secret(state.config.access_token_jwt_secret.as_bytes()),
        &validation,
    )
    .map_err(|_| AppError::unauthorized("invalid or expired access token"))?;
    Ok(data.claims)
}

fn oauth_cookie_name(provider: Provider) -> &'static str {
    match provider {
        Provider::Google => "oauth_google_state",
        Provider::Facebook => "oauth_facebook_state",
    }
}

fn oauth_state_cookie(provider: Provider, value: String, secure: bool) -> Cookie<'static> {
    Cookie::build((oauth_cookie_name(provider), value))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(secure)
        .max_age(CookieDuration::seconds(OAUTH_STATE_TTL.as_secs() as i64))
        .build()
}

fn oauth_state_delete_cookie(provider: Provider, secure: bool) -> Cookie<'static> {
    Cookie::build((oauth_cookie_name(provider), ""))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(secure)
        .max_age(CookieDuration::seconds(0))
        .build()
}

fn is_https_url(url: &str) -> bool {
    url.starts_with("https://")
}

fn frontend_callback_base(state: &AppState) -> String {
    let raw = state
        .config
        .frontend_base_url
        .as_deref()
        .unwrap_or(state.config.frontend_origin.as_str());
    raw.trim_end_matches('/').to_string()
}

pub async fn start_oauth(
    state: &AppState,
    jar: PrivateCookieJar,
    provider: Provider,
    query: OAuthStartQuery,
) -> Result<(PrivateCookieJar, Redirect), AppError> {
    let oauth_state = random_urlsafe(32);
    let code_verifier = random_urlsafe(64);
    let redirect_to = sanitize_redirect_path(query.redirect.as_deref().unwrap_or("/events"));

    let cookie_payload = OAuthStateCookie {
        provider: provider.as_str().to_string(),
        state: oauth_state.clone(),
        code_verifier: matches!(provider, Provider::Google).then_some(code_verifier.clone()),
        redirect_to,
        issued_at_unix: unix_ts(SystemTime::now())?,
    };

    let redirect_uri = format!("{}/auth/{}/callback", state.config.app_base_url, provider.as_str());
    let secure_cookie = is_https_url(&state.config.app_base_url);

    let auth_url = match provider {
        Provider::Google => {
            let code_challenge = pkce_challenge(&code_verifier);
            format!(
                "https://accounts.google.com/o/oauth2/v2/auth?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method=S256",
                url_encode(&state.config.google_client_id),
                url_encode(&redirect_uri),
                url_encode("openid email profile"),
                url_encode(&oauth_state),
                url_encode(&code_challenge),
            )
        }
        Provider::Facebook => format!(
            "https://www.facebook.com/v20.0/dialog/oauth?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}",
            url_encode(&state.config.facebook_client_id),
            url_encode(&redirect_uri),
            url_encode("email"),
            url_encode(&oauth_state),
        ),
    };

    let jar = jar.add(oauth_state_cookie(provider, cookie_payload.to_cookie_value()?, secure_cookie));
    Ok((jar, Redirect::temporary(&auth_url)))
}

pub async fn oauth_callback(
    state: &AppState,
    jar: PrivateCookieJar,
    provider: Provider,
    query: OAuthCallbackQuery,
) -> Result<(PrivateCookieJar, Redirect), AppError> {
    if let Some(error) = query.error {
        return Err(AppError::bad_request(format!("provider returned error: {error}")));
    }

    let auth_state = query
        .state
        .ok_or_else(|| AppError::bad_request("missing oauth state"))?;
    let code = query
        .code
        .ok_or_else(|| AppError::bad_request("missing oauth code"))?;

    let cookie_name = oauth_cookie_name(provider);
    let cookie_value = jar
        .get(cookie_name)
        .map(|cookie| cookie.value().to_string())
        .ok_or_else(|| AppError::bad_request("missing oauth state cookie"))?;

    let state_cookie = OAuthStateCookie::from_cookie_value(&cookie_value)?;
    let issued_at = SystemTime::UNIX_EPOCH
        .checked_add(Duration::from_secs(state_cookie.issued_at_unix))
        .ok_or_else(|| AppError::bad_request("invalid oauth state timestamp"))?;

    if is_expired(issued_at, OAUTH_STATE_TTL) {
        return Err(AppError::bad_request("oauth state expired"));
    }

    if state_cookie.provider != provider.as_str() {
        return Err(AppError::bad_request("oauth provider mismatch"));
    }

    if state_cookie.state != auth_state {
        return Err(AppError::bad_request("invalid oauth state"));
    }

    let email = match provider {
        Provider::Google => google_exchange_and_email(state, &code, state_cookie.code_verifier.as_deref()).await?,
        Provider::Facebook => facebook_exchange_and_email(state, &code).await?,
    };

    let user = repository::find_or_create_user(&state.db, provider, &email).await?;
    if provider == Provider::Google && is_admin_email(state, &email) {
        repository::ensure_admin_role(&state.db, user.id).await?;
    }

    let refresh_token = random_urlsafe(48);
    let refresh_expires_at = Utc::now() + ChronoDuration::seconds(REFRESH_SESSION_TTL.as_secs() as i64);
    let session_id = repository::insert_session(
        &state.db,
        user.id,
        &hash_token(&refresh_token),
        refresh_expires_at,
    )
    .await?;

    let exchange_code = random_urlsafe(24);
    {
        let mut codes = state.exchange_codes.lock().await;
        codes.retain(|_, value| !is_expired(value.created_at, EXCHANGE_CODE_TTL));
        codes.insert(
            exchange_code.clone(),
            ExchangeCodeRecord {
                session_id,
                created_at: SystemTime::now(),
                redirect_to: state_cookie.redirect_to,
                refresh_token,
            },
        );
    }

    let frontend_callback = format!(
        "{}/#/auth/callback?code={}",
        frontend_callback_base(state),
        url_encode(&exchange_code)
    );

    let jar = jar.remove(oauth_state_delete_cookie(provider, is_https_url(&state.config.app_base_url)));
    Ok((jar, Redirect::to(&frontend_callback)))
}

pub async fn exchange_session(state: &AppState, code: &str) -> Result<ExchangeResponse, AppError> {
    let exchange = {
        let mut codes = state.exchange_codes.lock().await;
        let value = codes
            .remove(code)
            .ok_or_else(|| AppError::unauthorized("invalid exchange code"))?;
        if is_expired(value.created_at, EXCHANGE_CODE_TTL) {
            return Err(AppError::unauthorized("exchange code expired"));
        }
        value
    };

    let session = repository::find_session_by_id(&state.db, exchange.session_id)
        .await?
        .ok_or_else(|| AppError::unauthorized("invalid session"))?;
    if !session_active(&session) {
        return Err(AppError::unauthorized("invalid session"));
    }

    let user = repository::get_user_by_id(&state.db, session.user_id)
        .await?
        .ok_or_else(|| AppError::unauthorized("invalid user"))?;

    let (access_token, expires_at) = issue_access_token(state, session.id, &user)?;

    Ok(ExchangeResponse {
        access_token,
        expires_at_unix: unix_ts(expires_at)?,
        refresh_token: exchange.refresh_token,
        redirect: exchange.redirect_to,
    })
}

pub async fn refresh(
    state: &AppState,
    headers: &HeaderMap,
    payload: &RefreshRequest,
) -> Result<RefreshResponse, AppError> {
    if is_https_url(&state.config.app_base_url) {
        validate_refresh_origin(headers, &state.config.frontend_origin)?;
    }

    if payload.refresh_token.is_empty() {
        return Err(AppError::unauthorized("missing refresh token"));
    }

    let incoming_hash = hash_token(&payload.refresh_token);
    let next_refresh_token = random_urlsafe(48);

    let Some(rotated) = repository::rotate_session_refresh_token(
        &state.db,
        &incoming_hash,
        &hash_token(&next_refresh_token),
        Utc::now() + ChronoDuration::seconds(REFRESH_SESSION_TTL.as_secs() as i64),
    )
    .await?
    else {
        if let Some(reused) = repository::find_session_by_previous_refresh_hash(&state.db, &incoming_hash).await? {
            repository::compromise_session(&state.db, reused.id).await?;
            return Err(AppError::unauthorized("refresh token reuse detected"));
        }
        return Err(AppError::unauthorized("invalid refresh token"));
    };

    let user = repository::get_user_by_id(&state.db, rotated.user_id)
        .await?
        .ok_or_else(|| AppError::unauthorized("invalid user"))?;

    let (access_token, expires_at) = issue_access_token(state, rotated.id, &user)?;
    Ok(RefreshResponse {
        access_token,
        expires_at_unix: unix_ts(expires_at)?,
        refresh_token: next_refresh_token,
    })
}

pub async fn me(state: &AppState, headers: &HeaderMap) -> Result<MeResponse, AppError> {
    let claims = decode_access_token(state, &bearer_from_headers(headers)?)?;
    let session_id = Uuid::parse_str(&claims.sid).map_err(|_| AppError::unauthorized("invalid session claim"))?;
    let session = repository::find_session_by_id(&state.db, session_id)
        .await?
        .ok_or_else(|| AppError::unauthorized("invalid session"))?;

    if !session_active(&session) {
        return Err(AppError::unauthorized("invalid session"));
    }

    let user = repository::get_user_by_id(&state.db, session.user_id)
        .await?
        .ok_or_else(|| AppError::unauthorized("invalid user"))?;

    Ok(MeResponse {
        provider: user.provider,
        email: user.email,
        roles: user.roles,
    })
}

pub async fn logout(
    state: &AppState,
    headers: &HeaderMap,
    refresh_token: Option<&str>,
) -> Result<(), AppError> {
    let refresh_hash = refresh_token
        .filter(|value| !value.is_empty())
        .map(hash_token);

    let session_id_from_bearer = bearer_from_headers(headers)
        .ok()
        .and_then(|token| decode_access_token(state, &token).ok())
        .and_then(|claims| Uuid::parse_str(&claims.sid).ok());

    let session_id_from_refresh = if let Some(hash) = refresh_hash {
        repository::find_session_by_refresh_hash(&state.db, &hash)
            .await?
            .map(|value| value.id)
    } else {
        None
    };

    if let Some(session_id) = session_id_from_bearer {
        repository::revoke_session(&state.db, session_id).await?;
    }
    if let Some(session_id) = session_id_from_refresh {
        repository::revoke_session(&state.db, session_id).await?;
    }

    Ok(())
}

fn session_active(session: &repository::DbSession) -> bool {
    if session.revoked_at.is_some() || session.compromised_at.is_some() {
        return false;
    }
    session.expires_at > Utc::now()
}

fn issue_access_token(
    state: &AppState,
    session_id: Uuid,
    user: &repository::DbUser,
) -> Result<(String, SystemTime), AppError> {
    let now_unix = unix_ts(SystemTime::now())?;
    let exp_unix = now_unix + ACCESS_TOKEN_TTL.as_secs();
    let claims = AccessTokenClaims {
        sub: user.id.to_string(),
        sid: session_id.to_string(),
        provider: user.provider.clone(),
        email: user.email.clone(),
        roles: user.roles.clone(),
        iat: now_unix as usize,
        exp: exp_unix as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.access_token_jwt_secret.as_bytes()),
    )
    .map_err(|_| AppError::internal("failed to sign access token"))?;

    let expires_at = SystemTime::UNIX_EPOCH + Duration::from_secs(exp_unix);
    Ok((token, expires_at))
}

fn sanitize_redirect_path(path: &str) -> String {
    if is_safe_redirect_path(path) {
        path.to_string()
    } else {
        "/events".to_string()
    }
}

fn is_safe_redirect_path(path: &str) -> bool {
    if !path.starts_with('/') || path.starts_with("//") || path.contains('\\') {
        return false;
    }
    !path.chars().any(|value| value.is_control())
}

fn validate_refresh_origin(headers: &HeaderMap, frontend_origin: &str) -> Result<(), AppError> {
    let origin = headers
        .get(header::ORIGIN)
        .ok_or_else(|| AppError::unauthorized("missing origin header"))?
        .to_str()
        .map_err(|_| AppError::unauthorized("invalid origin header"))?;

    if normalize_origin(origin) != normalize_origin(frontend_origin) {
        return Err(AppError::unauthorized("origin not allowed"));
    }

    Ok(())
}

fn normalize_origin(value: &str) -> &str {
    value.trim_end_matches('/')
}

fn random_urlsafe(byte_len: usize) -> String {
    let mut bytes = vec![0_u8; byte_len];
    OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn pkce_challenge(verifier: &str) -> String {
    let digest = Sha256::digest(verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(digest)
}

fn is_expired(from: SystemTime, ttl: Duration) -> bool {
    match from.elapsed() {
        Ok(elapsed) => elapsed > ttl,
        Err(_) => false,
    }
}

fn unix_ts(t: SystemTime) -> Result<u64, AppError> {
    t.duration_since(SystemTime::UNIX_EPOCH)
        .map(|value| value.as_secs())
        .map_err(|_| AppError::internal("invalid timestamp"))
}

fn url_encode(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

fn hash_token(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    URL_SAFE_NO_PAD.encode(digest)
}

fn bearer_from_headers(headers: &HeaderMap) -> Result<String, AppError> {
    let auth = headers
        .get(header::AUTHORIZATION)
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

fn is_admin_email(state: &AppState, email: &str) -> bool {
    let Some(raw) = &state.config.admin_emails else {
        return false;
    };

    let normalized = email.trim().to_lowercase();
    raw.split(',')
        .map(str::trim)
        .map(str::to_lowercase)
        .any(|value| value == normalized)
}

async fn google_exchange_and_email(
    state: &AppState,
    code: &str,
    code_verifier: Option<&str>,
) -> Result<String, AppError> {
    #[derive(Deserialize)]
    struct GoogleTokenResponse {
        access_token: String,
    }

    #[derive(Deserialize)]
    struct GoogleUserInfo {
        email: Option<String>,
        email_verified: Option<bool>,
    }

    let redirect_uri = format!("{}/auth/google/callback", state.config.app_base_url);
    let verifier = code_verifier.ok_or_else(|| AppError::internal("missing pkce verifier"))?;

    let token = state
        .http_client
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri.as_str()),
            ("client_id", state.config.google_client_id.as_str()),
            ("client_secret", state.config.google_client_secret.as_str()),
            ("code_verifier", verifier),
        ])
        .send()
        .await
        .map_err(|e| AppError::bad_gateway(format!("google token exchange failed: {e}")))?
        .error_for_status()
        .map_err(|e| AppError::bad_gateway(format!("google token exchange failed: {e}")))?
        .json::<GoogleTokenResponse>()
        .await
        .map_err(|e| AppError::bad_gateway(format!("invalid google token response: {e}")))?;

    let user = state
        .http_client
        .get("https://openidconnect.googleapis.com/v1/userinfo")
        .bearer_auth(token.access_token)
        .send()
        .await
        .map_err(|e| AppError::bad_gateway(format!("google userinfo request failed: {e}")))?
        .error_for_status()
        .map_err(|e| AppError::bad_gateway(format!("google userinfo request failed: {e}")))?
        .json::<GoogleUserInfo>()
        .await
        .map_err(|e| AppError::bad_gateway(format!("invalid google userinfo response: {e}")))?;

    let email = user
        .email
        .ok_or_else(|| AppError::bad_request("google did not return email"))?;

    if user.email_verified != Some(true) {
        return Err(AppError::bad_request("google email is not verified"));
    }

    Ok(email.to_lowercase())
}

async fn facebook_exchange_and_email(state: &AppState, code: &str) -> Result<String, AppError> {
    #[derive(Deserialize)]
    struct FacebookTokenResponse {
        access_token: String,
    }

    #[derive(Deserialize)]
    struct FacebookUser {
        email: Option<String>,
    }

    let redirect_uri = format!("{}/auth/facebook/callback", state.config.app_base_url);

    let token = state
        .http_client
        .get("https://graph.facebook.com/v20.0/oauth/access_token")
        .query(&[
            ("client_id", state.config.facebook_client_id.as_str()),
            ("client_secret", state.config.facebook_client_secret.as_str()),
            ("redirect_uri", redirect_uri.as_str()),
            ("code", code),
        ])
        .send()
        .await
        .map_err(|e| AppError::bad_gateway(format!("facebook token exchange failed: {e}")))?
        .error_for_status()
        .map_err(|e| AppError::bad_gateway(format!("facebook token exchange failed: {e}")))?
        .json::<FacebookTokenResponse>()
        .await
        .map_err(|e| AppError::bad_gateway(format!("invalid facebook token response: {e}")))?;

    let user = state
        .http_client
        .get("https://graph.facebook.com/me")
        .query(&[("fields", "id,email")])
        .bearer_auth(token.access_token)
        .send()
        .await
        .map_err(|e| AppError::bad_gateway(format!("facebook me request failed: {e}")))?
        .error_for_status()
        .map_err(|e| AppError::bad_gateway(format!("facebook me request failed: {e}")))?
        .json::<FacebookUser>()
        .await
        .map_err(|e| AppError::bad_gateway(format!("invalid facebook user response: {e}")))?;

    let email = user
        .email
        .ok_or_else(|| AppError::bad_request("facebook did not return email (scope not granted?)"))?;

    Ok(email.to_lowercase())
}
