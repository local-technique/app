<script setup lang="ts">
import { RefreshCcw } from "@lucide/vue";
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { createCategory, deleteCategory, listCategories, updateCategory } from "../categories/api";
import CategoryIcon from "../categories/CategoryIcon.vue";
import type { CategoryInput, CategoryItem } from "../categories/types";
import type { LocaleCode } from "../common/i18n";

const { t, locale } = useI18n();
const categories = ref<CategoryItem[]>([]);
const editingId = ref("");
const defaultColor = "#9aaab1";
const defaultColors = ["#d73a49", "#f4511e", "#fbca04", "#0e8a16", "#006b75", "#1d76db", "#0052cc", "#5319e7", "#e99695", "#f9d0c4", "#fef2c0", "#c2e0c6", "#bfdadc", "#c5def5", "#bfd4f2", "#d4c5f9"];
const form = ref<CategoryInput>({ key: "", icon: "tag", color: defaultColor, labels: { en: "", fr: "" } });
const loading = ref(false);
const error = ref(false);
const colorPickerOpen = ref(false);

function activeLocale(): LocaleCode { return locale.value === "en" ? "en" : "fr"; }

async function load(): Promise<void> {
  loading.value = true; error.value = false;
  try { categories.value = await listCategories(activeLocale(), true); } catch { error.value = true; } finally { loading.value = false; }
}

function randomColor(): string { return `#${Math.floor(Math.random() * 0x1000000).toString(16).padStart(6, "0")}`; }
function reset(): void { editingId.value = ""; form.value = { key: "", icon: "tag", color: defaultColor, labels: { en: "", fr: "" } }; }
function edit(category: CategoryItem): void { editingId.value = category.id; form.value = { key: category.key, icon: category.icon, color: category.color, labels: { en: category.labels.en ?? "", fr: category.labels.fr ?? "" } }; }
function closeColorPicker(event: FocusEvent): void {
  if (event.currentTarget instanceof HTMLElement && event.relatedTarget instanceof Node && event.currentTarget.contains(event.relatedTarget)) return;
  colorPickerOpen.value = false;
}

async function save(): Promise<void> {
  error.value = false;
  try {
    if (editingId.value) await updateCategory(editingId.value, form.value);
    else await createCategory(form.value);
    reset(); await load();
  } catch { error.value = true; }
}

async function remove(id: string): Promise<void> {
  if (!window.confirm(t("labels.deleteCategoryConfirm"))) return;
  error.value = false;
  try { await deleteCategory(id); await load(); } catch { error.value = true; }
}

void load();
</script>

<template>
  <main class="page-wrap admin-page">
    <h1 class="page-title">{{ t("labels.adminCategoriesTitle") }}</h1>
    <p><a href="https://lucide.dev/icons/" target="_blank" rel="noreferrer">{{ t("labels.categoryIconHelp") }}</a></p>
    <form class="category-form" @submit.prevent="save">
      <label>{{ t("labels.categoryKey") }}<input v-model="form.key" required /></label>
      <label>{{ t("labels.categoryIcon") }}<span class="icon-input-row"><CategoryIcon data-testid="category-icon-input-preview" :name="form.icon" :style="{ color: form.color }" /><input v-model="form.icon" required /><span class="color-field" @focusin="colorPickerOpen = true" @focusout="closeColorPicker"><label class="sr-only" for="category-color-input">Color</label><span class="color-input-wrap"><span class="color-preview" :style="{ background: form.color }" aria-hidden="true"></span><input id="category-color-input" v-model="form.color" pattern="#[0-9a-fA-F]{6}" required @focus="colorPickerOpen = true" /><button class="icon-button" type="button" aria-label="Set random color" @click="form.color = randomColor()"><RefreshCcw :size="18" aria-hidden="true" /></button></span><span v-if="colorPickerOpen" class="color-popover"><span>Choose from default colors</span><span class="color-grid"><button v-for="color in defaultColors" :key="color" class="color-swatch" type="button" :aria-label="`Use color ${color}`" :style="{ background: color }" @click="form.color = color"></button></span></span></span></span></label>
      <label>{{ t("labels.labelEn") }}<input v-model="form.labels.en" required /></label>
      <label>{{ t("labels.labelFr") }}<input v-model="form.labels.fr" required /></label>
      <footer><button class="secondary-button" type="button" @click="reset">{{ t("labels.cancel") }}</button><button class="primary-button" type="submit">{{ t("labels.save") }}</button></footer>
    </form>
    <p v-if="loading" class="empty-state">{{ t("labels.loadingCategories") }}</p>
    <p v-if="error" class="empty-state">{{ t("labels.categoriesError") }}</p>
    <div class="admin-table-wrap" v-if="categories.length">
      <table class="admin-table"><thead><tr><th>{{ t("labels.categoryId") }}</th><th>{{ t("labels.categoryKey") }}</th><th>{{ t("labels.categoryIcon") }}</th><th>{{ t("labels.label") }}</th><th>{{ t("labels.actions") }}</th></tr></thead><tbody><tr v-for="category in categories" :key="category.id"><td>{{ category.id.slice(0, 7) }}</td><td>{{ category.key }}</td><td><span class="icon-cell" :style="{ color: category.color }"><CategoryIcon :data-testid="`category-icon-list-${category.key}`" :name="category.icon" />{{ category.icon }}</span></td><td>{{ category.label }}</td><td><button class="secondary-button" type="button" @click="edit(category)">{{ t("labels.edit") }}</button> <button class="secondary-button" type="button" @click="remove(category.id)">{{ t("labels.delete") }}</button></td></tr></tbody></table>
    </div>
  </main>
</template>

<style scoped>
.admin-page { width: 100%; max-width: none; }
.category-form { display: grid; gap: 0.7rem; max-width: 560px; margin: 1rem 0; }
.category-form label, .color-field { display: grid; gap: 0.35rem; color: var(--muted-fg); font-weight: 700; }
.category-form input { border: 1px solid var(--control-border); border-radius: 0.7rem; padding: 0.65rem; background: var(--control-bg); color: var(--control-fg); }
.icon-input-row, .icon-cell { display: inline-flex; align-items: center; gap: 0.55rem; }
.icon-input-row input { flex: 1; min-width: 0; }
.icon-input-row .color-field { flex: 0 0 auto; }
.color-input-wrap { display: grid; grid-template-columns: 2.25rem minmax(0, 12rem) auto; gap: 0.45rem; align-items: center; }
.color-field { position: relative; }
.color-preview { border: 1px solid var(--control-border); border-radius: 0.45rem; height: 2.25rem; width: 2.25rem; }
.icon-button { align-items: center; border: 1px solid var(--control-border); border-radius: 0.55rem; background: var(--control-bg); color: var(--control-fg); cursor: pointer; display: inline-flex; height: 2.25rem; justify-content: center; width: 2.25rem; }
.color-popover { border: 1px solid var(--control-border); border-radius: 0.45rem; background: var(--panel-bg); color: var(--page-fg); display: grid; gap: 0.8rem; left: 2.7rem; padding: 1rem; position: absolute; top: calc(100% + 0.35rem); width: fit-content; z-index: 20; }
.color-grid { display: grid; grid-template-columns: repeat(8, 2rem); gap: 0.45rem; }
.color-swatch { border: 0; border-radius: 0.4rem; cursor: pointer; height: 2rem; width: 2rem; }
.sr-only { clip: rect(0, 0, 0, 0); border: 0; height: 1px; margin: -1px; overflow: hidden; padding: 0; position: absolute; white-space: nowrap; width: 1px; }
.category-form footer { display: flex; gap: 0.7rem; }
.admin-table-wrap { overflow-x: auto; border: 1px solid var(--border-color); border-radius: 0.9rem; background: var(--panel-bg); }
.admin-table { width: 100%; border-collapse: collapse; min-width: 720px; }
.admin-table th, .admin-table td { padding: 0.72rem; border-bottom: 1px solid var(--border-color); text-align: left; }
.secondary-button, .primary-button { border: 1px solid var(--control-border); border-radius: 0.55rem; padding: 0.45rem 0.7rem; background: var(--control-bg); color: var(--control-fg); cursor: pointer; }
.primary-button { border-color: rgba(72, 144, 255, 0.7); background: rgba(72, 144, 255, 0.22); }
</style>
