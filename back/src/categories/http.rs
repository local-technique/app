use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;

use crate::app::state::AppState;
use crate::categories::model::{CategoryCreateRequest, CategoryListQuery, CategoryUpdateRequest};
use crate::categories::service;
use crate::common::auth::Principal;
use crate::common::error::AppError;
use crate::common::role::Role;

#[utoipa::path(
    get,
    path = "/categories",
    tag = "categories",
    security((),),
    params(CategoryListQuery),
    responses(
        (status = 200, description = "List of categories available to CO_OWNERSHIP_BOARD", body = Vec<crate::categories::model::CategoryItem>),
        (status = 403, description = "Forbidden — requires ADMIN or CO_OWNERSHIP_BOARD"),
    ),
    description = "List categories. Requires ADMIN or CO_OWNERSHIP_BOARD."
)]
pub async fn list(
    principal: Principal,
    State(state): State<AppState>,
    Query(query): Query<CategoryListQuery>,
) -> Result<Json<Vec<crate::categories::model::CategoryItem>>, AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwnershipBoard, Role::CoOwnershipBoardOps])?;
    Ok(Json(service::list(&state.db, query.locale.as_deref()).await?))
}

#[utoipa::path(
    get,
    path = "/admin/categories",
    tag = "categories",
    security((),),
    params(CategoryListQuery),
    responses(
        (status = 200, description = "Full category list for admins", body = Vec<crate::categories::model::CategoryItem>),
        (status = 403, description = "Forbidden — requires ADMIN"),
    ),
    description = "List all categories (admin view). Requires ADMIN."
)]
pub async fn admin_list(
    principal: Principal,
    State(state): State<AppState>,
    Query(query): Query<CategoryListQuery>,
) -> Result<Json<Vec<crate::categories::model::CategoryItem>>, AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwnershipBoardOps])?;
    Ok(Json(service::list(&state.db, query.locale.as_deref()).await?))
}

#[utoipa::path(
    post,
    path = "/admin/categories",
    tag = "categories",
    security((),),
    request_body = CategoryCreateRequest,
    responses(
        (status = 201, description = "Created category", body = crate::categories::model::CategoryItem),
        (status = 403, description = "Forbidden — requires ADMIN"),
    ),
    description = "Create a category. Requires ADMIN."
)]
pub async fn create(
    principal: Principal,
    State(state): State<AppState>,
    Json(payload): Json<CategoryCreateRequest>,
) -> Result<(StatusCode, Json<crate::categories::model::CategoryItem>), AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwnershipBoardOps])?;
    let value = service::create(&state.db, &payload).await?;
    Ok((StatusCode::CREATED, Json(value)))
}

#[utoipa::path(
    put,
    path = "/admin/categories/{id}",
    tag = "categories",
    security((),),
    params(
        ("id" = String, Path, description = "Category ID"),
    ),
    request_body = CategoryUpdateRequest,
    responses(
        (status = 200, description = "Updated category", body = crate::categories::model::CategoryItem),
        (status = 403, description = "Forbidden — requires ADMIN"),
    ),
    description = "Update a category. Requires ADMIN."
)]
pub async fn update(
    principal: Principal,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<CategoryUpdateRequest>,
) -> Result<Json<crate::categories::model::CategoryItem>, AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwnershipBoardOps])?;
    Ok(Json(service::update(&state.db, &id, &payload).await?))
}

#[utoipa::path(
    delete,
    path = "/admin/categories/{id}",
    tag = "categories",
    security((),),
    params(
        ("id" = String, Path, description = "Category ID"),
    ),
    responses(
        (status = 204, description = "Category deleted"),
        (status = 403, description = "Forbidden — requires ADMIN"),
    ),
    description = "Delete a category. Requires ADMIN."
)]
pub async fn delete(
    principal: Principal,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwnershipBoardOps])?;
    service::delete(&state.db, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}
