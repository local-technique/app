use sqlx::PgPool;
use uuid::Uuid;

use crate::attachments::model::{AttachmentItem, AttachmentUploadUser, CreateAttachmentResponse};
use crate::common::error::AppError;
use crate::s3::S3Signer;

fn sanitize_filename(name: &str) -> String {
    let sanitized: String = name.chars()
        .map(|c| if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '_' })
        .collect();
    if sanitized.len() > 200 {
        let ext = std::path::Path::new(&sanitized)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let base = &sanitized[..200.min(sanitized.len())];
        format!("{}.{}", base, ext)
    } else {
        sanitized
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn create(
    db: &PgPool,
    s3: &S3Signer,
    item_type: &str,
    item_id: &str,
    user_id: Uuid,
    file_name: String,
    mime_type: String,
    size_bytes: i64,
) -> Result<CreateAttachmentResponse, AppError> {
    let id = Uuid::new_v4();
    let storage_key = format!("{}/{}/{}-{}", item_type, item_id, id, sanitize_filename(&file_name));

    sqlx::query(
        "INSERT INTO attachments (id, item_type, item_id, file_name, mime_type, size_bytes, storage_key, uploaded_by_user_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
    )
    .bind(id)
    .bind(item_type)
    .bind(item_id)
    .bind(&file_name)
    .bind(&mime_type)
    .bind(size_bytes)
    .bind(&storage_key)
    .bind(user_id)
    .execute(db)
    .await?;

    let presigned_put_url = s3.generate_presigned_put_url(&storage_key, 3600);
    let url = s3.generate_presigned_get_url(&storage_key, 3600);
    let download_url = format!("/{}/{}/attachments/{}/download", item_type, item_id, id);

    Ok(CreateAttachmentResponse {
        attachment: AttachmentItem {
            id: id.to_string(),
            file_name,
            mime_type,
            size_bytes,
            url,
            download_url,
            uploaded_by: None,
            created_at: chrono::Utc::now().to_rfc3339(),
        },
        presigned_put_url,
    })
}

pub async fn list(
    db: &PgPool,
    s3: &S3Signer,
    item_type: &str,
    item_id: &str,
) -> Result<Vec<AttachmentItem>, AppError> {
    let rows = sqlx::query_as::<_, AttachmentRow>(
        r#"
        SELECT a.id, a.file_name, a.mime_type, a.size_bytes, a.storage_key, a.created_at,
               u.id as user_id, u.email, u.first_name, u.last_name
        FROM attachments a
        LEFT JOIN users u ON a.uploaded_by_user_id = u.id
        WHERE a.item_type = $1 AND a.item_id = $2 AND a.deleted_at IS NULL
        ORDER BY a.created_at DESC
        "#
    )
    .bind(item_type)
    .bind(item_id)
    .fetch_all(db)
    .await?;

    Ok(rows.into_iter().map(|row| {
        let url = s3.generate_presigned_get_url(&row.storage_key, 3600);
        AttachmentItem {
            id: row.id.to_string(),
            file_name: row.file_name,
            mime_type: row.mime_type,
            size_bytes: row.size_bytes,
            url,
            download_url: format!("/{}/{}/attachments/{}/download", item_type, item_id, row.id),
            uploaded_by: row.user_id.map(|uid| AttachmentUploadUser {
                id: uid.to_string(),
                email: row.email.unwrap_or_default(),
                first_name: row.first_name,
                last_name: row.last_name,
            }),
            created_at: row.created_at.to_rfc3339(),
        }
    }).collect())
}

pub async fn delete(
    db: &PgPool,
    item_type: &str,
    item_id: &str,
    attachment_id: &str,
    current_user_id: Uuid,
) -> Result<bool, AppError> {
    let attachment_uuid = Uuid::parse_str(attachment_id).map_err(|_| AppError::bad_request("invalid attachment id"))?;

    let row = sqlx::query_as::<_, AttachmentOwnerRow>(
        "SELECT uploaded_by_user_id, storage_key FROM attachments WHERE id = $1 AND item_type = $2 AND item_id = $3 AND deleted_at IS NULL"
    )
    .bind(attachment_uuid)
    .bind(item_type)
    .bind(item_id)
    .fetch_optional(db)
    .await?;

    match row {
        Some(r) => {
            let is_owner = r.uploaded_by_user_id == Some(current_user_id);
            if !is_owner {
                return Err(AppError::forbidden("you can only delete your own attachments"));
            }
            sqlx::query("UPDATE attachments SET deleted_at = now() WHERE id = $1")
                .bind(attachment_uuid)
                .execute(db)
                .await?;
            Ok(true)
        }
        None => Ok(false),
    }
}

pub async fn delete_as_admin(
    db: &PgPool,
    item_type: &str,
    item_id: &str,
    attachment_id: &str,
) -> Result<bool, AppError> {
    let attachment_uuid = Uuid::parse_str(attachment_id).map_err(|_| AppError::bad_request("invalid attachment id"))?;

    let result = sqlx::query("UPDATE attachments SET deleted_at = now() WHERE id = $1 AND item_type = $2 AND item_id = $3 AND deleted_at IS NULL")
        .bind(attachment_uuid)
        .bind(item_type)
        .bind(item_id)
        .execute(db)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn get_storage_key(
    db: &PgPool,
    attachment_id: &str,
) -> Result<Option<String>, AppError> {
    let id = Uuid::parse_str(attachment_id).map_err(|_| AppError::bad_request("invalid attachment id"))?;
    let row = sqlx::query_scalar::<_, String>(
        "SELECT storage_key FROM attachments WHERE id = $1 AND deleted_at IS NULL"
    )
    .bind(id)
    .fetch_optional(db)
    .await?;
    Ok(row)
}

#[derive(sqlx::FromRow)]
struct AttachmentRow {
    id: Uuid,
    file_name: String,
    mime_type: String,
    size_bytes: i64,
    storage_key: String,
    created_at: chrono::DateTime<chrono::Utc>,
    user_id: Option<Uuid>,
    email: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
}

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
struct AttachmentOwnerRow {
    uploaded_by_user_id: Option<Uuid>,
    storage_key: String,
}
