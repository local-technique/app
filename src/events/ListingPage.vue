<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useRoute, useRouter } from "vue-router";
import type { LocaleCode } from "../common/localeContent";
import { mockEventsRepository } from "./repositories/mockEventsRepository";
import { groupByStatus, toEventViewModel } from "./utils";

const { t, locale } = useI18n();
const route = useRoute();
const router = useRouter();
const query = ref(typeof route.query.q === "string" ? route.query.q : "");
const events = ref([] as Awaited<ReturnType<typeof mockEventsRepository.list>>);
let loadVersion = 0;

function activeLocale(): LocaleCode {
  return locale.value === "en" ? "en" : "fr";
}

async function loadEvents(value: string): Promise<void> {
  const requestVersion = ++loadVersion;
  const result = await mockEventsRepository.list(activeLocale(), value);
  if (requestVersion === loadVersion) {
    events.value = result;
  }
}

watch(
  [query, () => locale.value],
  async ([nextQuery]) => {
    await loadEvents(nextQuery);
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
  const filtered = events.value.map((event) => toEventViewModel(event, activeLocale()));
  return groupByStatus(filtered);
});

const hasAnyMatch = computed(() => {
  const groups = grouped.value;
  return groups.current.length > 0 || groups.toCome.length > 0 || groups.past.length > 0;
});

const isSearchActive = computed(() => query.value.trim().length > 0);
const detailQuery = computed(() => {
  const trimmed = query.value.trim();
  return trimmed ? { q: trimmed } : {};
});

function hasSection(name: "current" | "toCome" | "past"): boolean {
  return grouped.value[name].length > 0;
}
</script>

<template>
  <main class="page-wrap">
    <h1 class="page-title">{{ t("nav.events") }}</h1>

    <div class="search-bar">
      <input id="events-search" v-model="query" type="search" :placeholder="t('labels.searchEvents')" />
    </div>

    <section class="timeline-section" v-if="hasSection('current')">
      <h2>{{ t("labels.current") }}</h2>
      <div class="timeline-list">
        <article class="timeline-card" v-for="event in grouped.current" :key="event.id">
          <h3 class="timeline-card-title">
            <RouterLink :to="{ path: `/events/${event.id}`, query: detailQuery }">{{ event.title }}</RouterLink>
          </h3>
          <p class="timeline-warning" v-if="event.warning">{{ t("labels.warningPrefix") }} {{ event.warning }}</p>
          <p class="timeline-meta">{{ event.dateLabel }}</p>
          <p class="timeline-meta" v-if="event.location">{{ event.location }}</p>
        </article>
      </div>
    </section>

    <section class="timeline-section" v-if="hasSection('toCome')">
      <h2>{{ t("labels.toCome") }}</h2>
      <div class="timeline-list">
        <article class="timeline-card" v-for="event in grouped.toCome" :key="event.id">
          <h3 class="timeline-card-title">
            <RouterLink :to="{ path: `/events/${event.id}`, query: detailQuery }">{{ event.title }}</RouterLink>
          </h3>
          <p class="timeline-warning" v-if="event.warning">{{ t("labels.warningPrefix") }} {{ event.warning }}</p>
          <p class="timeline-meta">{{ event.dateLabel }}</p>
          <p class="timeline-meta" v-if="event.location">{{ event.location }}</p>
        </article>
      </div>
    </section>

    <section class="timeline-section" v-if="hasSection('past')" data-status="past">
      <h2>{{ t("labels.past") }}</h2>
      <div class="timeline-list">
        <article class="timeline-card timeline-card-past" v-for="event in grouped.past" :key="event.id">
          <h3 class="timeline-card-title">
            <RouterLink :to="{ path: `/events/${event.id}`, query: detailQuery }">{{ event.title }}</RouterLink>
          </h3>
          <p class="timeline-warning" v-if="event.warning">{{ t("labels.warningPrefix") }} {{ event.warning }}</p>
          <p class="timeline-meta">{{ event.dateLabel }}</p>
          <p class="timeline-meta" v-if="event.location">{{ event.location }}</p>
        </article>
      </div>
    </section>

    <p class="empty-state" v-if="isSearchActive && !hasAnyMatch">{{ t("labels.noEventsMatch") }}</p>
  </main>
</template>
