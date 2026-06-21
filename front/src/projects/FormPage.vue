<script setup lang="ts">
import { computed, inject, onMounted, onUnmounted, ref, watch, type Ref } from "vue";
import { Info } from "@lucide/vue";
import { useI18n } from "vue-i18n";
import { useRoute, useRouter } from "vue-router";
import { listCategories } from "../categories/api";
import CategorySelect from "../categories/CategorySelect.vue";
import StatusSelect from "../common/components/StatusSelect.vue";
import type { CategoryItem } from "../categories/types";
import DateInput from "../common/components/DateInput.vue";
import { toDateLocalInput, toUtcFromDateLocalInput, todayDateInput } from "../common/dateInput";
import { parseUtc } from "../common/date";
import type { LocaleCode } from "../common/i18n";
import TimelineList from "../common/components/TimelineList.vue";
import type { TimelineEntry } from "../common/components/TimelineList.vue";
import { apiProjectsRepository } from "./repositories/apiProjectsRepository";
import type { ProjectStoredStatus } from "./types";

const { t } = useI18n();
const route = useRoute();
const router = useRouter();
const existingId = computed(() => (typeof route.params.id === "string" ? route.params.id : ""));
const isEdit = computed(() => existingId.value.length > 0);
const cancelPath = computed(() => (isEdit.value ? `/projects/${encodeURIComponent(existingId.value)}` : "/projects"));
const categories = ref<CategoryItem[]>([]);
const selectedLocale = inject("selectedLocale", ref<LocaleCode>("en")) as Ref<LocaleCode>;
const saving = ref(false);
const loadFailed = ref(false);
const saveFailed = ref(false);
const form = ref({ id: "", categoryId: "", startUtc: "", endUtc: "", statusType: "waiting" as ProjectStoredStatus, statusText: "", title: "", description: "" });
const statusPlaceholder = computed(() => form.value.statusType === "waiting" ? t("labels.statusPlaceholderBlocking") : t("labels.statusPlaceholderOngoing"));
const timeline = ref<TimelineEntry[]>([]);
const showMarkdownHelp = ref(false);
const markdownHelpUrl = computed(() => selectedLocale.value === "fr" ? "https://www.markdownlang.com/fr/cheatsheet/" : "https://www.markdownlang.com/cheatsheet/");
function onDocumentClick(e: MouseEvent): void {
  if (showMarkdownHelp.value) {
    const target = e.target as HTMLElement;
    if (!target.closest(".markdown-info-icon") && !target.closest(".markdown-help-popover")) {
      showMarkdownHelp.value = false;
    }
  }
}
function toggleMarkdownHelp(): void {
  showMarkdownHelp.value = !showMarkdownHelp.value;
}
onMounted(() => document.addEventListener("click", onDocumentClick));
onUnmounted(() => document.removeEventListener("click", onDocumentClick));

function field(fields: Array<{ fieldKey: string; value: string }>, key: string): string { return fields.find((item) => item.fieldKey === key)?.value ?? ""; }

function applyFields(fields: Array<{ fieldKey: string; value: string; fallbackLocale?: string | null }>): void {
  for (const field of fields) {
    if (field.fieldKey === "title") form.value.title = field.value;
    if (field.fieldKey === "description") form.value.description = field.value;
    if (field.fieldKey === "status_text") form.value.statusText = field.value;
  }
}

function formatTimeline(editTimeline: Array<{ id: string; atUtc: string | null; fields: Array<{ fieldKey: string; value: string }> }>): TimelineEntry[] {
  return editTimeline.map((item) => {
    const atDate = item.atUtc ? parseUtc(item.atUtc) : null;
    return {
      id: item.id,
      atLabel: atDate ? new Intl.DateTimeFormat(selectedLocale.value, { dateStyle: "medium", timeStyle: "short" }).format(atDate) : "Pending",
      atDateLabel: atDate ? new Intl.DateTimeFormat(selectedLocale.value, { dateStyle: "medium" }).format(atDate) : "",
      atTimeLabel: atDate ? new Intl.DateTimeFormat(selectedLocale.value, { timeStyle: "short" }).format(atDate) : "",
      isPending: !item.atUtc,
      title: field(item.fields, "title"),
      details: field(item.fields, "details"),
    };
  });
}

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
    if (!data) {
      loadFailed.value = true;
      return;
    }
    form.value.id = data.id;
    form.value.categoryId = data.categoryId;
    form.value.startUtc = toDateLocalInput(data.startUtc);
    form.value.endUtc = toDateLocalInput(data.endUtc);
    form.value.statusType = data.statusType;
    applyFields(data.fields);
    timeline.value = formatTimeline(data.timeline);
  } catch {
    loadFailed.value = true;
  }
}

watch(() => route.params.id, () => void load(), { immediate: true });

async function save(): Promise<void> {
  saving.value = true;
  saveFailed.value = false;
  try {
    const createdKey = await apiProjectsRepository.save(
      {
        categoryId: form.value.categoryId,
        startUtc: toUtcFromDateLocalInput(form.value.startUtc) ?? "",
        endUtc: toUtcFromDateLocalInput(form.value.endUtc),
        statusType: form.value.statusType,
        locale: selectedLocale.value,
        fields: { title: form.value.title, description: form.value.description, status_text: form.value.statusText },
        replaceTimeline: false,
        timeline: [],
      },
      isEdit.value ? existingId.value : undefined,
    );
    await router.push(`/projects/${encodeURIComponent(isEdit.value ? existingId.value : String(createdKey))}`);
  } catch {
    saveFailed.value = true;
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <main class="page-wrap">
    <h1 class="page-title">{{ isEdit ? t("labels.editProject") : t("labels.createProject") }}</h1>
    <p v-if="loadFailed" class="empty-state">{{ t("labels.projectsLoadFailed") }}</p>
    <form v-else class="project-form" @submit.prevent="save">
      <div class="title-category-row">
        <CategorySelect v-model="form.categoryId" :categories="categories" />
        <span v-if="isEdit" class="title-key-wrap"><span class="title-key">{{ form.id }}</span></span>
        <input v-model="form.title" required class="title-input" :placeholder="t('labels.title')" />
      </div>

      <span class="status-input-group">
        <StatusSelect v-model="form.statusType" />
        <input v-model="form.statusText" required :placeholder="statusPlaceholder" :aria-label="t('labels.projectStatusText')" />
      </span>

      <div class="date-row">
        <div class="date-field"><label>{{ t("labels.startUtc") }}</label><DateInput v-model="form.startUtc" required /></div>
        <div v-if="isEdit" class="date-field"><label>{{ t("labels.endUtc") }}</label><DateInput v-model="form.endUtc" /></div>
      </div>

      <label class="desc-field">
        <span class="desc-label">{{ t("labels.description") }}<span class="info-icon-wrap"><Info :size="16" class="markdown-info-icon" @click.prevent="toggleMarkdownHelp" @keydown.enter.prevent="toggleMarkdownHelp" tabindex="0" role="button" :aria-label="t('labels.markdownHelpTitle')" />
          <div v-if="showMarkdownHelp" class="markdown-help-popover" @click.stop>
            <p>{{ t("labels.markdownHelpText") }}</p>
            <a :href="markdownHelpUrl" target="_blank" rel="noopener noreferrer">{{ t("labels.markdownHelpLink") }}</a>
          </div>
        </span></span>
        <textarea v-model="form.description" required />
      </label>

      <section v-if="timeline.length" class="timeline-section">
        <h2>{{ t("labels.projectTimeline") }}</h2>
        <TimelineList :entries="timeline" />
      </section>

      <p v-if="saveFailed" class="empty-state">{{ t("labels.saveFailed") }}</p>
      <footer class="form-actions"><RouterLink class="secondary-button" :to="cancelPath">{{ t("labels.cancel") }}</RouterLink><button class="primary-button" type="submit" :disabled="saving">{{ saving ? t("labels.saving") : t("labels.save") }}</button></footer>
    </form>
  </main>
</template>

<style scoped>
.project-form { display: grid; gap: 0.8rem; margin-top: 1rem; max-width: 680px; }
.project-form label { display: grid; gap: 0.35rem; color: var(--muted-fg); font-weight: 700; }
.project-form input, .project-form select, .project-form textarea { border: 1px solid var(--control-border); border-radius: 0.7rem; padding: 0.65rem; background: var(--control-bg); color: var(--control-fg); }
.project-form textarea { min-height: 9rem; }
.title-category-row { display: flex; align-items: center; gap: 0.5rem; }
.title-key-wrap { font-size: clamp(1.45rem, 2vw, 2rem); display: inline-flex; }
.title-key { color: var(--muted-fg); font-family: ui-monospace, monospace; font-weight: 600; white-space: nowrap; flex-shrink: 0; }
.title-input { flex: 1; min-width: 0; }
.date-row { display: grid; grid-template-columns: 1fr 1fr; gap: 0.8rem; }
.date-field { display: flex; align-items: center; gap: 0.4rem; }
.date-field label { color: var(--muted-fg); font-weight: 700; white-space: nowrap; font-size: 0.85rem; }
.date-field input { flex: 1; min-width: 0; }
.status-input-group { display: flex; align-items: stretch; width: 100%; }
.status-input-group select { flex: 0 0 auto; min-width: 9.5rem; border-top-right-radius: 0; border-bottom-right-radius: 0; border-right: 0; font-weight: 700; }
.status-input-group input { flex: 1 1 auto; min-width: 0; border-top-left-radius: 0; border-bottom-left-radius: 0; }
.form-actions { display: flex; gap: 0.7rem; justify-content: flex-end; }
.secondary-button, .primary-button { border: 1px solid var(--control-border); border-radius: 0.55rem; padding: 0.55rem 0.8rem; background: var(--control-bg); color: var(--control-fg); text-decoration: none; }
.primary-button { border-color: rgba(72, 144, 255, 0.7); background: rgba(72, 144, 255, 0.22); }
.markdown-info-icon { cursor: pointer; color: var(--muted-fg); }
.info-icon-wrap { display: inline-flex; }
.desc-field .desc-label { position: relative; display: inline-flex; align-items: center; gap: 0.3rem; }
.markdown-help-popover { position: absolute; top: calc(100% + 0.5rem); left: 5.2rem; z-index: 100; width: 270px; border: 1px solid var(--border-color); border-radius: 0.5rem; padding: 0.75rem; background: var(--panel-bg); box-shadow: 0 4px 16px rgba(0, 0, 0, 0.18); display: grid; gap: 0.5rem; font-weight: 400; color: var(--control-fg); }
.markdown-help-popover::before { content: ''; position: absolute; top: -6px; left: 0.6rem; width: 10px; height: 10px; background: var(--panel-bg); border: 1px solid var(--border-color); border-right: none; border-bottom: none; transform: rotate(45deg); }
.markdown-help-popover a { color: #4890ff; }
.markdown-help-popover p { margin: 0; }
@media (max-width: 520px) {
  .status-input-group { display: grid; }
  .status-input-group select { min-width: 0; border-right: 1px solid var(--control-border); border-bottom: 0; border-radius: 0.7rem 0.7rem 0 0; }
  .status-input-group input { border-radius: 0 0 0.7rem 0.7rem; }
  .date-row { grid-template-columns: 1fr; }
}
</style>
