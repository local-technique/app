<script setup lang="ts">
export type LatestEntry = {
  id: string;
  atLabel: string;
  isPending: boolean;
  title: string;
};

defineProps<{ entry: LatestEntry }>();
</script>

<template>
  <aside class="latest-timeline-entry latest-timeline-entry-stretched">
    <p class="timeline-meta" v-if="!entry.isPending">{{ entry.atLabel }}</p>
    <p class="latest-timeline-title">
      <span v-if="entry.isPending" class="pending-badge">{{ entry.atLabel }}</span>
      <span>{{ entry.title }}</span>
    </p>
  </aside>
</template>

<style scoped>
.latest-timeline-entry { border-top: 1px solid var(--border-color); display: grid; flex: 0 0 100%; gap: 0.3rem; padding-top: 0.55rem; }
.latest-timeline-title { color: #77b3ff; display: block; margin: 0; font-size: 0.88rem; font-weight: 650; line-height: 1.25; }
.latest-timeline-title .pending-badge { margin-right: 0.38rem; vertical-align: 0.08em; }
.pending-badge { background: rgba(255, 139, 26, 0.2); border: 1px solid rgba(255, 139, 26, 0.62); border-radius: 999px; color: #ff8b1a; display: inline-flex; font-size: 0.78rem; line-height: 1.15; padding: 0.18rem 0.45rem; }
</style>
