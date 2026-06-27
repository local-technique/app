# Timeline Entry Permissions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Restrict timeline entry edit/delete to the original creator, Admin, or CoOwnershipBoardOps, enforced backend + reflected frontend.

**Architecture:** Add `created_by_user_id` column to all 3 timeline tables. Backend service layer checks ownership before update/delete. Frontend passes `currentUserId` to `EditableTimelineList` for per-entry button visibility. Avatar shows creator (not last modifier); tooltip shows both.

**Tech Stack:** Rust/axum/sqlx/postgres backend, Vue 3/TypeScript frontend, lucide-vue icons.

---

### Task 1: New migration — add `created_by_user_id` to timeline tables

**Files:**
- Create: `back/migrations/0021_add_timeline_creator.sql`

- [ ] **Step 1: Create migration file**

```sql
ALTER TABLE incident_timeline ADD COLUMN created_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL;
ALTER TABLE maintenance_timeline ADD COLUMN created_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL;
ALTER TABLE project_timeline ADD COLUMN created_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL;

UPDATE incident_timeline SET created_by_user_id = last_modified_by_user_id WHERE created_by_user_id IS NULL;
UPDATE maintenance_timeline SET created_by_user_id = last_modified_by_user_id WHERE created_by_user_id IS NULL;
UPDATE project_timeline SET created_by_user_id = last_modified_by_user_id WHERE created_by_user_id IS NULL;
```

- [ ] **Step 2: Run the migration**

Run: `cd back && sqlx migrate run`
Expected: migration 0021 applied

---

### Task 2: Backend — add `created_by` to timeline response models + `id` to `MeResponse`

**Files:**
- Modify: `back/src/auth/model.rs`
- Modify: `back/src/auth/service.rs`
- Modify: `back/src/incidents/model.rs`
- Modify: `back/src/maintenances/model.rs`
- Modify: `back/src/projects/model.rs`

- [ ] **Step 1: Add `id` field to `MeResponse`**

In `back/src/auth/model.rs`, add to `MeResponse`:
```rust
pub struct MeResponse {
    pub id: String,
    pub provider: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub roles: Vec<String>,
}
```

- [ ] **Step 2: Return `user.id` in `me()` service**

In `back/src/auth/service.rs`, in the `me()` function, change to:
```rust
    Ok(MeResponse {
        id: user.id.to_string(),
        provider: user.provider,
        email: user.email,
        first_name: user.first_name,
        last_name: user.last_name,
        roles: user.roles,
    })
```

- [ ] **Step 3: Add `created_by: Option<AuditUser>` to `IncidentTimelineItem`**

In `back/src/incidents/model.rs`:
```rust
pub struct IncidentTimelineItem {
    pub id: String,
    pub at_utc: Option<String>,
    pub title: String,
    pub details: String,
    pub created_by: Option<AuditUser>,
    pub last_modified_by: Option<AuditUser>,
}
```

- [ ] **Step 4: Add `created_by: Option<AuditUser>` to `MaintenanceTimelineItem`**

Same change in `back/src/maintenances/model.rs`:
```rust
pub struct MaintenanceTimelineItem {
    pub id: String,
    pub at_utc: Option<String>,
    pub title: String,
    pub details: String,
    pub created_by: Option<AuditUser>,
    pub last_modified_by: Option<AuditUser>,
}
```

- [ ] **Step 5: Add `created_by: Option<AuditUser>` to `ProjectTimelineItem`**

Same change in `back/src/projects/model.rs`:
```rust
pub struct ProjectTimelineItem {
    pub id: String,
    pub at_utc: Option<String>,
    pub title: String,
    pub details: String,
    pub created_by: Option<AuditUser>,
    pub last_modified_by: Option<AuditUser>,
}
```

---

### Task 3: Backend — incident repository changes for `created_by_user_id`

**Files:**
- Modify: `back/src/incidents/repository.rs`

- [ ] **Step 1: Update `by_id` timeline query to select `created_by` user**

In the timeline query within `by_id()`, add `created_by` user columns:
```sql
SELECT
  t.id,
  t.at_utc,
  tu.id AS tl_user_id,
  tu.email AS tl_user_email,
  tu.first_name AS tl_user_first_name,
  tu.last_name AS tl_user_last_name,
  cu.id AS tl_created_by_id,
  cu.email AS tl_created_by_email,
  cu.first_name AS tl_created_by_first_name,
  cu.last_name AS tl_created_by_last_name,
  ...
FROM incident_timeline t
LEFT JOIN users tu ON tu.id = t.last_modified_by_user_id
LEFT JOIN users cu ON cu.id = t.created_by_user_id
WHERE t.incident_id = $1
```

In the timeline mapping, add `created_by` alongside `last_modified_by`:
```rust
let created_by_id: Option<Uuid> = value.try_get("tl_created_by_id")?;
let created_by_email: Option<String> = value.try_get("tl_created_by_email")?;
let created_by_first_name: Option<String> = value.try_get("tl_created_by_first_name")?;
let created_by_last_name: Option<String> = value.try_get("tl_created_by_last_name")?;
// ... use created_by_* for the `created_by` field similar to last_modified_by
```

- [ ] **Step 2: Update `create_timeline_entry` to set `created_by_user_id`**

In the INSERT:
```sql
INSERT INTO incident_timeline (id, incident_id, at_utc, sort_order, last_modified_by_user_id, created_by_user_id)
VALUES ($1, $2, $3, $4, $5, $6)
```
Add the `user_id` binding as a 6th parameter for `created_by_user_id`.

In the result query after creation, also select `created_by` user info:
```sql
SELECT
  ...,
  u.id AS tu_id,
  u.email AS tu_email,
  u.first_name AS tu_first_name,
  u.last_name AS tu_last_name,
  cu.id AS tl_created_by_id,
  cu.email AS tl_created_by_email,
  cu.first_name AS tl_created_by_first_name,
  cu.last_name AS tl_created_by_last_name
FROM incident_timeline t
LEFT JOIN users u ON u.id = t.last_modified_by_user_id
LEFT JOIN users cu ON cu.id = t.created_by_user_id
WHERE t.id = $1
```

- [ ] **Step 3: Update `update_timeline_entry` to also select `created_by`**

After the update query runs, in the result SELECT:
```sql
SELECT
  ...,
  cu.id AS tl_created_by_id,
  cu.email AS tl_created_by_email,
  cu.first_name AS tl_created_by_first_name,
  cu.last_name AS tl_created_by_last_name
FROM incident_timeline t
LEFT JOIN users cu ON cu.id = t.created_by_user_id
WHERE t.id = $1
```

- [ ] **Step 4: Update `save_partial` upsert to preserve `created_by_user_id`**

The upsert in `save_partial()` should include `created_by_user_id` in the INSERT columns but NOT in the ON CONFLICT SET clause:
```sql
INSERT INTO incident_timeline (id, incident_id, at_utc, sort_order, last_modified_by_user_id, created_by_user_id)
VALUES ($1, $2, $3, $4, $5, $6)
ON CONFLICT (id) DO UPDATE SET at_utc = EXCLUDED.at_utc, sort_order = EXCLUDED.sort_order, last_modified_by_user_id = EXCLUDED.last_modified_by_user_id
WHERE incident_timeline.incident_id = EXCLUDED.incident_id
```
Add binding for `user_id` as 6th param. Since `created_by_user_id` is not in the SET clause, existing rows keep their original `created_by_user_id`; new rows get it set.

---

### Task 4: Backend — maintenance repository changes for `created_by_user_id`

**Files:**
- Modify: `back/src/maintenances/repository.rs`

Make the identical changes as in Task 3 but for the maintenance_timeline tables. Same SQL patterns (replace `incident_timeline` with `maintenance_timeline`, `incident_timeline_i18n` with `maintenance_timeline_i18n`).

---

### Task 5: Backend — project repository changes for `created_by_user_id`

**Files:**
- Modify: `back/src/projects/repository.rs`

Make the identical changes as in Task 3 but for the project_timeline tables.

---

### Task 6: Backend — service layer authorization for incidents

**Files:**
- Modify: `back/src/incidents/service.rs`

- [ ] **Step 1: Add authorization to `update_timeline_entry`**

Before calling the repository update, check permissions. Add a helper function or inline check:

```rust
use crate::common::role::Role;

async fn check_timeline_authorization(
    db: &sqlx::PgPool,
    entry_id: &str,
    principal: &Principal,
) -> Result<(), AppError> {
    let created_by: Option<uuid::Uuid> = sqlx::query_scalar(
        "SELECT created_by_user_id FROM incident_timeline WHERE id = $1::uuid",
    )
    .bind(entry_id)
    .fetch_optional(db)
    .await?
    .flatten();

    let is_admin = principal.roles.iter().any(|r| r == Role::Admin.code());
    let is_ops = principal.roles.iter().any(|r| r == Role::CoOwnershipBoardOps.code());
    let is_creator = created_by == Some(principal.user_id);

    if is_admin || is_ops || is_creator {
        Ok(())
    } else {
        Err(AppError::forbidden("not allowed to edit this timeline entry"))
    }
}
```

Call this from `update_timeline_entry` before the repository call, and from `delete_timeline_entry`.

- [ ] **Step 2: Add authorization to `delete_timeline_entry`**

Same check as in Step 1 — call `check_timeline_authorization` before deleting.

---

### Task 7: Backend — service layer authorization for maintenances

**Files:**
- Modify: `back/src/maintenances/service.rs`

Same changes as Task 6, but for maintenance_timeline tables.

---

### Task 8: Backend — service layer authorization for projects

**Files:**
- Modify: `back/src/projects/service.rs`

Same changes as Task 6, but for project_timeline tables.

---

### Task 9: Backend — HTTP handler permissions (all 3 items)

**Files:**
- Modify: `back/src/incidents/http.rs`
- Modify: `back/src/maintenances/http.rs`
- Modify: `back/src/projects/http.rs`

No changes needed to handler role checks — the coarse `ensure_any_role(&[Admin, CoOwnershipBoard, CoOwnershipBoardOps])` remains as the gate, and the fine-grained check happens in the service layer.

However, the `delete_timeline` function should pass the `principal` and `entry_id` to the service for the ownership check.

- [ ] **Step 1: Update `delete_timeline` in incidents/http.rs**

Change from:
```rust
service::delete_timeline_entry(&state.db, &id, &entry_id).await?;
```
To:
```rust
service::delete_timeline_entry(&state.db, &id, &entry_id, &principal).await?;
```

- [ ] **Step 2: Same change for maintenances/http.rs**

- [ ] **Step 3: Same change for projects/http.rs**

- [ ] **Step 4: Update each service's `delete_timeline_entry` signature**

```rust
pub async fn delete_timeline_entry(
    db: &sqlx::PgPool,
    incident_code: &str,
    entry_id: &str,
    principal: &Principal,
) -> Result<(), AppError> {
    // authorize
    check_timeline_authorization(db, entry_id, principal).await?;
    // ... rest of function
}
```

---

### Task 10: Frontend — session types and current user ID

**Files:**
- Modify: `front/src/auth/session.ts`

- [ ] **Step 1: Add `currentUserId` to session**

```typescript
export const currentUserId = reactive<{ value: string | null }>({ value: null });
```

Update the `MeResponse` type:
```typescript
type MeResponse = {
  id?: string;
  roles?: string[];
};
```

Update `fetchCurrentUserRoles`:
```typescript
const payload = (await response.json()) as MeResponse;
currentUserId.value = payload.id ?? null;
currentUserRoles.roles = Array.isArray(payload.roles) ? payload.roles : [];
```

Update `clearSession`:
```typescript
currentUserId.value = null;
```

- [ ] **Step 2: Add `getCurrentUserId` export**

```typescript
export function getCurrentUserId(): string | null {
  return currentUserId.value;
}
```

---

### Task 11: Frontend — add `createdBy` to timeline entry API types

**Files:**
- Modify: `front/src/incidents/types.ts`
- Modify: `front/src/events/types.ts`
- Modify: `front/src/projects/types.ts`

- [ ] **Step 1: Update `IncidentTimelineEntry` type**

```typescript
export type IncidentTimelineEntry = {
  id: string;
  atUtc: string | null;
  title: IncidentLocalizedText;
  details?: IncidentLocalizedText;
  createdBy?: { id: string; email: string; firstName?: string | null; lastName?: string | null } | null;
  lastModifiedBy?: { id: string; email: string; firstName?: string | null; lastName?: string | null } | null;
};
```

- [ ] **Step 2: Same for `EventTimelineEntry`**

- [ ] **Step 3: Same for `ProjectTimelineEntry`**

---

### Task 12: Frontend — API repository mapping for `created_by`

**Files:**
- Modify: `front/src/incidents/repositories/apiIncidentsRepository.ts`
- Modify: `front/src/events/repositories/apiEventsRepository.ts`
- Modify: `front/src/projects/repositories/apiProjectsRepository.ts`

- [ ] **Step 1: Add `created_by` to `ApiIncidentTimelineItem`**

```typescript
type ApiIncidentTimelineItem = {
  id: string;
  at_utc: string | null;
  title: string;
  details: string;
  created_by?: { id: string; email: string; first_name?: string | null; last_name?: string | null } | null;
  last_modified_by?: { id: string; email: string; first_name?: string | null; last_name?: string | null } | null;
};
```

- [ ] **Step 2: Map `created_by` in `toTimelineEntry`**

```typescript
function toTimelineEntry(locale: LocaleCode, item: ApiIncidentTimelineItem): IncidentTimelineEntry {
  return {
    id: item.id,
    atUtc: item.at_utc,
    title: localized(locale, item.title ?? ""),
    details: localized(locale, item.details ?? ""),
    createdBy: mapUserRef(item.created_by),
    lastModifiedBy: mapUserRef(item.last_modified_by),
  };
}
```

- [ ] **Step 3: Same changes for events API repository**

- [ ] **Step 4: Same changes for projects API repository**

---

### Task 13: Frontend — utils/viewmodel changes to pass `createdBy` through

**Files:**
- Modify: `front/src/incidents/utils.ts`
- Modify: `front/src/events/utils.ts`
- Modify: `front/src/projects/utils.ts`

- [ ] **Step 1: Update `TimelineEntryViewModel` type in incidents utils**

The `TimelineEntry` type in `TimelineList.vue` already has `lastModifiedBy?: { initials; fullName } | null`. We need to:
- Add `createdBy: { initials: string; fullName: string; id: string } | null` and `lastModifiedBy: { initials: string; fullName: string } | null`
- In `toTimelineEntryViewModel`, compute both `createdBy` and `lastModifiedBy` display info

```typescript
export type IncidentTimelineEntryViewModel = {
  id: string;
  atUtc: string | null;
  atLabel: string;
  atDateLabel: string;
  atTimeLabel: string;
  isPending: boolean;
  title: string;
  details: string;
  createdBy?: { initials: string; fullName: string; id: string } | null;
  lastModifiedBy?: { initials: string; fullName: string } | null;
};
```

Update `toTimelineEntryViewModel`:
```typescript
function toTimelineEntryViewModel(entry: IncidentTimelineEntry, locale: LocaleCode): IncidentTimelineEntryViewModel {
  const atDate = entry.atUtc ? parseUtc(entry.atUtc) : null;

  // Helper to compute display info
  function toUserDisplay(user: { id: string; email: string; firstName?: string | null; lastName?: string | null } | null | undefined): { initials: string; fullName: string; id: string } | null {
    if (!user) return null;
    const firstChar = user.firstName?.[0] ?? user.lastName?.[0] ?? user.email[0] ?? '';
    const lastChar = user.firstName && user.lastName ? user.lastName[0] : null;
    const initials = firstChar && lastChar ? `${firstChar}${lastChar}`.toUpperCase() : firstChar.toUpperCase();
    const fullName = user.firstName && user.lastName ? `${user.firstName} ${user.lastName}` : (user.firstName ?? user.lastName ?? user.email ?? '');
    return { initials, fullName, id: user.id };
  }

  const createdBy = toUserDisplay(entry.createdBy);
  const lastModifiedBy = entry.lastModifiedBy ? toUserDisplay(entry.lastModifiedBy) : null;

  return {
    id: entry.id,
    atUtc: entry.atUtc,
    atLabel: atDate ? formatLocalDateTime(atDate, locale) : "Pending",
    atDateLabel: atDate ? new Intl.DateTimeFormat(locale, { dateStyle: "medium" }).format(atDate) : "",
    atTimeLabel: atDate ? new Intl.DateTimeFormat(locale, { timeStyle: "short" }).format(atDate) : "",
    isPending: !entry.atUtc,
    title: resolve(entry.title, locale),
    details: resolve(entry.details, locale),
    createdBy: createdBy ? { initials: createdBy.initials, fullName: createdBy.fullName, id: createdBy.id } : null,
    lastModifiedBy: createdBy && lastModifiedBy && createdBy.id !== lastModifiedBy.id
      ? { initials: lastModifiedBy.initials, fullName: lastModifiedBy.fullName }
      : null,
  };
}
```

Note: `lastModifiedBy` is only set in the view model when it's different from `createdBy` (to avoid redundant display).

- [ ] **Step 2: Same changes for events utils**

- [ ] **Step 3: Same changes for projects utils**

---

### Task 14: Frontend — update `TimelineEntry` type in `TimelineList.vue`

**Files:**
- Modify: `front/src/common/components/TimelineList.vue`

- [ ] **Step 1: Add `createdBy` and update `lastModifiedBy` types**

```typescript
export type TimelineEntry = {
  id: string;
  atUtc: string | null;
  atLabel: string;
  atDateLabel: string;
  atTimeLabel: string;
  isPending: boolean;
  title: string;
  details: string;
  createdBy?: { initials: string; fullName: string; id: string } | null;
  lastModifiedBy?: { initials: string; fullName: string } | null;
};
```

No template changes needed for `TimelineList.vue` since it only shows the avatar without edit buttons. The avatar currently shows `entry.lastModifiedBy` — change it to `entry.createdBy`:

```html
<span v-if="entry.createdBy" class="tl-user-avatar" :title="entry.createdBy.fullName">{{ entry.createdBy.initials }}</span>
```

Add tooltip second line if `lastModifiedBy` exists:
```html
<span v-if="entry.createdBy" class="tl-user-avatar"
  :title="entry.lastModifiedBy ? entry.createdBy.fullName + '\n' + $t('labels.lastEditedBy', { name: entry.lastModifiedBy.fullName }) : entry.createdBy.fullName">
  {{ entry.createdBy.initials }}
</span>
```

You may need to add a translation key `lastEditedBy` (e.g., `"Last edited by {name}"`).

---

### Task 15: Frontend — update `EditableTimelineList.vue` for per-entry permissions

**Files:**
- Modify: `front/src/common/components/EditableTimelineList.vue`

- [ ] **Step 1: Add `currentUserId` prop and update props**

```typescript
const props = defineProps<{
  entries: TimelineEntry[];
  canEdit: boolean;
  currentUserId: string | null;
}>();
```

- [ ] **Step 2: Add per-entry permission helper**

```typescript
function canEditEntry(entry: TimelineEntry): boolean {
  if (!props.currentUserId) return false;
  if (!entry.createdBy) return false;
  // Admin and CoOwnershipBoardOps have full access
  if (hasRole("ADMIN") || hasRole("CO_OWNERSHIP_BOARD_OPS")) return true;
  // Creator can edit their own entries
  return entry.createdBy.id === props.currentUserId;
}
```

You'll need to import `hasRole` from session:
```typescript
import { hasRole } from "../../auth/session";
```

- [ ] **Step 3: Update template — use `canEditEntry` for per-entry buttons**

The "add" button stays gated by `canEdit`. The edit/delete buttons on each entry become gated by `canEditEntry(entry)`:

```html
<div v-if="canEditEntry(entry)" class="timeline-entry-actions">
  <button class="timeline-action-btn" style="background: rgba(72, 144, 255, 0.78)" @click="startEdit(entry)">
    <Pencil :size="14" />
  </button>
  <button class="timeline-action-btn" style="background: rgba(220, 38, 38, 0.85)" @click="emit('delete', entry.id)">
    <Trash2 :size="14" />
  </button>
</div>
```

- [ ] **Step 4: Update avatar to show `createdBy` instead of `lastModifiedBy`**

Change:
```html
<span v-if="entry.lastModifiedBy" class="tl-user-avatar" :title="entry.lastModifiedBy.fullName">{{ entry.lastModifiedBy.initials }}</span>
```
To:
```html
<span v-if="entry.createdBy" class="tl-user-avatar"
  :title="entry.lastModifiedBy ? entry.createdBy.fullName + '\n' + t('labels.lastEditedBy', { name: entry.lastModifiedBy.fullName }) : entry.createdBy.fullName">
  {{ entry.createdBy.initials }}
</span>
```

- [ ] **Step 5: Also update the edit form "editing" display — keep the edit card's user avatar showing the creator**

The editing form template section already shows Save/Cancel buttons instead of Pencil/Trash — this still works. Just make sure the non-editing display shows `entry.createdBy`.

---

### Task 16: Frontend — pass `currentUserId` from detail pages

**Files:**
- Modify: `front/src/incidents/DetailPage.vue`
- Modify: `front/src/events/DetailPage.vue`
- Modify: `front/src/projects/DetailPage.vue`

- [ ] **Step 1: Import `getCurrentUserId`**

```typescript
import { currentUserRoles, getCurrentUserId, hasAnyRole } from "../auth/session";
```

- [ ] **Step 2: Pass `currentUserId` to EditableTimelineList**

```html
<EditableTimelineList
  :entries="model.timeline"
  :can-edit="canEdit"
  :current-user-id="getCurrentUserId()"
  @add="handleTimelineAdd"
  @update="handleTimelineUpdate"
  @delete="handleTimelineDelete"
/>
```

- [ ] **Step 3: Same changes for events DetailPage.vue**

- [ ] **Step 4: Same changes for projects DetailPage.vue**

---

### Task 17: Frontend — add translation key

**Files:**
- Modify: Check both `en.json` and `fr.json` translation files

- [ ] **Step 1: Add `lastEditedBy` translation**

In `en.json`:
```json
"lastEditedBy": "Last edited by {name}"
```

In `fr.json`:
```json
"lastEditedBy": "Dernière modification par {name}"
```

Find the translation files in the codebase (likely under `front/src/**/i18n/` or similar).

---

### Task 18: Verify and build

- [ ] **Step 1: Backend build check**

Run: `cd back && cargo build --all-features`
Expected: compiles without warnings

- [ ] **Step 2: Backend tests**

Run: `cd back && cargo test --all-features`
Expected: all tests pass

- [ ] **Step 3: Frontend lint**

Run: `cd front && npm run lint`
Expected: no errors

- [ ] **Step 4: Frontend build**

Run: `cd front && npm run build`
Expected: builds successfully
