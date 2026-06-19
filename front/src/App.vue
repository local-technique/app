<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, provide, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useRoute, useRouter } from "vue-router";
import AuthGuard from "./auth/AuthGuard.vue";
import { currentUserRoles, hasAnyRole, hasRole, initAuth } from "./auth/session";

initAuth();
import MobileBottomNav from "./common/components/MobileBottomNav.vue";
import MobileMenu from "./common/components/MobileMenu.vue";
import SidebarNav from "./common/components/SidebarNav.vue";
import { getStoredLocale, setStoredLocale, type LocaleCode } from "./common/i18n";
import { applyTheme, getStoredTheme, prefersDarkMode, setStoredTheme, type ThemeMode } from "./common/theme";

const { locale } = useI18n();
const route = useRoute();
const router = useRouter();

const selectedLocale = ref<LocaleCode>(getStoredLocale());
const selectedTheme = ref<ThemeMode>(getStoredTheme());
const mobileMenuOpen = ref(false);

provide("selectedLocale", selectedLocale);
provide("selectedTheme", selectedTheme);
const routerReady = ref(router.currentRoute.value.matched.length > 0);
const showCoOwnerLinks = computed(() => currentUserRoles.loaded && hasAnyRole(["ADMIN", "CO_OWNER", "CO_OWNERSHIP_BOARD", "CO_OWNERSHIP_BOARD_OPS"]));
const showAdminLink = computed(() => currentUserRoles.loaded && hasRole("ADMIN"));
const showAdminCategoryLink = computed(() => currentUserRoles.loaded && hasAnyRole(["ADMIN", "CO_OWNERSHIP_BOARD_OPS"]));

let mediaQuery: MediaQueryList | null = null;
let mediaQueryListener: (() => void) | null = null;

function getPrefersDark(): boolean {
  return prefersDarkMode();
}

function syncTheme(): void {
  applyTheme(selectedTheme.value, { prefersDark: getPrefersDark() });
}

locale.value = selectedLocale.value;
syncTheme();

watch(selectedLocale, (value) => {
  locale.value = value;
  setStoredLocale(value);
});

watch(selectedTheme, (value) => {
  setStoredTheme(value);
  syncTheme();
});

onMounted(() => {
  router.isReady().finally(() => {
    routerReady.value = true;
  });

  if (typeof window === "undefined" || typeof window.matchMedia !== "function") {
    return;
  }

  mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
  mediaQueryListener = () => {
    if (selectedTheme.value === "system") {
      syncTheme();
    }
  };

  mediaQuery.addEventListener("change", mediaQueryListener);
});

onBeforeUnmount(() => {
  if (mediaQuery && mediaQueryListener) {
    mediaQuery.removeEventListener("change", mediaQueryListener);
  }
});
</script>

<template>
  <div v-if="routerReady" class="app-root">
    <div class="app-shell">
      <aside class="desktop-sidebar" v-if="route.path !== '/login'">
        <div class="desktop-brand">CoPro</div>
        <SidebarNav
          :show-co-owner-links="showCoOwnerLinks"
          :show-admin-link="showAdminLink"
          :show-admin-category-link="showAdminCategoryLink"
        />
      </aside>

      <main class="app-main">
        <AuthGuard>
          <RouterView />
        </AuthGuard>
      </main>
    </div>

    <MobileMenu
      v-if="route.path !== '/login'"
      :open="mobileMenuOpen"
      :show-co-owner-links="showCoOwnerLinks"
      :show-admin-link="showAdminLink"
      :show-admin-category-link="showAdminCategoryLink"
      @close="mobileMenuOpen = false"
      @navigate="mobileMenuOpen = false"
    />

    <MobileBottomNav
      v-if="route.path !== '/login'"
      :show-co-owner-links="showCoOwnerLinks"
      :show-admin-link="showAdminLink"
      :show-admin-category-link="showAdminCategoryLink"
      @open-more="mobileMenuOpen = true"
    />
  </div>
</template>

<style scoped>
.app-root {
  min-height: 100vh;
  background: var(--app-surface);
  color: var(--page-fg);
  font-family: Inter, "Segoe UI", system-ui, sans-serif;
}

.app-shell {
  display: grid;
  grid-template-columns: 276px minmax(0, 1fr);
  min-height: 100vh;
}

.desktop-sidebar {
  border-right: 1px solid var(--border-color);
  background: var(--panel-bg);
}

.desktop-brand {
  padding: 1.1rem 1rem 0.4rem;
  font-size: 1.05rem;
  font-weight: 700;
  letter-spacing: 0.01em;
}

.app-main {
  padding: 1.6rem 1.35rem 2.2rem;
}

:deep(.page-wrap) {
  width: min(960px, 100%);
}

:deep(.page-title) {
  margin: 0;
  font-size: clamp(1.45rem, 2vw, 2rem);
  line-height: 1.1;
  letter-spacing: -0.015em;
}

:deep(.search-bar) {
  margin-top: 1rem;
  margin-bottom: 1.15rem;
  display: grid;
  gap: 0.42rem;
  max-width: 420px;
}

:deep(.search-bar label) {
  font-size: 0.8rem;
  color: var(--muted-fg);
  text-transform: uppercase;
  letter-spacing: 0.07em;
}

:deep(.search-bar input) {
  border: 1px solid var(--control-border);
  border-radius: 0.72rem;
  min-height: 2.35rem;
  padding: 0 0.7rem;
  background: var(--control-bg);
  color: var(--control-fg);
}

:deep(.timeline-section) {
  margin-top: 1.05rem;
}

:deep(section[data-status="past"]) {
  opacity: 0.64;
}

:deep(section[data-status="past"] .timeline-card),
:deep(.timeline-card-past) {
  background: color-mix(in srgb, var(--panel-bg) 70%, transparent);
}

:deep(section[data-status="past"] .timeline-card-title a),
:deep(.timeline-card-past .timeline-card-title a) {
  color: color-mix(in srgb, var(--page-fg) 66%, var(--muted-fg));
}

:deep(section[data-status="past"] .timeline-card-title a:visited),
:deep(section[data-status="past"] .timeline-card-title a:hover),
:deep(section[data-status="past"] .timeline-card-title a:active),
:deep(.timeline-card-past .timeline-card-title a:visited),
:deep(.timeline-card-past .timeline-card-title a:hover),
:deep(.timeline-card-past .timeline-card-title a:active) {
  color: color-mix(in srgb, var(--page-fg) 66%, var(--muted-fg));
}

:deep(section[data-status="past"] .timeline-meta),
:deep(.timeline-card-past .timeline-meta) {
  color: color-mix(in srgb, var(--muted-fg) 92%, #7d8796);
}

:deep(.timeline-section > h2) {
  margin: 0 0 0.68rem;
  font-size: 1.2rem;
  letter-spacing: -0.01em;
}

:deep(.timeline-list) {
  display: grid;
  gap: 0.66rem;
}

:deep(.timeline-card) {
  display: grid;
  gap: 0.5rem;
  border: 1px solid var(--border-color);
  border-radius: 0.92rem;
  padding: 0.86rem 0.95rem;
  background: color-mix(in srgb, var(--panel-bg) 88%, transparent);
  box-shadow: 0 1px 0 rgba(0, 0, 0, 0.12);
}

:deep(.timeline-card-title) {
  margin: 0;
  font-size: 1rem;
  line-height: 1.3;
}

:deep(.timeline-card-title a) {
  color: var(--page-fg);
  text-decoration: none;
}

:deep(.timeline-card-title a:hover) {
  text-decoration: underline;
}

:deep(a) {
  color: #4f9fff;
}

:deep(a:hover) {
  color: #77b3ff;
}

:deep(.timeline-meta) {
  margin: 0;
  font-size: 0.9rem;
  color: var(--muted-fg);
}

:deep(.entity-key) {
  font-weight: 400;
  margin-right: 0.4rem;
}

:deep(.card-status) {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  color: var(--muted-fg);
  font-weight: 700;
}

:deep(.timeline-warning) {
  margin: 0;
  font-size: 0.89rem;
  font-weight: 700;
  color: #f35a67;
}

:deep(.empty-state) {
  margin-top: 1.2rem;
  padding: 0.9rem 1rem;
  border: 1px dashed var(--border-color);
  border-radius: 0.85rem;
  color: var(--muted-fg);
}

@media (max-width: 960px) {
  .app-shell {
    display: block;
    min-height: calc(100vh - 58px);
  }

  .desktop-sidebar {
    display: none;
  }

  .app-main {
    padding: 1.1rem 0.85rem 5rem;
  }

  :deep(.timeline-card) {
    padding: 0.8rem 0.82rem;
  }
}
</style>
