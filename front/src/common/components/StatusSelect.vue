<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { Activity, ChevronDown, Hourglass } from "@lucide/vue";

const { t } = useI18n();
const model = defineModel<string>({ required: true });

const options = computed(() => [
  { value: "waiting", icon: Hourglass, label: t("labels.blocked") },
  { value: "ongoing", icon: Activity, label: t("labels.ongoing") },
] as const);

const open = ref(false);
const selected = computed(() => options.value.find((o) => o.value === model.value) ?? options.value[0]);
const isWaiting = computed(() => model.value === "waiting");

function select(value: string): void {
  model.value = value;
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
  <div class="StatusSelect" @mouseleave="close">
    <button class="status-trigger" :class="{ 'status-waiting': isWaiting }" type="button" @click="toggle">
      <component :is="selected.icon" :size="16" />
      <span class="status-trigger-label">{{ selected.label }}</span>
      <ChevronDown :size="14" class="status-arrow" />
    </button>
    <div v-if="open" class="status-dropdown">
      <button
        v-for="opt in options"
        :key="opt.value"
        class="status-option"
        :class="{ 'status-option-selected': opt.value === model, 'status-waiting': opt.value === 'waiting' }"
        type="button"
        @click="select(opt.value)"
      >
        <component :is="opt.icon" :size="16" />
        <span class="status-option-label">{{ opt.label }}</span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.StatusSelect {
  position: relative;
  display: inline-flex;
}

.status-trigger {
  display: inline-flex;
  align-items: center;
  gap: 0.3rem;
  border: 1px solid var(--control-border);
  border-radius: 0.7rem 0 0 0.7rem;
  border-right: 0;
  padding: 0.65rem 0.6rem;
  background: var(--control-bg);
  color: var(--control-fg);
  cursor: pointer;
  font-size: 0.92rem;
  white-space: nowrap;
  min-height: 2.35rem;
}

.status-trigger:hover {
  border-color: var(--muted-fg);
  border-right: 0;
}

.status-trigger-label {
  font-weight: 700;
  letter-spacing: 0.02em;
  line-height: 1;
}

.status-arrow {
  color: var(--muted-fg);
}

.status-dropdown {
  position: absolute;
  top: calc(100% + 2px);
  left: 0;
  z-index: 50;
  display: grid;
  gap: 2px;
  background: var(--panel-bg);
  border: 1px solid var(--control-border);
  border-radius: 0.7rem;
  padding: 4px;
  min-width: 140px;
  box-shadow: 0 4px 12px rgba(0,0,0,0.15);
}

.status-option {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  border: none;
  border-radius: 0.55rem;
  padding: 0.45rem 0.6rem;
  background: transparent;
  cursor: pointer;
  text-align: left;
  width: 100%;
  color: var(--control-fg);
  font-size: 0.92rem;
}

.status-option:hover {
  background: rgba(127,127,127,0.1);
}

.status-option-selected {
  background: rgba(72,144,255,0.12);
}

.status-option-label {
  font-weight: 600;
  line-height: 1;
}

.status-waiting { color: #e67e22; }
</style>
