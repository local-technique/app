<script setup lang="ts">
import { computed, ref } from "vue";
import { ChevronDown } from "@lucide/vue";
import CategoryIcon from "./CategoryIcon.vue";
import type { CategoryItem } from "./types";

const props = defineProps<{
  categories: CategoryItem[];
  modelValue: string;
}>();
const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

const open = ref(false);
const selected = computed(() => props.categories.find((c) => c.id === props.modelValue) ?? null);

function select(cat: CategoryItem): void {
  emit("update:modelValue", cat.id);
  open.value = false;
}

function toggle(): void {
  open.value = !open.value;
}

function close(): void {
  open.value = false;
}
</script>

<template>
  <div class="CategorySelect" @mouseleave="close">
    <button class="category-trigger" type="button" :aria-label="selected?.label" @click="toggle">
      <CategoryIcon v-if="selected" :name="selected.icon" :size="18" :style="{ color: selected.color }" />
      <span v-if="selected" class="category-trigger-code">{{ selected.key }}</span>
      <span v-else class="category-trigger-code category-trigger-placeholder">{{ "Select" }}</span>
      <ChevronDown :size="14" class="category-arrow" />
    </button>
    <div v-if="open" class="category-dropdown">
      <button
        v-for="cat in categories"
        :key="cat.id"
        class="category-option"
        :class="{ 'category-option-selected': cat.id === modelValue }"
        type="button"
        @click="select(cat)"
      >
        <span class="cat-option-icon" :style="{ color: cat.color }">
          <CategoryIcon :name="cat.icon" :size="18" />
        </span>
        <span class="cat-option-key" :style="{ color: cat.color }">{{ cat.key }}</span>
        <span class="cat-option-label">{{ cat.label }}</span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.CategorySelect {
  position: relative;
  display: inline-flex;
}

.category-trigger {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  border: 1px solid var(--control-border);
  border-radius: 0.7rem;
  padding: 0.45rem 0.6rem;
  background: var(--control-bg);
  color: var(--control-fg);
  cursor: pointer;
  font-size: 0.92rem;
  min-height: 2.35rem;
  white-space: nowrap;
}

.category-trigger:hover {
  border-color: var(--muted-fg);
}

.category-trigger-code {
  font-weight: 800;
  letter-spacing: 0.04em;
}

.category-arrow {
  color: var(--muted-fg);
  margin-left: 0.15rem;
}

.category-trigger-placeholder {
  color: var(--muted-fg);
  font-weight: 400;
}

.category-dropdown {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  z-index: 50;
  display: grid;
  gap: 2px;
  background: var(--panel-bg);
  border: 1px solid var(--control-border);
  border-radius: 0.7rem;
  padding: 4px;
  min-width: 180px;
  box-shadow: 0 4px 12px rgba(0,0,0,0.15);
}

.category-option {
  display: grid;
  grid-template-columns: auto auto 1fr;
  align-items: center;
  gap: 0.3rem 0.4rem;
  border: none;
  border-radius: 0.55rem;
  padding: 0.35rem 0.5rem;
  background: transparent;
  cursor: pointer;
  text-align: left;
  width: 100%;
}

.category-option:hover {
  background: rgba(127,127,127,0.1);
}

.category-option-selected {
  background: rgba(72,144,255,0.12);
}

.cat-option-key {
  font-size: 0.78rem;
  font-weight: 800;
  letter-spacing: 0.04em;
  line-height: 1;
  text-transform: uppercase;
}

.cat-option-label {
  grid-column: 3;
  font-size: 0.72rem;
  color: var(--muted-fg);
  line-height: 1;
  white-space: nowrap;
}

.cat-option-icon {
  display: inline-flex;
  align-items: center;
  line-height: 0;
}
</style>
