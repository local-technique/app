use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect};
use axum::Json;
use crate::app::state::AppState;
use crate::attachments::model::{AttachmentItem, CreateAttachmentRequest, CreateAttachmentResponse};
use crate::attachments::service;
use crate::common::auth::Principal;
use crate::common::error::AppError;
use crate::common::role::Role;

fn normalize_item_type(item_type: &str) -> Result<&str, AppError> {
    match item_type {
        "incidents" => Ok("incident"),
        "maintenances" => Ok("maintenance"),
        "projects" => Ok("project"),
        _ => Err(AppError::bad_request("invalid item type")),
    }
}

pub async fn create(
    principal: Principal,
    State(state): State<AppState>,
    Path((item_type, item_id)): Path<(String, String)>,
    Json(payload): Json<CreateAttachmentRequest>,
) -> Result<(StatusCode, Json<CreateAttachmentResponse>), AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwnershipBoard, Role::CoOwnershipBoardOps])?;
    let item_type = normalize_item_type(&item_type)?;

    if payload.size_bytes > 10 * 1024 * 1024 {
        return Err(AppError::bad_request("file size exceeds 10 MB limit"));
    }

    let is_image = payload.mime_type.starts_with("image/");
    let is_pdf = payload.mime_type == "application/pdf";
    if !is_image && !is_pdf {
        return Err(AppError::bad_request("only images and PDFs are allowed"));
    }

    let result = service::create(
        &state.db,
        &state.s3,
        item_type,
        &item_id,
        principal.user_id,
        payload.file_name,
        payload.mime_type,
        payload.size_bytes,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(result)))
}

pub async fn list(
    principal: Principal,
    State(state): State<AppState>,
    Path((item_type, item_id)): Path<(String, String)>,
) -> Result<Json<Vec<AttachmentItem>>, AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwner, Role::CoOwnershipBoard, Role::CoOwnershipBoardOps])?;
    let item_type = normalize_item_type(&item_type)?;
    let items = service::list(&state.db, &state.s3, item_type, &item_id).await?;
    Ok(Json(items))
}

pub async fn delete(
    principal: Principal,
    State(state): State<AppState>,
    Path((item_type, item_id, attachment_id)): Path<(String, String, String)>,
) -> Result<StatusCode, AppError> {
    let item_type = normalize_item_type(&item_type)?;
    let deleted = if principal.ensure_any_role(&[Role::Admin, Role::CoOwnershipBoardOps]).is_ok() {
        service::delete_as_admin(&state.db, item_type, &item_id, &attachment_id).await?
    } else {
        principal.ensure_any_role(&[Role::CoOwnershipBoard])?;
        service::delete(&state.db, item_type, &item_id, &attachment_id, principal.user_id).await?
    };

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found("attachment not found"))
    }
}

pub async fn download(
    principal: Principal,
    State(state): State<AppState>,
    Path((_item_type, _item_id, attachment_id)): Path<(String, String, String)>,
) -> Result<impl IntoResponse, AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwner, Role::CoOwnershipBoard, Role::CoOwnershipBoardOps])?;

    let storage_key = service::get_storage_key(&state.db, &attachment_id)
        .await?
        .ok_or_else(|| AppError::not_found("attachment not found"))?;

    let url = state.s3.generate_presigned_get_url(&storage_key, 3600);
    Ok(Redirect::to(&url))
}
