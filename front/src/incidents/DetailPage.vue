<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { ArrowLeft } from "lucide-vue-next";
import { useI18n } from "vue-i18n";
import { useRoute } from "vue-router";
import AttachmentList from "../common/components/AttachmentList.vue";
import AttachmentPreview from "../common/components/AttachmentPreview.vue";
import type { AttachmentItem } from "../common/attachments";
import type { LocaleCode } from "../common/localeContent";
import { apiIncidentsRepository } from "./repositories/apiIncidentsRepository";
import { toIncidentViewModel } from "./utils";

const { t, locale } = useI18n();

const route = useRoute();
const incidentId = ref(typeof route.params.id === "string" ? route.params.id : "");
const incident = ref<Awaited<ReturnType<typeof apiIncidentsRepository.byId>>>(null);
const loadFailed = ref(false);
const selectedAttachmentId = ref("");

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

function handleAttachmentSelect(item: AttachmentItem): void {
  selectedAttachmentId.value = item.id;
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
    <h1 class="page-title">{{ model.title }}</h1>
    <p class="timeline-meta">ID: {{ model.id }}</p>

    <section class="timeline-card detail-block">
      <p class="timeline-meta">{{ model.dateLabel }}</p>
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

    <section class="timeline-section">
      <h2>{{ t("labels.incidentTimeline") }}</h2>
      <div class="timeline-list">
        <article class="timeline-card" v-for="entry in model.timeline" :key="entry.id">
          <h3 class="timeline-card-title">{{ entry.title }}</h3>
          <p class="timeline-meta">{{ entry.atLabel }}</p>
          <p v-if="entry.details">{{ entry.details }}</p>
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
</style>
