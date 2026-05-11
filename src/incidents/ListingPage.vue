<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useRoute, useRouter } from "vue-router";
import type { LocaleCode } from "../common/localeContent";
import { mockIncidentsRepository } from "./repositories/mockIncidentsRepository";
import { groupByStatus, toIncidentViewModel } from "./utils";

const { t, locale } = useI18n();
const route = useRoute();
const router = useRouter();
const query = ref(typeof route.query.q === "string" ? route.query.q : "");
const incidents = ref([] as Awaited<ReturnType<typeof mockIncidentsRepository.list>>);
let loadVersion = 0;

function activeLocale(): LocaleCode {
  return locale.value === "en" ? "en" : "fr";
}

async function loadIncidents(value: string): Promise<void> {
  const requestVersion = ++loadVersion;
  const result = await mockIncidentsRepository.list(activeLocale(), value);
  if (requestVersion === loadVersion) {
    incidents.value = result;
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

  const nextQuery = { ...route.query } as Record<string, unknown>;
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

    <div class="search-bar">
      <input id="incidents-search" v-model="query" type="search" :placeholder="t('labels.searchIncidents')" />
    </div>

    <section class="timeline-section" v-if="hasSection('current')">
      <h2>{{ t("labels.current") }}</h2>
      <div class="timeline-list">
        <article class="timeline-card" v-for="incident in grouped.current" :key="incident.id">
          <p class="timeline-meta">{{ incident.raw.categoryCode }}</p>
          <h3 class="timeline-card-title">
            <RouterLink :to="{ path: `/incidents/${incident.id}`, query: detailQuery }">{{ incident.title }}</RouterLink>
          </h3>
          <p class="timeline-meta">{{ incident.dateLabel }}</p>
          <p class="timeline-meta" v-if="incident.location">{{ incident.location }}</p>
        </article>
      </div>
    </section>

    <section class="timeline-section" v-if="hasSection('past')" data-status="past">
      <h2>{{ t("labels.past") }}</h2>
      <div class="timeline-list">
        <article class="timeline-card timeline-card-past" v-for="incident in grouped.past" :key="incident.id">
          <p class="timeline-meta">{{ incident.raw.categoryCode }}</p>
          <h3 class="timeline-card-title">
            <RouterLink :to="{ path: `/incidents/${incident.id}`, query: detailQuery }">{{ incident.title }}</RouterLink>
          </h3>
          <p class="timeline-meta">{{ incident.dateLabel }}</p>
          <p class="timeline-meta" v-if="incident.location">{{ incident.location }}</p>
        </article>
      </div>
    </section>

    <p class="empty-state" v-if="isSearchActive && !hasAnyMatch">{{ t("labels.noIncidentsMatch") }}</p>
  </main>
</template>
