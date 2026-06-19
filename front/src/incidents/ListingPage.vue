<script setup lang="ts">
import { Activity, CheckCircle2, Hourglass } from "@lucide/vue";
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useRoute, useRouter } from "vue-router";
import { currentUserRoles, hasAnyRole } from "../auth/session";
import CategoryBadge from "../categories/CategoryBadge.vue";
import LatestTimelineEntry from "../common/components/LatestTimelineEntry.vue";
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
const detailQuery = computed(() => {
  const trimmed = query.value.trim();
  return trimmed ? { q: trimmed } : {};
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
        <article class="timeline-card incident-list-card" v-for="incident in grouped.current" :key="incident.id" :style="incident.raw.category ? { '--category-color': incident.raw.category.color } : undefined">
          <CategoryBadge v-if="incident.raw.category" :category-key="incident.raw.category.key" :icon="incident.raw.category.icon" :color="incident.raw.category.color" :label="incident.raw.category.label" variant="rail" />
          <div class="incident-card-main">
            <h3 class="timeline-card-title">
              <span class="timeline-meta entity-key">{{ incident.id }}</span>
              <RouterLink :to="{ path: `/incidents/${incident.id}`, query: detailQuery }">{{ incident.title }}</RouterLink>
            </h3>
            <template v-if="incident.statusText || incident.statusType !== 'ongoing'">
              <p class="card-status"><component :is="incident.statusType === 'ongoing' ? Activity : Hourglass" :size="16" /> {{ incident.statusText || t('labels.blocked') }}</p>
            </template>
            <p class="timeline-meta">{{ incident.dateLabel }}</p>
            <p class="timeline-meta" v-if="incident.location">{{ incident.location }}</p>
          </div>
          <LatestTimelineEntry v-if="incident.timeline[0]" :entry="incident.timeline[0]" />
        </article>
      </div>
    </section>

    <section class="timeline-section" v-if="hasSection('past')" data-status="past">
      <h2>{{ t("labels.past") }}</h2>
      <div class="timeline-list">
        <article class="timeline-card timeline-card-past incident-list-card" v-for="incident in grouped.past" :key="incident.id" :style="incident.raw.category ? { '--category-color': incident.raw.category.color } : undefined">
          <CategoryBadge v-if="incident.raw.category" :category-key="incident.raw.category.key" :icon="incident.raw.category.icon" :color="incident.raw.category.color" :label="incident.raw.category.label" variant="rail" />
          <div class="incident-card-main">
            <h3 class="timeline-card-title">
              <span class="timeline-meta entity-key">{{ incident.id }}</span>
              <RouterLink :to="{ path: `/incidents/${incident.id}`, query: detailQuery }">{{ incident.title }}</RouterLink>
            </h3>
            <template v-if="incident.statusText || incident.statusType !== 'ongoing'">
              <p class="card-status"><CheckCircle2 :size="16" /> {{ incident.statusText || t('labels.finished') }}</p>
            </template>
            <p class="timeline-meta">{{ incident.dateLabel }}</p>
            <p class="timeline-meta" v-if="incident.location">{{ incident.location }}</p>
          </div>
          <LatestTimelineEntry v-if="incident.timeline[0]" :entry="incident.timeline[0]" />
        </article>
      </div>
    </section>

    <p class="empty-state" v-if="isSearchActive && !hasAnyMatch && !loadFailed">{{ t("labels.noIncidentsMatch") }}</p>
    <p class="empty-state" v-if="loadFailed">{{ t("labels.incidentsLoadFailed") }}</p>
  </main>
</template>

<style scoped>
.primary-action { display: inline-flex; margin-top: 0.8rem; border: 1px solid rgba(72, 144, 255, 0.7); border-radius: 0.55rem; padding: 0.55rem 0.8rem; background: rgba(72, 144, 255, 0.22); color: var(--control-fg); text-decoration: none; font-weight: 700; }
.incident-list-card { align-items: stretch; display: flex; flex-wrap: wrap; gap: 0.72rem; overflow: hidden; position: relative; }
.incident-list-card::before { background: var(--category-color, rgba(72, 144, 255, 0.55)); content: ""; position: absolute; inset: 0 auto 0 0; width: 0.28rem; }
.incident-card-main { display: grid; flex: 1; gap: 0.5rem; min-width: 0; }

@media (min-width: 760px) {
  .incident-list-card { display: grid; grid-template-columns: auto minmax(0, 1fr) minmax(13rem, 30%); column-gap: 1.2rem; }
}
</style>
