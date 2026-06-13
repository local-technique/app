use axum::extract::State;
use axum::Json;

use crate::app::state::AppState;
use crate::common::auth::Principal;

pub async fn get_openapi_json(
    State(state): State<AppState>,
    _principal: Principal,
) -> Json<serde_json::Value> {
    Json(state.openapi_spec.clone())
}
