# Generated Keys Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Generate maintenance, incident, and project public keys in PostgreSQL while renaming user-facing identifiers from `code`/public `id` to `key` across the API and UI.

**Architecture:** Apply one migration that preserves existing readable values, converts category internal IDs to UUIDs, and adds table-local key sequences. Update backend models/repositories/services/handlers so `id` is internal UUID data and `key` is the public route/display identifier. Update frontend API mappers, forms, category admin, and tests to use generated keys and category UUID references.

**Tech Stack:** PostgreSQL migrations, Rust/Axum/sqlx backend, Vue 3/Vite/Vitest frontend.

---

## File Map

- Create `back/migrations/0013_generated_public_keys.sql`: schema/data migration for key columns, category UUID IDs, foreign keys, and sequences.
- Modify `back/src/categories/model.rs`, `service.rs`, `repository.rs`, `http.rs`: category `key` API shape, UUID generation, key validation, create response.
- Modify `back/src/maintenances/model.rs`, `service.rs`, `repository.rs`, `http.rs`: generated create keys and `key` responses.
- Modify `back/src/incidents/model.rs`, `service.rs`, `repository.rs`, `http.rs`: generated create keys and `key` responses while preserving timeline UUID behavior.
- Modify `back/src/projects/model.rs`, `service.rs`, `repository.rs`, `http.rs`: generated create keys and `key` responses.
- Modify `front/src/categories/types.ts`, `api.ts`, `CategoryBadge.vue`, `CategoryBadge.test.ts`, `front/src/admin/CategoriesPage.vue`, `CategoriesPage.test.ts`: category `key` naming and admin UX.
- Modify `front/src/events/**`, `front/src/incidents/**`, `front/src/projects/**`: public `key` fields, create forms without key inputs, save returns generated key.
- Modify `front/src/common/i18n.ts`: label text from public ID/code wording to key wording.

## Task 1: Database Migration

**Files:**
- Create: `back/migrations/0013_generated_public_keys.sql`

- [ ] **Step 1: Write migration**

Add this migration. It preserves existing category keys, converts category internal IDs to UUIDs, updates referencing rows, renames public identifier columns to `key`, and creates sequence defaults for future inserts.

```sql
ALTER TABLE maintenances RENAME COLUMN code TO key;
ALTER TABLE incidents RENAME COLUMN code TO key;
ALTER TABLE projects RENAME COLUMN code TO key;
ALTER TABLE event_categories RENAME COLUMN code TO key;

ALTER TABLE incidents DROP CONSTRAINT IF EXISTS incidents_category_code_fkey;
ALTER TABLE maintenances DROP CONSTRAINT IF EXISTS maintenances_category_code_fkey;
ALTER TABLE projects DROP CONSTRAINT IF EXISTS projects_category_code_fkey;

ALTER TABLE event_category_i18n DROP CONSTRAINT IF EXISTS event_category_i18n_category_id_fkey;

ALTER TABLE event_categories ADD COLUMN IF NOT EXISTS uuid_id UUID;
UPDATE event_categories SET uuid_id = gen_random_uuid() WHERE uuid_id IS NULL;

ALTER TABLE incidents ADD COLUMN IF NOT EXISTS category_uuid UUID;
UPDATE incidents i SET category_uuid = c.uuid_id FROM event_categories c WHERE i.category_code = c.id;

ALTER TABLE maintenances ADD COLUMN IF NOT EXISTS category_uuid UUID;
UPDATE maintenances m SET category_uuid = c.uuid_id FROM event_categories c WHERE m.category_code = c.id;

ALTER TABLE projects ADD COLUMN IF NOT EXISTS category_uuid UUID;
UPDATE projects p SET category_uuid = c.uuid_id FROM event_categories c WHERE p.category_code = c.id;

ALTER TABLE event_category_i18n ADD COLUMN IF NOT EXISTS category_uuid UUID;
UPDATE event_category_i18n ci SET category_uuid = c.uuid_id FROM event_categories c WHERE ci.category_id = c.id;

ALTER TABLE incidents ALTER COLUMN category_uuid SET NOT NULL;
ALTER TABLE maintenances ALTER COLUMN category_uuid SET NOT NULL;
ALTER TABLE projects ALTER COLUMN category_uuid SET NOT NULL;
ALTER TABLE event_category_i18n ALTER COLUMN category_uuid SET NOT NULL;

ALTER TABLE incidents DROP COLUMN category_code;
ALTER TABLE maintenances DROP COLUMN category_code;
ALTER TABLE projects DROP COLUMN category_code;
ALTER TABLE event_category_i18n DROP COLUMN category_id;

ALTER TABLE incidents RENAME COLUMN category_uuid TO category_id;
ALTER TABLE maintenances RENAME COLUMN category_uuid TO category_id;
ALTER TABLE projects RENAME COLUMN category_uuid TO category_id;
ALTER TABLE event_category_i18n RENAME COLUMN category_uuid TO category_id;

ALTER TABLE event_categories DROP CONSTRAINT event_categories_pkey;
ALTER TABLE event_categories DROP COLUMN id;
ALTER TABLE event_categories RENAME COLUMN uuid_id TO id;
ALTER TABLE event_categories ADD PRIMARY KEY (id);

ALTER TABLE event_category_i18n ADD CONSTRAINT event_category_i18n_category_id_fkey FOREIGN KEY (category_id) REFERENCES event_categories(id) ON DELETE CASCADE;
ALTER TABLE incidents ADD CONSTRAINT incidents_category_id_fkey FOREIGN KEY (category_id) REFERENCES event_categories(id) ON DELETE RESTRICT;
ALTER TABLE maintenances ADD CONSTRAINT maintenances_category_id_fkey FOREIGN KEY (category_id) REFERENCES event_categories(id) ON DELETE RESTRICT;
ALTER TABLE projects ADD CONSTRAINT projects_category_id_fkey FOREIGN KEY (category_id) REFERENCES event_categories(id) ON DELETE RESTRICT;

CREATE SEQUENCE IF NOT EXISTS maintenance_key_seq;
CREATE SEQUENCE IF NOT EXISTS incident_key_seq;
CREATE SEQUENCE IF NOT EXISTS project_key_seq;

SELECT setval('maintenance_key_seq', greatest(9, coalesce((SELECT max((regexp_match(key, '^EVT-([1-9][0-9]*)$'))[1]::bigint) FROM maintenances WHERE key ~ '^EVT-[1-9][0-9]*$'), 9)), true);
SELECT setval('incident_key_seq', greatest(9, coalesce((SELECT max((regexp_match(key, '^INC-([1-9][0-9]*)$'))[1]::bigint) FROM incidents WHERE key ~ '^INC-[1-9][0-9]*$'), 9)), true);
SELECT setval('project_key_seq', greatest(9, coalesce((SELECT max((regexp_match(key, '^PRJ-([1-9][0-9]*)$'))[1]::bigint) FROM projects WHERE key ~ '^PRJ-[1-9][0-9]*$'), 9)), true);

CREATE INDEX IF NOT EXISTS incidents_category_idx ON incidents(category_id);
CREATE INDEX IF NOT EXISTS maintenances_category_idx ON maintenances(category_id);
DROP INDEX IF EXISTS projects_category_idx;
CREATE INDEX IF NOT EXISTS projects_category_idx ON projects(category_id);
```

- [ ] **Step 2: Validate SQL references**

Run: `rg "category_code|\.code| code" back/migrations back/src -g "*.sql" -g "*.rs"`

Expected: only old migrations before `0013` and not-yet-updated Rust references show matches.

## Task 2: Backend Category API

**Files:**
- Modify: `back/src/categories/model.rs`
- Modify: `back/src/categories/service.rs`
- Modify: `back/src/categories/repository.rs`
- Modify: `back/src/categories/http.rs`

- [ ] **Step 1: Update category models**

Replace `code` with `key`, remove create `id`, and keep update by route UUID.

```rust
#[derive(Serialize)]
pub struct CategoryItem {
    pub id: String,
    pub key: String,
    pub icon: String,
    pub color: String,
    pub label: String,
    pub labels: HashMap<String, String>,
}

#[derive(Deserialize)]
pub struct CategoryCreateRequest {
    pub key: String,
    pub icon: String,
    pub color: String,
    pub labels: HashMap<String, String>,
}

#[derive(Deserialize)]
pub struct CategoryUpdateRequest {
    pub key: String,
    pub icon: String,
    pub color: String,
    pub labels: HashMap<String, String>,
}
```

- [ ] **Step 2: Add failing validation tests**

In `back/src/categories/service.rs`, add tests for uppercase normalization and rejection.

```rust
#[test]
fn normalize_category_key_uppercases_trimmed_ascii_key() {
    assert_eq!(super::normalize_category_key_for_test(" hea ").expect("valid key"), "HEA");
}

#[test]
fn normalize_category_key_rejects_invalid_values() {
    for value in ["AB", "ABCDEF", "A-B", "ééé"] {
        let error = super::normalize_category_key_for_test(value).expect_err("invalid key");
        assert_eq!(error.message, "category key must be 3 to 5 ASCII alphanumeric characters");
    }
}
```

Run: `cargo test --all-features categories::service::tests::normalize_category_key -- --nocapture`

Expected: FAIL because helper is not implemented.

- [ ] **Step 3: Implement service changes**

Use `Uuid::new_v4()` in create, normalize category keys, and return `CategoryItem` from create/update.

```rust
fn normalize_category_key(value: &str) -> Result<String, AppError> {
    let value = normalize_text_value(value).to_uppercase();
    let valid = (3..=5).contains(&value.len()) && value.chars().all(|ch| ch.is_ascii_alphanumeric());
    if valid { Ok(value) } else { Err(AppError::bad_request("category key must be 3 to 5 ASCII alphanumeric characters")) }
}

#[cfg(test)]
fn normalize_category_key_for_test(value: &str) -> Result<String, AppError> { normalize_category_key(value) }
```

- [ ] **Step 4: Update repository SQL**

Change SQL from `code` to `key`, references from `category_code` to `category_id`, and make `create`/`update` return `CategoryItem` by calling a helper query after writing.

```sql
SELECT c.id, c.key, c.icon, c.color, coalesce((...), c.key) AS label FROM event_categories c ORDER BY c.key ASC
```

- [ ] **Step 5: Run category tests**

Run: `cargo test --all-features categories -- --nocapture`

Expected: PASS.

## Task 3: Backend Generated Keys For Maintenance, Incidents, And Projects

**Files:**
- Modify: `back/src/maintenances/model.rs`, `service.rs`, `repository.rs`, `http.rs`
- Modify: `back/src/incidents/model.rs`, `service.rs`, `repository.rs`, `http.rs`
- Modify: `back/src/projects/model.rs`, `service.rs`, `repository.rs`, `http.rs`

- [ ] **Step 1: Update response/request model shape**

For each domain, rename public response field `id` to `key`, rename `category_code` to `category_id`, rename `CategoryDisplay.code` to `key`, and remove `id` from create request structs. Add a shared create response in each model file.

```rust
#[derive(Serialize)]
pub struct CreatedKeyResponse { pub key: String }
```

For update flows, keep a save request that can receive the route key from the handler. The simplest low-risk shape is `pub key: Option<String>` with create ignoring it and update setting it.

- [ ] **Step 2: Update service validation**

Do not require public keys on create. Continue requiring `category_id`. For update, require the route key assigned by the handler before repository update.

```rust
if validated.category_id.is_empty() { return Err(AppError::bad_request("category_id is required")); }
```

- [ ] **Step 3: Update repository insert SQL**

Create paths must generate keys and return them. Update paths must use the route key and must not alter it. Split current upsert methods into explicit `create` and `update` branches, or keep one method with `payload.key` deciding branch.

Maintenance create SQL:

```sql
INSERT INTO maintenances (id, key, category_id, start_utc, end_utc, notified_at_utc, updated_at, last_modified_at, last_modified_by_user_id)
VALUES ($1, 'EVT-' || nextval('maintenance_key_seq'), $2, $3, $4, $5, now(), now(), $6)
RETURNING id, key
```

Incident create SQL:

```sql
INSERT INTO incidents (id, key, category_id, start_utc, end_utc, updated_at, last_modified_at, last_modified_by_user_id)
VALUES ($1, 'INC-' || nextval('incident_key_seq'), $2, $3, $4, now(), now(), $5)
RETURNING id, key
```

Project create SQL:

```sql
INSERT INTO projects (id, key, category_id, start_utc, end_utc, status_type, updated_at, last_modified_at, last_modified_by_user_id)
VALUES ($1, 'PRJ-' || nextval('project_key_seq'), $2, $3, $4, $5, now(), now(), $6)
RETURNING id, key
```

- [ ] **Step 4: Update route lookups and searches**

Change `WHERE code = $1` to `WHERE key = $1`, joins to `c.id = *.category_id`, display to `c.key`, and search to include public `*.key` plus category `c.key`/label.

- [ ] **Step 5: Update HTTP create handlers**

Return JSON with status `201 Created` instead of an empty status.

```rust
pub async fn create(...) -> Result<(StatusCode, Json<CreatedKeyResponse>), AppError> {
    principal.ensure_any_role(&[Role::Admin, Role::CoOwnershipBoard])?;
    let key = service::create(&state.db, &payload, principal.user_id).await?;
    Ok((StatusCode::CREATED, Json(CreatedKeyResponse { key })))
}
```

- [ ] **Step 6: Run backend compile check**

Run: `cargo test --all-features --no-run`

Expected: PASS compile, or compiler errors only in frontend-unrelated Rust files that this task then fixes.

## Task 4: Frontend Category Rename And Admin UX

**Files:**
- Modify: `front/src/categories/types.ts`
- Modify: `front/src/categories/api.ts`
- Modify: `front/src/categories/CategoryBadge.vue`
- Modify: `front/src/categories/CategoryBadge.test.ts`
- Modify: `front/src/admin/CategoriesPage.vue`
- Modify: `front/src/admin/CategoriesPage.test.ts`

- [ ] **Step 1: Update category types**

```ts
export type CategoryItem = { id: string; key: string; icon: string; color: string; label: string; labels: Record<string, string> };
export type CategoryInput = { key: string; icon: string; color: string; labels: Record<string, string> };
```

- [ ] **Step 2: Update mock category payloads**

In `listCategories`, return `{ id: "0000000-...", key: "HEA", ... }` in test mode. Use keys in dropdowns and badges.

- [ ] **Step 3: Update admin form**

Initialize form without `id`, render a category key input, remove the category ID input, and display `category.id.slice(0, 7)` in the table.

```ts
const form = ref<CategoryInput>({ key: "", icon: "tag", color: defaultColor, labels: { en: "", fr: "" } });
function reset(): void { editingId.value = ""; form.value = { key: "", icon: "tag", color: defaultColor, labels: { en: "", fr: "" } }; }
function edit(category: CategoryItem): void { editingId.value = category.id; form.value = { key: category.key, icon: category.icon, color: category.color, labels: { en: category.labels.en ?? "", fr: category.labels.fr ?? "" } }; }
```

- [ ] **Step 4: Run frontend category tests**

Run: `npm run test -- --run src/admin/CategoriesPage.test.ts src/categories/CategoryBadge.test.ts`

Expected: PASS.

## Task 5: Frontend Generated Keys For Events, Incidents, And Projects

**Files:**
- Modify: `front/src/events/types.ts`, `repositories/apiEventsRepository.ts`, `FormPage.vue`, tests.
- Modify: `front/src/incidents/types.ts`, `repositories/apiIncidentsRepository.ts`, `FormPage.vue`, tests.
- Modify: `front/src/projects/types.ts`, `repositories/apiProjectsRepository.ts`, `FormPage.vue`, tests.

- [ ] **Step 1: Rename public fields in API mappers**

API item types use `key`, `category_id`, and `category?: { id; key; icon; color; label }`. Frontend domain objects may keep `id` as the route key if changing every route consumer is too broad, but API payloads must map from `key`.

```ts
type ApiCreatedKeyResponse = { key: string };
```

- [ ] **Step 2: Make repository save return generated key**

Change repository interfaces and implementations from `Promise<void>` to `Promise<string | void>` for save, returning a key on create and nothing on update.

```ts
async function sendJson<T>(url: string, method: string, body?: unknown): Promise<T | null> {
  const response = await fetch(url, { method, headers: { ...authHeaders(), "Content-Type": "application/json" }, body: body === undefined ? undefined : JSON.stringify(body) });
  if (!response.ok) throw new Error(`request failed with status ${response.status}`);
  return response.status === 204 ? null : ((await response.json()) as T);
}
```

- [ ] **Step 3: Remove public key inputs from create forms**

In each form, render public key as read-only only when editing. For create, omit it entirely. Remove `id` from create payloads.

```vue
<p v-if="isEdit" class="metadata-row"><strong>{{ t("labels.key") }}</strong><span>{{ form.id }}</span></p>
```

- [ ] **Step 4: Route after create with returned key**

```ts
const createdKey = await apiEventsRepository.save(payload, isEdit.value ? existingId.value : undefined);
await router.push(`/events/${encodeURIComponent(isEdit.value ? existingId.value : String(createdKey))}`);
```

Use equivalent paths for incidents and projects.

- [ ] **Step 5: Run targeted frontend tests**

Run: `npm run test -- --run src/events/FormPage.test.ts src/incidents/FormPage.test.ts src/projects/FormPage.test.ts src/events/repositories/apiEventsRepository.test.ts src/incidents/repositories/apiIncidentsRepository.test.ts src/projects/repositories/apiProjectsRepository.test.ts`

Expected: PASS.

## Task 6: Labels, Search, And Final Verification

**Files:**
- Modify: `front/src/common/i18n.ts`
- Search/update: remaining frontend/backend `code` references where they are user-facing category/public identifiers.

- [ ] **Step 1: Update labels**

Change labels such as `eventId`, `projectId`, `categoryCode`, and `categoryId` where user-facing to key wording. Keep internal UUID table column labels as `ID` in admin category list.

- [ ] **Step 2: Repository-wide search**

Run: `rg "category_code|categoryCode|\.code| code:|Project ID|Event ID|categoryCode" back/src front/src`

Expected: no active generated-key API references remain. Only unrelated text or old migrations may remain.

- [ ] **Step 3: Run frontend checks**

Run in `front/`: `npm run lint`

Expected: PASS.

Run in `front/`: `npm run test`

Expected: PASS.

Run in `front/`: `npm run build`

Expected: PASS.

- [ ] **Step 4: Run backend checks**

Run in `back/`: `cargo clippy --all-features -- -D warnings`

Expected: PASS.

Run in `back/`: `cargo test --all-features`

Expected: PASS.

Run in `back/`: `cargo build --all-features`

Expected: PASS.

## Notes

- Do not commit during this plan unless the user explicitly requests a commit.
- Do not add backward-compatible `code` aliases.
- Do not renumber existing maintenance, incident, or project keys.
