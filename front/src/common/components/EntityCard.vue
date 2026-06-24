<script setup lang="ts">
import { Activity, CalendarClock, CheckCircle2, Hourglass } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import CategoryBadge from "../../categories/CategoryBadge.vue";
import LatestTimelineEntry from "./LatestTimelineEntry.vue";
import type { LatestEntry } from "./LatestTimelineEntry.vue";

const props = defineProps<{
  id: string;
  title: string;
  statusType: "waiting" | "ongoing" | "finished" | "planned";
  statusText: string;
  dateLabel: string;
  to: string;
  query?: Record<string, string>;
  categoryKey: string;
  categoryIcon: string;
  categoryColor?: string;
  categoryLabel?: string;
  timelineEntry?: LatestEntry | null;
  warning?: string;
  location?: string;
}>();

const { t } = useI18n();

const iconComponent = computed(() => {
  if (props.statusType === "ongoing") return Activity;
  if (props.statusType === "planned") return CalendarClock;
  if (props.statusType === "finished") return CheckCircle2;
  return Hourglass;
});

const showStatusLine = computed(() => {
  if (props.statusType === "finished") return false;
  if (props.statusType === "ongoing" && !props.statusText) return false;
  if (props.statusType === "planned" && !props.statusText) return false;
  return true;
});

const isBlocked = computed(() => props.statusType === "waiting");

const statusLabelKey = computed(() => props.statusType === "waiting" ? "blocked" : props.statusType);

const statusDisplayText = computed(() => {
  if (props.statusText) return t("labels." + statusLabelKey.value) + " - " + props.statusText;
  if (props.statusType === "finished") return t("labels.finished");
  if (props.statusType === "planned") return "";
  return t("labels." + statusLabelKey.value);
});

const hasCategory = computed(() => props.categoryKey.length > 0);

const isFinished = computed(() => props.statusType === "finished");
</script>

<template>
  <article :class="['timeline-card', 'entity-card', { 'timeline-card-past': isFinished }]" :style="categoryColor ? { '--category-color': categoryColor } : undefined">
    <CategoryBadge v-if="hasCategory" :category-key="categoryKey" :icon="categoryIcon" :color="categoryColor" :label="categoryLabel" variant="rail" />
    <div class="entity-card-content">
      <h3 class="timeline-card-title">
        <component :is="iconComponent" :size="16" class="status-icon" /> <span class="timeline-meta entity-key">{{ id }}</span>
        <RouterLink :to="query ? { path: to, query } : to">{{ title }}</RouterLink>
      </h3>
      <template v-if="showStatusLine">
        <p :class="['card-status', { 'card-status-blocked': isBlocked }]">{{ statusDisplayText }}</p>
      </template>
      <p class="timeline-meta">{{ dateLabel }}</p>
      <p v-if="warning" class="timeline-warning">{{ t("labels.warningPrefix") }} {{ warning }}</p>
      <p v-if="location" class="timeline-meta">{{ location }}</p>
    </div>
    <LatestTimelineEntry v-if="timelineEntry" :entry="timelineEntry" />
  </article>
</template>

<style scoped>
.entity-card { align-items: stretch; display: flex; gap: 0.72rem; overflow: hidden; position: relative; flex-wrap: wrap; }
.entity-card::before { background: var(--category-color, rgba(72, 144, 255, 0.55)); content: ""; position: absolute; inset: 0 auto 0 0; width: 0.28rem; }
.entity-card-content { display: grid; gap: 0.5rem; min-width: 0; }
.status-icon { vertical-align: -0.15em; margin-right: 0.2rem; }
.card-status-blocked { color: #e67e22; }
.card-status { margin: 0; }
@media (min-width: 760px) {
  .entity-card { display: grid; grid-template-columns: auto minmax(0, 1fr) minmax(13rem, 30%); column-gap: 1.2rem; }
}
</style>
