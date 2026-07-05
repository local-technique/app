<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { Camera, CloudUpload } from "@lucide/vue";
import { authenticatedFetch } from "../../auth/authenticatedFetch";

const props = defineProps<{
  uploadEndpoint: string;
}>();

const emit = defineEmits<{
  (event: "uploaded"): void;
  (event: "error", message: string): void;
}>();

const { t } = useI18n();
const uploading = ref(false);
const dragOver = ref(false);

function isAllowed(file: File): boolean {
  const isImage = file.type.startsWith("image/");
  const isPdf = file.type === "application/pdf";
  if (!isImage && !isPdf) {
    emit("error", t("labels.attachmentTypeError"));
    return false;
  }
  if (file.size > 10 * 1024 * 1024) {
    emit("error", t("labels.attachmentSizeError"));
    return false;
  }
  return true;
}

async function uploadFile(file: File): Promise<void> {
  if (!isAllowed(file)) return;
  uploading.value = true;
  try {
    const createResp = await authenticatedFetch(props.uploadEndpoint, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        file_name: file.name,
        mime_type: file.type,
        size_bytes: file.size,
      }),
    });
    if (!createResp.ok) {
      const err = await createResp.json().catch(() => ({ error: "upload failed" }));
      emit("error", err.error ?? "upload failed");
      return;
    }
    const { presigned_put_url } = await createResp.json();

    const putResp = await fetch(presigned_put_url, {
      method: "PUT",
      body: file,
    });
    if (!putResp.ok) {
      emit("error", "file upload to storage failed");
      return;
    }

    emit("uploaded");
  } finally {
    uploading.value = false;
  }
}

function handleFilePick(event: Event): void {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (file) uploadFile(file);
  input.value = "";
}

function handleDrop(event: DragEvent): void {
  dragOver.value = false;
  const file = event.dataTransfer?.files?.[0];
  if (file) uploadFile(file);
}

function handleDragOver(event: DragEvent): void {
  event.preventDefault();
  dragOver.value = true;
}

function handleDragLeave(): void {
  dragOver.value = false;
}
</script>

<template>
  <div
    class="upload-card"
    :class="{ 'drag-over': dragOver }"
    @dragover="handleDragOver"
    @dragleave="handleDragLeave"
    @drop.prevent="handleDrop"
  >
    <label class="upload-trigger">
      <input type="file" accept="image/*,application/pdf" hidden @change="handleFilePick" />
      <template v-if="uploading">
        <span class="uploading-text">{{ t("labels.uploading") }}...</span>
      </template>
      <template v-else>
        <CloudUpload :size="36" class="upload-icon" />
        <span class="upload-label" v-if="!dragOver">{{ t("labels.uploadFile") }}</span>
        <span class="upload-label drop-active" v-else>{{ t("labels.dropToUpload") }}</span>
      </template>
    </label>
    <label class="camera-overlay" :title="t('labels.takePhoto')">
      <Camera :size="14" />
      <input type="file" accept="image/*" capture="environment" hidden @change="handleFilePick" />
    </label>
  </div>
</template>

<style scoped>
.upload-card {
  position: relative;
  width: 160px;
  height: 160px;
  border: 2px dashed var(--border-color);
  border-radius: 0.8rem;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: border-color 0.2s, background-color 0.2s;
  cursor: pointer;
}

.upload-card:hover {
  border-color: var(--muted-fg);
}

.upload-card.drag-over {
  border-color: var(--accent-color, #4a90d9);
  background: color-mix(in srgb, var(--accent-color, #4a90d9) 8%, transparent);
}

.upload-trigger {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.35rem;
  cursor: pointer;
  width: 100%;
  height: 100%;
  justify-content: center;
}

.upload-icon {
  color: var(--muted-fg);
}

.upload-card:hover .upload-icon {
  color: var(--page-fg);
}

.upload-label {
  font-size: 0.82rem;
  color: var(--muted-fg);
}

.drop-active {
  color: var(--accent-color, #4a90d9);
  font-weight: 600;
}

.uploading-text {
  font-size: 0.85rem;
  font-style: italic;
  color: var(--muted-fg);
}

.camera-overlay {
  position: absolute;
  bottom: 0.35rem;
  right: 0.35rem;
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  border-radius: 50%;
  background: #fff;
  color: var(--muted-fg);
  cursor: pointer;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.25);
  opacity: 0;
  transition: opacity 0.15s;
}

.upload-card:hover .camera-overlay,
.upload-card:focus-within .camera-overlay {
  opacity: 1;
}

.camera-overlay:hover {
  color: var(--page-fg);
}

@media (hover: none) {
  .camera-overlay {
    opacity: 1;
  }
}
</style>
