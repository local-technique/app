<script setup lang="ts">
import { Calendar } from "@lucide/vue";
import { ref } from "vue";

const model = defineModel<string>({ required: true });
const props = defineProps<{ required?: boolean }>();
const uid = `di-${Math.random().toString(36).slice(2, 9)}`;

function onCalendarPick(e: Event): void {
  const target = e.target as HTMLInputElement;
  if (target.value) model.value = target.value;
}

function triggerCalendar(): void {
  const el = document.getElementById(uid) as HTMLInputElement | null;
  if (el?.showPicker) el.showPicker();
}
</script>

<template>
  <span class="date-input-wrap">
    <input :id="uid" type="date" class="date-native" tabindex="-1" @change="onCalendarPick" />
    <input v-model="model" type="text" inputmode="numeric" placeholder="YYYY-MM-DD" pattern="\d{4}-\d{2}-\d{2}" class="date-text" :required="props.required" />
    <button type="button" class="date-cal-btn" @click="triggerCalendar" tabindex="-1"><Calendar :size="16" /></button>
  </span>
</template>

<style scoped>
.date-input-wrap { position: relative; display: inline-flex; align-items: center; }
.date-native { position: absolute; opacity: 0; width: 0; height: 0; overflow: hidden; pointer-events: none; }
.date-text { flex: 1; min-width: 0; width: 100%; border: 1px solid var(--control-border); border-radius: 0.7rem; padding: 0.65rem 2rem 0.65rem 0.65rem; background: var(--control-bg); color: var(--control-fg); box-sizing: border-box; }
.date-cal-btn { position: absolute; right: 0.3rem; display: flex; align-items: center; justify-content: center; padding: 0.3rem; cursor: pointer; border: none; background: none; color: var(--muted-fg); border-radius: 0.4rem; }
.date-cal-btn:hover { color: var(--control-fg); background: var(--control-border); }
</style>
