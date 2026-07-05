<script setup lang="ts">
import { ref } from "vue";
import { Trash2 } from "@lucide/vue";
import { useI18n } from "vue-i18n";
import { authenticatedFetch } from "../../auth/authenticatedFetch";
import AttachmentPreview from "./AttachmentPreview.vue";
import AttachmentUpload from "./AttachmentUpload.vue";
import type { AttachmentItem } from "../attachments";

const props = withDefaults(defineProps<{
  itemType: "incidents" | "maintenances" | "projects";
  itemId: string;
  attachments: AttachmentItem[];
  canDeleteAny?: boolean;
}>(), {
  canDeleteAny: false,
});

const emit = defineEmits<{
  (event: "refresh"): void;
}>();

const { t } = useI18n();
const showDeleteModal = ref(false);
const pendingDelete = ref<AttachmentItem | null>(null);

function apiBaseUrl(): string {
  return import.meta.env.VITE_API_BASE_URL ?? "http://localhost:8080";
}

function getCurrentUserId(): string | undefined {
  const raw = sessionStorage.getItem("copro-current-user");
  if (!raw) return undefined;
  try {
    return JSON.parse(raw).id;
  } catch {
    return undefined;
  }
}

function confirmDelete(item: AttachmentItem): void {
  pendingDelete.value = item;
  showDeleteModal.value = true;
}

async function handleDelete(): Promise<void> {
  if (!pendingDelete.value) return;
  const item = pendingDelete.value;
  showDeleteModal.value = false;
  pendingDelete.value = null;
  try {
    const resp = await authenticatedFetch(
      `${apiBaseUrl()}/${props.itemType}/${encodeURIComponent(props.itemId)}/attachments/${encodeURIComponent(item.id)}`,
      { method: "DELETE" },
    );
    if (resp.ok) emit("refresh");
  } catch {
    /* ignore */
  }
}

function cancelDelete(): void {
  showDeleteModal.value = false;
  pendingDelete.value = null;
}
</script>

<template>
  <section class="timeline-section">
    <h2>{{ t("labels.attachments") }}</h2>
    <div class="attachment-grid">
      <AttachmentPreview
        v-for="item in attachments"
        :key="item.id"
        :attachment="item"
        :can-delete="canDeleteAny || item.uploadedBy?.id === getCurrentUserId()"
        @delete="confirmDelete"
      />
      <AttachmentUpload
        :upload-endpoint="`${apiBaseUrl()}/${itemType}/${itemId}/attachments`"
        @uploaded="emit('refresh')"
        @error="console.error"
      />
    </div>
  </section>

  <Teleport to="body">
    <div v-if="showDeleteModal" class="modal-overlay" @click="cancelDelete">
      <div class="modal-card" @click.stop>
        <h3 class="modal-title">{{ t("labels.delete") }}</h3>
        <p>{{ t("labels.deleteAttachmentConfirm") }}</p>
        <div class="modal-actions">
          <button class="secondary-button" @click="cancelDelete">{{ t("labels.cancel") }}</button>
          <button class="delete-button" @click="handleDelete"><Trash2 :size="16" /> {{ t("labels.delete") }}</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.attachment-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 1rem;
}

.modal-overlay {
  position: fixed;
  inset: 0;
  z-index: 1000;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: Inter, "Segoe UI", system-ui, sans-serif;
}

.modal-card {
  background: var(--panel-bg);
  border: 1px solid var(--border-color);
  border-radius: 0.75rem;
  padding: 1.5rem;
  max-width: 400px;
  width: 90%;
  display: grid;
  gap: 0.75rem;
}

.modal-title {
  margin: 0;
}

.modal-actions {
  display: flex;
  gap: 0.5rem;
  justify-content: flex-end;
}

.delete-button {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  border: 1px solid rgba(220, 38, 38, 0.5);
  border-radius: 0.55rem;
  padding: 0.35rem 0.6rem;
  background: rgba(220, 38, 38, 0.85);
  color: #fff;
  cursor: pointer;
  font-size: inherit;
  font-weight: 600;
  text-decoration: none;
}
</style>
