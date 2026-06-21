<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { Activity, ArrowLeft, ArrowRight, CalendarClock, CheckCircle2, CircleCheck, Hourglass, MapPin, SquarePen, Trash2, UserPen } from "@lucide/vue";
import { useI18n } from "vue-i18n";
import { useRoute } from "vue-router";
import { currentUserRoles, hasAnyRole } from "../auth/session";
import CategoryIcon from "../categories/CategoryIcon.vue";
import AttachmentList from "../common/components/AttachmentList.vue";
import AttachmentPreview from "../common/components/AttachmentPreview.vue";
import type { AttachmentItem } from "../common/attachments";
import type { LocaleCode } from "../common/localeContent";
import { renderProjectMarkdown } from "../projects/utils";
import { apiIncidentsRepository } from "./repositories/apiIncidentsRepository";
import { toIncidentViewModel } from "./utils";

const { t, locale } = useI18n();

const route = useRoute();
const incidentId = ref(typeof route.params.id === "string" ? route.params.id : "");
const incident = ref<Awaited<ReturnType<typeof apiIncidentsRepository.byId>>>(null);
const loadFailed = ref(false);
const selectedAttachmentId = ref("");
const showDeleteModal = ref(false);

function activeLocale(): LocaleCode {
  return locale.value === "en" ? "en" : "fr";
}

async function loadIncident(): Promise<void> {
  try {
    incident.value = await apiIncidentsRepository.byId(incidentId.value, activeLocale());
    loadFailed.value = false;
  } catch {
    incident.value = null;
    loadFailed.value = true;
  }
  selectedAttachmentId.value = incident.value?.attachments[0]?.id ?? "";
}

onMounted(async () => {
  await loadIncident();
});

watch(
  () => route.params.id,
  async (nextId) => {
    incidentId.value = typeof nextId === "string" ? nextId : "";
    await loadIncident();
  },
);

watch(
  () => locale.value,
  async () => {
    await loadIncident();
  },
);

const model = computed(() => (incident.value ? toIncidentViewModel(incident.value, activeLocale()) : null));
const descriptionHtml = computed(() => (model.value ? renderProjectMarkdown(model.value.description) : ""));
const backQuery = computed(() => {
  const q = route.query.q;
  return typeof q === "string" && q.length > 0 ? { q } : {};
});
const selectedAttachment = computed<AttachmentItem | null>(() => {
  const attachments = model.value?.raw.attachments ?? [];
  if (attachments.length === 0) {
    return null;
  }

  return attachments.find((item) => item.id === selectedAttachmentId.value) ?? attachments[0] ?? null;
});
const canEdit = computed(() => currentUserRoles.loaded && hasAnyRole(["ADMIN", "CO_OWNERSHIP_BOARD", "CO_OWNERSHIP_BOARD_OPS"]));
const canDelete = computed(() => currentUserRoles.loaded && hasAnyRole(["ADMIN", "CO_OWNERSHIP_BOARD_OPS"]));
const auditLabel = computed(() => {
  if (!incident.value?.lastModifiedAt) return "";
  return t("labels.lastModified", {
    date: new Intl.DateTimeFormat(locale.value, { dateStyle: "medium", timeStyle: "short" }).format(new Date(incident.value.lastModifiedAt)),
    user: incident.value.lastModifiedBy?.email ?? t("labels.unknownUser"),
  });
});

function handleAttachmentSelect(item: AttachmentItem): void {
  selectedAttachmentId.value = item.id;
}

function deleteIncident(): void {
  showDeleteModal.value = true;
}
async function confirmDelete(): Promise<void> {
  showDeleteModal.value = false;
  try {
    await apiIncidentsRepository.delete(incidentId.value);
    window.location.hash = "#/incidents";
  } catch {
    loadFailed.value = true;
  }
}
function cancelDelete(): void {
  showDeleteModal.value = false;
}
</script>

<template>
  <main class="page-wrap" v-if="model">
    <p class="back-link back-link-top">
      <RouterLink class="back-link-ui" :to="{ path: '/incidents', query: backQuery }">
        <ArrowLeft :size="15" :stroke-width="2" />
        <span>{{ t("labels.backToIncidents") }}</span>
      </RouterLink>
    </p>
    <h1 class="page-title page-title-inline">
      <span v-if="model.raw.category" class="title-icon-wrap"><CategoryIcon :name="model.raw.category.icon" :size="24" :style="{ color: model.raw.category.color }" /></span>
      <span class="title-key">{{ model.id }}</span>
      <span class="title-text">{{ model.title }}</span>
    </h1>
    <div class="detail-actions-row">
      <p class="incident-status" :class="{ 'status-blocked': model.statusType === 'waiting' }"><component :is="model.statusType === 'ongoing' ? Activity : model.statusType === 'finished' ? CheckCircle2 : Hourglass" :size="16" /> {{ model.statusText ? t('labels.' + (model.statusType === 'waiting' ? 'blocked' : model.statusType)) + ' - ' + model.statusText : t('labels.' + (model.statusType === 'waiting' ? 'blocked' : model.statusType)) }}</p>
      <p class="detail-actions"><RouterLink v-if="canEdit" class="secondary-button" :to="`/incidents/${model.id}/edit`"><SquarePen :size="16" /> {{ t("labels.edit") }}</RouterLink><button v-if="canDelete" class="delete-button" type="button" @click="deleteIncident"><Trash2 :size="16" /> {{ t("labels.delete") }}</button></p>
    </div>
    <p class="timeline-meta date-line">
      <CalendarClock :size="16" />
      <template v-if="model.startDateFormatted && model.endDateFormatted">
        {{ model.startDateFormatted }} <ArrowRight :size="14" class="arrow-icon" /> {{ model.endDateFormatted }}
      </template>
      <template v-else-if="model.startDateFormatted">
        {{ t("labels.dateStart") }} {{ model.startDateFormatted }}
      </template>
      <template v-else>
        {{ t("labels.datesToBeConfirmed") }}
      </template>
    </p>

    <section class="timeline-card detail-block">
      <div class="rendered-description" v-html="descriptionHtml"></div>
    </section>
    <p class="timeline-meta detail-location" v-if="model.location"><MapPin :size="16" /> {{ model.location }}</p>
    <p class="timeline-meta audit-line" v-if="auditLabel"><UserPen :size="16" /> {{ auditLabel }}</p>

    <AttachmentList
      :items="model.raw.attachments"
      :selected-id="selectedAttachment?.id"
      @select="handleAttachmentSelect"
    />
    <AttachmentPreview v-if="selectedAttachment" :attachment="selectedAttachment" />

    <section class="timeline-section">
      <h2>{{ t("labels.incidentTimeline") }}</h2>
      <div class="incident-timeline-list">
        <article class="timeline-row" v-for="entry in model.timeline" :key="entry.id">
          <div class="timeline-date-slot">
            <span v-if="entry.isPending" class="pending-badge">{{ entry.atLabel }}</span>
            <span v-else class="timeline-date-label">
              <span>{{ entry.atDateLabel }}</span>
              <span class="timeline-time-label">{{ entry.atTimeLabel }}</span>
            </span>
          </div>
          <div class="timeline-axis" aria-hidden="true"><span class="timeline-dot" /></div>
          <div class="timeline-card timeline-entry-card">
            <h3 class="timeline-card-title timeline-entry-title">
              <CircleCheck v-if="!entry.isPending" class="timeline-entry-icon" :size="16" :stroke-width="2.4" aria-hidden="true" />
              <span>{{ entry.title }}</span>
            </h3>
            <p v-if="entry.details" class="timeline-entry-details">{{ entry.details }}</p>
          </div>
        </article>
      </div>
    </section>

    <p class="back-link">
      <RouterLink class="back-link-ui" :to="{ path: '/incidents', query: backQuery }">
        <ArrowLeft :size="15" :stroke-width="2" />
        <span>{{ t("labels.backToIncidents") }}</span>
      </RouterLink>
    </p>
  </main>
  <main class="page-wrap" v-else-if="loadFailed">
    <h1 class="page-title">{{ t("labels.incidentLoadFailed") }}</h1>
    <p class="back-link">
      <RouterLink class="back-link-ui" :to="{ path: '/incidents', query: backQuery }">
        <ArrowLeft :size="15" :stroke-width="2" />
        <span>{{ t("labels.backToIncidents") }}</span>
      </RouterLink>
    </p>
  </main>
  <main class="page-wrap" v-else>
    <h1 class="page-title">{{ t("labels.incidentNotFound") }}</h1>
    <p class="back-link">
      <RouterLink class="back-link-ui" :to="{ path: '/incidents', query: backQuery }">
        <ArrowLeft :size="15" :stroke-width="2" />
        <span>{{ t("labels.backToIncidents") }}</span>
      </RouterLink>
    </p>
  </main>

  <Teleport to="body">
    <div v-if="showDeleteModal" class="modal-overlay" @click="cancelDelete">
      <div class="modal-card" @click.stop>
        <h3 class="modal-title">{{ t("labels.delete") }}</h3>
        <p>{{ t("labels.deleteIncidentConfirm") }}</p>
        <div class="modal-actions">
          <button class="secondary-button" @click="cancelDelete">{{ t("labels.cancel") }}</button>
          <button class="delete-button" @click="confirmDelete"><Trash2 :size="16" /> {{ t("labels.delete") }}</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.detail-block {
  margin-top: 0.9rem;
}

.back-link {
  margin-top: 1rem;
}

.back-link-top {
  margin-top: 0;
  margin-bottom: 0.45rem;
}

.back-link-ui {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  color: var(--muted-fg);
  text-decoration: none;
  font-size: 0.92rem;
  font-weight: 600;
}

.back-link-ui:hover {
  color: var(--page-fg);
}

.incident-status { display: inline-flex; align-items: center; gap: 0.35rem; font-size: 0.95rem; font-weight: 700; white-space: nowrap; margin: 0; }
.incident-status.status-blocked { color: #e67e22; }
.rendered-description :deep(p) { margin: 0.7rem 0 0; }
.rendered-description :deep(ul) { margin: 0.7rem 0 0; padding-left: 1.3rem; }
.rendered-description :deep(code) { border-radius: 0.35rem; padding: 0.1rem 0.25rem; background: rgba(127, 127, 127, 0.18); }
.detail-actions { display: flex; gap: 0.5rem; margin: 0 0 0 auto; }
.delete-button { display: inline-flex; align-items: center; gap: 0.35rem; border: 1px solid rgba(220, 38, 38, 0.5); border-radius: 0.55rem; padding: 0.35rem 0.6rem; background: rgba(220, 38, 38, 0.85); color: #fff; cursor: pointer; font-size: inherit; font-weight: 600; text-decoration: none; }
.modal-overlay { position: fixed; inset: 0; z-index: 1000; background: rgba(0, 0, 0, 0.5); display: flex; align-items: center; justify-content: center; }
.modal-card { background: var(--panel-bg); border: 1px solid var(--border-color); border-radius: 0.75rem; padding: 1.5rem; max-width: 400px; width: 90%; display: grid; gap: 0.75rem; }
.modal-title { margin: 0; }
.modal-actions { display: flex; gap: 0.5rem; justify-content: flex-end; }

.incident-timeline-list {
  --timeline-accent: rgba(72, 144, 255, 0.78);
  display: grid;
  gap: 0;
  margin-top: 0.75rem;
}

.timeline-row {
  display: grid;
  grid-template-columns: minmax(5.7rem, 8.2rem) 1.25rem minmax(0, 1fr);
  column-gap: 0.6rem;
  position: relative;
}

.timeline-date-slot {
  color: var(--muted-fg);
  font-size: 0.78rem;
  font-weight: 800;
  line-height: 1.25;
  padding-top: 0.35rem;
  text-align: right;
  white-space: nowrap;
}

.timeline-date-label {
  display: inline-block;
}

.timeline-time-label::before {
  content: ", ";
}

.timeline-axis {
  display: flex;
  justify-content: center;
  position: relative;
}

.timeline-axis::before {
  background: var(--timeline-accent);
  bottom: -0.2rem;
  content: "";
  position: absolute;
  top: 0.55rem;
  width: 2px;
}

.timeline-row:last-child .timeline-axis::before {
  bottom: calc(100% - 0.6rem);
}

.timeline-dot {
  background: var(--page-bg);
  border: 3px solid var(--timeline-accent);
  border-radius: 999px;
  height: 0.75rem;
  margin-top: 0.28rem;
  width: 0.75rem;
  z-index: 1;
}

.timeline-entry-card {
  margin-bottom: calc(0.32rem + 10px);
  padding: 0.32rem 0.62rem;
}

.timeline-entry-title {
  align-items: center;
  display: flex;
  gap: 0.38rem;
  line-height: 1.15;
  margin: 0;
}

.timeline-entry-details {
  color: var(--muted-fg);
  font-size: 0.82rem;
  line-height: 1.2;
  margin: 0.12rem 0 0;
}

.timeline-entry-icon {
  color: var(--timeline-accent);
  flex: 0 0 auto;
}

.pending-badge {
  background: rgba(255, 139, 26, 0.2);
  border: 1px solid rgba(255, 139, 26, 0.62);
  border-radius: 999px;
  color: #ff8b1a;
  display: inline-flex;
  padding: 0.18rem 0.45rem;
}

@media (max-width: 560px) {
  .timeline-row {
    grid-template-columns: 4.7rem 1rem minmax(0, 1fr);
    column-gap: 0.45rem;
  }

  .timeline-date-slot {
    font-size: 0.7rem;
    white-space: normal;
  }

  .timeline-date-label {
    display: grid;
    gap: 0.05rem;
    justify-items: end;
  }

  .timeline-time-label::before {
    content: "";
  }
}
</style>
