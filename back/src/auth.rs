use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use axum::extract::{FromRef, Path, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use axum::Json;
use axum_extra::extract::cookie::{Cookie, Key, SameSite};
use axum_extra::extract::PrivateCookieJar;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use cookie::time::Duration as CookieDuration;
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::config::Config;

const OAUTH_STATE_TTL: Duration = Duration::from_secs(10 * 60);
const ACCESS_TOKEN_TTL: Duration = Duration::from_secs(15 * 60);
const REFRESH_SESSION_TTL: Duration = Duration::from_secs(7 * 24 * 60 * 60);
const EXCHANGE_CODE_TTL: Duration = Duration::from_secs(2 * 60);

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub cookie_key: Key,
    pub http_client: reqwest::Client,
    sessions: std::sync::Arc<tokio::sync::Mutex<HashMap<String, Session>>>,
    access_index: std::sync::Arc<tokio::sync::Mutex<HashMap<String, AccessToken>>>,
    exchange_codes: std::sync::Arc<tokio::sync::Mutex<HashMap<String, ExchangeCode>>>,
}

impl AppState {
    pub fn new(config: Config, cookie_key: Key) -> Self {
        Self {
            config,
            cookie_key,
            http_client: reqwest::Client::new(),
            sessions: std::sync::Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            access_index: std::sync::Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            exchange_codes: std::sync::Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }
}

impl FromRef<AppState> for Key {
    fn from_ref(input: &AppState) -> Self {
        input.cookie_key.clone()
    }
}

#[derive(Debug, Clone)]
struct Session {
    provider: Provider,
    email: String,
    refresh_token: String,
    refresh_expires_at: SystemTime,
}

#[derive(Debug, Clone)]
struct AccessToken {
    session_id: String,
    expires_at: SystemTime,
}

#[derive(Debug, Clone)]
struct ExchangeCode {
    session_id: String,
    created_at: SystemTime,
    redirect_to: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum Provider {
    Google,
    Facebook,
}

impl Provider {
    fn parse(raw: &str) -> Result<Self, AppError> {
        match raw {
            "google" => Ok(Self::Google),
            "facebook" => Ok(Self::Facebook),
            _ => Err(AppError::bad_request("unsupported provider")),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Google => "google",
            Self::Facebook => "facebook",
        }
    }
}

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
        serde_json::from_str::<Self>(value)
            .map_err(|_| AppError::bad_request("invalid oauth state cookie"))
    }
}

fn oauth_cookie_name(provider: Provider) -> &'static str {
    match provider {
        Provider::Google => "oauth_google_state",
        Provider::Facebook => "oauth_facebook_state",
    }
}

fn refresh_cookie_name() -> &'static str {
    "refresh_token"
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

fn refresh_cookie(value: String, secure: bool) -> Cookie<'static> {
    let same_site = if secure {
        SameSite::None
    } else {
        SameSite::Lax
    };

    Cookie::build((refresh_cookie_name(), value))
        .path("/")
        .http_only(true)
        .same_site(same_site)
        .secure(secure)
        .max_age(CookieDuration::seconds(REFRESH_SESSION_TTL.as_secs() as i64))
        .build()
}

fn refresh_delete_cookie(secure: bool) -> Cookie<'static> {
    let same_site = if secure {
        SameSite::None
    } else {
        SameSite::Lax
    };

    Cookie::build((refresh_cookie_name(), ""))
        .path("/")
        .http_only(true)
        .same_site(same_site)
        .secure(secure)
        .max_age(CookieDuration::seconds(0))
        .build()
}

fn is_https_url(url: &str) -> bool {
    url.starts_with("https://")
}

#[derive(Deserialize)]
pub struct OAuthStartQuery {
    pub redirect: Option<String>,
}

#[derive(Deserialize)]
pub struct OAuthCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

#[derive(Deserialize)]
pub struct ExchangeRequest {
    pub code: String,
}

#[derive(Serialize)]
pub struct ExchangeResponse {
    pub access_token: String,
    pub expires_at_unix: u64,
    pub redirect: String,
}

#[derive(Serialize)]
pub struct RefreshResponse {
    pub access_token: String,
    pub expires_at_unix: u64,
}

#[derive(Serialize)]
pub struct MeResponse {
    pub provider: String,
    pub email: String,
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

pub async fn start_oauth(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Path(provider_raw): Path<String>,
    Query(query): Query<OAuthStartQuery>,
) -> Result<(PrivateCookieJar, Redirect), AppError> {
    let provider = Provider::parse(&provider_raw)?;
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

    let jar = jar.add(oauth_state_cookie(
        provider,
        cookie_payload.to_cookie_value()?,
        secure_cookie,
    ));

    Ok((jar, Redirect::temporary(&auth_url)))
}

pub async fn oauth_callback(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Path(provider_raw): Path<String>,
    Query(query): Query<OAuthCallbackQuery>,
) -> Result<(PrivateCookieJar, Redirect), AppError> {
    let provider = Provider::parse(&provider_raw)?;

    if let Some(error) = query.error {
        return Err(AppError::bad_request(format!("provider returned error: {error}")));
    }

    let auth_state = query
        .state
        .ok_or_else(|| AppError::bad_request("missing oauth state"))?;
    let code = query
        .code
        .ok_or_else(|| AppError::bad_request("missing oauth code"))?;

    let secure_cookie = is_https_url(&state.config.app_base_url);
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
        Provider::Google => {
            google_exchange_and_email(&state, &code, state_cookie.code_verifier.as_deref()).await?
        }
        Provider::Facebook => facebook_exchange_and_email(&state, &code).await?,
    };

    let session_id = random_urlsafe(32);
    let refresh_token = random_urlsafe(48);
    let refresh_expires_at = SystemTime::now() + REFRESH_SESSION_TTL;

    {
        let mut sessions = state.sessions.lock().await;
        sessions.retain(|_, session| !is_expired(session.refresh_expires_at, Duration::ZERO));
        sessions.insert(
            session_id.clone(),
            Session {
                provider,
                email,
                refresh_token: refresh_token.clone(),
                refresh_expires_at,
            },
        );
    }

    let exchange_code = random_urlsafe(24);
    {
        let mut codes = state.exchange_codes.lock().await;
        codes.retain(|_, value| !is_expired(value.created_at, EXCHANGE_CODE_TTL));
        codes.insert(
            exchange_code.clone(),
            ExchangeCode {
                session_id,
                created_at: SystemTime::now(),
                redirect_to: state_cookie.redirect_to,
            },
        );
    }

    let frontend_callback = format!(
        "{}/#/auth/callback?code={}",
        state.config.frontend_origin,
        url_encode(&exchange_code)
    );

    let jar = jar
        .remove(oauth_state_delete_cookie(provider, secure_cookie))
        .add(refresh_cookie(refresh_token, secure_cookie));

    Ok((jar, Redirect::to(&frontend_callback)))
}

pub async fn exchange_session(
    State(state): State<AppState>,
    Json(payload): Json<ExchangeRequest>,
) -> Result<impl IntoResponse, AppError> {
    let exchange = {
        let mut codes = state.exchange_codes.lock().await;
        let value = codes
            .remove(&payload.code)
            .ok_or_else(|| AppError::unauthorized("invalid exchange code"))?;
        if is_expired(value.created_at, EXCHANGE_CODE_TTL) {
            return Err(AppError::unauthorized("exchange code expired"));
        }
        value
    };

    {
        let sessions = state.sessions.lock().await;
        sessions
            .get(&exchange.session_id)
            .ok_or_else(|| AppError::unauthorized("invalid session"))?;
    }

    let (access_token, expires_at) = issue_access_token(&state, &exchange.session_id).await?;

    Ok((
        [(header::CACHE_CONTROL, "no-store")],
        Json(ExchangeResponse {
            access_token,
            expires_at_unix: unix_ts(expires_at)?,
            redirect: exchange.redirect_to,
        }),
    ))
}

pub async fn refresh(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    headers: HeaderMap,
) -> Result<(PrivateCookieJar, Json<RefreshResponse>), AppError> {
    if is_https_url(&state.config.app_base_url) {
        validate_refresh_origin(&headers, &state.config.frontend_origin)?;
    }

    let refresh_token = jar
        .get(refresh_cookie_name())
        .map(|c| c.value().to_string())
        .ok_or_else(|| AppError::unauthorized("missing refresh token"))?;

    let secure = is_https_url(&state.config.app_base_url);

    let (session_id, next_refresh_token) = {
        let mut sessions = state.sessions.lock().await;
        sessions.retain(|_, session| !is_expired(session.refresh_expires_at, Duration::ZERO));
        let session_id = sessions
            .iter()
            .find(|(_, session)| session.refresh_token == refresh_token)
            .map(|(id, _)| id.clone())
            .ok_or_else(|| AppError::unauthorized("invalid refresh token"))?;

        let next_refresh_token = random_urlsafe(48);
        if let Some(target) = sessions.get_mut(&session_id) {
            target.refresh_token = next_refresh_token.clone();
            target.refresh_expires_at = SystemTime::now() + REFRESH_SESSION_TTL;
        }

        (session_id, next_refresh_token)
    };

    let (access_token, expires_at) = issue_access_token(&state, &session_id).await?;
    let jar = jar.add(refresh_cookie(next_refresh_token, secure));

    Ok((
        jar,
        Json(RefreshResponse {
            access_token,
            expires_at_unix: unix_ts(expires_at)?,
        }),
    ))
}

pub async fn me(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<MeResponse>, AppError> {
    let session_id = validate_access_token(&state, &bearer_from_headers(&headers)?).await?;

    let session = {
        let mut sessions = state.sessions.lock().await;
        sessions.retain(|_, v| !is_expired(v.refresh_expires_at, Duration::ZERO));
        sessions
            .get(&session_id)
            .cloned()
            .ok_or_else(|| AppError::unauthorized("invalid session"))?
    };

    Ok(Json(MeResponse {
        provider: session.provider.as_str().to_string(),
        email: session.email,
    }))
}

pub async fn logout(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    headers: HeaderMap,
) -> Result<(PrivateCookieJar, StatusCode), AppError> {
    let refresh_from_cookie = jar.get(refresh_cookie_name()).map(|c| c.value().to_string());

    let session_id_from_bearer = match bearer_from_headers(&headers) {
        Ok(token) => validate_access_token(&state, &token).await.ok(),
        Err(_) => None,
    };

    {
        let mut sessions = state.sessions.lock().await;

        if let Some(session_id) = session_id_from_bearer.clone() {
            sessions.remove(&session_id);
        }

        if let Some(refresh_token) = refresh_from_cookie
            && let Some(session_id) = sessions
                .iter()
                .find(|(_, session)| session.refresh_token == refresh_token)
                .map(|(id, _)| id.clone())
        {
            sessions.remove(&session_id);
        }
    }

    {
        let mut access = state.access_index.lock().await;
        if let Some(session_id) = session_id_from_bearer {
            access.retain(|_, v| v.session_id != session_id);
        } else {
            access.retain(|_, v| !is_expired(v.expires_at, Duration::ZERO));
        }
    }

    let secure = is_https_url(&state.config.app_base_url);
    let jar = jar.remove(refresh_delete_cookie(secure));
    Ok((jar, StatusCode::NO_CONTENT))
}

async fn issue_access_token(state: &AppState, session_id: &str) -> Result<(String, SystemTime), AppError> {
    let token = random_urlsafe(48);
    let expires_at = SystemTime::now() + ACCESS_TOKEN_TTL;
    let mut access = state.access_index.lock().await;
    access.retain(|_, v| !is_expired(v.expires_at, Duration::ZERO));
    access.insert(
        token.clone(),
        AccessToken {
            session_id: session_id.to_string(),
            expires_at,
        },
    );
    Ok((token, expires_at))
}

async fn validate_access_token(state: &AppState, token: &str) -> Result<String, AppError> {
    let mut access = state.access_index.lock().await;
    access.retain(|_, v| !is_expired(v.expires_at, Duration::ZERO));
    access
        .get(token)
        .map(|v| v.session_id.clone())
        .ok_or_else(|| AppError::unauthorized("invalid or expired access token"))
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

fn sanitize_redirect_path(path: &str) -> String {
    if is_safe_redirect_path(path) {
        path.to_string()
    } else {
        "/events".to_string()
    }
}

fn is_safe_redirect_path(path: &str) -> bool {
    if !path.starts_with('/') {
        return false;
    }

    if path.starts_with("//") {
        return false;
    }

    if path.contains('\\') {
        return false;
    }

    !path.chars().any(|c| c.is_control())
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
        .map(|v| v.as_secs())
        .map_err(|_| AppError::internal("invalid timestamp"))
}

fn url_encode(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
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

pub struct AppError {
    status: StatusCode,
    message: String,
}

impl AppError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: message.into(),
        }
    }

    fn bad_gateway(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_GATEWAY,
            message: message.into(),
        }
    }

    fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.into(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = Json(ErrorBody {
            error: self.message,
        });
        (self.status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redirect_path_must_start_with_slash() {
        assert_eq!(sanitize_redirect_path("/events?q=abc"), "/events?q=abc");
        assert_eq!(sanitize_redirect_path("https://evil.com"), "/events");
        assert_eq!(sanitize_redirect_path("events"), "/events");
        assert_eq!(sanitize_redirect_path("//evil.example"), "/events");
        assert_eq!(sanitize_redirect_path("/\\evil"), "/events");
        assert_eq!(sanitize_redirect_path("/safe\n"), "/events");
    }

    #[test]
    fn expired_helper_works() {
        let past = SystemTime::now() - Duration::from_secs(10);
        assert!(is_expired(past, Duration::from_secs(1)));
        assert!(!is_expired(SystemTime::now(), Duration::from_secs(30)));
    }
}
