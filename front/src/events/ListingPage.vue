<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useRoute, useRouter } from "vue-router";
import { currentUserRoles, hasAnyRole } from "../auth/session";
import CategoryBadge from "../categories/CategoryBadge.vue";
import type { LocaleCode } from "../common/localeContent";
import { apiEventsRepository } from "./repositories/apiEventsRepository";
import { groupByStatus, toEventViewModel } from "./utils";

const { t, locale } = useI18n();
const route = useRoute();
const router = useRouter();
const query = ref(typeof route.query.q === "string" ? route.query.q : "");
const events = ref([] as Awaited<ReturnType<typeof apiEventsRepository.list>>);
const loadFailed = ref(false);
let loadVersion = 0;

function activeLocale(): LocaleCode {
  return locale.value === "en" ? "en" : "fr";
}

async function loadEvents(value: string): Promise<void> {
  const requestVersion = ++loadVersion;
  try {
    const result = await apiEventsRepository.list(activeLocale(), value);
    if (requestVersion === loadVersion) {
      loadFailed.value = false;
      events.value = result;
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

  const nextQuery = { ...route.query };
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
const canCreate = computed(() => currentUserRoles.loaded && hasAnyRole(["ADMIN", "CO_OWNERSHIP_BOARD", "CO_OWNERSHIP_BOARD_OPS"]));
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
    <p v-if="canCreate"><RouterLink class="primary-action" to="/events/new">{{ t("labels.createEvent") }}</RouterLink></p>

    <div class="search-bar">
      <input id="events-search" v-model="query" type="search" :placeholder="t('labels.searchEvents')" />
    </div>

    <section class="timeline-section" v-if="hasSection('current')">
      <h2>{{ t("labels.current") }}</h2>
      <div class="timeline-list">
        <article class="timeline-card category-card" v-for="event in grouped.current" :key="event.id" :style="event.raw.category ? { '--category-color': event.raw.category.color } : undefined">
          <CategoryBadge v-if="event.raw.category" :category-key="event.raw.category.key" :icon="event.raw.category.icon" :color="event.raw.category.color" :label="event.raw.category.label" variant="rail" />
          <div class="category-card-content">
            <h3 class="timeline-card-title">
              <RouterLink :to="{ path: `/events/${event.id}`, query: detailQuery }">{{ event.title }}</RouterLink>
            </h3>
            <p class="timeline-warning" v-if="event.warning">{{ t("labels.warningPrefix") }} {{ event.warning }}</p>
            <p class="timeline-meta">{{ event.dateLabel }}</p>
            <p class="timeline-meta" v-if="event.location">{{ event.location }}</p>
          </div>
        </article>
      </div>
    </section>

    <section class="timeline-section" v-if="hasSection('toCome')">
      <h2>{{ t("labels.toCome") }}</h2>
      <div class="timeline-list">
        <article class="timeline-card category-card" v-for="event in grouped.toCome" :key="event.id" :style="event.raw.category ? { '--category-color': event.raw.category.color } : undefined">
          <CategoryBadge v-if="event.raw.category" :category-key="event.raw.category.key" :icon="event.raw.category.icon" :color="event.raw.category.color" :label="event.raw.category.label" variant="rail" />
          <div class="category-card-content">
            <h3 class="timeline-card-title">
              <RouterLink :to="{ path: `/events/${event.id}`, query: detailQuery }">{{ event.title }}</RouterLink>
            </h3>
            <p class="timeline-warning" v-if="event.warning">{{ t("labels.warningPrefix") }} {{ event.warning }}</p>
            <p class="timeline-meta">{{ event.dateLabel }}</p>
            <p class="timeline-meta" v-if="event.location">{{ event.location }}</p>
          </div>
        </article>
      </div>
    </section>

    <section class="timeline-section" v-if="hasSection('past')" data-status="past">
      <h2>{{ t("labels.past") }}</h2>
      <div class="timeline-list">
        <article class="timeline-card timeline-card-past category-card" v-for="event in grouped.past" :key="event.id" :style="event.raw.category ? { '--category-color': event.raw.category.color } : undefined">
          <CategoryBadge v-if="event.raw.category" :category-key="event.raw.category.key" :icon="event.raw.category.icon" :color="event.raw.category.color" :label="event.raw.category.label" variant="rail" />
          <div class="category-card-content">
            <h3 class="timeline-card-title">
              <RouterLink :to="{ path: `/events/${event.id}`, query: detailQuery }">{{ event.title }}</RouterLink>
            </h3>
            <p class="timeline-warning" v-if="event.warning">{{ t("labels.warningPrefix") }} {{ event.warning }}</p>
            <p class="timeline-meta">{{ event.dateLabel }}</p>
            <p class="timeline-meta" v-if="event.location">{{ event.location }}</p>
          </div>
        </article>
      </div>
    </section>

    <p class="empty-state" v-if="isSearchActive && !hasAnyMatch && !loadFailed">{{ t("labels.noEventsMatch") }}</p>
    <p class="empty-state" v-if="loadFailed">{{ t("labels.eventsLoadFailed") }}</p>
  </main>
</template>

<style scoped>
.primary-action { display: inline-flex; margin-top: 0.8rem; border: 1px solid rgba(72, 144, 255, 0.7); border-radius: 0.55rem; padding: 0.55rem 0.8rem; background: rgba(72, 144, 255, 0.22); color: var(--control-fg); text-decoration: none; font-weight: 700; }
.category-card { align-items: stretch; display: flex; gap: 0.72rem; overflow: hidden; position: relative; }
.category-card::before { background: var(--category-color, rgba(72, 144, 255, 0.55)); content: ""; position: absolute; inset: 0 auto 0 0; width: 0.28rem; }
.category-card-content { min-width: 0; }
</style>
