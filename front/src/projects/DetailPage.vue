<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { Activity, ArrowLeft, ArrowRight, CalendarClock, CheckCircle2, Hourglass, SquarePen, Trash2, UserPen } from "@lucide/vue";
import { useI18n } from "vue-i18n";
import { useRoute } from "vue-router";
import { currentUserRoles, hasAnyRole } from "../auth/session";
import CategoryIcon from "../categories/CategoryIcon.vue";
import AttachmentList from "../common/components/AttachmentList.vue";
import EditableTimelineList from "../common/components/EditableTimelineList.vue";
import type { LocaleCode } from "../common/localeContent";
import { apiProjectsRepository } from "./repositories/apiProjectsRepository";
import { renderProjectMarkdown, toProjectViewModel } from "./utils";

const { t, locale } = useI18n();
const route = useRoute();
const projectId = ref(typeof route.params.id === "string" ? route.params.id : "");
const project = ref<Awaited<ReturnType<typeof apiProjectsRepository.byId>>>(null);
const loadFailed = ref(false);
const showDeleteModal = ref(false);

function activeLocale(): LocaleCode {
  return locale.value === "en" ? "en" : "fr";
}

async function loadProject(): Promise<void> {
  try {
    project.value = await apiProjectsRepository.byId(projectId.value, activeLocale());
    loadFailed.value = false;
  } catch {
    project.value = null;
    loadFailed.value = true;
  }
}

onMounted(async () => loadProject());
watch(() => route.params.id, async (nextId) => {
  projectId.value = typeof nextId === "string" ? nextId : "";
  await loadProject();
});
watch(() => locale.value, async () => loadProject());

const model = computed(() => (project.value ? toProjectViewModel(project.value, activeLocale()) : null));
const backQuery = computed(() => {
  const q = route.query.q;
  return typeof q === "string" && q.length > 0 ? { q } : {};
});
const canEdit = computed(() => currentUserRoles.loaded && hasAnyRole(["ADMIN", "CO_OWNERSHIP_BOARD", "CO_OWNERSHIP_BOARD_OPS"]));
const canDelete = computed(() => currentUserRoles.loaded && hasAnyRole(["ADMIN", "CO_OWNERSHIP_BOARD_OPS"]));
const auditLabel = computed(() => {
  if (!project.value?.lastModifiedAt) return "";
  const user = project.value.lastModifiedBy;
  const userName = user?.firstName && user?.lastName ? `${user.firstName} ${user.lastName}` : (user?.email ?? t("labels.unknownUser"));
  return t("labels.lastModified", {
    date: new Intl.DateTimeFormat(locale.value, { dateStyle: "medium", timeStyle: "short" }).format(new Date(project.value.lastModifiedAt)),
    user: userName,
  });
});
const descriptionHtml = computed(() => (model.value ? renderProjectMarkdown(model.value.description) : ""));
const statusLabel = computed(() => {
  if (!model.value) return "";
  return model.value.statusText;
});
const statusIcon = computed(() => {
  if (!model.value) return Hourglass;
  if (model.value.statusType === "finished") return CheckCircle2;
  if (model.value.statusType === "ongoing") return Activity;
  if (model.value.statusType === "planned") return CalendarClock;
  return Hourglass;
});
const editPath = computed(() => (model.value ? `/projects/${encodeURIComponent(model.value.id)}/edit` : "/projects"));

async function handleTimelineAdd(payload: { atUtc: string | null; sortOrder: number; fields: Record<string, string> }): Promise<void> {
  try { await apiProjectsRepository.createTimelineEntry(projectId.value, activeLocale(), payload); } catch { /* ignore */ }
  await loadProject();
}
async function handleTimelineUpdate(entryId: string, payload: { atUtc: string | null; sortOrder: number; fields: Record<string, string> }): Promise<void> {
  try { await apiProjectsRepository.updateTimelineEntry(projectId.value, entryId, activeLocale(), payload); } catch { /* ignore */ }
  await loadProject();
}
async function handleTimelineDelete(entryId: string): Promise<void> {
  try { await apiProjectsRepository.deleteTimelineEntry(projectId.value, entryId); } catch { /* ignore */ }
  await loadProject();
}
function deleteProject(): void {
  showDeleteModal.value = true;
}
async function confirmDelete(): Promise<void> {
  showDeleteModal.value = false;
  try {
    await apiProjectsRepository.delete(projectId.value);
    window.location.hash = "#/projects";
  } catch {
    loadFailed.value = true;
  }
}
function cancelDelete(): void {
  showDeleteModal.value = false;
}
</script>

<template>
  <main v-if="model" class="page-wrap">
    <p class="back-link back-link-top"><RouterLink class="back-link-ui" :to="{ path: '/projects', query: backQuery }"><ArrowLeft :size="15" :stroke-width="2" /><span>{{ t("labels.backToProjects") }}</span></RouterLink></p>
    <h1 class="page-title page-title-inline">
      <span v-if="model.raw.category" class="title-icon-wrap"><CategoryIcon :name="model.raw.category.icon" :size="24" :style="{ color: model.raw.category.color }" /></span>
      <span class="title-key">{{ model.id }}</span>
      <span class="title-text">{{ model.title }}</span>
    </h1>
    <div class="detail-actions-row">
      <p class="project-status" :class="{ 'status-blocked': model.statusType === 'waiting' }"><component :is="statusIcon" :size="16" /> {{ statusLabel ? t('labels.' + (model.statusType === 'waiting' ? 'blocked' : model.statusType)) + ' - ' + statusLabel : t('labels.' + (model.statusType === 'waiting' ? 'blocked' : model.statusType)) }}</p>
      <p class="detail-actions"><RouterLink v-if="canEdit" class="secondary-button" :to="editPath"><SquarePen :size="16" /> <span class="btn-label">{{ t("labels.edit") }}</span></RouterLink><button v-if="canDelete" class="delete-button" type="button" @click="deleteProject"><Trash2 :size="16" /> <span class="btn-label">{{ t("labels.delete") }}</span></button></p>
    </div>
    <p class="timeline-meta date-line">
      <CalendarClock :size="16" />
      <template v-if="model.startDateFormatted && model.endDateFormatted">
        {{ model.startDateFormatted }} <ArrowRight :size="14" class="arrow-icon" /> {{ model.endDateFormatted }}
      </template>
      <template v-else-if="model.startDateFormatted">
        {{ t("labels.dateStart") }} {{ model.startDateFormatted }}
      </template>
      <template v-else-if="model.endDateFormatted">
        {{ t("labels.dateUntil", { date: model.endDateFormatted }) }}
      </template>
      <template v-else>
        {{ t("labels.datesToBeConfirmed") }}
      </template>
    </p>

    <section class="timeline-card detail-block">
      <div class="project-description" v-html="descriptionHtml"></div>
    </section>
    <p v-if="auditLabel" class="timeline-meta audit-line"><UserPen :size="16" /> {{ auditLabel }}</p>

    <AttachmentList :items="model.raw.attachments" />

    <section class="timeline-section">
      <h2>{{ t("labels.projectTimeline") }}</h2>
      <EditableTimelineList
        :entries="model.timeline"
        :can-edit="canEdit"
        @add="handleTimelineAdd"
        @update="handleTimelineUpdate"
        @delete="handleTimelineDelete"
      />
    </section>

    <p class="back-link"><RouterLink class="back-link-ui" :to="{ path: '/projects', query: backQuery }"><ArrowLeft :size="15" :stroke-width="2" /><span>{{ t("labels.backToProjects") }}</span></RouterLink></p>
  </main>
  <main v-else-if="loadFailed" class="page-wrap">
    <h1 class="page-title">{{ t("labels.projectsLoadFailed") }}</h1>
    <p class="back-link"><RouterLink class="back-link-ui" :to="{ path: '/projects', query: backQuery }">{{ t("labels.backToProjects") }}</RouterLink></p>
  </main>
  <main v-else class="page-wrap">
    <h1 class="page-title">{{ t("labels.projectNotFound") }}</h1>
    <p class="back-link"><RouterLink class="back-link-ui" :to="{ path: '/projects', query: backQuery }">{{ t("labels.backToProjects") }}</RouterLink></p>
  </main>

  <Teleport to="body">
    <div v-if="showDeleteModal" class="modal-overlay" @click="cancelDelete">
      <div class="modal-card" @click.stop>
        <h3 class="modal-title">{{ t("labels.delete") }}</h3>
        <p>{{ t("labels.deleteProjectConfirm") }}</p>
        <div class="modal-actions">
          <button class="secondary-button" @click="cancelDelete">{{ t("labels.cancel") }}</button>
          <button class="delete-button" @click="confirmDelete"><Trash2 :size="16" /> {{ t("labels.delete") }}</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.detail-block { margin-top: 0.9rem; }
.back-link { margin-top: 1rem; }
.back-link-top { margin-top: 0; margin-bottom: 0.45rem; }
.back-link-ui { display: inline-flex; align-items: center; gap: 0.35rem; color: var(--muted-fg); text-decoration: none; font-size: 0.92rem; font-weight: 600; }
.back-link-ui:hover { color: var(--page-fg); }
.project-status { font-size: 0.95rem; font-weight: 700; margin: 0; min-width: 0; }
.project-status :deep(svg) { vertical-align: -3px; width: 18px; height: 18px; stroke-width: 2.5; margin-right: 0.35rem; }
.project-status.status-blocked { color: #e67e22; }
.project-description :deep(p) { margin: 0.7rem 0 0; }
.project-description :deep(ul) { margin: 0.7rem 0 0; padding-left: 1.3rem; }
.project-description :deep(code) { border-radius: 0.35rem; padding: 0.1rem 0.25rem; background: rgba(127, 127, 127, 0.18); }
.project-description :deep(pre) {
  margin: 0.7rem 0 0;
  border-radius: 0.35rem;
  padding: 0.75rem 1rem;
  background: rgba(127, 127, 127, 0.1);
  overflow-x: auto;
}
.project-description :deep(pre code) {
  background: none;
  padding: 0;
}
.project-description :deep(blockquote) {
  margin: 0.7rem 0 0;
  padding: 0.25rem 0 0.25rem 1rem;
  border-left: 3px solid rgba(127, 127, 127, 0.3);
  color: rgba(127, 127, 127, 0.9);
}
.project-description :deep(hr) {
  margin: 0.7rem 0;
  border: none;
  border-top: 1px solid rgba(127, 127, 127, 0.25);
}
.project-description :deep(input[type="checkbox"]) {
  margin: 0 0.35rem 0 0;
  pointer-events: none;
}
.project-description :deep(ol) {
  margin: 0.7rem 0 0;
  padding-left: 1.3rem;
}
.project-description :deep(mark) {
  border-radius: 0.2rem;
  padding: 0.05rem 0.15rem;
  background: rgba(255, 230, 0, 0.35);
}
.project-description :deep(sub),
.project-description :deep(sup) {
  font-size: 0.75em;
}
.project-description :deep(del) {
  text-decoration: line-through;
}
.detail-actions-row { display: grid; grid-template-columns: 1fr auto; gap: 0.5rem; }
.detail-actions { display: flex; gap: 0.5rem; }
.delete-button { display: inline-flex; align-items: center; gap: 0.35rem; border: 1px solid rgba(220, 38, 38, 0.5); border-radius: 0.55rem; padding: 0.35rem 0.6rem; background: rgba(220, 38, 38, 0.85); color: #fff; cursor: pointer; font-size: inherit; font-weight: 600; text-decoration: none; }
.modal-overlay { position: fixed; inset: 0; z-index: 1000; background: rgba(0, 0, 0, 0.5); display: flex; align-items: center; justify-content: center; }
.modal-card { background: var(--panel-bg); border: 1px solid var(--border-color); border-radius: 0.75rem; padding: 1.5rem; max-width: 400px; width: 90%; display: grid; gap: 0.75rem; }
.modal-title { margin: 0; }
.modal-actions { display: flex; gap: 0.5rem; justify-content: flex-end; }
@media (max-width: 560px) { .btn-label { display: none; } }
</style>
