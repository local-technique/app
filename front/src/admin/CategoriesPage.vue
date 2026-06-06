<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { createCategory, deleteCategory, listCategories, updateCategory } from "../categories/api";
import CategoryIcon from "../categories/CategoryIcon.vue";
import type { CategoryInput, CategoryItem } from "../categories/types";
import type { LocaleCode } from "../common/i18n";

const { t, locale } = useI18n();
const categories = ref<CategoryItem[]>([]);
const editingId = ref("");
const form = ref<CategoryInput>({ id: "", code: "", icon: "tag", labels: { en: "", fr: "" } });
const loading = ref(false);
const error = ref(false);

function activeLocale(): LocaleCode { return locale.value === "en" ? "en" : "fr"; }

async function load(): Promise<void> {
  loading.value = true; error.value = false;
  try { categories.value = await listCategories(activeLocale(), true); } catch { error.value = true; } finally { loading.value = false; }
}

function reset(): void { editingId.value = ""; form.value = { id: "", code: "", icon: "tag", labels: { en: "", fr: "" } }; }
function edit(category: CategoryItem): void { editingId.value = category.id; form.value = { id: category.id, code: category.code, icon: category.icon, labels: { en: category.labels.en ?? "", fr: category.labels.fr ?? "" } }; }

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
      <label>{{ t("labels.categoryId") }}<input v-model="form.id" :disabled="Boolean(editingId)" required /></label>
      <label>{{ t("labels.categoryCode") }}<input v-model="form.code" required /></label>
      <label>{{ t("labels.categoryIcon") }}<span class="icon-input-row"><CategoryIcon data-testid="category-icon-input-preview" :name="form.icon" /><input v-model="form.icon" required /></span></label>
      <label>{{ t("labels.labelEn") }}<input v-model="form.labels.en" required /></label>
      <label>{{ t("labels.labelFr") }}<input v-model="form.labels.fr" required /></label>
      <footer><button class="secondary-button" type="button" @click="reset">{{ t("labels.cancel") }}</button><button class="primary-button" type="submit">{{ t("labels.save") }}</button></footer>
    </form>
    <p v-if="loading" class="empty-state">{{ t("labels.loadingCategories") }}</p>
    <p v-if="error" class="empty-state">{{ t("labels.categoriesError") }}</p>
    <div class="admin-table-wrap" v-if="categories.length">
      <table class="admin-table"><thead><tr><th>{{ t("labels.categoryId") }}</th><th>{{ t("labels.categoryCode") }}</th><th>{{ t("labels.categoryIcon") }}</th><th>{{ t("labels.label") }}</th><th>{{ t("labels.actions") }}</th></tr></thead><tbody><tr v-for="category in categories" :key="category.id"><td>{{ category.id }}</td><td>{{ category.code }}</td><td><span class="icon-cell"><CategoryIcon :data-testid="`category-icon-list-${category.id}`" :name="category.icon" />{{ category.icon }}</span></td><td>{{ category.label }}</td><td><button class="secondary-button" type="button" @click="edit(category)">{{ t("labels.edit") }}</button> <button class="secondary-button" type="button" @click="remove(category.id)">{{ t("labels.delete") }}</button></td></tr></tbody></table>
    </div>
  </main>
</template>

<style scoped>
.admin-page { width: 100%; max-width: none; }
.category-form { display: grid; gap: 0.7rem; max-width: 560px; margin: 1rem 0; }
.category-form label { display: grid; gap: 0.35rem; color: var(--muted-fg); font-weight: 700; }
.category-form input { border: 1px solid var(--control-border); border-radius: 0.7rem; padding: 0.65rem; background: var(--control-bg); color: var(--control-fg); }
.icon-input-row, .icon-cell { display: inline-flex; align-items: center; gap: 0.55rem; }
.icon-input-row input { flex: 1; min-width: 0; }
.category-form footer { display: flex; gap: 0.7rem; }
.admin-table-wrap { overflow-x: auto; border: 1px solid var(--border-color); border-radius: 0.9rem; background: var(--panel-bg); }
.admin-table { width: 100%; border-collapse: collapse; min-width: 720px; }
.admin-table th, .admin-table td { padding: 0.72rem; border-bottom: 1px solid var(--border-color); text-align: left; }
.secondary-button, .primary-button { border: 1px solid var(--control-border); border-radius: 0.55rem; padding: 0.45rem 0.7rem; background: var(--control-bg); color: var(--control-fg); cursor: pointer; }
.primary-button { border-color: rgba(72, 144, 255, 0.7); background: rgba(72, 144, 255, 0.22); }
</style>
