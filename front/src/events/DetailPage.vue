<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { Activity, ArrowLeft, ArrowRight, CalendarClock, CheckCircle2, Hourglass, MapPin, UserPen } from "@lucide/vue";
import { useI18n } from "vue-i18n";
import { useRoute } from "vue-router";
import { currentUserRoles, hasAnyRole } from "../auth/session";
import CategoryIcon from "../categories/CategoryIcon.vue";
import AttachmentList from "../common/components/AttachmentList.vue";
import AttachmentPreview from "../common/components/AttachmentPreview.vue";
import TimelineList from "../common/components/TimelineList.vue";
import type { AttachmentItem } from "../common/attachments";
import type { LocaleCode } from "../common/localeContent";
import { renderProjectMarkdown } from "../projects/utils";
import { apiEventsRepository } from "./repositories/apiEventsRepository";
import { toEventViewModel } from "./utils";

const { t, locale } = useI18n();

const route = useRoute();
const eventId = ref(typeof route.params.id === "string" ? route.params.id : "");
const event = ref<Awaited<ReturnType<typeof apiEventsRepository.byId>>>(null);
const loadFailed = ref(false);
const selectedAttachmentId = ref("");

function activeLocale(): LocaleCode {
  return locale.value === "en" ? "en" : "fr";
}

async function loadEvent(): Promise<void> {
  try {
    event.value = await apiEventsRepository.byId(eventId.value, activeLocale());
    loadFailed.value = false;
  } catch {
    event.value = null;
    loadFailed.value = true;
  }
  selectedAttachmentId.value = event.value?.attachments[0]?.id ?? "";
}

onMounted(async () => {
  await loadEvent();
});

watch(
  () => route.params.id,
  async (nextId) => {
    eventId.value = typeof nextId === "string" ? nextId : "";
    await loadEvent();
  },
);

watch(
  () => locale.value,
  async () => {
    await loadEvent();
  },
);

const model = computed(() => (event.value ? toEventViewModel(event.value, activeLocale()) : null));
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
  if (!event.value?.lastModifiedAt) return "";
  return t("labels.lastModified", {
    date: new Intl.DateTimeFormat(locale.value, { dateStyle: "medium", timeStyle: "short" }).format(new Date(event.value.lastModifiedAt)),
    user: event.value.lastModifiedBy?.email ?? t("labels.unknownUser"),
  });
});

function handleAttachmentSelect(item: AttachmentItem): void {
  selectedAttachmentId.value = item.id;
}

async function deleteEvent(): Promise<void> {
  if (!window.confirm(t("labels.deleteEventConfirm"))) return;
  try {
    await apiEventsRepository.delete(eventId.value);
    window.location.hash = "#/events";
  } catch {
    loadFailed.value = true;
  }
}
</script>

<template>
  <main class="page-wrap" v-if="model">
    <p class="back-link back-link-top">
      <RouterLink class="back-link-ui" :to="{ path: '/events', query: backQuery }">
        <ArrowLeft :size="15" :stroke-width="2" />
        <span>{{ t("labels.backToEvents") }}</span>
      </RouterLink>
    </p>
    <h1 class="page-title page-title-inline">
      <span v-if="model.raw.category" class="title-icon-wrap"><CategoryIcon :name="model.raw.category.icon" :size="24" :style="{ color: model.raw.category.color }" /></span>
      <span class="title-key">{{ model.id }}</span>
      <span class="title-text">{{ model.title }}</span>
    </h1>
    <div class="detail-actions-row">
      <p class="detail-actions"><RouterLink v-if="canEdit" class="secondary-button" :to="`/events/${model.id}/edit`">{{ t("labels.edit") }}</RouterLink><button v-if="canDelete" class="secondary-button" type="button" @click="deleteEvent">{{ t("labels.delete") }}</button></p>
      <span class="detail-spacer"></span>
      <p class="event-status" :class="{ 'status-blocked': model.statusType === 'waiting' }"><component :is="model.statusType === 'ongoing' ? Activity : model.statusType === 'finished' ? CheckCircle2 : Hourglass" :size="16" /> {{ model.statusText || t('labels.' + model.statusType) }}</p>
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

    <p class="timeline-warning detail-warning" v-if="model.warning"><span class="warning-icon">{{ t("labels.warningPrefix") }}</span>{{ model.warning }}</p>
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

    <section v-if="model.timeline.length" class="timeline-section">
      <h2>{{ t("labels.maintenanceTimeline") }}</h2>
      <TimelineList :entries="model.timeline" />
    </section>

    <p class="back-link">
      <RouterLink class="back-link-ui" :to="{ path: '/events', query: backQuery }">
        <ArrowLeft :size="15" :stroke-width="2" />
        <span>{{ t("labels.backToEvents") }}</span>
      </RouterLink>
    </p>
  </main>
  <main class="page-wrap" v-else-if="loadFailed">
    <h1 class="page-title">{{ t("labels.eventLoadFailed") }}</h1>
    <p class="back-link">
      <RouterLink class="back-link-ui" :to="{ path: '/events', query: backQuery }">
        <ArrowLeft :size="15" :stroke-width="2" />
        <span>{{ t("labels.backToEvents") }}</span>
      </RouterLink>
    </p>
  </main>
  <main class="page-wrap" v-else>
    <h1 class="page-title">{{ t("labels.eventNotFound") }}</h1>
    <p class="back-link">
      <RouterLink class="back-link-ui" :to="{ path: '/events', query: backQuery }">
        <ArrowLeft :size="15" :stroke-width="2" />
        <span>{{ t("labels.backToEvents") }}</span>
      </RouterLink>
    </p>
  </main>
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

.event-status { display: inline-flex; align-items: center; gap: 0.35rem; font-size: 1.2rem; white-space: nowrap; }
.event-status.status-blocked { color: #e67e22; }
.detail-warning { display: flex; align-items: center; gap: 0.4rem; margin-top: 0.5rem; font-size: 1.2rem; }
.warning-icon { display: inline-flex; align-items: center; justify-content: center; width: 16px; flex-shrink: 0; }
.rendered-description :deep(p) { margin: 0.7rem 0 0; }
.rendered-description :deep(ul) { margin: 0.7rem 0 0; padding-left: 1.3rem; }
.rendered-description :deep(code) { border-radius: 0.35rem; padding: 0.1rem 0.25rem; background: rgba(127, 127, 127, 0.18); }
</style>
