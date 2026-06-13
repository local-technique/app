use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;

use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use sqlx::PgPool;
use tokio::sync::Mutex;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub cookie_key: Key,
    pub http_client: reqwest::Client,
    pub db: PgPool,
    pub exchange_codes: Arc<Mutex<HashMap<String, ExchangeCodeRecord>>>,
    pub openapi_spec: serde_json::Value,
}

impl AppState {
    pub fn new(config: Config, cookie_key: Key, db: PgPool, openapi_spec: serde_json::Value) -> Self {
        Self {
            config,
            cookie_key,
            http_client: reqwest::Client::new(),
            db,
            exchange_codes: Arc::new(Mutex::new(HashMap::new())),
            openapi_spec,
        }
    }
}

impl FromRef<AppState> for Key {
    fn from_ref(input: &AppState) -> Self {
        input.cookie_key.clone()
    }
}

#[derive(Debug, Clone)]
pub struct ExchangeCodeRecord {
    pub session_id: uuid::Uuid,
    pub created_at: SystemTime,
    pub redirect_to: String,
    pub refresh_token: String,
}
