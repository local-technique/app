use axum::{
    Router,
    routing::{get, post},
};

use crate::app::state::AppState;
use crate::{admin, auth, incidents, maintenances, translations};

pub fn build(state: AppState, cors: tower_http::cors::CorsLayer) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/auth/{provider}/start", get(auth::http::start_oauth))
        .route("/auth/{provider}/callback", get(auth::http::oauth_callback))
        .route("/auth/exchange", post(auth::http::exchange_session))
        .route("/auth/refresh", post(auth::http::refresh))
        .route("/auth/logout", post(auth::http::logout))
        .route("/me", get(auth::http::me))
        .route("/admin/roles", get(admin::http::roles))
        .route("/admin/users", get(admin::http::users))
        .route("/admin/users/{user_id}/roles", axum::routing::put(admin::http::update_user_roles))
        .route("/incidents", get(incidents::http::list).post(incidents::http::create))
        .route(
            "/incidents/{id}",
            get(incidents::http::detail)
                .put(incidents::http::update)
                .delete(incidents::http::delete),
        )
        .route("/incidents/{id}/translations", get(incidents::http::translations))
        .route(
            "/incidents/{id}/translations/replace",
            post(incidents::http::replace_translations),
        )
        .route("/maintenances", get(maintenances::http::list).post(maintenances::http::create))
        .route(
            "/maintenances/{id}",
            get(maintenances::http::detail)
                .put(maintenances::http::update)
                .delete(maintenances::http::delete),
        )
        .route(
            "/maintenances/{id}/translations",
            get(maintenances::http::translations),
        )
        .route(
            "/maintenances/{id}/translations/replace",
            post(maintenances::http::replace_translations),
        )
        .route("/translations", get(translations::http::list_matrix))
        .route("/translations/bulk", post(translations::http::upsert_bulk))
        .layer(cors)
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}
