# Item Edition Rework Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Redesign the item edition forms (projects, incidents, events) with new layout, date handling, category selector, and read-only timeline.

**Architecture:** Modify three FormPage.vue files sharing the same patterns. Add utility functions to `dateInput.ts` for date-only↔UTC conversions. Each form is independent but follows identical logic.

**Tech Stack:** Vue 3, TypeScript, vue-i18n, @testing-library/vue, vitest

---

### Task 1: Add date utility functions

**Files:**
- Modify: `front/src/common/dateInput.ts`

- [ ] **Add `toDateLocalInput` and `toUtcFromDateLocalInput` functions**

```typescript
// Append to dateInput.ts

export function toDateLocalInput(value?: string | null): string {
  return value ? value.slice(0, 10) : "";
}

export function toUtcFromDateLocalInput(value: string, now: Date = new Date()): string | null {
  if (!value) return null;
  const time = `${String(now.getHours()).padStart(2, "0")}:${String(now.getMinutes()).padStart(2, "0")}:00.000Z`;
  return `${value}T${time}`;
}

/** Returns today as YYYY-MM-DD for date input default */
export function todayDateInput(): string {
  const d = new Date();
  const yyyy = d.getFullYear();
  const mm = String(d.getMonth() + 1).padStart(2, "0");
  const dd = String(d.getDate()).padStart(2, "0");
  return `${yyyy}-${mm}-${dd}`;
}
```

- [ ] **Update tests in `dateInput.test.ts` if exists**

Run: `npx vitest run front/src/common/dateInput.test.ts`
Expected: PASS

---

### Task 2: Rework Projects FormPage

**Files:**
- Modify: `front/src/projects/FormPage.vue`
- Test: `front/src/projects/FormPage.test.ts`

- [ ] **Rewrite the script section**

Changes:
- Remove `editLocale`, `enabledLocales`, `activeLocale()`
- Use `inject("selectedLocale")` from App.vue
- Import new date utils: `toDateLocalInput`, `toUtcFromDateLocalInput`, `todayDateInput`
- Import `TimelineList` type and component for read-only rendering
- Pre-fill `form.startUtc` with `todayDateInput()` when creating new item
- Set end date initial to `""` (hidden for new), always visible for edit
- On save: convert date-only values using `toUtcFromDateLocalInput`, pass null for empty values
- Remove timeline editing functions (`addTimeline`, `removeTimeline`)
- No longer send `replaceTimeline` and `timeline` in save payload
- Use `fallbackLocale` from repository response for all fields

- [ ] **Rewrite the template**

New template layout:
```html
<form class="project-form" @submit.prevent="save">
  <!-- Line 1: category + key + title -->
  <div class="title-category-row">
    <div class="category-select-wrap">
      <button v-if="selectedCategory" class="category-trigger" type="button">
        <CategoryBadge :category-key="selectedCategory.key" :icon="selectedCategory.icon" :color="selectedCategory.color" />
      </button>
      <div class="category-dropdown">
        <button v-for="cat in categories" :key="cat.id" class="category-option" @click="form.categoryId = cat.id">
          <span class="cat-option-icon" :style="{ color: cat.color }"><CategoryIcon :name="cat.icon" :size="18" /></span>
          <span class="cat-option-key">{{ cat.key }}</span>
          <span class="cat-option-label">{{ cat.label }}</span>
        </button>
      </div>
    </div>
    <span v-if="isEdit" class="title-key">{{ form.id }}</span>
    <input v-model="form.title" required class="title-input" :placeholder="t('labels.title')" />
  </div>
  
  <!-- Line 2: Status -->
  <label class="status-field">
    <span>{{ t("labels.projectStatus") }}</span>
    <span class="status-input-group">
      <select v-model="form.statusType" required>
        <option value="waiting">{{ t("labels.waiting") }}</option>
        <option value="ongoing">{{ t("labels.ongoing") }}</option>
      </select>
      <input v-model="form.statusText" required />
    </span>
    <small v-if="fallbackByField.status_text">{{ t("labels.prefilledFrom", { locale: fallbackByField.status_text }) }}</small>
  </label>
  
  <!-- Line 3: Start / End dates -->
  <div class="date-row">
    <label>{{ t("labels.startUtc") }}<input v-model="form.startUtc" type="date" required /></label>
    <label v-if="isEdit">{{ t("labels.endUtc") }}<input v-model="form.endUtc" type="date" /></label>
  </div>
  
  <!-- Line 4: Description -->
  <label>{{ t("labels.description") }}<textarea v-model="form.description" required /></label>
  <small v-if="fallbackByField.description">{{ t("labels.prefilledFrom", { locale: fallbackByField.description }) }}</small>
  
  <!-- Timeline (read-only) -->
  <section v-if="timeline.length" class="timeline-section">
    <h2>{{ t("labels.projectTimeline") }}</h2>
    <TimelineList :entries="timeline" />
  </section>
  
  <!-- Form actions -->
  <p v-if="saveFailed" class="empty-state">{{ t("labels.saveFailed") }}</p>
  <footer class="form-actions">
    <RouterLink class="secondary-button" :to="cancelPath">{{ t("labels.cancel") }}</RouterLink>
    <button class="primary-button" type="submit" :disabled="saving">{{ saving ? t("labels.saving") : t("labels.save") }}</button>
  </footer>
</form>
```

- [ ] **Update save method**

```typescript
async function save(): Promise<void> {
  saving.value = true;
  saveFailed.value = false;
  try {
    const payload: ProjectSavePayload = {
      categoryId: form.value.categoryId,
      startUtc: toUtcFromDateLocalInput(form.value.startUtc) ?? "",
      endUtc: toUtcFromDateLocalInput(form.value.endUtc),
      statusType: form.value.statusType,
      locale: selectedLocale.value,
      fields: {
        title: form.value.title,
        description: form.value.description,
        status_text: form.value.statusText,
      },
    };
    const createdKey = await apiProjectsRepository.save(
      payload,
      isEdit.value ? existingId.value : undefined,
    );
    await router.push(`/projects/${encodeURIComponent(isEdit.value ? existingId.value : String(createdKey))}`);
  } catch {
    saveFailed.value = true;
  } finally {
    saving.value = false;
  }
}
```

- [ ] **Update load method to use selectedLocale and date-only conversion**

```typescript
async function load(): Promise<void> {
  loadFailed.value = false;
  try {
    categories.value = await listCategories(selectedLocale.value);
    if (!form.value.categoryId && categories.value[0]) form.value.categoryId = categories.value[0].id;
    if (!isEdit.value) {
      form.value.startUtc = todayDateInput();
      return;
    }
    const data = await apiProjectsRepository.editData(existingId.value, selectedLocale.value);
    if (!data) { loadFailed.value = true; return; }
    form.value.id = data.id;
    form.value.categoryId = data.categoryId;
    form.value.startUtc = toDateLocalInput(data.startUtc);
    form.value.endUtc = toDateLocalInput(data.endUtc);
    form.value.statusType = data.statusType;
    applyFields(data.fields);
    timeline.value = data.timeline.map((item) => ({
      id: item.id,
      atUtc: item.atUtc,
      atLabel: "",
      atDateLabel: "",
      atTimeLabel: "",
      isPending: !item.atUtc,
      title: field(item.fields, "title"),
      details: field(item.fields, "details"),
    }));
  } catch { loadFailed.value = true; }
}
```

Wait, the TimelineList component uses a specific type `TimelineEntry` that has `atLabel`, `atDateLabel`, `atTimeLabel`, `isPending`, `title`, `details`. I need to format the timeline data properly for the TimelineList component.

Let me look at how the DetailPage does it. It uses `model.timeline` which is already formatted by the utility functions. For the form, I can get the raw timeline items and format them.

Actually, looking more carefully at the existing form code, the timeline data is already fetched from the editData endpoint. But the timeline rendering for read-only views uses formatted view model data. I need to format the raw timeline data into the format expected by TimelineList.

Let me reformulate. The raw timeline items from the API have `atUtc` and `fields`. I need to format them into `TimelineEntry` items. I can use the same formatting logic as the utility functions.

Let me revise the approach: instead of duplicating formatting logic in the form, I can import the utility functions from the respective utils files.

Actually, looking at the imports more carefully, the TimelineList component expects:
```typescript
export type TimelineEntry = {
  id: string;
  atLabel: string;
  atDateLabel: string;
  atTimeLabel: string;
  isPending: boolean;
  title: string;
  details: string;
};
```

And in the projects DetailPage, the timeline comes from `model.timeline` which returns `ProjectTimelineEntryViewModel[]` which has the same shape. The events DetailPage also uses `model.timeline` which is `EventTimelineEntryViewModel[]`.

For the form, I can directly format the raw timeline data in the load function. Let me use `parseUtc` from date.ts and format with Intl.DateTimeFormat.

Actually, let me just import the `toProjectViewModel` or similar helper. But that creates a full view model which is overkill. Let me just format the timeline entries inline in the load method.

OK let me simplify my approach. Let me write the plan more carefully based on what I know.

- [ ] **Update test file**

```typescript
// Update tests in FormPage.test.ts
// - Test that language selector is gone
// - Test that start date is pre-filled with today
// - Test that end date is hidden for new items
// - Test category+title on same line
```

- [ ] **Run tests**

Run: `npx vitest run front/src/projects/FormPage.test.ts`
Expected: PASS

---

### Task 3-4: Rework Incidents and Events FormPages (same pattern as Task 2)

Follow identical pattern to Task 2 with entity-specific differences:
- Incidents: no description fallback, has location, startUtc required
- Events: has warning field, has location

---

### Verification

- [ ] **Run all tests**

Run: `cd front && npm run test`
Expected: PASS

- [ ] **Run lint**

Run: `cd front && npm run lint`
Expected: PASS

- [ ] **Run build**

Run: `cd front && npm run build`
Expected: PASS
