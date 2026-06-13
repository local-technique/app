# OpenAPI Documentation Feature

## Purpose

Expose the existing backend API as an OpenAPI 3.1 spec via a new endpoint,
and render it in the frontend as a documentation page accessible from
Settings.

## Backend

### Approach — utoipa

Add `utoipa` with the `axum` feature to annotate route handlers and
derive OpenAPI schema from existing request/response types.

### Dependencies

- `utoipa` (features: `axum`)

### Steps

1. **New module `back/src/api_doc/`** with:
   - `mod.rs` — `#[derive(OpenApi)]` struct named `ApiDoc` that
     references every path set (one per module) via `paths(...)`.
     Also references all response/schema types via `components(schemas(...))`.
   - `http.rs` — handler `get_openapi_json` returning
     `Json<utoipa::openapi::OpenApi>` by calling `ApiDoc::openapi()`.

2. **Annotate each existing handler** with `#[utoipa::path(...)]`:
   - `get`/`post`/`put`/`delete` method and full path
   - `tag` — grouped by resource (incidents, maintenances, projects, etc.)
   - `params(...)` — query/Path/extractor types
   - `request_body` — for POST/PUT with JSON body
   - `responses(...)` — status code + body type
   - `description` — includes role requirements like "Requires ADMIN, CO_OWNER, or CO_OWNERSHIP_BOARD."

3. **Add route** in `app/router.rs`:
   ```rust
   .route("/openapi.json", get(api_doc::http::get_openapi_json))
   ```

4. **Serve spec** — authenticated (requires auth, like `/settings/token`).
   The spec contains per-endpoint role requirements in descriptions.
   Using `Principal` extractor ensures only authenticated users can
   access the spec, so api doc is available to anyone with a CoPro account.

### Role documentation strategy

Since `utoipa` macros execute at compile time and cannot introspect
`principal.ensure_role(Role::Admin)` calls, role requirements are
documented in the `description` field of each `#[utoipa::path(...)]`
annotation.

## Frontend

### Approach — Scalar API Reference Vue component

Use `@scalar/api-reference` to render the OpenAPI spec in a clean
two-panel layout (endpoint sidebar + detail panel) that resembles
GitHub REST API docs.

### Steps

1. **Install dependency**:
   ```
   npm install @scalar/api-reference
   ```

2. **New component** `front/src/views/settings/ApiDocPage.vue`:
   - Fetches `/openapi.json` from the backend on mount
   - Passes the spec JSON to Scalar's `ApiReference` component
   - Applies CSS variable mappings to match project theme:
     - `--scalar-color-1` → `var(--page-fg)`
     - `--scalar-background-1` → `var(--page-bg)`
     - `--scalar-background-2` → `var(--panel-bg)`
     - `--scalar-border-color` → `var(--border-color)`
     - `--scalar-color-accent` → `#4f9fff` (link blue from project)
     - `--scalar-color-3` → `var(--muted-fg)`

3. **New route** in `front/src/router/index.ts`:
   ```ts
   { path: "/settings/api-doc", component: ApiDocPage, meta: { requiresAuth: true } }
   ```

4. **Link in SettingsPage.vue** — add below the API token panel:
   ```html
   <section class="settings-section">
     <a href="#/settings/api-doc" class="api-doc-link">{{ t("labels.apiDocumentation") }}</a>
   </section>
   ```

5. **Translation keys** — add `labels.apiDocumentation` to i18n files.

## Files Changed

### Backend
- `back/Cargo.toml` — add `utoipa`
- `back/src/api_doc/mod.rs` (new) — `OpenApi` derive struct
- `back/src/api_doc/http.rs` (new) — serve endpoint
- `back/src/main.rs` — add `mod api_doc;`
- `back/src/app/router.rs` — add route
- `back/src/incidents/http.rs` — annotate handlers
- `back/src/maintenances/http.rs` — annotate handlers
- `back/src/projects/http.rs` — annotate handlers
- `back/src/categories/http.rs` — annotate handlers
- `back/src/auth/http.rs` — annotate handlers
- `back/src/admin/http.rs` — annotate handlers
- `back/src/translations/http.rs` — annotate handlers
- `back/src/api_tokens/http.rs` — annotate handlers

### Frontend
- `front/package.json` — add `@scalar/api-reference`
- `front/src/views/settings/ApiDocPage.vue` (new) — page component
- `front/src/router/index.ts` — add route
- `front/src/views/settings/SettingsPage.vue` — add link
- `front/src/common/i18n.ts` (or locale files) — add translation key
