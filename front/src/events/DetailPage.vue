<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { ArrowLeft } from "@lucide/vue";
import { useI18n } from "vue-i18n";
import { useRoute } from "vue-router";
import { currentUserRoles, hasAnyRole, hasRole } from "../auth/session";
import CategoryBadge from "../categories/CategoryBadge.vue";
import AttachmentList from "../common/components/AttachmentList.vue";
import AttachmentPreview from "../common/components/AttachmentPreview.vue";
import type { AttachmentItem } from "../common/attachments";
import type { LocaleCode } from "../common/localeContent";
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
const canEdit = computed(() => currentUserRoles.loaded && hasAnyRole(["ADMIN", "CO_OWNERSHIP_BOARD"]));
const canDelete = computed(() => currentUserRoles.loaded && hasRole("ADMIN"));
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
    <h1 class="page-title">{{ model.title }}</h1>
    <p class="detail-actions"><RouterLink v-if="canEdit" class="secondary-button" :to="`/events/${model.id}/edit`">{{ t("labels.edit") }}</RouterLink><button v-if="canDelete" class="secondary-button" type="button" @click="deleteEvent">{{ t("labels.delete") }}</button></p>
    <p class="timeline-meta">ID: {{ model.id }}</p>
    <p class="timeline-meta category-meta" v-if="model.raw.category">
      <CategoryBadge :code="model.raw.category.code" :icon="model.raw.category.icon" :color="model.raw.category.color" :label="model.raw.category.label" />
      <span>- {{ model.raw.category.label }}</span>
    </p>
    <p class="timeline-meta" v-if="auditLabel">{{ auditLabel }}</p>

    <section class="timeline-card detail-block">
      <p class="timeline-meta">{{ model.dateLabel }}</p>
      <p class="timeline-warning" v-if="model.warning">{{ t("labels.warningPrefix") }} {{ model.warning }}</p>
      <p class="timeline-meta" v-if="model.location">{{ model.location }}</p>
      <p>{{ model.shortDescription }}</p>
      <p>{{ model.longDescription }}</p>
    </section>

    <AttachmentList
      :items="model.raw.attachments"
      :selected-id="selectedAttachment?.id"
      @select="handleAttachmentSelect"
    />
    <AttachmentPreview v-if="selectedAttachment" :attachment="selectedAttachment" />

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

.detail-actions { display: flex; gap: 0.6rem; flex-wrap: wrap; }
.secondary-button { border: 1px solid var(--control-border); border-radius: 0.55rem; padding: 0.45rem 0.7rem; background: var(--control-bg); color: var(--control-fg); cursor: pointer; text-decoration: none; }
.category-meta { align-items: center; display: flex; gap: 0.35rem; }
</style>
