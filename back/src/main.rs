use std::net::SocketAddr;

use axum::{
    Router,
};
use axum_extra::extract::cookie::Key;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use http::{header::CONTENT_TYPE, HeaderValue, Method};
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod app;
mod admin;
mod auth;
mod categories;
mod common;
mod config;
mod incidents;
mod maintenances;
mod translations;

#[tokio::main]
async fn main() {
    let config = config::Config::from_env().expect("failed to load config from env");

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cors = CorsLayer::new()
        .allow_origin(
            config
                .frontend_origin
                .parse::<HeaderValue>()
                .expect("FRONTEND_ORIGIN must be a valid header value"),
        )
        .allow_credentials(true)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([http::header::AUTHORIZATION, CONTENT_TYPE]);

    let key_bytes = STANDARD
        .decode(config.cookie_key_base64.as_bytes())
        .expect("COOKIE_KEY_BASE64 must be valid base64");
    let cookie_key = Key::try_from(key_bytes.as_slice())
        .expect("COOKIE_KEY_BASE64 must decode to at least 64 bytes");

    let db = common::db::connect(&config.database_url)
        .await
        .expect("failed to connect to database");
    common::db::migrate(&db)
        .await
        .expect("failed to run database migrations");

    let state = app::state::AppState::new(config.clone(), cookie_key, db);
    let app: Router = app::router::build(state, cors);

    let addr = config
        .listen_addr
        .as_deref()
        .unwrap_or("0.0.0.0:8080")
        .parse::<SocketAddr>()
        .expect("LISTEN_ADDR must be a valid socket address");
    info!(%addr, "starting backend server");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind tcp listener");

    axum::serve(listener, app)
        .await
        .expect("backend server failed");
}
