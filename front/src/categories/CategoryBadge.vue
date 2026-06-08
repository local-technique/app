<script setup lang="ts">
import CategoryIcon from "./CategoryIcon.vue";

defineProps<{
  code: string;
  icon: string;
  color?: string;
  label?: string;
  variant?: "rail" | "inline";
}>();
</script>

<template>
  <span
    :class="['category-badge', `category-badge-${variant ?? 'inline'}`]"
    :style="{
      '--category-color': color ?? '#9aaab1',
      ...((variant ?? 'inline') === 'rail' ? { alignContent: 'center', gap: '0.18rem' } : { gap: '0.21rem' }),
    }"
    :aria-label="label ? `${code} - ${label}` : code"
  >
    <CategoryIcon class="category-badge-icon" :name="icon" :size="variant === 'rail' ? 24 : 18" />
    <span class="category-badge-code">{{ code }}</span>
  </span>
</template>

<style scoped>
.category-badge { color: var(--category-color, var(--muted-fg)); }
.category-badge-icon { flex: 0 0 auto; }
.category-badge-rail { align-content: center; align-items: center; display: grid; gap: 0.18rem; justify-items: center; min-width: 3.2rem; padding-right: 0.72rem; position: relative; }
.category-badge-rail::after { background: var(--category-color, var(--border-color)); bottom: 0.1rem; content: ""; position: absolute; right: 0; top: 0.1rem; width: 1px; }
.category-badge-rail .category-badge-code { font-size: 0.72rem; font-weight: 800; letter-spacing: 0.06em; line-height: 1; text-transform: uppercase; }
.category-badge-inline { align-items: center; display: inline-flex; gap: 0.21rem; }
.category-badge-inline .category-badge-code { font-weight: 800; letter-spacing: 0.04em; }
</style>
