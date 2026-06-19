<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useRoute, useRouter } from "vue-router";
import { listCategories } from "../categories/api";
import CategoryBadge from "../categories/CategoryBadge.vue";
import type { CategoryItem } from "../categories/types";
import { toDateTimeLocalInput, toUtcFromDateTimeLocalInput } from "../common/dateInput";
import type { LocaleCode } from "../common/i18n";
import { apiEventsRepository } from "./repositories/apiEventsRepository";
import type { EventStoredStatus } from "./types";

const { t, locale } = useI18n();
const route = useRoute();
const router = useRouter();
const existingId = computed(() => (typeof route.params.id === "string" ? route.params.id : ""));
const isEdit = computed(() => existingId.value.length > 0);
const categories = ref<CategoryItem[]>([]);
const enabledLocales = ref<string[]>(["en", "fr"]);
const editLocale = ref<LocaleCode>(locale.value === "en" ? "en" : "fr");
const saving = ref(false);
const loadFailed = ref(false);
const saveFailed = ref(false);
const fallbackByField = ref<Record<string, string>>({});
const form = ref({
  id: "",
  categoryId: "",
  startUtc: "",
  endUtc: "",
  notifiedAtUtc: "",
  statusType: "ongoing" as EventStoredStatus,
  statusText: "",
  title: "",
  shortDescription: "",
  longDescription: "",
  warning: "",
  location: "",
});
const timeline = ref<Array<{ id: string; atUtc: string; title: string; details: string }>>([]);
const selectedCategory = computed(() => categories.value.find((category) => category.id === form.value.categoryId) ?? null);

function activeLocale(): LocaleCode {
  return editLocale.value === "en" ? "en" : "fr";
}

function field(fields: Array<{ fieldKey: string; value: string }>, key: string): string { return fields.find((item) => item.fieldKey === key)?.value ?? ""; }

function applyFields(fields: Array<{ fieldKey: string; value: string; fallbackLocale?: string | null }>): void {
  fallbackByField.value = {};
  for (const field of fields) {
    if (field.fallbackLocale) {
      fallbackByField.value[field.fieldKey] = field.fallbackLocale;
    }
    if (field.fieldKey === "title") form.value.title = field.value;
    if (field.fieldKey === "short_description") form.value.shortDescription = field.value;
    if (field.fieldKey === "long_description") form.value.longDescription = field.value;
    if (field.fieldKey === "warning") form.value.warning = field.value;
    if (field.fieldKey === "location") form.value.location = field.value;
    if (field.fieldKey === "status_text") form.value.statusText = field.value;
  }
}

async function load(): Promise<void> {
  loadFailed.value = false;
  try {
    categories.value = await listCategories(activeLocale());
    if (!form.value.categoryId && categories.value[0]) {
      form.value.categoryId = categories.value[0].id;
    }
    if (!isEdit.value) {
      enabledLocales.value = ["en", "fr"];
      return;
    }
    const data = await apiEventsRepository.editData(existingId.value, activeLocale());
    if (!data) {
      loadFailed.value = true;
      return;
    }
    enabledLocales.value = data.enabledLocales;
    form.value.id = data.id;
    form.value.categoryId = data.categoryId;
    form.value.startUtc = toDateTimeLocalInput(data.startUtc);
    form.value.endUtc = toDateTimeLocalInput(data.endUtc);
    form.value.notifiedAtUtc = toDateTimeLocalInput(data.notifiedAtUtc);
    form.value.statusType = data.statusType;
    applyFields(data.fields);
    timeline.value = data.timeline.map((item) => ({ id: item.id, atUtc: toDateTimeLocalInput(item.atUtc), title: field(item.fields, "title"), details: field(item.fields, "details") }));
  } catch {
    loadFailed.value = true;
  }
}

watch([() => route.params.id, editLocale], () => void load(), { immediate: true });

function addTimeline(): void { timeline.value.push({ id: crypto.randomUUID(), atUtc: "", title: "", details: "" }); }
function removeTimeline(id: string): void { timeline.value = timeline.value.filter((item) => item.id !== id); }

async function save(): Promise<void> {
  saving.value = true;
  saveFailed.value = false;
  try {
      const createdKey = await apiEventsRepository.save(
      {
        categoryId: form.value.categoryId,
        startUtc: toUtcFromDateTimeLocalInput(form.value.startUtc) ?? "",
        endUtc: toUtcFromDateTimeLocalInput(form.value.endUtc),
        notifiedAtUtc: toUtcFromDateTimeLocalInput(form.value.notifiedAtUtc),
        statusType: form.value.statusType,
        locale: activeLocale(),
        fields: {
          title: form.value.title,
          short_description: form.value.shortDescription,
          long_description: form.value.longDescription,
          warning: form.value.warning,
          location: form.value.location,
          status_text: form.value.statusText,
        },
        replaceTimeline: true,
        timeline: timeline.value.map((item, index) => ({ id: item.id, atUtc: toUtcFromDateTimeLocalInput(item.atUtc), sortOrder: index + 1, fields: { title: item.title, details: item.details } })),
      },
      isEdit.value ? existingId.value : undefined,
    );
    await router.push(`/events/${encodeURIComponent(isEdit.value ? existingId.value : String(createdKey))}`);
  } catch {
    saveFailed.value = true;
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <main class="page-wrap">
    <h1 class="page-title">{{ isEdit ? t("labels.editEvent") : t("labels.createEvent") }}</h1>
    <p v-if="loadFailed" class="empty-state">{{ t("labels.eventLoadFailed") }}</p>
    <form v-else class="event-form" @submit.prevent="save">
      <label>{{ t("labels.editLocale") }}<select v-model="editLocale"><option v-for="item in enabledLocales" :key="item" :value="item">{{ item.toUpperCase() }}</option></select></label>
      <p v-if="isEdit" class="metadata-row"><strong>{{ t("labels.eventKey") }}</strong><span>{{ form.id }}</span></p>
      <label>{{ t("labels.category") }}<span class="category-select-row"><CategoryBadge v-if="selectedCategory" :category-key="selectedCategory.key" :icon="selectedCategory.icon" :color="selectedCategory.color" :label="selectedCategory.label" /><select v-model="form.categoryId" required><option v-for="category in categories" :key="category.id" :value="category.id">{{ category.key }} - {{ category.label }}</option></select></span></label>
      <label>{{ t("labels.startUtc") }}<input v-model="form.startUtc" type="datetime-local" required /></label>
      <label>{{ t("labels.endUtc") }}<input v-model="form.endUtc" type="datetime-local" /></label>
      <label>{{ t("labels.notifiedAtUtc") }}<input v-model="form.notifiedAtUtc" type="datetime-local" /></label>
      <label>{{ t("labels.title") }}<input v-model="form.title" required /><small v-if="fallbackByField.title">{{ t("labels.prefilledFrom", { locale: fallbackByField.title }) }}</small></label>
      <label>{{ t("labels.shortDescription") }}<textarea v-model="form.shortDescription" required /></label>
      <label>{{ t("labels.longDescription") }}<textarea v-model="form.longDescription" required /></label>
      <label>{{ t("labels.warning") }}<input v-model="form.warning" /></label>
      <label>{{ t("labels.location") }}<input v-model="form.location" /></label>
      <label class="status-field">
        <span>{{ t("labels.eventStatus") }}</span>
        <span class="status-input-group">
          <select v-model="form.statusType" required :aria-label="t('labels.eventStatusType')">
            <option value="waiting">{{ t("labels.waiting") }}</option>
            <option value="ongoing">{{ t("labels.ongoing") }}</option>
          </select>
          <input v-model="form.statusText" required :aria-label="t('labels.eventStatusText')" />
        </span>
        <small v-if="fallbackByField.status_text">{{ t("labels.prefilledFrom", { locale: fallbackByField.status_text }) }}</small>
      </label>
      <section class="timeline-section"><h2>{{ t("labels.maintenanceTimeline") }}</h2><button class="secondary-button" type="button" @click="addTimeline">{{ t("labels.addTimelineEntry") }}</button><article class="timeline-card" v-for="entry in timeline" :key="entry.id"><label>{{ t("labels.startUtc") }}<input v-model="entry.atUtc" type="datetime-local" /></label><label>{{ t("labels.title") }}<input v-model="entry.title" required /></label><label>{{ t("labels.details") }}<textarea v-model="entry.details" /></label><button class="secondary-button" type="button" @click="removeTimeline(entry.id)">{{ t("labels.remove") }}</button></article></section>
      <p v-if="saveFailed" class="empty-state">{{ t("labels.saveFailed") }}</p>
      <footer class="form-actions"><RouterLink class="secondary-button" :to="isEdit ? `/events/${existingId}` : '/events'">{{ t("labels.cancel") }}</RouterLink><button class="primary-button" type="submit" :disabled="saving">{{ saving ? t("labels.saving") : t("labels.save") }}</button></footer>
    </form>
  </main>
</template>

<style scoped>
.event-form { display: grid; gap: 0.8rem; margin-top: 1rem; max-width: 680px; }
.event-form label { display: grid; gap: 0.35rem; color: var(--muted-fg); font-weight: 700; }
.event-form input, .event-form select, .event-form textarea { border: 1px solid var(--control-border); border-radius: 0.7rem; padding: 0.65rem; background: var(--control-bg); color: var(--control-fg); }
.event-form textarea { min-height: 7rem; }
.category-select-row { align-items: center; display: flex; gap: 0.6rem; }
.category-select-row select { flex: 1 1 auto; min-width: 0; }
.status-input-group { display: flex; align-items: stretch; width: 100%; }
.status-input-group select { flex: 0 0 auto; min-width: 9.5rem; border-top-right-radius: 0; border-bottom-right-radius: 0; border-right: 0; font-weight: 700; }
.status-input-group input { flex: 1 1 auto; min-width: 0; border-top-left-radius: 0; border-bottom-left-radius: 0; }
.form-actions { display: flex; gap: 0.7rem; justify-content: flex-end; }
.secondary-button, .primary-button { border: 1px solid var(--control-border); border-radius: 0.55rem; padding: 0.55rem 0.8rem; background: var(--control-bg); color: var(--control-fg); text-decoration: none; }
.primary-button { border-color: rgba(72, 144, 255, 0.7); background: rgba(72, 144, 255, 0.22); }
</style>
