# File Attachments Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Support file upload, download, and visualization (images + PDFs) on item detail pages using Cloudflare R2 (production) or MinIO (local dev).

**Architecture:** A single `attachments` DB table tracks file metadata with soft-delete. A custom S3 signing module (AWS SigV4 via `reqwest` + `hmac`) generates presigned URLs so the browser uploads/downloads files directly to/from R2/MinIO — the backend never proxies file bytes. Each item type (incidents, maintenances, projects) gets a set of attachment routes. The frontend adds a reusable upload widget (file picker, camera capture, drag-drop zone) to each DetailPage and wires real attachment data from the API.

**Tech Stack:** Rust (Axum), `reqwest` + `hmac` + `hex` for S3 signing, Vue 3, MinIO (local dev), Cloudflare R2 (prod)

---

### Task 1: Add MinIO service to docker-compose.yml

**Files:**
- Modify: `docker-compose.yml`

- [ ] **Step 1: Add MinIO service**

Add a `minio` service after the existing `postgres` service:

```yaml
  minio:
    image: minio/minio:latest
    container_name: minio
    ports:
      - 9000:9000
      - 9001:9001
    environment:
      MINIO_ROOT_USER: minioadmin
      MINIO_ROOT_PASSWORD: minioadmin
    command: server /data --console-address ":9001"
    volumes:
      - ./minio-data:/data
```

- [ ] **Step 2: Add bucket creation on startup**

Add a `minio-create-bucket` service that runs once and exits:

```yaml
  minio-create-bucket:
    image: minio/mc:latest
    depends_on:
      - minio
    entrypoint: >
      /bin/sh -c "
      sleep 3;
      mc alias set local http://minio:9000 minioadmin minioadmin;
      mc mb local/attachments --ignore-existing;
      echo 'Bucket created';
      "
```

- [ ] **Step 3: Add .gitignore for minio-data**

```bash
echo "minio-data/" >> .gitignore
```

- [ ] **Step 4: Verify docker-compose config**

Run: `docker-compose config` (current directory)
Expected: no errors, MinIO services appear in output

---

### Task 2: Backend — add Cargo dependencies

**Files:**
- Modify: `back/Cargo.toml`

- [ ] **Step 1: Add hmac and hex crates**

Add these lines after the existing `base64 = "0.22"` line:

```toml
hex = "0.4"
hmac = "0.12"
```

---

### Task 3: Backend — create DB migration for attachments table

**Files:**
- Create: `back/migrations/0024_attachments.sql`

- [ ] **Step 1: Write the migration**

```sql
CREATE TABLE attachments (
  id          UUID PRIMARY KEY,
  item_type   TEXT NOT NULL CHECK (item_type IN ('incident', 'maintenance', 'project')),
  item_id     UUID NOT NULL,
  file_name   TEXT NOT NULL,
  mime_type   TEXT NOT NULL,
  size_bytes  BIGINT NOT NULL,
  storage_key TEXT NOT NULL,
  uploaded_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
  created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at  TIMESTAMPTZ
);

CREATE INDEX attachments_active_idx ON attachments(item_type, item_id) WHERE deleted_at IS NULL;
```

---

### Task 4: Backend — add R2 config fields

**Files:**
- Modify: `back/src/config.rs`

- [ ] **Step 1: Add R2 fields to Config struct**

```rust
    pub r2_endpoint: String,
    pub r2_attachments_bucket: String,
    pub r2_access_key_id: String,
    pub r2_secret_access_key: String,
```

The struct should now look like this (only the R2 additions shown, keep existing fields):

```rust
#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub app_base_url: String,
    pub frontend_origin: String,
    pub frontend_base_url: Option<String>,
    pub google_client_id: String,
    pub google_client_secret: String,
    pub microsoft_client_id: String,
    pub microsoft_client_secret: String,
    pub cookie_key_base64: String,
    pub access_token_jwt_secret: String,
    pub database_url: String,
    pub admin_emails: Option<String>,
    pub listen_addr: Option<String>,
    pub r2_endpoint: String,
    pub r2_attachments_bucket: String,
    pub r2_access_key_id: String,
    pub r2_secret_access_key: String,
}
```

---

### Task 5: Backend — implement S3 signing module

**Files:**
- Create: `back/src/s3.rs`

- [ ] **Step 1: Write the S3 signing module**

```rust
use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::time::SystemTime;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct S3Signer {
    endpoint: String,
    bucket: String,
    access_key: String,
    secret_key: String,
    region: String,
}

impl S3Signer {
    pub fn new(endpoint: String, bucket: String, access_key: String, secret_key: String) -> Self {
        Self {
            endpoint,
            bucket,
            access_key,
            secret_key,
            region: "auto".to_string(),
        }
    }

    pub fn generate_presigned_get_url(&self, storage_key: &str, expires_secs: u64) -> String {
        let expires = expires_secs.to_string();
        let now = Utc::now();
        let date_str = now.format("%Y%m%d").to_string();
        let date_time_str = now.format("%Y%m%dT%H%M%SZ").to_string();

        let resource = format!("/{}/{}", self.bucket, storage_key);
        let host = format!("{}/{}", self.endpoint.trim_end_matches('/'), self.bucket);

        let mut headers = vec![
            ("host", &self.host_name()),
        ];

        let signed_headers = "host";
        let query_params = format!(
            "X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential={}%2F{}%2F{}%2Fs3%2Faws4_request&X-Amz-Date={}&X-Amz-Expires={}&X-Amz-SignedHeaders={}",
            &self.access_key, &date_str, &self.region, &date_time_str, &expires, signed_headers
        );

        let canonical_request = format!(
            "GET\n/{}\n{}\n{}\n\n{}\nUNSIGNED-PAYLOAD",
            storage_key, query_params, self._canonical_headers(&headers), signed_headers
        );

        let algorithm = "AWS4-HMAC-SHA256";
        let credential_scope = format!("{}/{}/s3/aws4_request", date_str, self.region);
        let string_to_sign = format!(
            "{}\n{}\n{}\n{}",
            algorithm,
            date_time_str,
            credential_scope,
            hex::encode(sha2::Sha256::digest(canonical_request.as_bytes()))
        );

        let signing_key = self._signing_key(&date_str);
        let signature = hex::encode(self._hmac(&signing_key, string_to_sign.as_bytes()));

        format!(
            "{}/{}?{}&X-Amz-Signature={}",
            self.endpoint.trim_end_matches('/'),
            storage_key,
            query_params,
            signature
        )
    }

    pub fn generate_presigned_put_url(&self, storage_key: &str, expires_secs: u64) -> String {
        let expires = expires_secs.to_string();
        let now = Utc::now();
        let date_str = now.format("%Y%m%d").to_string();
        let date_time_str = now.format("%Y%m%dT%H%M%SZ").to_string();

        let mut headers = vec![
            ("host", &self.host_name()),
        ];

        let signed_headers = "host";
        let query_params = format!(
            "X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential={}%2F{}%2F{}%2Fs3%2Faws4_request&X-Amz-Date={}&X-Amz-Expires={}&X-Amz-SignedHeaders={}",
            &self.access_key, &date_str, &self.region, &date_time_str, &expires, signed_headers
        );

        let canonical_request = format!(
            "PUT\n/{}\n{}\n{}\n\n{}\nUNSIGNED-PAYLOAD",
            storage_key, query_params, self._canonical_headers(&headers), signed_headers
        );

        let algorithm = "AWS4-HMAC-SHA256";
        let credential_scope = format!("{}/{}/s3/aws4_request", date_str, self.region);
        let string_to_sign = format!(
            "{}\n{}\n{}\n{}",
            algorithm,
            date_time_str,
            credential_scope,
            hex::encode(sha2::Sha256::digest(canonical_request.as_bytes()))
        );

        let signing_key = self._signing_key(&date_str);
        let signature = hex::encode(self._hmac(&signing_key, string_to_sign.as_bytes()));

        format!(
            "{}/{}?{}&X-Amz-Signature={}",
            self.endpoint.trim_end_matches('/'),
            storage_key,
            query_params,
            signature
        )
    }

    pub async fn delete_object(&self, storage_key: &str) -> Result<(), String> {
        let url = format!("{}/{}", self.endpoint.trim_end_matches('/'), storage_key);
        let now = Utc::now();
        let date_str = now.format("%Y%m%d").to_string();
        let date_time_str = now.format("%Y%m%dT%H%M%SZ").to_string();

        let mut headers = vec![
            ("host", &self.host_name()),
        ];

        let signed_headers = "host";
        let canonical_request = format!(
            "DELETE\n/{}\n\n{}\n\n{}\nUNSIGNED-PAYLOAD",
            storage_key, self._canonical_headers(&headers), signed_headers
        );

        let credential_scope = format!("{}/{}/s3/aws4_request", date_str, self.region);
        let string_to_sign = format!(
            "AWS4-HMAC-SHA256\n{}\n{}\n{}",
            date_time_str,
            credential_scope,
            hex::encode(sha2::Sha256::digest(canonical_request.as_bytes()))
        );

        let signing_key = self._signing_key(&date_str);
        let signature = hex::encode(self._hmac(&signing_key, string_to_sign.as_bytes()));

        let auth_header = format!(
            "AWS4-HMAC-SHA256 Credential={}/{}/s3/aws4_request, SignedHeaders={}, Signature={}",
            self.access_key, credential_scope, signed_headers, signature
        );

        let client = reqwest::Client::new();
        let response = client
            .delete(&url)
            .header("Authorization", &auth_header)
            .header("x-amz-date", &date_time_str)
            .send()
            .await
            .map_err(|e| format!("delete request failed: {}", e))?;

        if response.status().is_success() || response.status() == reqwest::StatusCode::NO_CONTENT {
            Ok(())
        } else {
            Err(format!("delete failed with status: {}", response.status()))
        }
    }

    fn host_name(&self) -> String {
        self.endpoint
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_end_matches('/')
            .to_string()
    }

    fn _canonical_headers(&self, headers: &[(&str, &str)]) -> String {
        headers.iter()
            .map(|(k, v)| format!("{}:{}", k.to_lowercase(), v.trim()))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn _hmac(&self, key: &[u8], data: &[u8]) -> Vec<u8> {
        let mut mac = HmacSha256::new_from_slice(key).expect("HMAC key");
        mac.update(data);
        mac.finalize().into_bytes().to_vec()
    }

    fn _signing_key(&self, date_str: &str) -> Vec<u8> {
        let k_secret = format!("AWS4{}", self.secret_key);
        let k_date = self._hmac(k_secret.as_bytes(), date_str.as_bytes());
        let k_region = self._hmac(&k_date, self.region.as_bytes());
        let k_service = self._hmac(&k_region, b"s3");
        self._hmac(&k_service, b"aws4_request")
    }
}
```

- [ ] **Step 2: Register module in main.rs**

Add to the module declarations (in alphabetical order):

```rust
mod s3;
```

---

### Task 6: Backend — create attachments module

**Files:**
- Create: `back/src/attachments/mod.rs`
- Create: `back/src/attachments/model.rs`
- Create: `back/src/attachments/service.rs`
- Create: `back/src/attachments/http.rs`
- Modify: `back/src/main.rs`

- [ ] **Step 1: Create mod.rs**

```rust
pub mod http;
pub mod model;
pub mod service;
```

- [ ] **Step 2: Create model.rs**

```rust
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
```

- [ ] **Step 3: Create service.rs**

```rust
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
    .bind(Uuid::parse_str(item_id).map_err(|_| AppError::bad_request("invalid item id"))?)
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
    .bind(Uuid::parse_str(item_id).map_err(|_| AppError::bad_request("invalid item id"))?)
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
    let item_uuid = Uuid::parse_str(item_id).map_err(|_| AppError::bad_request("invalid item id"))?;
    let attachment_uuid = Uuid::parse_str(attachment_id).map_err(|_| AppError::bad_request("invalid attachment id"))?;

    let row = sqlx::query_as::<_, AttachmentOwnerRow>(
        "SELECT uploaded_by_user_id, storage_key FROM attachments WHERE id = $1 AND item_type = $2 AND item_id = $3 AND deleted_at IS NULL"
    )
    .bind(attachment_uuid)
    .bind(item_type)
    .bind(item_uuid)
    .fetch_optional(db)
    .await?;

    match row {
        Some(r) => {
            let is_owner = r.uploaded_by_user_id == Some(current_user_id);
            if !is_owner {
                return Err(AppError::forbidden("you can only delete your own attachments unless admin"));
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
    let item_uuid = Uuid::parse_str(item_id).map_err(|_| AppError::bad_request("invalid item id"))?;
    let attachment_uuid = Uuid::parse_str(attachment_id).map_err(|_| AppError::bad_request("invalid attachment id"))?;

    let result = sqlx::query("UPDATE attachments SET deleted_at = now() WHERE id = $1 AND item_type = $2 AND item_id = $3 AND deleted_at IS NULL")
        .bind(attachment_uuid)
        .bind(item_type)
        .bind(item_uuid)
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

struct AttachmentOwnerRow {
    uploaded_by_user_id: Option<Uuid>,
    storage_key: String,
}
```

- [ ] **Step 4: Create http.rs**

```rust
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect};
use axum::Json;
use uuid::Uuid;

use crate::app::state::AppState;
use crate::attachments::model::{AttachmentItem, CreateAttachmentRequest, CreateAttachmentResponse};
use crate::attachments::service;
use crate::common::auth::Principal;
use crate::common::error::AppError;
use crate::common::role::Role;

pub async fn create(
    principal: Principal,
    State(state): State<AppState>,
    Path((item_type, item_id)): Path<(String, String)>,
    Json(payload): Json<CreateAttachmentRequest>,
) -> Result<(StatusCode, Json<CreateAttachmentResponse>), AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwnershipBoard, Role::CoOwnershipBoardOps])?;

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
        &item_type,
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
    let items = service::list(&state.db, &state.s3, &item_type, &item_id).await?;
    Ok(Json(items))
}

pub async fn delete(
    principal: Principal,
    State(state): State<AppState>,
    Path((item_type, item_id, attachment_id)): Path<(String, String, String)>,
) -> Result<StatusCode, AppError> {
    let is_admin = principal
        .ensure_any_role(&[Role::Admin, Role::CoOwnershipBoardOps])
        .is_ok();
    let deleted = if is_admin {
        service::delete_as_admin(&state.db, &item_type, &item_id, &attachment_id).await?
    } else {
        principal.ensure_any_role(&[Role::Admin, Role::CoOwnershipBoard, Role::CoOwnershipBoardOps])?;
        service::delete(&state.db, &item_type, &item_id, &attachment_id, principal.user_id).await?
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
    Path((item_type, item_id, attachment_id)): Path<(String, String, String)>,
) -> Result<impl IntoResponse, AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwner, Role::CoOwnershipBoard, Role::CoOwnershipBoardOps])?;

    let storage_key = service::get_storage_key(&state.db, &attachment_id)
        .await?
        .ok_or_else(|| AppError::not_found("attachment not found"))?;

    let url = state.s3.generate_presigned_get_url(&storage_key, 3600);
    Ok(Redirect::to(&url))
}
```

- [ ] **Step 5: Register attachments module in main.rs**

Add to module declarations (in alphabetical order after `api_tokens`):

```rust
mod attachments;
```

---

### Task 7: Backend — update AppState with S3Client

**Files:**
- Modify: `back/src/app/state.rs`
- Modify: `back/src/main.rs`

- [ ] **Step 1: Add s3 field to AppState**

Add the import and field:

```rust
use crate::s3::S3Signer;

// Inside AppState struct, after openapi_spec:
    pub s3: S3Signer,
```

- [ ] **Step 2: Update AppState::new**

Add `s3` parameter and field assignment. The new signature:

```rust
    pub fn new(config: Config, cookie_key: Key, db: PgPool, openapi_spec: serde_json::Value, s3: S3Signer) -> Self {
```

And in the body:

```rust
            s3,
```

- [ ] **Step 3: Initialize S3Signer in main.rs**

After `openapi_spec` creation and before `AppState::new`, add:

```rust
    let s3 = S3Signer::new(
        config.r2_endpoint.clone(),
        config.r2_attachments_bucket.clone(),
        config.r2_access_key_id.clone(),
        config.r2_secret_access_key.clone(),
    );
```

Then pass `s3` to `AppState::new`:

```rust
    let state = app::state::AppState::new(config.clone(), cookie_key, db, openapi_spec, s3);
```

---

### Task 8: Backend — register routes and update detail responses

**Files:**
- Modify: `back/src/app/router.rs`
- Modify: `back/src/incidents/model.rs`
- Modify: `back/src/maintenances/model.rs`
- Modify: `back/src/projects/model.rs`

- [ ] **Step 1: Add import for attachments http module**

At the top of `router.rs`, add `attachments` to the use crate line:

```rust
use crate::{admin, api_tokens, attachments, auth, categories, incidents, maintenances, projects, translations};
```

- [ ] **Step 2: Register attachment routes for each item type**

After the translations/replace route for incidents (after line 49), add:

```rust
        .route(
            "/incidents/{id}/attachments",
            get(attachments::http::list).post(attachments::http::create),
        )
        .route(
            "/incidents/{id}/attachments/{attachment_id}",
            axum::routing::delete(attachments::http::delete),
        )
        .route(
            "/incidents/{id}/attachments/{attachment_id}/download",
            get(attachments::http::download),
        )
```

After the translations/replace route for maintenances (after line 74), add:

```rust
        .route(
            "/maintenances/{id}/attachments",
            get(attachments::http::list).post(attachments::http::create),
        )
        .route(
            "/maintenances/{id}/attachments/{attachment_id}",
            axum::routing::delete(attachments::http::delete),
        )
        .route(
            "/maintenances/{id}/attachments/{attachment_id}/download",
            get(attachments::http::download),
        )
```

After the translations/replace route for projects (after line 96), add:

```rust
        .route(
            "/projects/{id}/attachments",
            get(attachments::http::list).post(attachments::http::create),
        )
        .route(
            "/projects/{id}/attachments/{attachment_id}",
            axum::routing::delete(attachments::http::delete),
        )
        .route(
            "/projects/{id}/attachments/{attachment_id}/download",
            get(attachments::http::download),
        )
```

- [ ] **Step 3: Add attachments field to IncidentDetail**

In `back/src/incidents/model.rs`, add to `IncidentDetail` struct:

```rust
    pub attachments: Vec<crate::attachments::model::AttachmentItem>,
```

- [ ] **Step 4: Add attachments field to MaintenanceDetail**

In `back/src/maintenances/model.rs`, locate `MaintenanceDetail` and add:

```rust
    pub attachments: Vec<crate::attachments::model::AttachmentItem>,
```

- [ ] **Step 5: Add attachments field to ProjectDetail**

In `back/src/projects/model.rs`, locate `ProjectDetail` and add:

```rust
    pub attachments: Vec<crate::attachments::model::AttachmentItem>,
```

- [ ] **Step 6: Update detail service functions to populate attachments**

In each service's `by_id` function (incidents, maintenances, projects), after building the detail struct and before returning it, add:

```rust
    let attachments = crate::attachments::service::list(&db, &s3, "incident", &id).await?;
    detail.attachments = attachments;
```

(Replace `"incident"` with `"maintenance"` or `"project"` as appropriate.)

Each detail service function needs access to the `S3Signer`. It must be passed from the http handler through the service. For the incidents service, the function signature changes from:

```rust
pub async fn by_id(db: &PgPool, id: &str, locale: Option<&str>) -> Result<Option<IncidentDetail>, AppError> {
```

To:

```rust
pub async fn by_id(db: &PgPool, s3: &S3Signer, id: &str, locale: Option<&str>) -> Result<Option<IncidentDetail>, AppError> {
```

And the http handler changes from:

```rust
    let Some(value) = service::by_id(&state.db, &id, query.locale.as_deref()).await? else {
```

To:

```rust
    let Some(value) = service::by_id(&state.db, &state.s3, &id, query.locale.as_deref()).await? else {
```

Same pattern for maintenances and projects.

---

### Task 9: Frontend — add downloadUrl to AttachmentItem type

**Files:**
- Modify: `front/src/common/attachments.ts`

- [ ] **Step 1: Add downloadUrl and uploadedBy fields**

```ts
export type AttachmentItem = {
  id: string;
  fileName: string;
  mimeType: string;
  sizeBytes: number;
  url: string;
  downloadUrl: string;
  uploadedBy?: { id: string; email: string } | null;
};
```

---

### Task 10: Frontend — create AttachmentUpload component

**Files:**
- Create: `front/src/common/components/AttachmentUpload.vue`

- [ ] **Step 1: Write the component**

```vue
<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { Upload, Camera } from "@lucide/vue";
import { authenticatedFetch } from "../../auth/authenticatedFetch";

const props = defineProps<{
  uploadEndpoint: string;
}>();

const emit = defineEmits<{
  (event: "uploaded"): void;
  (event: "error", message: string): void;
}>();

const { t } = useI18n();
const uploading = ref(false);
const dragOver = ref(false);

function isAllowed(file: File): boolean {
  const isImage = file.type.startsWith("image/");
  const isPdf = file.type === "application/pdf";
  if (!isImage && !isPdf) {
    emit("error", t("labels.attachmentTypeError"));
    return false;
  }
  if (file.size > 10 * 1024 * 1024) {
    emit("error", t("labels.attachmentSizeError"));
    return false;
  }
  return true;
}

async function uploadFile(file: File): Promise<void> {
  if (!isAllowed(file)) return;
  uploading.value = true;
  try {
    // Step 1: POST metadata -> get presigned PUT URL
    const createResp = await authenticatedFetch(props.uploadEndpoint, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        file_name: file.name,
        mime_type: file.type,
        size_bytes: file.size,
      }),
    });
    if (!createResp.ok) {
      const err = await createResp.json().catch(() => ({ error: "upload failed" }));
      emit("error", err.error ?? "upload failed");
      return;
    }
    const { presigned_put_url } = await createResp.json();

    // Step 2: PUT file bytes directly to R2/MinIO
    const putResp = await fetch(presigned_put_url, {
      method: "PUT",
      body: file,
    });
    if (!putResp.ok) {
      emit("error", "file upload to storage failed");
      return;
    }

    emit("uploaded");
  } finally {
    uploading.value = false;
  }
}

function handleFilePick(event: Event): void {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (file) uploadFile(file);
  input.value = "";
}

function handleDrop(event: DragEvent): void {
  dragOver.value = false;
  const file = event.dataTransfer?.files?.[0];
  if (file) uploadFile(file);
}

function handleDragOver(event: DragEvent): void {
  event.preventDefault();
  dragOver.value = true;
}

function handleDragLeave(): void {
  dragOver.value = false;
}
</script>

<template>
  <section class="upload-zone" :class="{ 'drag-over': dragOver }" @dragover="handleDragOver" @dragleave="handleDragLeave" @drop.prevent="handleDrop">
    <div class="upload-controls">
      <label class="upload-button secondary-button">
        <Upload :size="16" />
        <span>{{ t("labels.uploadFile") }}</span>
        <input type="file" accept="image/*,application/pdf" hidden @change="handleFilePick" />
      </label>
      <label class="upload-button secondary-button camera-button">
        <Camera :size="16" />
        <input type="file" accept="image/*" capture="environment" hidden @change="handleFilePick" />
      </label>
    </div>
    <p v-if="!dragOver" class="drop-hint">{{ t("labels.dropFilesHere") }}</p>
    <p v-else class="drop-hint drop-active">{{ t("labels.dropToUpload") }}</p>
    <p v-if="uploading" class="uploading-indicator">{{ t("labels.uploading") }}...</p>
  </section>
</template>

<style scoped>
.upload-zone {
  margin-top: 0.9rem;
  padding: 1rem;
  border: 2px dashed var(--border-color);
  border-radius: 0.8rem;
  text-align: center;
  transition: border-color 0.2s, background-color 0.2s;
}
.upload-zone.drag-over {
  border-color: var(--accent-color, #4a90d9);
  background: color-mix(in srgb, var(--accent-color, #4a90d9) 8%, transparent);
}
.upload-controls {
  display: flex;
  gap: 0.5rem;
  justify-content: center;
  margin-bottom: 0.5rem;
}
.upload-button {
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
}
.drop-hint {
  margin: 0;
  font-size: 0.85rem;
  color: var(--muted-fg);
}
.drop-active {
  color: var(--accent-color, #4a90d9);
  font-weight: 600;
}
.uploading-indicator {
  margin: 0.5rem 0 0;
  font-style: italic;
}
</style>
```

- [ ] **Step 2: Add i18n labels**

In the i18n locale files, add these keys:
- `attachmentTypeError`: "Only images and PDFs are allowed" / "Seuls les images et PDFs sont autorisés"
- `attachmentSizeError`: "File size must be under 10 MB" / "La taille du fichier doit être inférieure à 10 Mo"
- `uploadFile`: "Upload file" / "Ajouter un fichier"
- `dropFilesHere`: "Drop files here" / "Déposer les fichiers ici"
- `dropToUpload`: "Release to upload" / "Relâcher pour uploader"
- `uploading`: "Uploading" / "Upload en cours"

---

### Task 11: Frontend — update API repositories to wire real attachment data

**Files:**
- Modify: `front/src/incidents/repositories/apiIncidentsRepository.ts`
- Modify: `front/src/events/repositories/apiEventsRepository.ts`
- Modify: `front/src/projects/repositories/apiProjectsRepository.ts`

- [ ] **Step 1: Add attachment API types**

In each API repository file, add the API shape for attachments:

```ts
type ApiAttachmentItem = {
  id: string;
  file_name: string;
  mime_type: string;
  size_bytes: number;
  url: string;
  download_url: string;
};
```

- [ ] **Step 2: Remove `attachments: []` hardcoding**

In each `toXxxItem` function, replace:

```ts
    attachments: [],
```

With:

```ts
    attachments: "attachments" in value
      ? (value.attachments ?? []).map(toAttachmentItem)
      : [],
```

- [ ] **Step 3: Add toAttachmentItem mapper**

```ts
function toAttachmentItem(item: ApiAttachmentItem): AttachmentItem {
  return {
    id: item.id,
    fileName: item.file_name,
    mimeType: item.mime_type,
    sizeBytes: item.size_bytes,
    url: item.url,
    downloadUrl: item.download_url,
    uploadedBy: item.uploaded_by,
  };
}
```

Also add `uploaded_by` to the API type:

```ts
type ApiAttachmentItem = {
  id: string;
  file_name: string;
  mime_type: string;
  size_bytes: number;
  url: string;
  download_url: string;
  uploaded_by?: { id: string; email: string } | null;
};
```

- [ ] **Step 4: Add ApiAttachmentItem to detail types**

Add attachments to the `ApiIncidentDetail` type:

```ts
type ApiIncidentDetail = {
  // ... existing fields ...
  attachments: ApiAttachmentItem[];
};
```

Same for `ApiMaintenanceDetail` and `ApiProjectDetail` (names may vary by module).

---

### Task 12: Frontend — update DetailPages with upload, delete, and project preview

**Files:**
- Modify: `front/src/common/components/AttachmentList.vue`
- Modify: `front/src/incidents/DetailPage.vue`
- Modify: `front/src/events/DetailPage.vue`
- Modify: `front/src/projects/DetailPage.vue`

- [ ] **Step 1: Update AttachmentList with delete button**

Add Trash2 import at the top of `<script setup>`:

```ts
import { Trash2 } from "@lucide/vue";
```

Add `currentUserId` and `isAdmin` props:

```ts
const props = defineProps<{
  items: AttachmentItem[];
  selectedId?: string;
  currentUserId?: string;
  isAdmin?: boolean;
}>();
```

Update the emit to include `delete`:

```ts
const emit = defineEmits<{
  (event: "select", attachment: AttachmentItem): void;
  (event: "delete", attachment: AttachmentItem): void;
}>();
```

Add `canDelete` function:

```ts
function canDelete(item: AttachmentItem): boolean {
  if (props.isAdmin) return true;
  if (!props.currentUserId) return false;
  const uploadedById = item.uploadedBy?.id;
  return uploadedById === props.currentUserId;
}
```

Add delete button inside the `<li>` in the template, after the filename button:

```vue
        <button
          v-if="canDelete(item)"
          class="delete-attachment"
          @click.stop="emit('delete', item)"
          :title="t('labels.delete')"
        >
          <Trash2 :size="14" />
        </button>
```

Add styling:

```css
.delete-attachment {
  background: none;
  border: none;
  cursor: pointer;
  color: var(--muted-fg);
  padding: 0.2rem;
}
.delete-attachment:hover {
  color: #dc2626;
}
```

- [ ] **Step 2: Add AttachmentUpload and delete handler to each DetailPage**

Import AttachmentUpload:

```ts
import AttachmentUpload from "../common/components/AttachmentUpload.vue";
```

Template — after AttachmentList and before AttachmentPreview for incidents/events:

```vue
    <AttachmentUpload
      :upload-endpoint="`${apiBaseUrl()}/incidents/${incidentId}/attachments`"
      @uploaded="loadIncident"
      @error="console.error"
    />
    <AttachmentList
      :items="model.raw.attachments"
      :selected-id="selectedAttachment?.id"
      :current-user-id="getCurrentUserId()"
      :is-admin="canEdit"
      @select="handleAttachmentSelect"
      @delete="handleAttachmentDelete"
    />
```

Add the delete handler:

```ts
async function handleAttachmentDelete(item: AttachmentItem): Promise<void> {
  try {
    const resp = await authenticatedFetch(
      `${apiBaseUrl()}/incidents/${encodeURIComponent(incidentId.value)}/attachments/${encodeURIComponent(item.id)}`,
      { method: "DELETE" },
    );
    if (resp.ok) await loadIncident();
  } catch {
    /* ignore */
  }
}
```

(Adjust `apiBaseUrl()`, endpoint path, and `loadIncident`/`loadEvent`/`loadProject` for each DetailPage.)

- [ ] **Step 3: Add `deleteAttachment` placeholder in API repositories** (for mock mode)

Add to each repository interface and implementation:

```ts
  async deleteAttachment(itemId: string, attachmentId: string): Promise<void> {
    if (useMockData()) return;
    await sendJson(`${apiBaseUrl()}/${itemType}/${encodeURIComponent(itemId)}/attachments/${encodeURIComponent(attachmentId)}`, "DELETE");
  }
```

Where `itemType` is `"incidents"`, `"maintenances"`, or `"projects"` respectively.

- [ ] **Step 4: Update Project DetailPage**

Add `selectedAttachmentId` ref, `selectedAttachment` computed, `AttachmentPreview` import + usage, and `@select` handler — matching the pattern in incidents DetailPage (lines 23, 66-73, 86-88, 155-160 of `front/src/incidents/DetailPage.vue`).

Also add `apiBaseUrl` import for use with AttachmentUpload's `upload-endpoint` prop:

```ts
function apiBaseUrl(): string {
  return import.meta.env.VITE_API_BASE_URL ?? "http://localhost:8080";
}
```

---

### Task 13: Frontend — update AttachmentPreview to use downloadUrl

**Files:**
- Modify: `front/src/common/components/AttachmentPreview.vue`

- [ ] **Step 1: Replace `attachment.url` with `attachment.downloadUrl` for open links**

In the template, change both "Open file" links:

For PDF (`<a :href="attachment.url">`):
```html
        <a :href="attachment.downloadUrl" target="_blank" rel="noopener">{{ t("labels.openFile") }}</a>
```

For the preview link section:
```html
    <p class="preview-link" v-if="previewKind !== 'pdf'">
      <a :href="attachment.downloadUrl" target="_blank" rel="noopener">{{ t("labels.openFile") }}</a>
    </p>
```

The inline preview (`<img :src="attachment.url">` and `<object :data="attachment.url">`) stays as `attachment.url` (the presigned GET URL).

---

### Task 14: Run CI checks

- [ ] **Step 1: Backend lint**

Run: `cargo clippy --all-features -- -D warnings` in `back/`
Expected: no warnings or errors

- [ ] **Step 2: Backend build**

Run: `cargo build --all-features` in `back/`
Expected: compiles successfully

- [ ] **Step 3: Frontend lint**

Run: `npm run lint` in `front/`
Expected: no errors

- [ ] **Step 4: Frontend build**

Run: `npm run build` in `front/`
Expected: builds successfully
