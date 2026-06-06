<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useRoute, useRouter } from "vue-router";
import { listCategories } from "../categories/api";
import type { CategoryItem } from "../categories/types";
import { toDateTimeLocalInput, toUtcFromDateTimeLocalInput } from "../common/dateInput";
import type { LocaleCode } from "../common/i18n";
import { apiEventsRepository } from "./repositories/apiEventsRepository";

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
  title: "",
  shortDescription: "",
  longDescription: "",
  warning: "",
  location: "",
});

function activeLocale(): LocaleCode {
  return editLocale.value === "en" ? "en" : "fr";
}

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
    applyFields(data.fields);
  } catch {
    loadFailed.value = true;
  }
}

watch([() => route.params.id, editLocale], () => void load(), { immediate: true });

async function save(): Promise<void> {
  saving.value = true;
  saveFailed.value = false;
  try {
    await apiEventsRepository.save(
      {
        id: form.value.id.trim(),
        categoryId: form.value.categoryId,
        startUtc: toUtcFromDateTimeLocalInput(form.value.startUtc) ?? "",
        endUtc: toUtcFromDateTimeLocalInput(form.value.endUtc),
        notifiedAtUtc: toUtcFromDateTimeLocalInput(form.value.notifiedAtUtc),
        locale: activeLocale(),
        fields: {
          title: form.value.title,
          short_description: form.value.shortDescription,
          long_description: form.value.longDescription,
          warning: form.value.warning,
          location: form.value.location,
        },
      },
      isEdit.value ? existingId.value : undefined,
    );
    await router.push(`/events/${encodeURIComponent(form.value.id.trim())}`);
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
      <label>{{ t("labels.eventId") }}<input v-model="form.id" required :disabled="isEdit" /></label>
      <label>{{ t("labels.category") }}<select v-model="form.categoryId" required><option v-for="category in categories" :key="category.id" :value="category.id">{{ category.code }} - {{ category.label }}</option></select></label>
      <label>{{ t("labels.startUtc") }}<input v-model="form.startUtc" type="datetime-local" required /></label>
      <label>{{ t("labels.endUtc") }}<input v-model="form.endUtc" type="datetime-local" /></label>
      <label>{{ t("labels.notifiedAtUtc") }}<input v-model="form.notifiedAtUtc" type="datetime-local" /></label>
      <label>{{ t("labels.title") }}<input v-model="form.title" required /><small v-if="fallbackByField.title">{{ t("labels.prefilledFrom", { locale: fallbackByField.title }) }}</small></label>
      <label>{{ t("labels.shortDescription") }}<textarea v-model="form.shortDescription" required /></label>
      <label>{{ t("labels.longDescription") }}<textarea v-model="form.longDescription" required /></label>
      <label>{{ t("labels.warning") }}<input v-model="form.warning" /></label>
      <label>{{ t("labels.location") }}<input v-model="form.location" /></label>
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
.form-actions { display: flex; gap: 0.7rem; justify-content: flex-end; }
.secondary-button, .primary-button { border: 1px solid var(--control-border); border-radius: 0.55rem; padding: 0.55rem 0.8rem; background: var(--control-bg); color: var(--control-fg); text-decoration: none; }
.primary-button { border-color: rgba(72, 144, 255, 0.7); background: rgba(72, 144, 255, 0.22); }
</style>
