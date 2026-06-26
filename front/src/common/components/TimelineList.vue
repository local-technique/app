<script setup lang="ts">
import { CircleCheck } from "@lucide/vue";

export type TimelineEntry = {
  id: string;
  atUtc: string | null;
  atLabel: string;
  atDateLabel: string;
  atTimeLabel: string;
  isPending: boolean;
  title: string;
  details: string;
  lastModifiedBy?: { initials: string; fullName: string } | null;
};

defineProps<{ entries: TimelineEntry[] }>();
</script>

<template>
  <div class="TimelineList">
    <article class="timeline-row" v-for="entry in entries" :key="entry.id">
      <div class="timeline-date-slot">
        <span v-if="entry.isPending" class="pending-badge">{{ entry.atLabel }}</span>
        <span v-else class="timeline-date-label">
          <span>{{ entry.atDateLabel }}</span>
          <span class="timeline-time-label">{{ entry.atTimeLabel }}</span>
        </span>
      </div>
      <div class="timeline-axis" aria-hidden="true"><span class="timeline-dot" /></div>
      <div class="timeline-card timeline-entry-card">
        <span v-if="entry.lastModifiedBy" class="tl-user-avatar" :title="entry.lastModifiedBy.fullName">{{ entry.lastModifiedBy.initials }}</span>
        <div class="tl-card-body">
          <h3 class="timeline-card-title timeline-entry-title">
            <CircleCheck v-if="!entry.isPending" class="timeline-entry-icon" :size="16" :stroke-width="2.4" aria-hidden="true" />
            <span>{{ entry.title }}</span>
          </h3>
          <p v-if="entry.details" class="timeline-entry-details">{{ entry.details }}</p>
        </div>
      </div>
    </article>
  </div>
</template>

<style scoped>
.TimelineList {
  --timeline-accent: rgba(72, 144, 255, 0.78);
  display: grid;
  gap: 0;
  margin-top: 0.75rem;
}

.TimelineList .timeline-row {
  display: grid;
  grid-template-columns: minmax(5.7rem, 8.2rem) 1.25rem minmax(0, 1fr);
  column-gap: 0.6rem;
  position: relative;
}

.TimelineList .timeline-date-slot {
  color: var(--muted-fg);
  font-size: 0.78rem;
  font-weight: 800;
  line-height: 1.25;
  padding-top: 0.35rem;
  text-align: right;
  white-space: nowrap;
}

.TimelineList .timeline-date-label { display: inline-block; }

.TimelineList .timeline-time-label::before { content: ", "; }

.TimelineList .timeline-axis { display: flex; justify-content: center; position: relative; }

.TimelineList .timeline-axis::before {
  background: var(--timeline-accent);
  bottom: -0.2rem;
  content: "";
  position: absolute;
  top: 0.55rem;
  width: 2px;
}

.TimelineList .timeline-row:last-child .timeline-axis::before { bottom: calc(100% - 0.6rem); }

.TimelineList .timeline-dot {
  background: var(--page-bg);
  border: 3px solid var(--timeline-accent);
  border-radius: 999px;
  height: 0.75rem;
  margin-top: 0.28rem;
  width: 0.75rem;
  z-index: 1;
}

.TimelineList .timeline-entry-card { display: flex; gap: 0.5rem; align-items: flex-start; margin-bottom: calc(0.32rem + 10px); padding: 0.32rem 0.62rem; }

.TimelineList .tl-card-body { flex: 1; min-width: 0; }

.TimelineList .timeline-entry-title { align-items: center; display: flex; gap: 0.38rem; line-height: 1.15; margin: 0; }

.TimelineList .timeline-entry-details { color: var(--muted-fg); font-size: 0.82rem; line-height: 1.2; margin: 0.12rem 0 0; }

.TimelineList .timeline-entry-icon { color: var(--timeline-accent); flex: 0 0 auto; }

.TimelineList .tl-user-avatar {
  width: 1.5rem;
  height: 1.5rem;
  border-radius: 50%;
  background: rgba(127, 127, 127, 0.2);
  color: var(--muted-fg);
  cursor: default;
  flex: 0 0 auto;
  font-size: 0.55rem;
  font-weight: 700;
  line-height: 1;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.TimelineList .pending-badge {
  background: rgba(255, 139, 26, 0.2);
  border: 1px solid rgba(255, 139, 26, 0.62);
  border-radius: 999px;
  color: #ff8b1a;
  display: inline-flex;
  padding: 0.18rem 0.45rem;
}

@media (max-width: 560px) {
  .TimelineList .timeline-row { grid-template-columns: 4.7rem 1rem minmax(0, 1fr); column-gap: 0.45rem; }
  .TimelineList .timeline-date-slot { font-size: 0.7rem; white-space: normal; }
  .TimelineList .timeline-date-label { display: grid; gap: 0.05rem; justify-items: end; }
  .TimelineList .timeline-time-label::before { content: ""; }
}
</style>
