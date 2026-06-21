<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { Activity, ArrowLeft, ArrowRight, CalendarClock, CheckCircle2, Hourglass, UserPen } from "@lucide/vue";
import { useI18n } from "vue-i18n";
import { useRoute } from "vue-router";
import { currentUserRoles, hasAnyRole } from "../auth/session";
import CategoryIcon from "../categories/CategoryIcon.vue";
import AttachmentList from "../common/components/AttachmentList.vue";
import TimelineList from "../common/components/TimelineList.vue";
import type { LocaleCode } from "../common/localeContent";
import { apiProjectsRepository } from "./repositories/apiProjectsRepository";
import { renderProjectMarkdown, toProjectViewModel } from "./utils";

const { t, locale } = useI18n();
const route = useRoute();
const projectId = ref(typeof route.params.id === "string" ? route.params.id : "");
const project = ref<Awaited<ReturnType<typeof apiProjectsRepository.byId>>>(null);
const loadFailed = ref(false);

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
  return t("labels.lastModified", {
    date: new Intl.DateTimeFormat(locale.value, { dateStyle: "medium", timeStyle: "short" }).format(new Date(project.value.lastModifiedAt)),
    user: project.value.lastModifiedBy?.email ?? t("labels.unknownUser"),
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
  return Hourglass;
});
const editPath = computed(() => (model.value ? `/projects/${encodeURIComponent(model.value.id)}/edit` : "/projects"));

async function deleteProject(): Promise<void> {
  if (!window.confirm(t("labels.deleteProjectConfirm"))) return;
  try {
    await apiProjectsRepository.delete(projectId.value);
    window.location.hash = "#/projects";
  } catch {
    loadFailed.value = true;
  }
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
      <p class="detail-actions"><RouterLink v-if="canEdit" class="secondary-button" :to="editPath">{{ t("labels.edit") }}</RouterLink><button v-if="canDelete" class="secondary-button" type="button" @click="deleteProject">{{ t("labels.delete") }}</button></p>
      <span class="detail-spacer"></span>
      <p class="project-status" :class="{ 'status-blocked': model.statusType === 'waiting' }"><component :is="statusIcon" :size="16" /> {{ statusLabel || t('labels.' + model.statusType) }}</p>
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

    <section v-if="model.timeline.length" class="timeline-section">
      <h2>{{ t("labels.projectTimeline") }}</h2>
      <TimelineList :entries="model.timeline" />
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
</template>

<style scoped>
.detail-block { margin-top: 0.9rem; }
.back-link { margin-top: 1rem; }
.back-link-top { margin-top: 0; margin-bottom: 0.45rem; }
.back-link-ui { display: inline-flex; align-items: center; gap: 0.35rem; color: var(--muted-fg); text-decoration: none; font-size: 0.92rem; font-weight: 600; }
.back-link-ui:hover { color: var(--page-fg); }
.project-status { display: inline-flex; align-items: center; gap: 0.35rem; font-size: 1.2rem; white-space: nowrap; }
.project-status.status-blocked { color: #e67e22; }
.project-description :deep(p) { margin: 0.7rem 0 0; }
.project-description :deep(ul) { margin: 0.7rem 0 0; padding-left: 1.3rem; }
.project-description :deep(code) { border-radius: 0.35rem; padding: 0.1rem 0.25rem; background: rgba(127, 127, 127, 0.18); }
</style>
