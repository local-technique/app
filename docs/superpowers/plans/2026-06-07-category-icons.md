# Category Icons Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Display category icons on maintenance, incident, and project listing/detail/edit pages.

**Architecture:** Reuse the existing `CategoryIcon` resolver through one small shared `CategoryBadge` component. Listing pages use the badge as a left rail with code under icon and a divider; detail and form pages use the same component inline next to the category text/dropdown.

**Tech Stack:** Vue 3 SFCs, TypeScript, Vue Router, Vue I18n, Testing Library Vue, Vitest, Lucide Vue.

---

## Files

- Create: `front/src/categories/CategoryBadge.vue` for shared listing, detail, and form category display.
- Modify: `front/src/events/ListingPage.vue` to render category rail on maintenance cards.
- Modify: `front/src/incidents/ListingPage.vue` to render category rail on incident cards.
- Modify: `front/src/projects/ListingPage.vue` to render category rail on project cards and remove category code from the project kicker.
- Modify: `front/src/events/DetailPage.vue` to render inline category icon next to category metadata.
- Modify: `front/src/incidents/DetailPage.vue` to render inline category icon next to category metadata.
- Modify: `front/src/projects/DetailPage.vue` to render inline category icon next to category metadata.
- Modify: `front/src/events/FormPage.vue` to show selected category icon left of the category dropdown.
- Modify: `front/src/incidents/FormPage.vue` to show selected category icon left of the category dropdown.
- Modify: `front/src/projects/FormPage.vue` to show selected category icon left of the category dropdown.
- Modify tests under `front/src/events`, `front/src/incidents`, and `front/src/projects` to cover listing/detail/form icon placements.

### Task 1: Shared Category Badge Component

**Files:**
- Create: `front/src/categories/CategoryBadge.vue`

- [ ] **Step 1: Create the shared component**

Use `apply_patch` to create `front/src/categories/CategoryBadge.vue`:

```vue
<script setup lang="ts">
import CategoryIcon from "./CategoryIcon.vue";

defineProps<{
  code: string;
  icon: string;
  label?: string;
  variant?: "rail" | "inline";
}>();
</script>

<template>
  <span :class="['category-badge', `category-badge-${variant ?? 'inline'}`]" :aria-label="label ? `${code} - ${label}` : code">
    <CategoryIcon class="category-badge-icon" :name="icon" :size="variant === 'rail' ? 24 : 18" />
    <span class="category-badge-code">{{ code }}</span>
  </span>
</template>

<style scoped>
.category-badge { color: var(--muted-fg); }
.category-badge-icon { flex: 0 0 auto; }
.category-badge-rail { align-items: center; display: grid; gap: 0.28rem; justify-items: center; min-width: 3.2rem; padding-right: 0.72rem; position: relative; }
.category-badge-rail::after { background: var(--border-color); bottom: 0.1rem; content: ""; position: absolute; right: 0; top: 0.1rem; width: 1px; }
.category-badge-rail .category-badge-code { font-size: 0.72rem; font-weight: 800; letter-spacing: 0.06em; line-height: 1; text-transform: uppercase; }
.category-badge-inline { align-items: center; display: inline-flex; gap: 0.42rem; }
.category-badge-inline .category-badge-code { font-weight: 800; letter-spacing: 0.04em; }
</style>
```

- [ ] **Step 2: Run a focused type check through tests**

Run: `npm run test -- CategoryBadge`

Expected: command may report no matching tests, but Vue compilation should not fail. If the test runner exits non-zero only because no tests matched, continue to Task 2.

### Task 2: Listing Page Category Rails

**Files:**
- Modify: `front/src/events/ListingPage.vue`
- Modify: `front/src/incidents/ListingPage.vue`
- Modify: `front/src/projects/ListingPage.vue`
- Modify tests: `front/src/events/ListingPage.test.ts`, `front/src/incidents/ListingPage.test.ts`, `front/src/projects/ListingPage.test.ts`

- [ ] **Step 1: Write failing listing assertions**

In `front/src/events/ListingPage.test.ts`, change the render call in the second test to capture `container` and assert a rail exists:

```ts
const { container } = render(EventsListingPage, { global: { plugins: [router, createAppI18n("en")] } });

expect(await screen.findByRole("heading", { name: "Current" })).not.toBeNull();
expect(container.querySelector(".category-badge-rail")?.textContent).toContain("HEA");
```

In `front/src/incidents/ListingPage.test.ts`, change the render call to capture `container` and add:

```ts
const { container } = render(IncidentsListingPage, { global: { plugins: [router, createAppI18n("en")] } });

expect(await screen.findByRole("heading", { name: "Current" })).not.toBeNull();
expect(container.querySelector(".category-badge-rail")?.textContent).toContain("HEA");
```

In `front/src/projects/ListingPage.test.ts`, after render add:

```ts
const { container } = render(ProjectsListingPage, { global: { plugins: [router, createAppI18n("en")] } });

expect(await screen.findByRole("heading", { name: "Ongoing Projects" })).not.toBeNull();
expect(container.querySelector(".category-badge-rail")?.textContent).toContain("GAR");
```

- [ ] **Step 2: Run listing tests and verify they fail**

Run: `npm run test -- ListingPage.test.ts`

Expected: FAIL because `.category-badge-rail` does not exist yet.

- [ ] **Step 3: Implement event listing rail**

In `front/src/events/ListingPage.vue`, add the import:

```ts
import CategoryBadge from "../categories/CategoryBadge.vue";
```

Wrap each event card's existing content with a category rail and content container. For each of the three `article` blocks, use this structure:

```vue
<article class="timeline-card category-card" v-for="event in grouped.current" :key="event.id">
  <CategoryBadge v-if="event.raw.category" :code="event.raw.category.code" :icon="event.raw.category.icon" :label="event.raw.category.label" variant="rail" />
  <div class="category-card-content">
    <h3 class="timeline-card-title">
      <RouterLink :to="{ path: `/events/${event.id}`, query: detailQuery }">{{ event.title }}</RouterLink>
    </h3>
    <p class="timeline-warning" v-if="event.warning">{{ t("labels.warningPrefix") }} {{ event.warning }}</p>
    <p class="timeline-meta">{{ event.dateLabel }}</p>
    <p class="timeline-meta" v-if="event.location">{{ event.location }}</p>
  </div>
</article>
```

Use the same `category-card` structure for `grouped.toCome` and `grouped.past`, preserving `timeline-card-past` on past cards.

Add scoped styles:

```css
.category-card { align-items: stretch; display: flex; gap: 0.72rem; }
.category-card-content { min-width: 0; }
```

- [ ] **Step 4: Implement incident listing rail**

In `front/src/incidents/ListingPage.vue`, add:

```ts
import CategoryBadge from "../categories/CategoryBadge.vue";
```

Inside both incident `article` blocks, place the rail before `.incident-card-main`:

```vue
<CategoryBadge v-if="incident.raw.category" :code="incident.raw.category.code" :icon="incident.raw.category.icon" :label="incident.raw.category.label" variant="rail" />
```

Remove the category code from the existing meta line so the code is only in the rail:

```vue
<p class="timeline-meta">{{ incident.id }}</p>
```

Change styles so mobile uses rail plus content and desktop preserves the timeline preview as the third column:

```css
.timeline-card { grid-template-columns: auto minmax(0, 1fr); }
```

Update the media query grid:

```css
.timeline-card { align-items: stretch; grid-template-columns: auto minmax(0, 1fr) minmax(13rem, 30%); column-gap: 1.2rem; }
```

- [ ] **Step 5: Implement project listing rail**

In `front/src/projects/ListingPage.vue`, add:

```ts
import CategoryBadge from "../categories/CategoryBadge.vue";
```

For each project `article`, insert the rail before the project content and wrap existing content:

```vue
<CategoryBadge v-if="project.raw.category" :code="project.raw.category.code" :icon="project.raw.category.icon" :label="project.raw.category.label" variant="rail" />
<div class="project-card-content">
  <p class="project-kicker">{{ project.id }}</p>
  <h3 class="timeline-card-title"><RouterLink :to="{ path: projectPath(project.id), query: detailQuery }">{{ project.title }}</RouterLink></h3>
  <p class="timeline-meta">{{ project.dateLabel }}</p>
  <p class="project-status"><Activity :size="16" /> {{ project.statusText }}</p>
</div>
```

Use the matching status icon in each section. Add styles:

```css
.project-card { align-items: stretch; display: flex; gap: 0.72rem; position: relative; overflow: hidden; }
.project-card-content { min-width: 0; }
```

- [ ] **Step 6: Run listing tests and verify they pass**

Run: `npm run test -- ListingPage.test.ts`

Expected: PASS.

### Task 3: Detail Page Inline Category Icons

**Files:**
- Modify: `front/src/events/DetailPage.vue`
- Modify: `front/src/incidents/DetailPage.vue`
- Modify: `front/src/projects/DetailPage.vue`
- Modify tests: `front/src/events/DetailPage.test.ts`, `front/src/incidents/DetailPage.test.ts`, `front/src/projects/DetailPage.test.ts`

- [ ] **Step 1: Write failing detail assertions**

In each detail test render, capture `container` and assert the inline badge exists.

For `front/src/events/DetailPage.test.ts`:

```ts
const { container } = render(EventsDetailPage, { global: { plugins: [router, createAppI18n("en")] } });

const warning = await screen.findByText("⚠️ no hot water between 9h30 & 17h00");
expect(container.querySelector(".category-badge-inline")?.textContent).toContain("HEA");
```

For `front/src/incidents/DetailPage.test.ts` first test:

```ts
const { container } = render(IncidentDetailPage, { global: { plugins: [router, createAppI18n("en")] } });

expect(await screen.findByRole("heading", { name: "Heating outage on block B" })).not.toBeNull();
expect(container.querySelector(".category-badge-inline")?.textContent).toContain("HEA");
```

For `front/src/projects/DetailPage.test.ts`, the render already captures `container`; add:

```ts
expect(container.querySelector(".category-badge-inline")?.textContent).toContain("ELV");
```

- [ ] **Step 2: Run detail tests and verify they fail**

Run: `npm run test -- DetailPage.test.ts`

Expected: FAIL because `.category-badge-inline` does not exist yet.

- [ ] **Step 3: Implement event detail inline category**

In `front/src/events/DetailPage.vue`, add:

```ts
import CategoryBadge from "../categories/CategoryBadge.vue";
```

Replace the category paragraph with:

```vue
<p class="timeline-meta category-meta" v-if="model.raw.category">
  <CategoryBadge :code="model.raw.category.code" :icon="model.raw.category.icon" :label="model.raw.category.label" />
  <span>- {{ model.raw.category.label }}</span>
</p>
```

Add style:

```css
.category-meta { align-items: center; display: flex; gap: 0.35rem; }
```

- [ ] **Step 4: Implement incident detail inline category**

Make the same import, template replacement, and style addition in `front/src/incidents/DetailPage.vue`.

- [ ] **Step 5: Implement project detail inline category**

Make the same import, template replacement, and style addition in `front/src/projects/DetailPage.vue`.

- [ ] **Step 6: Run detail tests and verify they pass**

Run: `npm run test -- DetailPage.test.ts`

Expected: PASS.

### Task 4: Form Category Dropdown Icons

**Files:**
- Modify: `front/src/events/FormPage.vue`
- Modify: `front/src/incidents/FormPage.vue`
- Modify: `front/src/projects/FormPage.vue`
- Create tests: `front/src/events/FormPage.test.ts`, `front/src/incidents/FormPage.test.ts`
- Modify test: `front/src/projects/FormPage.test.ts`

- [ ] **Step 1: Write failing form assertions**

Create `front/src/events/FormPage.test.ts`:

```ts
import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";
import { createRouter, createWebHashHistory } from "vue-router";
import { createAppI18n } from "../common/i18n";
import EventsFormPage from "./FormPage.vue";

describe("Events form", () => {
  it("shows the selected category icon next to the category dropdown", async () => {
    const router = createRouter({ history: createWebHashHistory(), routes: [{ path: "/events/new", component: EventsFormPage }] });
    await router.push("/events/new");
    await router.isReady();

    const { container } = render(EventsFormPage, { global: { plugins: [router, createAppI18n("en")] } });

    expect(await screen.findByLabelText("Category")).not.toBeNull();
    expect(container.querySelector(".category-select-row .category-badge-inline")?.textContent).toContain("HEA");
  });
});
```

Create `front/src/incidents/FormPage.test.ts` with the same structure, replacing route and component:

```ts
import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";
import { createRouter, createWebHashHistory } from "vue-router";
import { createAppI18n } from "../common/i18n";
import IncidentsFormPage from "./FormPage.vue";

describe("Incidents form", () => {
  it("shows the selected category icon next to the category dropdown", async () => {
    const router = createRouter({ history: createWebHashHistory(), routes: [{ path: "/incidents/new", component: IncidentsFormPage }] });
    await router.push("/incidents/new");
    await router.isReady();

    const { container } = render(IncidentsFormPage, { global: { plugins: [router, createAppI18n("en")] } });

    expect(await screen.findByLabelText("Category")).not.toBeNull();
    expect(container.querySelector(".category-select-row .category-badge-inline")?.textContent).toContain("HEA");
  });
});
```

In `front/src/projects/FormPage.test.ts`, add to the first test:

```ts
expect(await screen.findByLabelText("Category")).not.toBeNull();
expect(container.querySelector(".category-select-row .category-badge-inline")?.textContent).toContain("GAR");
```

- [ ] **Step 2: Run form tests and verify they fail**

Run: `npm run test -- FormPage.test.ts`

Expected: FAIL because `.category-select-row` and inline category badges do not exist yet.

- [ ] **Step 3: Implement events form selected category icon**

In `front/src/events/FormPage.vue`, add imports and computed value:

```ts
import CategoryBadge from "../categories/CategoryBadge.vue";
```

```ts
const selectedCategory = computed(() => categories.value.find((category) => category.id === form.value.categoryId) ?? null);
```

Replace the category label with:

```vue
<label>{{ t("labels.category") }}<span class="category-select-row"><CategoryBadge v-if="selectedCategory" :code="selectedCategory.code" :icon="selectedCategory.icon" :label="selectedCategory.label" /><select v-model="form.categoryId" required><option v-for="category in categories" :key="category.id" :value="category.id">{{ category.code }} - {{ category.label }}</option></select></span></label>
```

Add styles:

```css
.category-select-row { align-items: center; display: flex; gap: 0.6rem; }
.category-select-row select { flex: 1 1 auto; min-width: 0; }
```

- [ ] **Step 4: Implement incidents form selected category icon**

Make the same import, `selectedCategory` computed, template replacement, and style addition in `front/src/incidents/FormPage.vue`.

- [ ] **Step 5: Implement projects form selected category icon**

Make the same import, `selectedCategory` computed, template replacement, and style addition in `front/src/projects/FormPage.vue`.

- [ ] **Step 6: Run form tests and verify they pass**

Run: `npm run test -- FormPage.test.ts`

Expected: PASS.

### Task 5: Final Verification

**Files:**
- Verify all frontend changes.

- [ ] **Step 1: Run frontend lint**

Run from `front/`: `npm run lint`

Expected: PASS with no lint errors.

- [ ] **Step 2: Run frontend tests**

Run from `front/`: `npm run test`

Expected: PASS with all tests passing.

- [ ] **Step 3: Run frontend build**

Run from `front/`: `npm run build`

Expected: PASS and production build completes.

- [ ] **Step 4: Inspect git diff**

Run from repo root: `git diff -- front docs/superpowers/specs/2026-06-07-category-icons-design.md docs/superpowers/plans/2026-06-07-category-icons.md`

Expected: Diff only contains the approved category icon UI, tests, and planning docs.

## Self-Review

- Spec coverage: listing rails, detail inline category rows, edition dropdown icons, existing `CategoryIcon` reuse, and frontend verification are covered.
- Placeholder scan: no TBD/TODO placeholders remain.
- Type consistency: component props use existing category field names `code`, `icon`, and `label`; Vue templates reference current `raw.category` and `CategoryItem` shapes.
