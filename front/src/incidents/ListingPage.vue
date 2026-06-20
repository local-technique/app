<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useRoute, useRouter } from "vue-router";
import { currentUserRoles, hasAnyRole } from "../auth/session";
import EntityCard from "../common/components/EntityCard.vue";
import type { LocaleCode } from "../common/localeContent";
import { apiIncidentsRepository } from "./repositories/apiIncidentsRepository";
import { groupByStatus, toIncidentViewModel } from "./utils";

const { t, locale } = useI18n();
const route = useRoute();
const router = useRouter();
const query = ref(typeof route.query.q === "string" ? route.query.q : "");
const incidents = ref([] as Awaited<ReturnType<typeof apiIncidentsRepository.list>>);
const loadFailed = ref(false);
let loadVersion = 0;

function activeLocale(): LocaleCode {
  return locale.value === "en" ? "en" : "fr";
}

async function loadIncidents(value: string): Promise<void> {
  const requestVersion = ++loadVersion;
  try {
    const result = await apiIncidentsRepository.list(activeLocale(), value);
    if (requestVersion === loadVersion) {
      loadFailed.value = false;
      incidents.value = result;
    }
  } catch {
    if (requestVersion === loadVersion) {
      loadFailed.value = true;
    }
  }
}

watch(
  [query, () => locale.value],
  async ([nextQuery]) => {
    await loadIncidents(nextQuery);
  },
  { flush: "post", immediate: true },
);

watch(
  () => route.query.q,
  (next) => {
    const value = typeof next === "string" ? next : "";
    if (value !== query.value) {
      query.value = value;
    }
  },
);

watch(query, async (nextValue) => {
  const trimmed = nextValue.trim();
  const current = typeof route.query.q === "string" ? route.query.q : "";
  if (trimmed === current) {
    return;
  }

  const nextQuery = { ...route.query };
  if (trimmed) {
    nextQuery.q = trimmed;
  } else {
    delete nextQuery.q;
  }

  await router.replace({ path: route.path, query: nextQuery });
});

const grouped = computed(() => {
  const models = incidents.value.map((incident) => toIncidentViewModel(incident, activeLocale()));
  return groupByStatus(models);
});

const hasAnyMatch = computed(() => {
  const groups = grouped.value;
  return groups.current.length > 0 || groups.past.length > 0;
});

const isSearchActive = computed(() => query.value.trim().length > 0);
const canCreate = computed(() => currentUserRoles.loaded && hasAnyRole(["ADMIN", "CO_OWNERSHIP_BOARD", "CO_OWNERSHIP_BOARD_OPS"]));
const detailQuery = computed<Record<string, string> | undefined>(() => {
  const trimmed = query.value.trim();
  return trimmed ? { q: trimmed } : undefined;
});

function hasSection(name: "current" | "past"): boolean {
  return grouped.value[name].length > 0;
}
</script>

<template>
  <main class="page-wrap">
    <h1 class="page-title">{{ t("nav.incidents") }}</h1>
    <p v-if="canCreate"><RouterLink class="primary-action" to="/incidents/new">{{ t("labels.createIncident") }}</RouterLink></p>

    <div class="search-bar">
      <input id="incidents-search" v-model="query" type="search" :placeholder="t('labels.searchIncidents')" />
    </div>

    <section class="timeline-section" v-if="hasSection('current')">
      <h2>{{ t("labels.current") }}</h2>
      <div class="timeline-list">
        <EntityCard v-for="incident in grouped.current" :key="incident.id"
          :id="incident.id" :title="incident.title"
          :status-type="incident.statusType" :status-text="incident.statusText"
          :date-label="incident.dateLabel" :to="`/incidents/${incident.id}`"
          :query="detailQuery"
          :category-key="incident.raw.category?.key ?? ''" :category-icon="incident.raw.category?.icon ?? ''"
          :category-color="incident.raw.category?.color" :category-label="incident.raw.category?.label"
          :timeline-entry="incident.timeline[0]"
          :location="incident.location || undefined"
        />
      </div>
    </section>

    <section class="timeline-section" v-if="hasSection('past')" data-status="past">
      <h2>{{ t("labels.past") }}</h2>
      <div class="timeline-list">
        <EntityCard v-for="incident in grouped.past" :key="incident.id"
          :id="incident.id" :title="incident.title"
          :status-type="incident.statusType" :status-text="incident.statusText"
          :date-label="incident.dateLabel" :to="`/incidents/${incident.id}`"
          :query="detailQuery"
          :category-key="incident.raw.category?.key ?? ''" :category-icon="incident.raw.category?.icon ?? ''"
          :category-color="incident.raw.category?.color" :category-label="incident.raw.category?.label"
          :timeline-entry="incident.timeline[0]"
          :location="incident.location || undefined"
        />
      </div>
    </section>

    <p class="empty-state" v-if="isSearchActive && !hasAnyMatch && !loadFailed">{{ t("labels.noIncidentsMatch") }}</p>
    <p class="empty-state" v-if="loadFailed">{{ t("labels.incidentsLoadFailed") }}</p>
  </main>
</template>

<style scoped>
.primary-action { display: inline-flex; margin-top: 0.8rem; border: 1px solid rgba(72, 144, 255, 0.7); border-radius: 0.55rem; padding: 0.55rem 0.8rem; background: rgba(72, 144, 255, 0.22); color: var(--control-fg); text-decoration: none; font-weight: 700; }
</style>
