# OpenAPI Documentation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Serve an OpenAPI 3.1 spec from the Axum backend at `/openapi.json` and render it in the frontend using Scalar API Reference.

**Architecture:** Add `utoipa` to derive OpenAPI from handler annotations and model derives. Backend serves the spec JSON. Frontend fetches and renders it via Scalar's Vue component on a new `/settings/api-doc` route, linked below the API token panel.

**Tech Stack:** Rust `utoipa` (OpenAPI), Vue 3, `@scalar/api-reference` (Vue component)

---

## File Structure

### Backend — new files
- `back/src/api_doc/mod.rs` — `#[derive(OpenApi)]` struct referencing all paths/schemas/tags
- `back/src/api_doc/http.rs` — handler returning `Json<utoipa::openapi::OpenApi>`

### Backend — modified files
- `back/Cargo.toml` — add `utoipa`
- `back/src/main.rs` — add `mod api_doc;`
- `back/src/app/router.rs` — add `/openapi.json` route
- `back/src/auth/http.rs` — annotate 6 handlers
- `back/src/auth/model.rs` — derive `ToSchema`, `IntoParams` on types
- `back/src/admin/http.rs` — annotate 3 handlers
- `back/src/admin/model.rs` — derive `ToSchema`
- `back/src/categories/http.rs` — annotate 5 handlers
- `back/src/categories/model.rs` — derive `ToSchema`, `IntoParams`
- `back/src/incidents/http.rs` — annotate 8 handlers
- `back/src/incidents/model.rs` — derive `ToSchema`, `IntoParams`
- `back/src/maintenances/http.rs` — annotate 8 handlers
- `back/src/maintenances/model.rs` — derive `ToSchema`, `IntoParams`
- `back/src/projects/http.rs` — annotate 8 handlers
- `back/src/projects/model.rs` — derive `ToSchema`, `IntoParams`
- `back/src/translations/http.rs` — annotate 2 handlers
- `back/src/translations/model.rs` — derive `ToSchema`
- `back/src/api_tokens/http.rs` — annotate 3 handlers
- `back/src/api_tokens/model.rs` — derive `ToSchema`

### Frontend — new files
- `front/src/views/settings/ApiDocPage.vue` — Scalar component page

### Frontend — modified files
- `front/package.json` — add `@scalar/api-reference`
- `front/src/router/index.ts` — add route
- `front/src/views/settings/SettingsPage.vue` — add link
- `front/src/common/i18n.ts` — add translation keys

---

### Task 1: Add utoipa dependency and scaffolding

**Files:**
- Modify: `Cargo.toml`
- Create: `back/src/api_doc/mod.rs`
- Create: `back/src/api_doc/http.rs`
- Modify: `back/src/main.rs`
- Modify: `back/src/app/router.rs`

- [ ] **Step 1: Add utoipa to Cargo.toml**

Add after the existing dependencies:

```toml
utoipa = { version = "5", features = ["axum", "uuid", "chrono"] }
utoipauto = "0.1"
```

- [ ] **Step 2: Create `back/src/api_doc/mod.rs`**

```rust
mod http;

pub use http::*;
```

- [ ] **Step 3: Create `back/src/api_doc/http.rs`**

```rust
use axum::extract::State;
use axum::Json;

use crate::app::state::AppState;
use crate::common::auth::Principal;

pub async fn get_openapi_json(
    State(state): State<AppState>,
    principal: Principal,
) -> Json<utoipa::openapi::OpenApi> {
    Json(crate::ApiDoc::openapi())
}
```

- [ ] **Step 4: Register module in `back/src/main.rs`**

Add after `mod api_tokens;`:

```rust
mod api_doc;
```

- [ ] **Step 5: Add route in `back/src/app/router.rs`**

Add after the existing routes, before `.layer(cors)`:

```rust
        .route("/openapi.json", get(crate::api_doc::http::get_openapi_json))
```

- [ ] **Step 6: Verify compilation**

Run: `cd back && cargo check`
Expected: Compilation succeeds (warnings about unused imports are fine)

- [ ] **Step 7: Commit**

```
git add back/Cargo.toml back/Cargo.lock back/src/api_doc/ back/src/main.rs back/src/app/router.rs
git commit -m "feat(back): add utoipa scaffolding for OpenAPI spec"
```

---

### Task 2: Annotate auth handlers and models

**Files:**
- Modify: `back/src/auth/http.rs`
- Modify: `back/src/auth/model.rs`

- [ ] **Step 1: Add `#[derive(ToSchema)]` and `impl IntoParams` on auth model types**

Read `back/src/auth/model.rs` to find all public types. Add `ToSchema` to the derive list on:
- `ExchangeRequest` (request body)
- `ExchangeResponse` (response)
- `RefreshRequest` (request body)
- `RefreshResponse` (response)
- `MeResponse` (response)
- `LogoutRequest` (request body)

Add `IntoParams` to `OAuthStartQuery` and `OAuthCallbackQuery`.

For each struct, append `ToSchema` to the existing derive attribute. For example:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ExchangeRequest {
    pub code: String,
}
```

Add `IntoParams` to query structs:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, IntoParams)]
pub struct OAuthStartQuery {
    pub redirect: Option<String>,
}
```

- [ ] **Step 2: Annotate `start_oauth` handler**

Insert before the function:

```rust
#[utoipa::path(
    get,
    path = "/auth/{provider}/start",
    tag = "auth",
    params(
        ("provider" = String, Path, description = "OAuth provider (google)"),
        OAuthStartQuery,
    ),
    responses(
        (status = 302, description = "Redirect to OAuth provider"),
        (status = 400, description = "Unsupported provider"),
    ),
    description = "Start OAuth login flow for the given provider."
)]
```

- [ ] **Step 3: Annotate `oauth_callback` handler**

```rust
#[utoipa::path(
    get,
    path = "/auth/{provider}/callback",
    tag = "auth",
    params(
        ("provider" = String, Path, description = "OAuth provider (google)"),
        OAuthCallbackQuery,
    ),
    responses(
        (status = 302, description = "Redirect to frontend with session code"),
        (status = 400, description = "Unsupported provider"),
    ),
    description = "OAuth callback endpoint - completes OAuth flow."
)]
```

- [ ] **Step 4: Annotate `exchange_session` handler**

```rust
#[utoipa::path(
    post,
    path = "/auth/exchange",
    tag = "auth",
    request_body = ExchangeRequest,
    responses(
        (status = 200, description = "Session tokens", body = ExchangeResponse),
    ),
    description = "Exchange an OAuth callback code for access and refresh tokens."
)]
```

- [ ] **Step 5: Annotate `refresh` handler**

```rust
#[utoipa::path(
    post,
    path = "/auth/refresh",
    tag = "auth",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "New access token", body = RefreshResponse),
    ),
    description = "Refresh an expired access token using a refresh token."
)]
```

- [ ] **Step 6: Annotate `me` handler**

```rust
#[utoipa::path(
    get,
    path = "/me",
    tag = "auth",
    security((),),
    responses(
        (status = 200, description = "Current user info", body = MeResponse),
        (status = 401, description = "Unauthorized"),
    ),
    description = "Get the currently authenticated user's email and roles."
)]
```

- [ ] **Step 7: Annotate `logout` handler**

```rust
#[utoipa::path(
    post,
    path = "/auth/logout",
    tag = "auth",
    security((),),
    request_body = LogoutRequest,
    responses(
        (status = 204, description = "Successfully logged out"),
        (status = 401, description = "Unauthorized"),
    ),
    description = "Log out by revoking the current session."
)]
```

- [ ] **Step 8: Verify compilation**

Run: `cd back && cargo check`
Expected: Compilation succeeds

- [ ] **Step 9: Commit**

```
git add back/src/auth/http.rs back/src/auth/model.rs
git commit -m "feat(back): annotate auth handlers with utoipa OpenAPI metadata"
```

---

### Task 3: Annotate admin and categories handlers and models

**Files:**
- Modify: `back/src/admin/http.rs`
- Modify: `back/src/admin/model.rs`
- Modify: `back/src/categories/http.rs`
- Modify: `back/src/categories/model.rs`

- [ ] **Step 1: Add `ToSchema` derives on admin model types**

Types: `RolesResponse`, `AdminUsersResponse`, `AdminUsersQuery`, `UpdateUserRolesRequest`, `UpdateUserRolesResponse`
Add `ToSchema` (and `IntoParams` for `AdminUsersQuery`).

- [ ] **Step 2: Annotate admin `roles` handler**

```rust
#[utoipa::path(
    get,
    path = "/admin/roles",
    tag = "admin",
    security((),),
    responses(
        (status = 200, description = "List of assignable roles", body = RolesResponse),
        (status = 403, description = "Forbidden — requires ADMIN"),
    ),
    description = "List assignable roles. Requires ADMIN."
)]
```

- [ ] **Step 3: Annotate admin `users` handler**

```rust
#[utoipa::path(
    get,
    path = "/admin/users",
    tag = "admin",
    security((),),
    params(AdminUsersQuery),
    responses(
        (status = 200, description = "Paginated user list", body = AdminUsersResponse),
        (status = 403, description = "Forbidden — requires ADMIN"),
    ),
    description = "List users with optional filters. Requires ADMIN."
)]
```

- [ ] **Step 4: Annotate admin `update_user_roles` handler**

```rust
#[utoipa::path(
    put,
    path = "/admin/users/{user_id}/roles",
    tag = "admin",
    security((),),
    params(
        ("user_id" = String, Path, description = "User UUID"),
    ),
    request_body = UpdateUserRolesRequest,
    responses(
        (status = 200, description = "Updated user roles", body = UpdateUserRolesResponse),
        (status = 403, description = "Forbidden — requires ADMIN"),
    ),
    description = "Update a user's roles. Requires ADMIN."
)]
```

- [ ] **Step 5: Add `ToSchema` and `IntoParams` derives on categories model types**

Types: `CategoryItem`, `CategoryCreateRequest`, `CategoryUpdateRequest`, `CategoryListQuery`
Add `ToSchema` to all, `IntoParams` to `CategoryListQuery`.

- [ ] **Step 6: Annotate categories `list` handler**

```rust
#[utoipa::path(
    get,
    path = "/categories",
    tag = "categories",
    security((),),
    params(CategoryListQuery),
    responses(
        (status = 200, description = "List of categories available to CO_OWNERSHIP_BOARD", body = Vec<CategoryItem>),
        (status = 403, description = "Forbidden — requires ADMIN or CO_OWNERSHIP_BOARD"),
    ),
    description = "List categories. Requires ADMIN or CO_OWNERSHIP_BOARD."
)]
```

- [ ] **Step 7: Annotate categories `admin_list` handler**

```rust
#[utoipa::path(
    get,
    path = "/admin/categories",
    tag = "categories",
    security((),),
    params(CategoryListQuery),
    responses(
        (status = 200, description = "Full category list for admins", body = Vec<CategoryItem>),
        (status = 403, description = "Forbidden — requires ADMIN"),
    ),
    description = "List all categories (admin view). Requires ADMIN."
)]
```

- [ ] **Step 8: Annotate categories `create` handler**

```rust
#[utoipa::path(
    post,
    path = "/admin/categories",
    tag = "categories",
    security((),),
    request_body = CategoryCreateRequest,
    responses(
        (status = 201, description = "Created category", body = CategoryItem),
        (status = 403, description = "Forbidden — requires ADMIN"),
    ),
    description = "Create a category. Requires ADMIN."
)]
```

- [ ] **Step 9: Annotate categories `update` handler**

```rust
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
        (status = 200, description = "Updated category", body = CategoryItem),
        (status = 403, description = "Forbidden — requires ADMIN"),
    ),
    description = "Update a category. Requires ADMIN."
)]
```

- [ ] **Step 10: Annotate categories `delete` handler**

```rust
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
```

- [ ] **Step 11: Verify compilation**

Run: `cd back && cargo check`
Expected: Compilation succeeds

- [ ] **Step 12: Commit**

```
git add back/src/admin/ back/src/categories/
git commit -m "feat(back): annotate admin and categories handlers with utoipa metadata"
```

---

### Task 4: Annotate incidents handlers and models

**Files:**
- Modify: `back/src/incidents/http.rs`
- Modify: `back/src/incidents/model.rs`

- [ ] **Step 1: Add `ToSchema` and `IntoParams` derives on incidents model types**

Types requiring `ToSchema`: `IncidentListItem`, `IncidentDetail`, `IncidentEditData`, `IncidentTranslationMatrixRow`, `CreatedKeyResponse`, `IncidentSaveRequest`, `IncidentTranslationsUpdateRequest`, `CategoryDisplay`, `AuditUser`, `IncidentTimelineItem`, `IncidentTranslationValue`, `EditFieldValue`, `IncidentTimelineEditItem`, `IncidentTimelineSaveItem`

Types requiring `IntoParams`: `IncidentListQuery`

- [ ] **Step 2: Annotate incidents `list` handler**

```rust
#[utoipa::path(
    get,
    path = "/incidents",
    tag = "incidents",
    security((),),
    params(IncidentListQuery),
    responses(
        (status = 200, description = "List of incidents", body = Vec<IncidentListItem>),
        (status = 403, description = "Forbidden — requires ADMIN, CO_OWNER, or CO_OWNERSHIP_BOARD"),
    ),
    description = "List incidents. Requires ADMIN, CO_OWNER, or CO_OWNERSHIP_BOARD."
)]
```

- [ ] **Step 3: Annotate incidents `create` handler**

```rust
#[utoipa::path(
    post,
    path = "/incidents",
    tag = "incidents",
    security((),),
    request_body = IncidentSaveRequest,
    responses(
        (status = 201, description = "Incident created", body = CreatedKeyResponse),
        (status = 403, description = "Forbidden — requires ADMIN or CO_OWNERSHIP_BOARD"),
    ),
    description = "Create an incident. Requires ADMIN or CO_OWNERSHIP_BOARD."
)]
```

- [ ] **Step 4: Annotate incidents `detail` handler**

```rust
#[utoipa::path(
    get,
    path = "/incidents/{id}",
    tag = "incidents",
    security((),),
    params(
        ("id" = String, Path, description = "Incident key"),
        IncidentListQuery,
    ),
    responses(
        (status = 200, description = "Incident detail", body = IncidentDetail),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    ),
    description = "Get incident details. Requires ADMIN, CO_OWNER, or CO_OWNERSHIP_BOARD."
)]
```

- [ ] **Step 5: Annotate incidents `update` handler**

```rust
#[utoipa::path(
    put,
    path = "/incidents/{id}",
    tag = "incidents",
    security((),),
    params(
        ("id" = String, Path, description = "Incident key"),
    ),
    request_body = IncidentSaveRequest,
    responses(
        (status = 204, description = "Updated"),
        (status = 403, description = "Forbidden — requires ADMIN or CO_OWNERSHIP_BOARD"),
    ),
    description = "Update an incident. Requires ADMIN or CO_OWNERSHIP_BOARD."
)]
```

- [ ] **Step 6: Annotate incidents `delete` handler**

```rust
#[utoipa::path(
    delete,
    path = "/incidents/{id}",
    tag = "incidents",
    security((),),
    params(
        ("id" = String, Path, description = "Incident key"),
    ),
    responses(
        (status = 204, description = "Deleted"),
        (status = 403, description = "Forbidden — requires ADMIN"),
        (status = 404, description = "Not found"),
    ),
    description = "Delete an incident. Requires ADMIN."
)]
```

- [ ] **Step 7: Annotate incidents `translations` handler**

```rust
#[utoipa::path(
    get,
    path = "/incidents/{id}/translations",
    tag = "incidents",
    security((),),
    params(
        ("id" = String, Path, description = "Incident key"),
    ),
    responses(
        (status = 200, description = "Translation matrix", body = Vec<IncidentTranslationMatrixRow>),
        (status = 403, description = "Forbidden — requires ADMIN"),
    ),
    description = "Get incident translations matrix. Requires ADMIN."
)]
```

- [ ] **Step 8: Annotate incidents `edit` handler**

```rust
#[utoipa::path(
    get,
    path = "/incidents/{id}/edit",
    tag = "incidents",
    security((),),
    params(
        ("id" = String, Path, description = "Incident key"),
        IncidentListQuery,
    ),
    responses(
        (status = 200, description = "Edit data for an incident", body = IncidentEditData),
        (status = 403, description = "Forbidden — requires ADMIN or CO_OWNERSHIP_BOARD"),
    ),
    description = "Get incident edit data. Requires ADMIN or CO_OWNERSHIP_BOARD."
)]
```

- [ ] **Step 9: Annotate incidents `replace_translations` handler**

```rust
#[utoipa::path(
    post,
    path = "/incidents/{id}/translations/replace",
    tag = "incidents",
    security((),),
    params(
        ("id" = String, Path, description = "Incident key"),
    ),
    request_body = IncidentTranslationsUpdateRequest,
    responses(
        (status = 204, description = "Translations replaced"),
        (status = 403, description = "Forbidden — requires ADMIN"),
    ),
    description = "Replace all translations for an incident. Requires ADMIN."
)]
```

- [ ] **Step 10: Verify compilation**

Run: `cd back && cargo check`
Expected: Compilation succeeds

- [ ] **Step 11: Commit**

```
git add back/src/incidents/
git commit -m "feat(back): annotate incidents handlers with utoipa metadata"
```

---

### Task 5: Annotate maintenances and projects handlers and models

**Files:**
- Modify: `back/src/maintenances/http.rs`
- Modify: `back/src/maintenances/model.rs`
- Modify: `back/src/projects/http.rs`
- Modify: `back/src/projects/model.rs`

- [ ] **Step 1: Add `ToSchema` and `IntoParams` derives on maintenances model types**

Types for `ToSchema`: `MaintenanceListItem`, `MaintenanceDetail`, `MaintenanceEditData`, `MaintenanceTranslationMatrixRow`, `CreatedKeyResponse`, `MaintenanceSaveRequest`, `MaintenanceTranslationsUpdateRequest`, `CategoryDisplay`, `AuditUser`, `EditFieldValue`, `MaintenanceTranslationValue`
Types for `IntoParams`: `MaintenanceListQuery`

- [ ] **Step 2: Annotate maintenances `list` handler**

```rust
#[utoipa::path(
    get,
    path = "/maintenances",
    tag = "maintenances",
    security((),),
    params(MaintenanceListQuery),
    responses(
        (status = 200, description = "List of maintenances", body = Vec<MaintenanceListItem>),
        (status = 403, description = "Forbidden — requires ADMIN, CO_OWNER, or CO_OWNERSHIP_BOARD"),
    ),
    description = "List maintenances. Requires ADMIN, CO_OWNER, or CO_OWNERSHIP_BOARD."
)]
```

- [ ] **Step 3: Annotate maintenances `create` handler**

```rust
#[utoipa::path(
    post,
    path = "/maintenances",
    tag = "maintenances",
    security((),),
    request_body = MaintenanceSaveRequest,
    responses(
        (status = 201, description = "Maintenance created", body = CreatedKeyResponse),
        (status = 403, description = "Forbidden — requires ADMIN or CO_OWNERSHIP_BOARD"),
    ),
    description = "Create a maintenance. Requires ADMIN or CO_OWNERSHIP_BOARD."
)]
```

- [ ] **Step 4: Annotate maintenances `detail` handler**

```rust
#[utoipa::path(
    get,
    path = "/maintenances/{id}",
    tag = "maintenances",
    security((),),
    params(
        ("id" = String, Path, description = "Maintenance key"),
        MaintenanceListQuery,
    ),
    responses(
        (status = 200, description = "Maintenance detail", body = MaintenanceDetail),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    ),
    description = "Get maintenance details. Requires ADMIN, CO_OWNER, or CO_OWNERSHIP_BOARD."
)]
```

- [ ] **Step 5: Annotate maintenances `update` handler**

```rust
#[utoipa::path(
    put,
    path = "/maintenances/{id}",
    tag = "maintenances",
    security((),),
    params(
        ("id" = String, Path, description = "Maintenance key"),
    ),
    request_body = MaintenanceSaveRequest,
    responses(
        (status = 204, description = "Updated"),
        (status = 403, description = "Forbidden — requires ADMIN or CO_OWNERSHIP_BOARD"),
    ),
    description = "Update a maintenance. Requires ADMIN or CO_OWNERSHIP_BOARD."
)]
```

- [ ] **Step 6: Annotate maintenances `delete` handler**

```rust
#[utoipa::path(
    delete,
    path = "/maintenances/{id}",
    tag = "maintenances",
    security((),),
    params(
        ("id" = String, Path, description = "Maintenance key"),
    ),
    responses(
        (status = 204, description = "Deleted"),
        (status = 403, description = "Forbidden — requires ADMIN"),
        (status = 404, description = "Not found"),
    ),
    description = "Delete a maintenance. Requires ADMIN."
)]
```

- [ ] **Step 7: Annotate maintenances `translations` handler**

```rust
#[utoipa::path(
    get,
    path = "/maintenances/{id}/translations",
    tag = "maintenances",
    security((),),
    params(
        ("id" = String, Path, description = "Maintenance key"),
    ),
    responses(
        (status = 200, description = "Translation matrix", body = Vec<MaintenanceTranslationMatrixRow>),
        (status = 403, description = "Forbidden — requires ADMIN"),
    ),
    description = "Get maintenance translations matrix. Requires ADMIN."
)]
```

- [ ] **Step 8: Annotate maintenances `edit` handler**

```rust
#[utoipa::path(
    get,
    path = "/maintenances/{id}/edit",
    tag = "maintenances",
    security((),),
    params(
        ("id" = String, Path, description = "Maintenance key"),
        MaintenanceListQuery,
    ),
    responses(
        (status = 200, description = "Edit data for a maintenance", body = MaintenanceEditData),
        (status = 403, description = "Forbidden — requires ADMIN or CO_OWNERSHIP_BOARD"),
    ),
    description = "Get maintenance edit data. Requires ADMIN or CO_OWNERSHIP_BOARD."
)]
```

- [ ] **Step 9: Annotate maintenances `replace_translations` handler**

```rust
#[utoipa::path(
    post,
    path = "/maintenances/{id}/translations/replace",
    tag = "maintenances",
    security((),),
    params(
        ("id" = String, Path, description = "Maintenance key"),
    ),
    request_body = MaintenanceTranslationsUpdateRequest,
    responses(
        (status = 204, description = "Translations replaced"),
        (status = 403, description = "Forbidden — requires ADMIN"),
    ),
    description = "Replace all translations for a maintenance. Requires ADMIN."
)]
```

- [ ] **Step 10: Add `ToSchema` and `IntoParams` derives on projects model types**

Types for `ToSchema`: `ProjectListItem`, `ProjectDetail`, `ProjectEditData`, `ProjectTranslationMatrixRow`, `CreatedKeyResponse`, `ProjectSaveRequest`, `ProjectTranslationsUpdateRequest`, `CategoryDisplay`, `AuditUser`, `EditFieldValue`, `ProjectTranslationValue`
Types for `IntoParams`: `ProjectListQuery`

- [ ] **Step 11: Annotate projects handlers**

Follow the same pattern as maintenances but with `tag = "projects"`, `/projects` paths, `Project*` types, and `ProjectListQuery` params.

- [ ] **Step 12: Verify compilation**

Run: `cd back && cargo check`
Expected: Compilation succeeds

- [ ] **Step 13: Commit**

```
git add back/src/maintenances/ back/src/projects/
git commit -m "feat(back): annotate maintenances and projects handlers with utoipa metadata"
```

---

### Task 6: Annotate translations and api_tokens handlers and models

**Files:**
- Modify: `back/src/translations/http.rs`
- Modify: `back/src/translations/model.rs`
- Modify: `back/src/api_tokens/http.rs`
- Modify: `back/src/api_tokens/model.rs`

- [ ] **Step 1: Add `ToSchema` derives on translations model types**

Types: `TranslationMatrixRow`, `BulkTranslationsUpdateRequest`, `BulkTranslationValue`

- [ ] **Step 2: Annotate translations `list_matrix` handler**

```rust
#[utoipa::path(
    get,
    path = "/translations",
    tag = "translations",
    security((),),
    responses(
        (status = 200, description = "Translation matrix", body = Vec<TranslationMatrixRow>),
        (status = 403, description = "Forbidden — requires ADMIN"),
    ),
    description = "List translation matrix. Requires ADMIN."
)]
```

- [ ] **Step 3: Annotate translations `upsert_bulk` handler**

```rust
#[utoipa::path(
    post,
    path = "/translations/bulk",
    tag = "translations",
    security((),),
    request_body = BulkTranslationsUpdateRequest,
    responses(
        (status = 204, description = "Translations upserted"),
        (status = 403, description = "Forbidden — requires ADMIN"),
    ),
    description = "Bulk upsert translations. Requires ADMIN."
)]
```

- [ ] **Step 4: Add `ToSchema` derives on api_tokens model types**

Types: `CreateTokenResponse`, `TokenInfoResponse`

- [ ] **Step 5: Annotate api_tokens `get_token` handler**

```rust
#[utoipa::path(
    get,
    path = "/settings/token",
    tag = "api_tokens",
    security((),),
    responses(
        (status = 200, description = "Token info", body = TokenInfoResponse),
        (status = 404, description = "No token found"),
    ),
    description = "Get current API token info."
)]
```

- [ ] **Step 6: Annotate api_tokens `create_token` handler**

```rust
#[utoipa::path(
    post,
    path = "/settings/token",
    tag = "api_tokens",
    security((),),
    responses(
        (status = 201, description = "Token created", body = CreateTokenResponse),
    ),
    description = "Create a new API token."
)]
```

- [ ] **Step 7: Annotate api_tokens `revoke_token` handler**

```rust
#[utoipa::path(
    delete,
    path = "/settings/token",
    tag = "api_tokens",
    security((),),
    responses(
        (status = 204, description = "Token revoked"),
    ),
    description = "Revoke the current API token."
)]
```

- [ ] **Step 8: Verify compilation**

Run: `cd back && cargo check`
Expected: Compilation succeeds

- [ ] **Step 9: Commit**

```
git add back/src/translations/ back/src/api_tokens/
git commit -m "feat(back): annotate translations and api_tokens handlers with utoipa metadata"
```

---

### Task 7: Wire up OpenApi struct and add health endpoint

**Files:**
- Modify: `back/src/api_doc/mod.rs`
- Modify: `back/src/app/router.rs`

- [ ] **Step 1: Read the existing `back/src/app/router.rs` to capture the `health` function**

Note that the `health` function is defined in `router.rs` at the bottom. We need to reference it in the OpenApi struct.

- [ ] **Step 2: Write the complete `back/src/api_doc/mod.rs`**

```rust
use utoipa::OpenApi;

mod http;
pub use http::*;

#[derive(OpenApi)]
#[openapi(
    paths(
        // Auth
        crate::auth::http::start_oauth,
        crate::auth::http::oauth_callback,
        crate::auth::http::exchange_session,
        crate::auth::http::refresh,
        crate::auth::http::me,
        crate::auth::http::logout,
        // Admin
        crate::admin::http::roles,
        crate::admin::http::users,
        crate::admin::http::update_user_roles,
        // Categories
        crate::categories::http::list,
        crate::categories::http::admin_list,
        crate::categories::http::create,
        crate::categories::http::update,
        crate::categories::http::delete,
        // Incidents
        crate::incidents::http::list,
        crate::incidents::http::create,
        crate::incidents::http::detail,
        crate::incidents::http::update,
        crate::incidents::http::delete,
        crate::incidents::http::translations,
        crate::incidents::http::edit,
        crate::incidents::http::replace_translations,
        // Maintenances
        crate::maintenances::http::list,
        crate::maintenances::http::create,
        crate::maintenances::http::detail,
        crate::maintenances::http::update,
        crate::maintenances::http::delete,
        crate::maintenances::http::translations,
        crate::maintenances::http::edit,
        crate::maintenances::http::replace_translations,
        // Projects
        crate::projects::http::list,
        crate::projects::http::create,
        crate::projects::http::detail,
        crate::projects::http::update,
        crate::projects::http::delete,
        crate::projects::http::translations,
        crate::projects::http::edit,
        crate::projects::http::replace_translations,
        // Translations
        crate::translations::http::list_matrix,
        crate::translations::http::upsert_bulk,
        // API Tokens
        crate::api_tokens::http::get_token,
        crate::api_tokens::http::create_token,
        crate::api_tokens::http::revoke_token,
        // Health
        crate::app::router::health,
    ),
    components(
        schemas(
            // Auth
            crate::auth::model::ExchangeRequest,
            crate::auth::model::ExchangeResponse,
            crate::auth::model::RefreshRequest,
            crate::auth::model::RefreshResponse,
            crate::auth::model::MeResponse,
            crate::auth::model::LogoutRequest,
            crate::auth::model::OAuthStartQuery,
            crate::auth::model::OAuthCallbackQuery,
            // Admin
            crate::admin::model::RolesResponse,
            crate::admin::model::AdminUsersResponse,
            crate::admin::model::UpdateUserRolesRequest,
            crate::admin::model::UpdateUserRolesResponse,
            // Categories
            crate::categories::model::CategoryItem,
            crate::categories::model::CategoryCreateRequest,
            crate::categories::model::CategoryUpdateRequest,
            crate::categories::model::CategoryListQuery,
            // Incidents
            crate::incidents::model::IncidentListItem,
            crate::incidents::model::IncidentDetail,
            crate::incidents::model::IncidentEditData,
            crate::incidents::model::IncidentTranslationMatrixRow,
            crate::incidents::model::CreatedKeyResponse,
            crate::incidents::model::IncidentSaveRequest,
            crate::incidents::model::IncidentTranslationsUpdateRequest,
            crate::incidents::model::CategoryDisplay,
            crate::incidents::model::AuditUser,
            crate::incidents::model::IncidentTimelineItem,
            crate::incidents::model::IncidentTranslationValue,
            crate::incidents::model::EditFieldValue,
            crate::incidents::model::IncidentTimelineEditItem,
            crate::incidents::model::IncidentTimelineSaveItem,
            // Maintenances
            crate::maintenances::model::MaintenanceListItem,
            crate::maintenances::model::MaintenanceDetail,
            crate::maintenances::model::MaintenanceEditData,
            crate::maintenances::model::MaintenanceTranslationMatrixRow,
            crate::maintenances::model::CreatedKeyResponse,
            crate::maintenances::model::MaintenanceSaveRequest,
            crate::maintenances::model::MaintenanceTranslationsUpdateRequest,
            crate::maintenances::model::CategoryDisplay,
            crate::maintenances::model::AuditUser,
            crate::maintenances::model::EditFieldValue,
            crate::maintenances::model::MaintenanceTranslationValue,
            // Projects
            crate::projects::model::ProjectListItem,
            crate::projects::model::ProjectDetail,
            crate::projects::model::ProjectEditData,
            crate::projects::model::ProjectTranslationMatrixRow,
            crate::projects::model::CreatedKeyResponse,
            crate::projects::model::ProjectSaveRequest,
            crate::projects::model::ProjectTranslationsUpdateRequest,
            crate::projects::model::CategoryDisplay,
            crate::projects::model::AuditUser,
            crate::projects::model::EditFieldValue,
            crate::projects::model::ProjectTranslationValue,
            // Translations
            crate::translations::model::TranslationMatrixRow,
            crate::translations::model::BulkTranslationsUpdateRequest,
            crate::translations::model::BulkTranslationValue,
            // API Tokens
            crate::api_tokens::model::CreateTokenResponse,
            crate::api_tokens::model::TokenInfoResponse,
        )
    ),
    tags(
        (name = "auth", description = "Authentication and session management"),
        (name = "admin", description = "Administrative operations"),
        (name = "categories", description = "Category management"),
        (name = "incidents", description = "Incident management"),
        (name = "maintenances", description = "Maintenance management"),
        (name = "projects", description = "Project management"),
        (name = "translations", description = "Translation management"),
        (name = "api_tokens", description = "API token management"),
        (name = "health", description = "Health check"),
    ),
)]
pub struct ApiDoc;
```

- [ ] **Step 3: Annotate and make `health` in `router.rs` public**

In `back/src/app/router.rs`, annotate the `health` function with utoipa metadata and make it pub:

```rust
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Service is healthy"),
    ),
    description = "Health check endpoint."
)]
pub async fn health() -> &'static str {
    "ok"
}
```

- [ ] **Step 4: Verify compilation**

Run: `cd back && cargo build`
If there are compilation errors about missing `ToSchema` or `IntoParams` derives, add them.
Expected: Compilation succeeds

- [ ] **Step 6: Commit**

```
git add back/src/api_doc/mod.rs back/src/app/router.rs
git commit -m "feat(back): wire up OpenApi struct with all paths and schemas"
```

---

### Task 8: Verify backend builds and tests pass

**Files:** None

- [ ] **Step 1: Run lint**

Run: `cd back && cargo clippy --all-features -- -D warnings`
If there are warnings about unused imports in `utoipa` path macros or missing docs, fix them.

Note: If utoipa macros generate warnings, add `#[allow(unused_imports)]` or similar at the module level.

- [ ] **Step 2: Run tests**

Run: `cd back && cargo test --all-features`
Expected: All tests pass

- [ ] **Step 3: Run build**

Run: `cd back && cargo build --all-features`
Expected: Build succeeds

---

### Task 9: Frontend — install Scalar, create ApiDocPage, add route and link

**Files:**
- Modify: `front/package.json`
- Create: `front/src/views/settings/ApiDocPage.vue`
- Modify: `front/src/router/index.ts`
- Modify: `front/src/views/settings/SettingsPage.vue`
- Modify: `front/src/common/i18n.ts`

- [ ] **Step 1: Install @scalar/api-reference**

Run: `cd front && npm install @scalar/api-reference`

- [ ] **Step 2: Create `front/src/views/settings/ApiDocPage.vue`**

```vue
<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { getAccessToken } from "../../auth/session";
import { ApiReference } from "@scalar/api-reference";
import "@scalar/api-reference/style.css";

const { t } = useI18n();

const spec = ref<Record<string, unknown> | null>(null);
const error = ref(false);

onMounted(async () => {
  try {
    const token = getAccessToken();
    const res = await fetch(
      `${import.meta.env.VITE_API_BASE_URL ?? "http://localhost:8080"}/openapi.json`,
      { headers: token ? { Authorization: `Bearer ${token}` } : undefined },
    );
    if (!res.ok) throw new Error("failed to fetch spec");
    spec.value = await res.json();
  } catch {
    error.value = true;
  }
});
</script>

<template>
  <main class="page-wrap">
    <h1 class="page-title">{{ t("labels.apiDocumentation") }}</h1>
    <p v-if="error" class="error-message">{{ t("labels.specLoadFailed") }}</p>
    <div v-else-if="!spec" class="empty-state">...</div>
    <div v-else class="scalar-wrapper">
      <ApiReference
        :configuration="{
          spec: { content: spec },
          hideDownloadButton: true,
          hideTestRequestButton: true,
        }" />
    </div>
  </main>
</template>

<style>
/* Scalar theme overrides — must be global (not scoped) to reach shadow DOM */
.scalar-wrapper {
  margin-top: 1rem;
  border: 1px solid var(--border-color);
  border-radius: 0.9rem;
  overflow: hidden;
  --scalar-color-1: var(--page-fg);
  --scalar-color-2: var(--page-fg);
  --scalar-color-3: var(--muted-fg);
  --scalar-color-accent: #4f9fff;
  --scalar-background-1: var(--page-bg);
  --scalar-background-2: var(--panel-bg);
  --scalar-background-3: var(--page-bg);
  --scalar-border-color: var(--border-color);
  --scalar-color-disabled: var(--muted-fg);
  --scalar-sidebar-background-1: var(--panel-bg);
}

.error-message {
  color: #f35a67;
  font-weight: 700;
}

.empty-state {
  margin-top: 1.2rem;
  color: var(--muted-fg);
}
</style>
```

- [ ] **Step 3: Add route in `front/src/router/index.ts`**

Add lazy import:
```ts
const ApiDocPage = () => import("../views/settings/ApiDocPage.vue");
```

Add route entry after the settings route:
```ts
{ path: "/settings/api-doc", component: ApiDocPage, meta: { requiresAuth: true } },
```

- [ ] **Step 4: Add link in `front/src/views/settings/SettingsPage.vue`**

After the closing `</section>` of the API token panel (line ~164), add:

```html
<section class="settings-section">
  <a href="#/settings/api-doc" class="api-doc-link">{{ t("labels.apiDocumentation") }}</a>
</section>
```

Add this CSS to the scoped styles:

```css
.api-doc-link {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  border: 1px solid var(--control-border);
  border-radius: 0.55rem;
  padding: 0.45rem 0.7rem;
  cursor: pointer;
  font-size: 0.85rem;
  text-decoration: none;
  color: var(--control-fg);
  background: transparent;
}

.api-doc-link:hover {
  border-color: rgba(72, 144, 255, 0.7);
  background: rgba(72, 144, 255, 0.22);
}
```

- [ ] **Step 5: Add translation keys in `front/src/common/i18n.ts`**

In the `en` labels section, add:
```ts
apiDocumentation: "API documentation",
specLoadFailed: "API documentation could not be loaded.",
```

In the `fr` labels section, add:
```ts
apiDocumentation: "Documentation de l'API",
specLoadFailed: "La documentation de l'API n'a pas pu être chargée.",
```

- [ ] **Step 6: Verify frontend builds**

Run: `cd front && npm run build`
Expected: Build succeeds

- [ ] **Step 7: Run lint**

Run: `cd front && npm run lint`
Expected: No type errors

- [ ] **Step 8: Run tests**

Run: `cd front && npm run test`
Expected: All tests pass

- [ ] **Step 9: Commit**

```
git add front/package.json front/package-lock.json front/src/
git commit -m "feat(front): add API documentation page with Scalar reference renderer"
```

---

## Self-Review Checklist

After completing all tasks, verify:
- [ ] `GET /openapi.json` returns valid OpenAPI 3.1 JSON
- [ ] All endpoints are listed with correct paths, methods, and tags
- [ ] Each endpoint has the required role documented in its description
- [ ] Scalar renders the spec correctly with project theming
- [ ] The API doc link is visible below the token panel on the settings page
- [ ] Backend compiles with `cargo build --all-features` and passes `cargo test`
- [ ] Frontend compiles with `npm run build` and passes `npm run test`
