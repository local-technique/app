use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct AttachmentItem {
    pub id: String,
    pub file_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub url: String,
    pub download_url: String,
    pub uploaded_by: Option<AttachmentUploadUser>,
    pub created_at: String,
}

#[derive(Serialize, ToSchema)]
pub struct AttachmentUploadUser {
    pub id: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct CreateAttachmentResponse {
    pub attachment: AttachmentItem,
    pub presigned_put_url: String,
}

#[derive(serde::Deserialize, ToSchema)]
pub struct CreateAttachmentRequest {
    pub file_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
}
