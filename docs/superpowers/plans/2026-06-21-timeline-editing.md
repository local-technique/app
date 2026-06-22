# Timeline Editing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add inline add/edit/delete of timeline entries for incidents, projects, and maintenances (events) — on both detail pages and form pages.

**Architecture:** New dedicated REST endpoints per entity (`POST/PUT/DELETE /{entity}/{id}/timeline[/{entryId}]`). Frontend `EditableTimelineList` component replaces read-only `TimelineList` and inline duplicated HTML. Role-gated at both frontend and backend (`Admin`, `CoOwnershipBoard`, `CoOwnershipBoardOps`).

**Tech Stack:** Rust/Axum/SQLX backend, Vue 3 + TypeScript frontend, PostgreSQL.

---

### Task 1: Backend — Add timeline CRUD model types

**Files:**
- Modify: `back/src/incidents/model.rs`
- Modify: `back/src/projects/model.rs`
- Modify: `back/src/maintenances/model.rs`

Each entity needs a request struct for creating/updating a timeline entry and a response struct.

- [ ] **Step 1: Add request/response types to `incidents/model.rs`**

```rust
// Add after `CreatedKeyResponse` at the bottom:
#[derive(serde::Deserialize, ToSchema)]
pub struct IncidentTimelineCreateRequest {
    pub at_utc: Option<String>,
    pub sort_order: i32,
    pub fields: HashMap<String, String>,
}

#[derive(serde::Deserialize, ToSchema)]
pub struct IncidentTimelineUpdateRequest {
    pub at_utc: Option<String>,
    pub sort_order: i32,
    pub fields: HashMap<String, String>,
}

// Response uses the existing IncidentTimelineItem struct (id, at_utc, title, details)
```

- [ ] **Step 2: Add same types to `projects/model.rs`**

```rust
#[derive(serde::Deserialize, ToSchema)]
pub struct ProjectTimelineCreateRequest {
    pub at_utc: Option<String>,
    pub sort_order: i32,
    pub fields: HashMap<String, String>,
}

#[derive(serde::Deserialize, ToSchema)]
pub struct ProjectTimelineUpdateRequest {
    pub at_utc: Option<String>,
    pub sort_order: i32,
    pub fields: HashMap<String, String>,
}

// Response uses the existing ProjectTimelineItem struct
```

- [ ] **Step 3: Add same types to `maintenances/model.rs`**

```rust
#[derive(serde::Deserialize, ToSchema)]
pub struct MaintenanceTimelineCreateRequest {
    pub at_utc: Option<String>,
    pub sort_order: i32,
    pub fields: HashMap<String, String>,
}

#[derive(serde::Deserialize, ToSchema)]
pub struct MaintenanceTimelineUpdateRequest {
    pub at_utc: Option<String>,
    pub sort_order: i32,
    pub fields: HashMap<String, String>,
}

// Response uses the existing MaintenanceTimelineItem struct
```

- [ ] **Step 4: Verify build**

Run: `cd back; cargo check`
Expected: compilation succeeds

---

### Task 2: Backend — Add repository methods for timeline CRUD

**Files:**
- Modify: `back/src/incidents/repository.rs`
- Modify: `back/src/projects/repository.rs`
- Modify: `back/src/maintenances/repository.rs`

Each repository gets three new async functions: `create_timeline_entry`, `update_timeline_entry`, `delete_timeline_entry`.

- [ ] **Step 1: Add to `incidents/repository.rs`**

Add after the `delete_by_code` function:

```rust
pub async fn create_timeline_entry(
    db: &sqlx::PgPool,
    incident_code: &str,
    at_utc: Option<DateTime<Utc>>,
    sort_order: i32,
    locale: &str,
    fields: &HashMap<String, String>,
) -> Result<IncidentTimelineItem, AppError> {
    let incident_id: Uuid = sqlx::query_scalar("SELECT id FROM incidents WHERE key = $1")
        .bind(incident_code)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::not_found("incident not found"))?;

    let mut tx = db.begin().await?;
    let timeline_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO incident_timeline (id, incident_id, at_utc, sort_order) VALUES ($1, $2, $3, $4)",
    )
    .bind(timeline_id)
    .bind(incident_id)
    .bind(at_utc)
    .bind(sort_order)
    .execute(&mut *tx)
    .await?;

    for (field_key, field_value) in fields {
        save_timeline_field(&mut tx, timeline_id, locale, field_key, field_value).await?;
    }
    tx.commit().await?;

    let locale_chain = crate::common::i18n::locale_chain(Some(locale));
    let title = sqlx::query_scalar::<_, String>(
        "SELECT coalesce((SELECT ti.field_value FROM incident_timeline_i18n ti JOIN unnest($2::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = ti.locale WHERE ti.timeline_id = $1 AND ti.field_key = 'title' ORDER BY lp.ord LIMIT 1), '')",
    )
    .bind(timeline_id)
    .bind(&locale_chain)
    .fetch_one(db)
    .await?;

    let details = sqlx::query_scalar::<_, String>(
        "SELECT coalesce((SELECT ti.field_value FROM incident_timeline_i18n ti JOIN unnest($2::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = ti.locale WHERE ti.timeline_id = $1 AND ti.field_key = 'details' ORDER BY lp.ord LIMIT 1), '')",
    )
    .bind(timeline_id)
    .bind(&locale_chain)
    .fetch_one(db)
    .await?;

    Ok(IncidentTimelineItem {
        id: timeline_id.to_string(),
        at_utc: at_utc.map(|v| v.to_rfc3339()),
        title,
        details,
    })
}

pub async fn update_timeline_entry(
    db: &sqlx::PgPool,
    incident_code: &str,
    entry_id: &str,
    at_utc: Option<DateTime<Utc>>,
    sort_order: i32,
    locale: &str,
    fields: &HashMap<String, String>,
) -> Result<Option<IncidentTimelineItem>, AppError> {
    let incident_id: Uuid = sqlx::query_scalar("SELECT id FROM incidents WHERE key = $1")
        .bind(incident_code)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::not_found("incident not found"))?;

    let timeline_id = Uuid::parse_str(entry_id).map_err(|_| AppError::bad_request("invalid timeline id"))?;

    let mut tx = db.begin().await?;
    let updated = sqlx::query(
        "UPDATE incident_timeline SET at_utc = $1, sort_order = $2 WHERE id = $3 AND incident_id = $4",
    )
    .bind(at_utc)
    .bind(sort_order)
    .bind(timeline_id)
    .bind(incident_id)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if updated == 0 {
        return Ok(None);
    }

    for (field_key, field_value) in fields {
        save_timeline_field(&mut tx, timeline_id, locale, field_key, field_value).await?;
    }
    tx.commit().await?;

    let locale_chain = crate::common::i18n::locale_chain(Some(locale));
    let title = sqlx::query_scalar::<_, String>(
        "SELECT coalesce((SELECT ti.field_value FROM incident_timeline_i18n ti JOIN unnest($2::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = ti.locale WHERE ti.timeline_id = $1 AND ti.field_key = 'title' ORDER BY lp.ord LIMIT 1), '')",
    )
    .bind(timeline_id)
    .bind(&locale_chain)
    .fetch_one(db)
    .await?;

    let details = sqlx::query_scalar::<_, String>(
        "SELECT coalesce((SELECT ti.field_value FROM incident_timeline_i18n ti JOIN unnest($2::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = ti.locale WHERE ti.timeline_id = $1 AND ti.field_key = 'details' ORDER BY lp.ord LIMIT 1), '')",
    )
    .bind(timeline_id)
    .bind(&locale_chain)
    .fetch_one(db)
    .await?;

    Ok(Some(IncidentTimelineItem {
        id: timeline_id.to_string(),
        at_utc: at_utc.map(|v| v.to_rfc3339()),
        title,
        details,
    }))
}

pub async fn delete_timeline_entry(
    db: &sqlx::PgPool,
    incident_code: &str,
    entry_id: &str,
) -> Result<bool, AppError> {
    let incident_id: Uuid = sqlx::query_scalar("SELECT id FROM incidents WHERE key = $1")
        .bind(incident_code)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::not_found("incident not found"))?;

    let timeline_id = Uuid::parse_str(entry_id).map_err(|_| AppError::bad_request("invalid timeline id"))?;
    let result = sqlx::query("DELETE FROM incident_timeline WHERE id = $1 AND incident_id = $2")
        .bind(timeline_id)
        .bind(incident_id)
        .execute(db)
        .await?;
    Ok(result.rows_affected() > 0)
}
```

- [ ] **Step 2: Add same 3 functions to `projects/repository.rs`**

Replace `incidents` → `projects`, `incident_timeline` → `project_timeline`, `incident_timeline_i18n` → `project_timeline_i18n`, `IncidentTimelineItem` → `ProjectTimelineItem`, `incident_code` → `project_code`, `incident not found` → `project not found`.

- [ ] **Step 3: Add same 3 functions to `maintenances/repository.rs`**

Replace `incidents` → `maintenances`, `incident_timeline` → `maintenance_timeline`, `incident_timeline_i18n` → `maintenance_timeline_i18n`, `IncidentTimelineItem` → `MaintenanceTimelineItem`, `incident_code` → `maintenance_code`, `incident not found` → `maintenance not found`.

- [ ] **Step 4: Verify build**

Run: `cd back; cargo check`
Expected: compilation succeeds

---

### Task 3: Backend — Add service functions for timeline CRUD

**Files:**
- Modify: `back/src/incidents/service.rs`
- Modify: `back/src/projects/service.rs`
- Modify: `back/src/maintenances/service.rs`

- [ ] **Step 1: Add to `incidents/service.rs`**

Add at the end before the `validate_translation_value` function:

```rust
pub async fn create_timeline_entry(
    db: &sqlx::PgPool,
    incident_code: &str,
    payload: &IncidentTimelineCreateRequest,
    locale: &str,
) -> Result<IncidentTimelineItem, AppError> {
    let enabled_locales = load_enabled_locales(db).await?;
    let locale = normalize_locale(locale)?;
    ensure_locale_enabled(&locale, &enabled_locales)?;
    let fields = validate_field_map(&payload.fields, &INCIDENT_TIMELINE_TRANSLATION_FIELD_KEYS, &["title"])?;
    let at_utc = payload
        .at_utc
        .as_deref()
        .filter(|v| !v.trim().is_empty())
        .map(DateTime::parse_from_rfc3339)
        .transpose()
        .map_err(|_| AppError::bad_request("invalid at_utc"))?
        .map(|v| v.with_timezone(&Utc));
    repository::create_timeline_entry(db, incident_code, at_utc, payload.sort_order, &locale, &fields).await
}

pub async fn update_timeline_entry(
    db: &sqlx::PgPool,
    incident_code: &str,
    entry_id: &str,
    payload: &IncidentTimelineUpdateRequest,
    locale: &str,
) -> Result<IncidentTimelineItem, AppError> {
    let enabled_locales = load_enabled_locales(db).await?;
    let locale = normalize_locale(locale)?;
    ensure_locale_enabled(&locale, &enabled_locales)?;
    let fields = validate_field_map(&payload.fields, &INCIDENT_TIMELINE_TRANSLATION_FIELD_KEYS, &["title"])?;
    let at_utc = payload
        .at_utc
        .as_deref()
        .filter(|v| !v.trim().is_empty())
        .map(DateTime::parse_from_rfc3339)
        .transpose()
        .map_err(|_| AppError::bad_request("invalid at_utc"))?
        .map(|v| v.with_timezone(&Utc));
    repository::update_timeline_entry(db, incident_code, entry_id, at_utc, payload.sort_order, &locale, &fields)
        .await?
        .ok_or_else(|| AppError::not_found("timeline entry not found"))
}

pub async fn delete_timeline_entry(
    db: &sqlx::PgPool,
    incident_code: &str,
    entry_id: &str,
) -> Result<(), AppError> {
    let deleted = repository::delete_timeline_entry(db, incident_code, entry_id).await?;
    if deleted {
        Ok(())
    } else {
        Err(AppError::not_found("timeline entry not found"))
    }
}
```

Add import: `use chrono::{DateTime, Utc};` at the top.

- [ ] **Step 2: Add same 3 functions to `projects/service.rs`**

Replace `IncidentTimelineCreateRequest` → `ProjectTimelineCreateRequest`, `IncidentTimelineUpdateRequest` → `ProjectTimelineUpdateRequest`, `IncidentTimelineItem` → `ProjectTimelineItem`, `INCIDENT_TIMELINE_TRANSLATION_FIELD_KEYS` → `PROJECT_TIMELINE_TRANSLATION_FIELD_KEYS` (check existing constant name — it uses `INCIDENT_TIMELINE_TRANSLATION_FIELD_KEYS` in incidents, check the equivalent in projects).

Projects service uses `PROJECT_TIMELINE_TRANSLATION_FIELD_KEYS` — verify the exact constant name from the projects service file.

- [ ] **Step 3: Add same 3 functions to `maintenances/service.rs`**

Replace for maintenances.

- [ ] **Step 4: Verify build**

Run: `cd back; cargo check`
Expected: compilation succeeds

---

### Task 4: Backend — Add HTTP handlers for timeline CRUD

**Files:**
- Modify: `back/src/incidents/http.rs`
- Modify: `back/src/projects/http.rs`
- Modify: `back/src/maintenances/http.rs`
- Modify: `back/src/app/router.rs`

- [ ] **Step 1: Add handlers to `incidents/http.rs`**

Add before the last line of the file:

```rust
#[utoipa::path(
    post,
    path = "/incidents/{id}/timeline",
    tag = "incidents",
    security((),),
    params(
        ("id" = String, Path, description = "Incident ID"),
    ),
    request_body = IncidentTimelineCreateRequest,
    responses(
        (status = 201, description = "Timeline entry created", body = IncidentTimelineItem),
        (status = 403, description = "Forbidden"),
    ),
    description = "Create a timeline entry. Requires ADMIN or CO_OWNERSHIP_BOARD."
)]
pub async fn create_timeline(
    principal: Principal,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<IncidentListQuery>,
    Json(payload): Json<IncidentTimelineCreateRequest>,
) -> Result<(StatusCode, Json<IncidentTimelineItem>), AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwnershipBoard, Role::CoOwnershipBoardOps])?;
    let locale = query.locale.unwrap_or_else(|| "en".to_string());
    let entry = service::create_timeline_entry(&state.db, &id, &payload, &locale).await?;
    Ok((StatusCode::CREATED, Json(entry)))
}

#[utoipa::path(
    put,
    path = "/incidents/{id}/timeline/{entryId}",
    tag = "incidents",
    security((),),
    params(
        ("id" = String, Path, description = "Incident ID"),
        ("entryId" = String, Path, description = "Timeline entry ID"),
    ),
    request_body = IncidentTimelineUpdateRequest,
    responses(
        (status = 200, description = "Timeline entry updated", body = IncidentTimelineItem),
        (status = 403, description = "Forbidden"),
    ),
    description = "Update a timeline entry. Requires ADMIN or CO_OWNERSHIP_BOARD."
)]
pub async fn update_timeline(
    principal: Principal,
    State(state): State<AppState>,
    Path((id, entry_id)): Path<(String, String)>,
    Query(query): Query<IncidentListQuery>,
    Json(payload): Json<IncidentTimelineUpdateRequest>,
) -> Result<Json<IncidentTimelineItem>, AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwnershipBoard, Role::CoOwnershipBoardOps])?;
    let locale = query.locale.unwrap_or_else(|| "en".to_string());
    let entry = service::update_timeline_entry(&state.db, &id, &entry_id, &payload, &locale).await?;
    Ok(Json(entry))
}

#[utoipa::path(
    delete,
    path = "/incidents/{id}/timeline/{entryId}",
    tag = "incidents",
    security((),),
    params(
        ("id" = String, Path, description = "Incident ID"),
        ("entryId" = String, Path, description = "Timeline entry ID"),
    ),
    responses(
        (status = 204, description = "Timeline entry deleted"),
        (status = 403, description = "Forbidden"),
    ),
    description = "Delete a timeline entry. Requires ADMIN or CO_OWNERSHIP_BOARD."
)]
pub async fn delete_timeline(
    principal: Principal,
    State(state): State<AppState>,
    Path((id, entry_id)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwnershipBoard, Role::CoOwnershipBoardOps])?;
    service::delete_timeline_entry(&state.db, &id, &entry_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
```

Update the use import line to include the new model types:

```rust
use crate::incidents::model::{
    CreatedKeyResponse, IncidentListQuery, IncidentSaveRequest, IncidentTranslationsUpdateRequest,
    IncidentTimelineCreateRequest, IncidentTimelineUpdateRequest,
};
```

- [ ] **Step 2: Add analogous 3 handlers to `projects/http.rs`**

Same pattern with `ProjectTimelineCreateRequest`, `ProjectTimelineUpdateRequest`, `ProjectTimelineItem`, `ProjectListQuery`.

- [ ] **Step 3: Add analogous 3 handlers to `maintenances/http.rs`**

Same pattern with `MaintenanceTimelineCreateRequest`, `MaintenanceTimelineUpdateRequest`, `MaintenanceTimelineItem`, `MaintenanceListQuery`.

- [ ] **Step 4: Register routes in `router.rs`**

Add for incidents (after the existing `/incidents/{id}/edit` line):

```rust
.route("/incidents/{id}/timeline", post(incidents::http::create_timeline))
.route(
    "/incidents/{id}/timeline/{entry_id}",
    axum::routing::put(incidents::http::update_timeline).delete(incidents::http::delete_timeline),
)
```

Add for projects:

```rust
.route("/projects/{id}/timeline", post(projects::http::create_timeline))
.route(
    "/projects/{id}/timeline/{entry_id}",
    axum::routing::put(projects::http::update_timeline).delete(projects::http::delete_timeline),
)
```

Add for maintenances:

```rust
.route("/maintenances/{id}/timeline", post(maintenances::http::create_timeline))
.route(
    "/maintenances/{id}/timeline/{entry_id}",
    axum::routing::put(maintenances::http::update_timeline).delete(maintenances::http::delete_timeline),
)
```

Ensure `axum::routing::post` is imported (add `post` to the routing import if not already).

- [ ] **Step 5: Verify build**

Run: `cd back; cargo check`
Expected: compilation succeeds

---

### Task 5: Frontend — Add API methods for timeline CRUD in repositories

**Files:**
- Modify: `front/src/incidents/repositories/apiIncidentsRepository.ts`
- Modify: `front/src/projects/repositories/apiProjectsRepository.ts`
- Modify: `front/src/events/repositories/apiEventsRepository.ts`

- [ ] **Step 1: Add `createTimelineEntry`, `updateTimelineEntry`, `deleteTimelineEntry` to `apiIncidentsRepository.ts`**

Add inside `ApiIncidentsRepository` class:

```typescript
async createTimelineEntry(id: string, preferredLanguage: LocaleCode, payload: { atUtc: string | null; sortOrder: number; fields: Record<string, string> }): Promise<ApiIncidentTimelineItem> {
  if (useMockData()) return { id: crypto.randomUUID(), at_utc: payload.atUtc, title: payload.fields.title ?? "", details: payload.fields.details ?? "" };
  const params = new URLSearchParams({ locale: preferredLanguage });
  const result = await sendJson<ApiIncidentTimelineItem>(`${apiBaseUrl()}/incidents/${encodeURIComponent(id)}/timeline?${params.toString()}`, "POST", {
    at_utc: payload.atUtc,
    sort_order: payload.sortOrder,
    fields: payload.fields,
  });
  if (!result) throw new Error("failed to create timeline entry");
  return result;
}

async updateTimelineEntry(id: string, entryId: string, preferredLanguage: LocaleCode, payload: { atUtc: string | null; sortOrder: number; fields: Record<string, string> }): Promise<ApiIncidentTimelineItem> {
  if (useMockData()) return { id: entryId, at_utc: payload.atUtc, title: payload.fields.title ?? "", details: payload.fields.details ?? "" };
  const params = new URLSearchParams({ locale: preferredLanguage });
  const result = await sendJson<ApiIncidentTimelineItem>(`${apiBaseUrl()}/incidents/${encodeURIComponent(id)}/timeline/${encodeURIComponent(entryId)}?${params.toString()}`, "PUT", {
    at_utc: payload.atUtc,
    sort_order: payload.sortOrder,
    fields: payload.fields,
  });
  if (!result) throw new Error("failed to update timeline entry");
  return result;
}

async deleteTimelineEntry(id: string, entryId: string): Promise<void> {
  if (useMockData()) return;
  await sendJson(`${apiBaseUrl()}/incidents/${encodeURIComponent(id)}/timeline/${encodeURIComponent(entryId)}`, "DELETE");
}
```

- [ ] **Step 2: Add same 3 methods to `apiProjectsRepository.ts`**

Replace `/incidents/` → `/projects/`.

- [ ] **Step 3: Add same 3 methods to `apiEventsRepository.ts`**

Replace `/incidents/` → `/maintenances/`.

- [ ] **Step 4: Verify TypeScript**

Run: `cd front; npx tsc --noEmit`
Expected: no errors

---

### Task 6: Frontend — Create EditableTimelineList.vue component

**Files:**
- Create: `front/src/common/components/EditableTimelineList.vue`

This is the main component. Props:
- `entries: TimelineEntry[]` (read-only view models)
- `canEdit: boolean`
- `entityId: string`
- `entityType: 'incidents' | 'projects' | 'events'`
- `locale: string`
- `onAdd`, `onUpdate`, `onDelete` — callback functions for mutations

When `canEdit` is false, renders identical to current `TimelineList`.

When `canEdit` is true:
- New-entry form at top (date/time input defaulted to current time, title input, details textarea, save icon + X icon)
- Each entry gets trash-2 (red bg) + pencil icons. Pencil switches to edit mode (inputs matching current values, save icon + X icon)
- Save icon persists, X icon cancels, trash-2 deletes immediately
- Save button disabled when title is empty

- [ ] **Step 1: Create the component script section**

The script section needs to:
1. Accept props as described
2. Track which entry is being edited (`editingId: string | null`)
3. Track new-entry form state
4. Handle add/edit/delete/click-outside

```typescript
<script setup lang="ts">
import { CircleCheck, Pencil, Save, Trash2, X } from "@lucide/vue";
import { computed, nextTick, onMounted, onUnmounted, ref, type Ref } from "vue";
import { parseUtc } from "../date";
import type { TimelineEntry } from "./TimelineList.vue";

const props = defineProps<{
  entries: TimelineEntry[];
  canEdit: boolean;
  entityId: string;
  entityType: "incidents" | "projects" | "events";
  locale: string;
}>();

const emit = defineEmits<{
  (e: "add", payload: { atUtc: string | null; sortOrder: number; fields: Record<string, string> }): void;
  (e: "update", entryId: string, payload: { atUtc: string | null; sortOrder: number; fields: Record<string, string> }): void;
  (e: "delete", entryId: string): void;
}>();

// Editing state
const editingId = ref<string | null>(null);
const showNewForm = ref(false);
const editForm = ref<{ atUtc: string; title: string; details: string }>({ atUtc: "", title: "", details: "" });
const newForm = ref<{ atUtc: string; title: string; details: string }>({ atUtc: "", title: "", details: "" });
const savingId = ref<string | null>(null);

function nowUtc(): string {
  return new Date().toISOString();
}

function initNewForm(): void {
  newForm.value = { atUtc: nowUtc(), title: "", details: "" };
  showNewForm.value = true;
}

function cancelNewForm(): void {
  showNewForm.value = false;
  newForm.value = { atUtc: "", title: "", details: "" };
}

function startEdit(entry: TimelineEntry): void {
  editingId.value = entry.id;
  editForm.value = {
    atUtc: "", // we need the raw atUtc; stored in a map
    title: entry.title,
    details: entry.details,
  };
}

function cancelEdit(): void {
  editingId.value = null;
}

async function saveEntry(entryId: string | null): Promise<void> {
  if (entryId === null) {
    // New entry
    if (!newForm.value.title.trim()) return;
    emit("add", {
      atUtc: newForm.value.atUtc || null,
      sortOrder: 0,
      fields: { title: newForm.value.title.trim(), details: newForm.value.details.trim() },
    });
    showNewForm.value = false;
    newForm.value = { atUtc: nowUtc(), title: "", details: "" };
  } else {
    if (!editForm.value.title.trim()) {
      editingId.value = null;
      return;
    }
    emit("update", entryId, {
      atUtc: editForm.value.atUtc || null,
      sortOrder: 0,
      fields: { title: editForm.value.title.trim(), details: editForm.value.details.trim() },
    });
    editingId.value = null;
  }
}

function handleDelete(entryId: string): void {
  if (editingId.value === entryId) editingId.value = null;
  emit("delete", entryId);
}
</script>
```

- [ ] **Step 2: Create the component template**

```html
<template>
  <div class="TimelineList TimelineList--editable">
    <!-- New entry form -->
    <article v-if="canEdit && showNewForm" class="timeline-row timeline-row--edit">
      <div class="timeline-date-slot">
        <input v-model="newForm.atUtc" type="datetime-local" class="timeline-input timeline-input--date" />
      </div>
      <div class="timeline-axis" aria-hidden="true"><span class="timeline-dot" /></div>
      <div class="timeline-card timeline-entry-card timeline-entry-card--edit">
        <div class="timeline-entry-form">
          <input v-model="newForm.title" placeholder="Title" class="timeline-input" />
          <textarea v-model="newForm.details" placeholder="Details (optional)" class="timeline-input timeline-input--details" rows="2" />
        </div>
        <span class="timeline-entry-actions">
          <button class="timeline-action-btn timeline-action-btn--save" @click.stop="saveEntry(null)" :disabled="!newForm.title.trim()" :title="saveLabel"><Save :size="14" /></button>
          <button class="timeline-action-btn timeline-action-btn--cancel" @click.stop="cancelNewForm" :title="cancelLabel"><X :size="14" /></button>
        </span>
      </div>
    </article>

    <!-- Add button -->
    <button v-if="canEdit && !showNewForm" class="timeline-add-btn" @click="initNewForm">+ {{ addLabel }}</button>

    <!-- Existing entries -->
    <article
      v-for="entry in entries"
      :key="entry.id"
      class="timeline-row"
      :class="{ 'timeline-row--edit': editingId === entry.id }"
    >
      <div class="timeline-date-slot">
        <span v-if="entry.isPending" class="pending-badge">{{ entry.atLabel }}</span>
        <input
          v-else-if="editingId === entry.id"
          v-model="editForm.atUtc"
          type="datetime-local"
          class="timeline-input timeline-input--date"
        />
        <span v-else class="timeline-date-label">
          <span>{{ entry.atDateLabel }}</span>
          <span class="timeline-time-label">{{ entry.atTimeLabel }}</span>
        </span>
      </div>
      <div class="timeline-axis" aria-hidden="true"><span class="timeline-dot" /></div>
      <div class="timeline-card timeline-entry-card" :class="{ 'timeline-entry-card--edit': editingId === entry.id }">
        <template v-if="editingId === entry.id">
          <div class="timeline-entry-form">
            <input v-model="editForm.title" class="timeline-input" />
            <textarea v-model="editForm.details" class="timeline-input timeline-input--details" rows="2" />
          </div>
          <span class="timeline-entry-actions">
            <button class="timeline-action-btn timeline-action-btn--save" @click.stop="saveEntry(entry.id)" :disabled="!editForm.title.trim()" :title="saveLabel"><Save :size="14" /></button>
            <button class="timeline-action-btn timeline-action-btn--cancel" @click.stop="cancelEdit" :title="cancelLabel"><X :size="14" /></button>
          </span>
        </template>
        <template v-else>
          <h3 class="timeline-card-title timeline-entry-title">
            <CircleCheck v-if="!entry.isPending" class="timeline-entry-icon" :size="16" :stroke-width="2.4" aria-hidden="true" />
            <span>{{ entry.title }}</span>
          </h3>
          <p v-if="entry.details" class="timeline-entry-details">{{ entry.details }}</p>
          <span v-if="canEdit" class="timeline-entry-actions">
            <button class="timeline-action-btn timeline-action-btn--edit" @click.stop="startEdit(entry)" :title="editLabel"><Pencil :size="14" /></button>
            <button class="timeline-action-btn timeline-action-btn--delete" @click.stop="handleDelete(entry.id)" :title="deleteLabel"><Trash2 :size="14" /></button>
          </span>
        </template>
      </div>
    </article>
  </div>
</template>
```

- [ ] **Step 3: Add styles**

```css
<style scoped>
.TimelineList--editable .timeline-row--edit .timeline-card { background: rgba(72, 144, 255, 0.06); }

.TimelineList--editable .timeline-card { position: relative; }

.timeline-entry-actions {
  position: absolute;
  right: 0.35rem;
  top: 0.35rem;
  display: flex;
  gap: 0.2rem;
}

.timeline-action-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: none;
  border-radius: 0.3rem;
  padding: 0.2rem;
  cursor: pointer;
  color: #fff;
  width: 1.5rem;
  height: 1.5rem;
}

.timeline-action-btn--edit { background: rgba(72, 144, 255, 0.78); }
.timeline-action-btn--delete { background: rgba(220, 38, 38, 0.85); }
.timeline-action-btn--save { background: rgba(34, 197, 94, 0.85); }
.timeline-action-btn--cancel { background: rgba(127, 127, 127, 0.6); }

.timeline-action-btn:hover { filter: brightness(1.15); }

.timeline-input {
  border: 1px solid var(--control-border);
  border-radius: 0.4rem;
  padding: 0.3rem 0.45rem;
  background: var(--control-bg);
  color: var(--control-fg);
  font-size: 0.82rem;
  width: 100%;
  box-sizing: border-box;
}

.timeline-input--date { width: auto; min-width: 11rem; }
.timeline-input--details { resize: vertical; min-height: 2.2rem; font-family: inherit; }

.timeline-entry-form {
  display: grid;
  gap: 0.3rem;
  padding-right: 3rem;
}

.timeline-add-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.3rem;
  border: 1px dashed var(--control-border);
  border-radius: 0.5rem;
  padding: 0.3rem 0.7rem;
  background: transparent;
  color: var(--muted-fg);
  cursor: pointer;
  font-size: 0.82rem;
  font-weight: 600;
  margin-bottom: 0.5rem;
  width: fit-content;
}

.timeline-add-btn:hover { border-color: var(--timeline-accent); color: var(--timeline-accent); }
</style>
```

- [ ] **Step 4: Verify frontend compiles**

Run: `cd front; npx tsc --noEmit`
Expected: no errors

---

### Task 7: Frontend — Integrate into DetailPage.vue (all 3 entities)

**Files:**
- Modify: `front/src/incidents/DetailPage.vue`
- Modify: `front/src/projects/DetailPage.vue`
- Modify: `front/src/events/DetailPage.vue`

- [ ] **Step 1: Replace timeline rendering in `incidents/DetailPage.vue`**

Replace the inline timeline HTML (lines 147-168) with:

```html
<section v-if="model.timeline.length || canEdit" class="timeline-section">
  <h2>{{ t("labels.incidentTimeline") }}</h2>
  <EditableTimelineList
    :entries="model.timeline"
    :can-edit="canEdit"
    :entity-id="incidentId"
    entity-type="incidents"
    :locale="activeLocale()"
    @add="handleTimelineAdd"
    @update="handleTimelineUpdate"
    @delete="handleTimelineDelete"
  />
</section>
```

Add import and handler functions:

```typescript
import EditableTimelineList from "../common/components/EditableTimelineList.vue";

async function handleTimelineAdd(payload: { atUtc: string | null; sortOrder: number; fields: Record<string, string> }): Promise<void> {
  await apiIncidentsRepository.createTimelineEntry(incidentId.value, activeLocale(), payload);
  await loadIncident();
}

async function handleTimelineUpdate(entryId: string, payload: { atUtc: string | null; sortOrder: number; fields: Record<string, string> }): Promise<void> {
  await apiIncidentsRepository.updateTimelineEntry(incidentId.value, entryId, activeLocale(), payload);
  await loadIncident();
}

async function handleTimelineDelete(entryId: string): Promise<void> {
  await apiIncidentsRepository.deleteTimelineEntry(incidentId.value, entryId);
  await loadIncident();
}
```

- [ ] **Step 2: Replace timeline in `projects/DetailPage.vue`**

Replace `<TimelineList>` usage with `<EditableTimelineList>` + handler functions.

- [ ] **Step 3: Replace timeline in `events/DetailPage.vue`**

Same as step 2, using `apiEventsRepository`.

- [ ] **Step 4: Verify frontend compiles**

Run: `cd front; npx tsc --noEmit`
Expected: no errors

---

### Task 8: Frontend — Integrate into FormPage.vue (all 3 entities)

**Files:**
- Modify: `front/src/incidents/FormPage.vue`
- Modify: `front/src/projects/FormPage.vue`
- Modify: `front/src/events/FormPage.vue`

For FormPage, the timeline is part of the form state. Instead of creating view models and passing to `TimelineList`, the component:
1. Tracks timeline entries as a local `ref` in edit-item format
2. Converts to `TimelineEntry[]` view models for display in `EditableTimelineList`
3. On each add/update/delete, mutates the local timeline ref
4. On main save, converts the local timeline to save payload format and includes it with `replace_timeline: true`

- [ ] **Step 1: Replace in `incidents/FormPage.vue`**

Changes needed:
1. Replace `import TimelineList from "../common/components/TimelineList.vue"` with `import EditableTimelineList from "../common/components/EditableTimelineList.vue"`
2. Replace the `timeline` ref type — change from `ref<TimelineEntry[]>([])` to storing the raw edit items alongside view models, or store a `timelineItems` ref of edit items and derive view models
3. Remove the `formatTimeline` function — replace with simpler view model conversion
4. Remove the `field` helper function for timeline (the one specific to EditFieldValue)
5. Replace the timeline section template: remove `<TimelineList>` and add `<EditableTimelineList>` with the same props as DetailPage, but without the API calls — instead mutate local state
6. In the `save()` function, change from `replaceTimeline: false, timeline: []` to `replaceTimeline: true, timeline: timelineItems.value.map(item => ({ id: item.id, atUtc: item.atUtc, sortOrder: item.sortOrder, fields: Object.fromEntries(item.fields.map(f => [f.fieldKey, f.value])) }))`
7. Add handler functions that mutate `timelineItems`:
   - `handleTimelineAdd` — pushes new item with a generated UUID
   - `handleTimelineUpdate` — finds and updates the item by ID
   - `handleTimelineDelete` — filters out the deleted item by ID
   - After each mutation, rebuild the view model array
8. Keep the `v-if="timeline.length"` condition — timeline section should always show when `canEdit` is true too

- [ ] **Step 2: Same for `projects/FormPage.vue`**

- [ ] **Step 3: Same for `events/FormPage.vue`**

- [ ] **Step 4: Verify frontend compiles**

Run: `cd front; npx tsc --noEmit`
Expected: no errors

---

### Task 9: Add i18n labels for timeline actions

**Files:**
- Modify: `front/src/common/i18n.ts`

- [ ] **Step 1: Add English labels**

```typescript
addTimelineEntry: "Add timeline entry",
editTimelineEntry: "Edit entry",
deleteTimelineEntry: "Delete entry",
save: "Save",
cancel: "Cancel",
```

- [ ] **Step 2: Add French labels**

```typescript
addTimelineEntry: "Ajouter une entrée",
editTimelineEntry: "Modifier l'entrée",
deleteTimelineEntry: "Supprimer l'entrée",
save: "Enregistrer",
cancel: "Annuler",
```

---

### Task 10: Run verification

- [ ] **Step 1: Backend checks**

Run: `cd back; cargo clippy --all-features -- -D warnings; cargo test --all-features; cargo build --all-features`
Expected: all pass

- [ ] **Step 2: Frontend checks**

Run: `cd front; npm run lint; npm run test; npm run build`
Expected: all pass
