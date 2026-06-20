<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useRoute, useRouter } from "vue-router";
import { currentUserRoles, hasAnyRole } from "../auth/session";
import EntityCard from "../common/components/EntityCard.vue";
import type { LocaleCode } from "../common/localeContent";
import { apiProjectsRepository } from "./repositories/apiProjectsRepository";
import { groupByStatus, toProjectViewModel } from "./utils";

const { t, locale } = useI18n();
const route = useRoute();
const router = useRouter();
const query = ref(typeof route.query.q === "string" ? route.query.q : "");
const projects = ref([] as Awaited<ReturnType<typeof apiProjectsRepository.list>>);
const loadFailed = ref(false);
let loadVersion = 0;

function activeLocale(): LocaleCode {
  return locale.value === "en" ? "en" : "fr";
}

async function loadProjects(value: string): Promise<void> {
  const requestVersion = ++loadVersion;
  try {
    const result = await apiProjectsRepository.list(activeLocale(), value);
    if (requestVersion === loadVersion) {
      loadFailed.value = false;
      projects.value = result;
    }
  } catch {
    if (requestVersion === loadVersion) {
      loadFailed.value = true;
      projects.value = [];
    }
  }
}

watch([query, () => locale.value], async ([nextQuery]) => loadProjects(nextQuery), { flush: "post", immediate: true });

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

const grouped = computed(() => groupByStatus(projects.value.map((project) => toProjectViewModel(project, activeLocale()))));
const hasAnyMatch = computed(() => grouped.value.ongoing.length > 0 || grouped.value.toCome.length > 0 || grouped.value.finished.length > 0);
const isSearchActive = computed(() => query.value.trim().length > 0);
const canCreate = computed(() => currentUserRoles.loaded && hasAnyRole(["ADMIN", "CO_OWNERSHIP_BOARD", "CO_OWNERSHIP_BOARD_OPS"]));
const detailQuery = computed<Record<string, string> | undefined>(() => {
  const trimmed = query.value.trim();
  return trimmed ? { q: trimmed } : undefined;
});
</script>

<template>
  <main class="page-wrap">
    <h1 class="page-title">{{ t("nav.projects") }}</h1>
    <p v-if="canCreate"><RouterLink class="primary-action" to="/projects/new">{{ t("labels.createProject") }}</RouterLink></p>

    <div class="search-bar">
      <input id="projects-search" v-model="query" type="search" :placeholder="t('labels.searchProjects')" />
    </div>

    <section v-if="grouped.ongoing.length" class="timeline-section projects-section projects-section-ongoing">
      <h2>{{ t("labels.ongoingProjects") }}</h2>
      <div class="timeline-list">
        <EntityCard v-for="project in grouped.ongoing" :key="project.id"
          :id="project.id" :title="project.title"
          :status-type="project.statusType" :status-text="project.statusText"
          :date-label="project.dateLabel" :to="`/projects/${project.id}`"
          :query="detailQuery"
          :category-key="project.raw.category?.key ?? ''" :category-icon="project.raw.category?.icon ?? ''"
          :category-color="project.raw.category?.color" :category-label="project.raw.category?.label"
          :timeline-entry="project.timeline[0]"
        />
      </div>
    </section>

    <section v-if="grouped.toCome.length" class="timeline-section projects-section projects-section-tocome">
      <h2>{{ t("labels.projectsToCome") }}</h2>
      <div class="timeline-list">
        <EntityCard v-for="project in grouped.toCome" :key="project.id"
          :id="project.id" :title="project.title"
          :status-type="project.statusType" :status-text="project.statusText"
          :date-label="project.dateLabel" :to="`/projects/${project.id}`"
          :query="detailQuery"
          :category-key="project.raw.category?.key ?? ''" :category-icon="project.raw.category?.icon ?? ''"
          :category-color="project.raw.category?.color" :category-label="project.raw.category?.label"
          :timeline-entry="project.timeline[0]"
        />
      </div>
    </section>

    <section v-if="grouped.finished.length" class="timeline-section projects-section" data-status="past">
      <h2>{{ t("labels.finishedProjects") }}</h2>
      <div class="timeline-list">
        <EntityCard v-for="project in grouped.finished" :key="project.id"
          :id="project.id" :title="project.title"
          :status-type="project.statusType" :status-text="project.statusText"
          :date-label="project.dateLabel" :to="`/projects/${project.id}`"
          :query="detailQuery"
          :category-key="project.raw.category?.key ?? ''" :category-icon="project.raw.category?.icon ?? ''"
          :category-color="project.raw.category?.color" :category-label="project.raw.category?.label"
          :timeline-entry="project.timeline[0]"
        />
      </div>
    </section>

    <p v-if="isSearchActive && !hasAnyMatch && !loadFailed" class="empty-state">{{ t("labels.noProjectsMatch") }}</p>
    <p v-if="loadFailed" class="empty-state">{{ t("labels.projectsLoadFailed") }}</p>
  </main>
</template>

<style scoped>
.primary-action { display: inline-flex; margin-top: 0.8rem; border: 1px solid rgba(72, 144, 255, 0.7); border-radius: 0.55rem; padding: 0.55rem 0.8rem; background: rgba(72, 144, 255, 0.22); color: var(--control-fg); text-decoration: none; font-weight: 700; }
</style>
