use axum::{
    Router,
    routing::{get, post},
};

use crate::app::state::AppState;
use crate::{admin, api_tokens, auth, categories, incidents, maintenances, projects, translations};

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
        .route("/categories", get(categories::http::list))
        .route("/admin/categories", get(categories::http::admin_list).post(categories::http::create))
        .route(
            "/admin/categories/{id}",
            axum::routing::put(categories::http::update).delete(categories::http::delete),
        )
        .route("/incidents", get(incidents::http::list).post(incidents::http::create))
        .route(
            "/incidents/{id}",
            get(incidents::http::detail)
                .put(incidents::http::update)
                .delete(incidents::http::delete),
        )
        .route("/incidents/{id}/translations", get(incidents::http::translations))
        .route("/incidents/{id}/edit", get(incidents::http::edit))
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
        .route("/maintenances/{id}/edit", get(maintenances::http::edit))
        .route(
            "/maintenances/{id}/translations/replace",
            post(maintenances::http::replace_translations),
        )
        .route("/projects", get(projects::http::list).post(projects::http::create))
        .route(
            "/projects/{id}",
            get(projects::http::detail)
                .put(projects::http::update)
                .delete(projects::http::delete),
        )
        .route("/projects/{id}/translations", get(projects::http::translations))
        .route("/projects/{id}/edit", get(projects::http::edit))
        .route(
            "/projects/{id}/translations/replace",
            post(projects::http::replace_translations),
        )
        .route("/translations", get(translations::http::list_matrix))
        .route("/translations/bulk", post(translations::http::upsert_bulk))
        .route("/settings/token", post(api_tokens::http::create_token))
        .route("/settings/token", get(api_tokens::http::get_token).delete(api_tokens::http::revoke_token))
        .layer(cors)
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}
