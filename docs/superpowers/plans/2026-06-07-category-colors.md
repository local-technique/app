# Category Colors Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Persist category colors and use them for category badges and listing card left borders across maintenance, incidents, and projects.

**Architecture:** Add `color` to the category database/API model, propagate it through all category-bearing responses, and render it through the shared `CategoryBadge`. Add a focused admin color picker component with GitHub-label-style swatch, hex input, default palette, and `RefreshCcw` random color button.

**Tech Stack:** Rust, Axum, SQLx, PostgreSQL migrations, Vue 3 SFCs, TypeScript, Vitest, Testing Library Vue, Lucide Vue.

---

## Files

- Create: `back/migrations/0012_category_colors.sql`
- Modify: `back/src/categories/model.rs`, `back/src/categories/repository.rs`, `back/src/categories/service.rs`
- Modify: `back/src/maintenances/model.rs`, `back/src/maintenances/repository.rs`
- Modify: `back/src/incidents/model.rs`, `back/src/incidents/repository.rs`
- Modify: `back/src/projects/model.rs`, `back/src/projects/repository.rs`
- Modify: `front/src/categories/types.ts`, `front/src/categories/api.ts`, `front/src/categories/CategoryBadge.vue`, `front/src/categories/CategoryBadge.test.ts`
- Create: `front/src/categories/CategoryColorInput.vue`, `front/src/categories/CategoryColorInput.test.ts`
- Modify: `front/src/admin/CategoriesPage.vue`, `front/src/admin/CategoriesPage.test.ts`
- Modify: listing/detail/form pages and mock data under `front/src/events`, `front/src/incidents`, and `front/src/projects`

### Task 1: Backend Category Color Persistence

**Files:**
- Create: `back/migrations/0012_category_colors.sql`
- Modify: `back/src/categories/model.rs`
- Modify: `back/src/categories/service.rs`
- Modify: `back/src/categories/repository.rs`

- [ ] **Step 1: Add failing backend validation tests**

In `back/src/categories/service.rs`, add test helpers and tests at the bottom of the file:

```rust
#[cfg(test)]
fn normalize_color_for_test(value: &str) -> Result<String, AppError> {
    normalize_color(value)
}

#[cfg(test)]
mod tests {
    #[test]
    fn normalize_color_accepts_hex_and_lowercases_it() {
        assert_eq!(super::normalize_color_for_test(" #9AAAB1 ").expect("valid color"), "#9aaab1");
    }

    #[test]
    fn normalize_color_rejects_invalid_values() {
        let error = super::normalize_color_for_test("blue").expect_err("invalid color");
        assert_eq!(error.message, "category color must be a #RRGGBB hex value");
    }
}
```

- [ ] **Step 2: Run backend category tests and verify failure**

Run from `back/`: `cargo test --all-features categories::service::tests::normalize_color`

Expected: FAIL because `normalize_color` is not defined.

- [ ] **Step 3: Add migration**

Create `back/migrations/0012_category_colors.sql`:

```sql
ALTER TABLE event_categories ADD COLUMN IF NOT EXISTS color TEXT NOT NULL DEFAULT '#9aaab1';

UPDATE event_categories
SET color = CASE id
  WHEN 'HEA' THEN '#d73a49'
  WHEN 'ELV' THEN '#0366d6'
  WHEN 'PLB' THEN '#0e8a16'
  WHEN 'ELC' THEN '#f9c513'
  WHEN 'GAR' THEN '#6f42c1'
  WHEN 'PMG' THEN '#005cc5'
  ELSE color
END;
```

- [ ] **Step 4: Update backend models**

In `back/src/categories/model.rs`, add `pub color: String` to `CategoryItem`, `CategoryCreateRequest`, and `CategoryUpdateRequest`.

In `back/src/categories/model.rs`, the structs should include:

```rust
pub struct CategoryItem {
    pub id: String,
    pub code: String,
    pub icon: String,
    pub color: String,
    pub label: String,
    pub labels: HashMap<String, String>,
}
```

```rust
pub struct CategoryCreateRequest {
    pub id: String,
    pub code: String,
    pub icon: String,
    pub color: String,
    pub labels: HashMap<String, String>,
}
```

```rust
pub struct CategoryUpdateRequest {
    pub code: String,
    pub icon: String,
    pub color: String,
    pub labels: HashMap<String, String>,
}
```

- [ ] **Step 5: Implement color validation**

In `back/src/categories/service.rs`, add:

```rust
fn normalize_color(value: &str) -> Result<String, AppError> {
    let value = normalize_text_value(value).to_lowercase();
    let valid = value.len() == 7
        && value.starts_with('#')
        && value.chars().skip(1).all(|ch| ch.is_ascii_hexdigit());
    if valid {
        Ok(value)
    } else {
        Err(AppError::bad_request("category color must be a #RRGGBB hex value"))
    }
}
```

In `create`, add:

```rust
let color = normalize_color(&payload.color)?;
repository::create(db, &id, &code, &icon, &color, &labels).await
```

In `update`, add:

```rust
let color = normalize_color(&payload.color)?;
repository::update(db, &id, &code, &icon, &color, &labels).await
```

- [ ] **Step 6: Update category repository**

In `back/src/categories/repository.rs`, select `c.color`, map it into `CategoryItem`, and update create/update signatures.

Expected repository snippets:

```rust
SELECT
  c.id,
  c.code,
  c.icon,
  c.color,
```

```rust
color: row.try_get("color")?,
```

```rust
pub async fn create(db: &sqlx::PgPool, id: &str, code: &str, icon: &str, color: &str, labels: &HashMap<String, String>) -> Result<(), AppError> {
    let mut tx = db.begin().await?;
    sqlx::query("INSERT INTO event_categories (id, code, icon, color) VALUES ($1, $2, $3, $4)")
        .bind(id)
        .bind(code)
        .bind(icon)
        .bind(color)
        .execute(&mut *tx)
        .await?;
    replace_labels(&mut tx, id, labels).await?;
    tx.commit().await?;
    Ok(())
}
```

```rust
pub async fn update(db: &sqlx::PgPool, id: &str, code: &str, icon: &str, color: &str, labels: &HashMap<String, String>) -> Result<(), AppError> {
    let mut tx = db.begin().await?;
    let result = sqlx::query("UPDATE event_categories SET code = $2, icon = $3, color = $4, updated_at = now() WHERE id = $1")
        .bind(id)
        .bind(code)
        .bind(icon)
        .bind(color)
        .execute(&mut *tx)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::not_found("category not found"));
    }
    replace_labels(&mut tx, id, labels).await?;
    tx.commit().await?;
    Ok(())
}
```

- [ ] **Step 7: Run backend category tests**

Run from `back/`: `cargo test --all-features categories::service::tests::normalize_color`

Expected: PASS.

### Task 2: Backend Category Color in Event/Incident/Project Responses

**Files:**
- Modify: `back/src/maintenances/model.rs`, `back/src/maintenances/repository.rs`
- Modify: `back/src/incidents/model.rs`, `back/src/incidents/repository.rs`
- Modify: `back/src/projects/model.rs`, `back/src/projects/repository.rs`

- [ ] **Step 1: Update category display structs**

In each model file, add `pub color: String` to `CategoryDisplay`:

```rust
pub struct CategoryDisplay {
    pub id: String,
    pub code: String,
    pub icon: String,
    pub color: String,
    pub label: String,
}
```

- [ ] **Step 2: Update repository selects and mappers**

In each repository, add `c.color AS category_color` next to `c.icon AS category_icon` in list and detail queries.

In each `CategoryDisplay` construction, add:

```rust
color: row.try_get("category_color")?,
```

- [ ] **Step 3: Run backend compile tests**

Run from `back/`: `cargo test --all-features categories::service::tests::normalize_color`

Expected: PASS and no compile errors from changed structs.

### Task 3: Frontend Category Types, Mocks, and Badge Color

**Files:**
- Modify: `front/src/categories/types.ts`, `front/src/categories/api.ts`, `front/src/categories/CategoryBadge.vue`, `front/src/categories/CategoryBadge.test.ts`
- Modify: `front/src/events/data/mockEvents.ts`, `front/src/incidents/data/mockIncidents.ts`, `front/src/projects/data/mockProjects.ts`
- Modify: API repository category type aliases under `front/src/events/repositories/apiEventsRepository.ts`, `front/src/incidents/repositories/apiIncidentsRepository.ts`, `front/src/projects/repositories/apiProjectsRepository.ts`

- [ ] **Step 1: Add failing badge color test**

In `front/src/categories/CategoryBadge.test.ts`, add:

```ts
it("applies the category color to the badge", () => {
  const { container } = render(CategoryBadge, { props: { code: "HEA", icon: "flame", label: "Heating", color: "#d73a49", variant: "rail" } });
  const badge = container.querySelector(".category-badge-rail");

  expect(badge).not.toBeNull();
  expect(getComputedStyle(badge as Element).getPropertyValue("--category-color").trim()).toBe("#d73a49");
});
```

- [ ] **Step 2: Run badge test and verify failure**

Run from `front/`: `npm run test -- src/categories/CategoryBadge.test.ts`

Expected: FAIL because `color` prop/CSS variable is not implemented.

- [ ] **Step 3: Update frontend category types**

In `front/src/categories/types.ts`, add `color: string` to `CategoryItem` and `CategoryInput`.

In event/incident/project type files, ensure category picks include color:

```ts
category?: Pick<CategoryItem, "id" | "code" | "icon" | "color" | "label">;
```

- [ ] **Step 4: Update `CategoryBadge`**

Add `color?: string` to props and set the CSS variable:

```vue
<span
  :class="['category-badge', `category-badge-${variant ?? 'inline'}`]"
  :style="{
    '--category-color': color ?? '#9aaab1',
    ...((variant ?? 'inline') === 'rail' ? { alignContent: 'center', gap: '0.18rem' } : { gap: '0.21rem' }),
  }"
  :aria-label="label ? `${code} - ${label}` : code"
>
```

Update CSS:

```css
.category-badge { color: var(--category-color, var(--muted-fg)); }
.category-badge-rail::after { background: var(--category-color, var(--border-color)); bottom: 0.1rem; content: ""; position: absolute; right: 0; top: 0.1rem; width: 1px; }
```

- [ ] **Step 5: Update frontend mock categories**

Use these colors consistently:

```ts
HEA: "#d73a49"
ELV: "#0366d6"
PLB: "#0e8a16"
ELC: "#f9c513"
GAR: "#6f42c1"
PMG: "#005cc5"
```

Add `color` to `listCategories` mock responses and each mock event/incident/project category object.

- [ ] **Step 6: Update API repository category shapes**

In `front/src/events/repositories/apiEventsRepository.ts`, `front/src/incidents/repositories/apiIncidentsRepository.ts`, and `front/src/projects/repositories/apiProjectsRepository.ts`, update each `category?: { ... }` shape to include `color: string`.

- [ ] **Step 7: Run badge test**

Run from `front/`: `npm run test -- src/categories/CategoryBadge.test.ts`

Expected: PASS.

### Task 4: Category Colors in Listing, Detail, and Form Pages

**Files:**
- Modify listing pages: `front/src/events/ListingPage.vue`, `front/src/incidents/ListingPage.vue`, `front/src/projects/ListingPage.vue`
- Modify detail pages: `front/src/events/DetailPage.vue`, `front/src/incidents/DetailPage.vue`, `front/src/projects/DetailPage.vue`
- Modify form pages: `front/src/events/FormPage.vue`, `front/src/incidents/FormPage.vue`, `front/src/projects/FormPage.vue`
- Modify related tests.

- [ ] **Step 1: Add failing listing color assertions**

In each listing test, assert the first category rail has the category color. Example for project listing:

```ts
const rail = container.querySelector(".category-badge-rail");
expect(rail).not.toBeNull();
expect(getComputedStyle(rail as Element).getPropertyValue("--category-color").trim()).toBe("#6f42c1");
```

For events/incidents HEA, expect `#d73a49`.

- [ ] **Step 2: Run listing tests and verify failure**

Run from `front/`: `npm run test -- ListingPage.test.ts`

Expected: FAIL because colors are not passed to badges/listing cards yet.

- [ ] **Step 3: Pass color to badges and cards**

For every `CategoryBadge`, add `:color="item.raw.category.color"` or `:color="selectedCategory.color"`.

For every listing card with a category, set the CSS variable on the article:

```vue
:style="project.raw.category ? { '--category-color': project.raw.category.color } : undefined"
```

Use the equivalent `event` or `incident` variable on those pages.

Update listing page styles so the left stripe uses category color:

```css
.project-card::before { background: var(--category-color, rgba(72, 144, 255, 0.55)); }
```

Add the same stripe rule to event and incident cards if they do not already have a left card border:

```css
.category-card, .timeline-card { position: relative; overflow: hidden; }
.category-card::before, .timeline-card::before { content: ""; position: absolute; inset: 0 auto 0 0; width: 0.28rem; background: var(--category-color, rgba(72, 144, 255, 0.55)); }
```

Avoid applying duplicate stripe rules to timeline entry cards inside incident detail; only listing cards should receive this variable/style.

- [ ] **Step 4: Run listing tests**

Run from `front/`: `npm run test -- ListingPage.test.ts`

Expected: PASS.

- [ ] **Step 5: Add detail/form color assertions and pass colors**

Update detail and form tests to assert `.category-badge-inline` has the expected `--category-color`.

Pass `:color="model.raw.category.color"` on detail badges and `:color="selectedCategory.color"` on form badges.

Run from `front/`: `npm run test -- DetailPage.test.ts FormPage.test.ts`

Expected: PASS.

### Task 5: GitHub-Style Admin Color Input

**Files:**
- Create: `front/src/categories/CategoryColorInput.vue`
- Create: `front/src/categories/CategoryColorInput.test.ts`
- Modify: `front/src/admin/CategoriesPage.vue`, `front/src/admin/CategoriesPage.test.ts`

- [ ] **Step 1: Add failing color input tests**

Create `front/src/categories/CategoryColorInput.test.ts`:

```ts
import { fireEvent, render, screen } from "@testing-library/vue";
import { describe, expect, it, vi } from "vitest";
import CategoryColorInput from "./CategoryColorInput.vue";

describe("CategoryColorInput", () => {
  it("edits hex colors and exposes default swatches", async () => {
    render(CategoryColorInput, { props: { modelValue: "#9aaab1", "onUpdate:modelValue": vi.fn() } });

    expect(screen.getByLabelText("Color")).not.toBeNull();
    await fireEvent.focus(screen.getByLabelText("Color"));
    expect(screen.getByText("Choose from default colors")).not.toBeNull();
    expect(screen.getByLabelText("Use color #d73a49")).not.toBeNull();
  });

  it("sets a random color from the refresh button", async () => {
    const update = vi.fn();
    render(CategoryColorInput, { props: { modelValue: "#9aaab1", "onUpdate:modelValue": update } });

    await fireEvent.click(screen.getByLabelText("Set random color"));

    expect(update).toHaveBeenCalledWith(expect.stringMatching(/^#[0-9a-f]{6}$/));
  });
});
```

- [ ] **Step 2: Run color input tests and verify failure**

Run from `front/`: `npm run test -- src/categories/CategoryColorInput.test.ts`

Expected: FAIL because the component does not exist.

- [ ] **Step 3: Implement `CategoryColorInput.vue`**

Create `front/src/categories/CategoryColorInput.vue` with:

```vue
<script setup lang="ts">
import { ref } from "vue";
import { RefreshCcw } from "@lucide/vue";

const DEFAULT_COLORS = ["#d73a49", "#f66a0a", "#f9c513", "#0e8a16", "#008672", "#0366d6", "#005cc5", "#6f42c1", "#ea4aaa", "#f9d0c4", "#fef2c0", "#c2e0c6", "#bfdadc", "#c5def5", "#bfd4f2", "#d4c5f9"];

const props = defineProps<{ modelValue: string }>();
const emit = defineEmits<{ "update:modelValue": [value: string] }>();
const open = ref(false);

function update(value: string): void {
  emit("update:modelValue", value.trim().toLowerCase());
}

function randomColor(): void {
  const value = Math.floor(Math.random() * 0xffffff).toString(16).padStart(6, "0");
  update(`#${value}`);
}
</script>

<template>
  <span class="category-color-input">
    <span class="color-preview" :style="{ backgroundColor: modelValue }" aria-hidden="true" />
    <input aria-label="Color" :value="modelValue" @focus="open = true" @input="update(($event.target as HTMLInputElement).value)" />
    <button class="color-random-button" type="button" aria-label="Set random color" @click="randomColor"><RefreshCcw :size="16" /></button>
    <span v-if="open" class="color-popover">
      <span class="color-popover-title">Choose from default colors</span>
      <button v-for="color in DEFAULT_COLORS" :key="color" class="color-swatch" type="button" :aria-label="`Use color ${color}`" :style="{ backgroundColor: color }" @click="update(color)" />
    </span>
  </span>
</template>

<style scoped>
.category-color-input { align-items: center; display: flex; gap: 0.45rem; position: relative; }
.category-color-input input { flex: 1 1 auto; min-width: 0; }
.color-preview { border: 1px solid var(--control-border); border-radius: 0.4rem; height: 2rem; width: 2rem; }
.color-random-button { align-items: center; border: 1px solid var(--control-border); border-radius: 0.55rem; background: var(--control-bg); color: var(--control-fg); cursor: pointer; display: inline-flex; height: 2.25rem; justify-content: center; width: 2.25rem; }
.color-popover { background: var(--panel-bg); border: 1px solid var(--border-color); border-radius: 0.45rem; box-shadow: 0 12px 26px rgba(0, 0, 0, 0.28); display: grid; gap: 0.5rem; grid-template-columns: repeat(8, 1.8rem); left: 2.45rem; padding: 0.75rem; position: absolute; top: calc(100% + 0.25rem); z-index: 5; }
.color-popover-title { color: var(--page-fg); grid-column: 1 / -1; font-weight: 650; }
.color-swatch { border: 0; border-radius: 0.35rem; cursor: pointer; height: 1.8rem; width: 1.8rem; }
</style>
```

- [ ] **Step 4: Run color input tests**

Run from `front/`: `npm run test -- src/categories/CategoryColorInput.test.ts`

Expected: PASS.

- [ ] **Step 5: Integrate color input into admin page**

In `front/src/admin/CategoriesPage.vue`:

- Import `CategoryColorInput`.
- Initialize/reset form with `color: '#9aaab1'`.
- When editing, copy `category.color`.
- Add one color label after icon:

```vue
<label>{{ t("labels.categoryColor") }}<CategoryColorInput v-model="form.color" /></label>
```

- Add color column to table showing swatch and hex.

In `front/src/common/i18n.ts`, add `categoryColor: "Color"` / `categoryColor: "Couleur"`.

- [ ] **Step 6: Update admin tests**

In `front/src/admin/CategoriesPage.test.ts`, assert:

```ts
expect(await screen.findByLabelText("Color")).not.toBeNull();
expect(await screen.findByText("#d73a49")).not.toBeNull();
```

Run from `front/`: `npm run test -- src/admin/CategoriesPage.test.ts src/categories/CategoryColorInput.test.ts`

Expected: PASS.

### Task 6: Final Verification

**Files:**
- All changed frontend/backend files.

- [ ] **Step 1: Run frontend lint**

Run from `front/`: `npm run lint`

Expected: PASS.

- [ ] **Step 2: Run frontend tests**

Run from `front/`: `npm run test`

Expected: PASS.

- [ ] **Step 3: Run frontend build**

Run from `front/`: `npm run build`

Expected: PASS.

- [ ] **Step 4: Run backend clippy**

Run from `back/`: `cargo clippy --all-features -- -D warnings`

Expected: PASS.

- [ ] **Step 5: Run backend tests**

Run from `back/`: `cargo test --all-features`

Expected: PASS.

- [ ] **Step 6: Run backend build**

Run from `back/`: `cargo build --all-features`

Expected: PASS.

- [ ] **Step 7: Inspect diff**

Run from repo root: `git diff --stat`

Expected: Diff contains category color migration, backend model/repository/service updates, frontend category color UI, display wiring, tests, and docs only.

## Self-Review

- Spec coverage: persisted color, validation, admin GitHub-style input, random color button, table display, badge color, listing border color, detail/form propagation, and verification commands are covered.
- Placeholder scan: no TODO/TBD placeholders remain.
- Type consistency: frontend and backend consistently use `color` and `#RRGGBB`; category display structs and API category objects include `color`.
- Repo instruction note: do not commit any plan or implementation changes unless the user explicitly requests a commit.
