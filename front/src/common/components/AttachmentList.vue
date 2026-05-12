<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import type { AttachmentItem } from "../attachments";
import { formatAttachmentSize } from "../attachments";

const props = defineProps<{
  items: AttachmentItem[];
  selectedId?: string;
}>();

const emit = defineEmits<{
  (event: "select", attachment: AttachmentItem): void;
}>();

const rows = computed(() =>
  props.items.map((item) => ({
    ...item,
    sizeLabel: formatAttachmentSize(item.sizeBytes),
  })),
);

const { t } = useI18n();

function selectAttachment(item: AttachmentItem): void {
  emit("select", item);
}
</script>

<template>
  <section class="timeline-section">
    <h2>{{ t("labels.attachments") }}</h2>

    <p class="empty-state" v-if="rows.length === 0">{{ t("labels.noAttachments") }}</p>

    <ul class="attachment-list" v-else>
      <li v-for="item in rows" :key="item.id">
        <button
          type="button"
          :aria-pressed="selectedId === item.id ? 'true' : 'false'"
          @click="selectAttachment(item)"
        >
          {{ item.fileName }}
        </button>
        <span class="timeline-meta">{{ item.mimeType }} - {{ item.sizeLabel }}</span>
      </li>
    </ul>
  </section>
</template>

<style scoped>
.attachment-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 0.55rem;
}

.attachment-list li {
  display: grid;
  gap: 0.35rem;
  padding: 0.72rem 0.8rem;
  border: 1px solid var(--border-color);
  border-radius: 0.8rem;
  background: color-mix(in srgb, var(--panel-bg) 88%, transparent);
}

.attachment-list button {
  border: 0;
  padding: 0;
  text-align: left;
  background: transparent;
  color: var(--page-fg);
  cursor: pointer;
  font-size: 0.94rem;
  font-weight: 600;
}
</style>
