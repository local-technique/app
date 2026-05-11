<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import type { AttachmentItem } from "../attachments";

const props = defineProps<{
  attachment: AttachmentItem;
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

const previewTitle = computed(() => `Preview ${props.attachment.fileName}`);
const { t } = useI18n();
</script>

<template>
  <section class="timeline-section">
    <h3>{{ t("labels.preview") }}</h3>

    <img
      v-if="previewKind === 'image'"
      :src="attachment.url"
      :alt="previewTitle"
      loading="lazy"
      class="preview-media"
    />

    <object
      v-else-if="previewKind === 'pdf'"
      :data="attachment.url"
      :type="attachment.mimeType"
      :title="previewTitle"
      class="preview-media"
    >
      <p>
        {{ t("labels.pdfPreviewUnavailable") }}
        <a :href="attachment.url" target="_blank" rel="noopener">{{ t("labels.openFile") }}</a>
      </p>
    </object>

    <p v-else>{{ t("labels.previewUnavailable") }}</p>

    <p class="preview-link" v-if="previewKind !== 'pdf'">
      <a :href="attachment.url" target="_blank" rel="noopener">{{ t("labels.openFile") }}</a>
    </p>
  </section>
</template>

<style scoped>
.preview-media {
  width: 100%;
  max-width: 100%;
  height: auto;
  min-height: 260px;
  border: 1px solid var(--border-color);
  border-radius: 0.8rem;
  background: color-mix(in srgb, var(--panel-bg) 88%, transparent);
}

.preview-link {
  margin-top: 0.65rem;
}
</style>
