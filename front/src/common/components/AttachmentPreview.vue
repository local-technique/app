<script setup lang="ts">
import { computed } from "vue";
import { Download, File, Trash2 } from "@lucide/vue";
import type { AttachmentItem } from "../attachments";
import { formatAttachmentSize } from "../attachments";

const props = defineProps<{
  attachment: AttachmentItem;
  canDelete?: boolean;
}>();

const emit = defineEmits<{
  (event: "delete", attachment: AttachmentItem): void;
}>();

const previewKind = computed(() => {
  if (props.attachment.mimeType === "application/pdf") {
    return "pdf";
  }
  if (props.attachment.mimeType.startsWith("image/")) {
    return "image";
  }
  return "unsupported";
});

const sizeLabel = computed(() => formatAttachmentSize(props.attachment.sizeBytes));
</script>

<template>
  <div class="preview-card">
    <a
      :href="attachment.url"
      target="_blank"
      rel="noopener"
      class="preview-link-wrap"
    >
      <img
        v-if="previewKind === 'image'"
        :src="attachment.url"
        :alt="attachment.fileName"
        loading="lazy"
        class="preview-media"
      />

      <div v-else-if="previewKind === 'pdf'" class="preview-media pdf-placeholder">
        <span class="pdf-label">PDF</span>
        <Download :size="32" class="pdf-icon" />
      </div>
      <div v-else class="preview-media file-placeholder">
        <File :size="48" class="file-icon" />
      </div>
    </a>

    <button
      v-if="canDelete"
      class="delete-overlay"
      @click.stop="emit('delete', attachment)"
      :title="$t('labels.delete')"
    >
      <Trash2 :size="14" />
    </button>

    <p class="preview-meta">
      <span class="file-name">{{ attachment.fileName }}</span>
      <span class="file-size">{{ sizeLabel }}</span>
    </p>
  </div>
</template>

<style scoped>
.preview-card {
  position: relative;
  width: 160px;
}

.preview-link-wrap {
  display: block;
  width: 160px;
  height: 160px;
  border-radius: 0.8rem;
  overflow: hidden;
  text-decoration: none;
}

.preview-media {
  width: 160px;
  height: 160px;
  object-fit: cover;
  border: 1px solid var(--border-color);
  border-radius: 0.8rem;
  background: color-mix(in srgb, var(--panel-bg) 88%, transparent);
}

.pdf-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.35rem;
  object-fit: none;
}

.pdf-label {
  font-size: 2.5rem;
  font-weight: 800;
  color: #dc2626;
  line-height: 1;
}

.pdf-icon {
  color: #dc2626;
}

.file-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  object-fit: none;
}

.file-icon {
  color: var(--muted-fg);
}

.delete-overlay {
  position: absolute;
  top: 0.35rem;
  right: 0.35rem;
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  border-radius: 50%;
  background: #fff;
  color: #dc2626;
  cursor: pointer;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.25);
  opacity: 0;
  transition: opacity 0.15s;
}

.preview-card:hover .delete-overlay {
  opacity: 1;
}

.preview-meta {
  margin: 0.35rem 0 0;
  display: flex;
  flex-direction: column;
  gap: 0.1rem;
  overflow: hidden;
}

.file-name {
  font-size: 0.82rem;
  font-weight: 600;
  color: var(--page-fg);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-size {
  font-size: 0.75rem;
  color: var(--muted-fg);
}
</style>
