<script setup lang="ts">
import { computed } from "vue";
import { BriefcaseBusiness, CalendarClock, FolderTree, Settings, Shield, TriangleAlert } from "@lucide/vue";
import { useRoute } from "vue-router";
import type { LocaleCode } from "../i18n";
import type { ThemeMode } from "../theme";

defineProps<{
  locale: LocaleCode;
  theme: ThemeMode;
  showCoOwnerLinks: boolean;
  showAdminLink: boolean;
  showAdminCategoryLink: boolean;
}>();

defineEmits<{
  "update:locale": [value: LocaleCode];
  "update:theme": [value: ThemeMode];
  navigate: [];
}>();

const route = useRoute();
const eventsActive = computed(() => route.path.startsWith("/events"));
const incidentsActive = computed(() => route.path.startsWith("/incidents"));
const projectsActive = computed(() => route.path.startsWith("/projects"));
const adminUsersActive = computed(() => route.path.startsWith("/admin/users"));
const adminCategoriesActive = computed(() => route.path.startsWith("/admin/categories"));
const settingsActive = computed(() => route.path.startsWith("/settings"));
</script>

<template>
  <nav class="sidebar-nav" aria-label="Main navigation">
    <section class="primary-links">
      <a v-if="showCoOwnerLinks" href="#/events" :class="{ active: eventsActive }" @click="$emit('navigate')">
        <CalendarClock :size="16" :stroke-width="2" />
        <span>{{ $t("nav.events") }}</span>
      </a>
      <a v-if="showCoOwnerLinks" href="#/incidents" :class="{ active: incidentsActive }" @click="$emit('navigate')">
        <TriangleAlert :size="16" :stroke-width="2" />
        <span>{{ $t("nav.incidents") }}</span>
      </a>
      <a v-if="showCoOwnerLinks" href="#/projects" :class="{ active: projectsActive }" @click="$emit('navigate')">
        <BriefcaseBusiness :size="16" :stroke-width="2" />
        <span>{{ $t("nav.projects") }}</span>
      </a>
      <a v-if="showAdminLink" href="#/admin/users" :class="{ active: adminUsersActive }" @click="$emit('navigate')">
        <Shield :size="16" :stroke-width="2" />
        <span>{{ $t("nav.adminUsers") }}</span>
      </a>
      <a v-if="showAdminCategoryLink" href="#/admin/categories" :class="{ active: adminCategoriesActive }" @click="$emit('navigate')">
        <FolderTree :size="16" :stroke-width="2" />
        <span>{{ $t("nav.adminCategories") }}</span>
      </a>
      <a href="#/settings" :class="{ active: settingsActive }" @click="$emit('navigate')">
        <Settings :size="16" :stroke-width="2" />
        <span>{{ $t("nav.settings") }}</span>
      </a>
    </section>

    <section class="controls-panel">
      <label for="app-locale">{{ $t("controls.language") }}</label>
      <select
        id="app-locale"
        :value="locale"
        @change="$emit('update:locale', ($event.target as HTMLSelectElement).value as LocaleCode)"
      >
        <option value="fr">FR</option>
        <option value="en">EN</option>
      </select>

      <label for="app-theme">{{ $t("controls.theme") }}</label>
      <select
        id="app-theme"
        :value="theme"
        @change="$emit('update:theme', ($event.target as HTMLSelectElement).value as ThemeMode)"
      >
        <option value="system">{{ $t("options.system") }}</option>
        <option value="light">{{ $t("options.light") }}</option>
        <option value="dark">{{ $t("options.dark") }}</option>
      </select>
    </section>
  </nav>
</template>

<style scoped>
.sidebar-nav {
  display: grid;
  gap: 1rem;
  align-content: start;
  padding: 1.15rem 1rem;
}

.primary-links {
  display: grid;
  gap: 0.45rem;
}

.sidebar-nav a {
  display: inline-flex;
  align-items: center;
  gap: 0.55rem;
  color: var(--muted-fg);
  text-decoration: none;
  border: 1px solid transparent;
  border-radius: 0.7rem;
  padding: 0.55rem 0.65rem;
  font-size: 0.92rem;
  font-weight: 600;
}

.sidebar-nav a:hover {
  color: var(--page-fg);
  background: rgba(43, 124, 255, 0.08);
}

.sidebar-nav a.active {
  color: var(--page-fg);
  background: rgba(72, 144, 255, 0.18);
  border-color: rgba(72, 144, 255, 0.34);
  box-shadow: 0 0 0 1px rgba(72, 144, 255, 0.2) inset;
}

.controls-panel {
  display: grid;
  gap: 0.55rem;
  margin-top: 0.4rem;
  padding: 0.75rem;
  border: 1px solid var(--border-color);
  border-radius: 0.9rem;
  background: rgba(0, 0, 0, 0.03);
}

html[data-theme="dark"] .controls-panel,
html[data-theme="system"][data-resolved-theme="dark"] .controls-panel {
  background: rgba(255, 255, 255, 0.03);
}

.sidebar-nav label {
  font-size: 0.76rem;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--muted-fg);
}

.sidebar-nav select {
  border: 1px solid var(--control-border);
  border-radius: 0.5rem;
  padding: 0.4rem 0.5rem;
  background: var(--control-bg);
  color: var(--control-fg);
  min-height: 2.2rem;
}
</style>
