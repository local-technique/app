<script setup lang="ts">
import { computed } from "vue";
import { CalendarClock, Shield, TriangleAlert } from "@lucide/vue";
import { useI18n } from "vue-i18n";
import { useRoute } from "vue-router";

defineEmits<{
  openMore: [];
}>();

defineProps<{
  showCoOwnerLinks: boolean;
  showAdminLink: boolean;
}>();

const { t } = useI18n();
const route = useRoute();
const eventsActive = computed(() => route.path.startsWith("/events"));
const incidentsActive = computed(() => route.path.startsWith("/incidents"));
const adminActive = computed(() => route.path.startsWith("/admin"));
</script>

<template>
  <nav class="mobile-bottom-nav" aria-label="Mobile primary navigation">
    <a v-if="showCoOwnerLinks" class="nav-item" :class="{ active: eventsActive }" href="#/events" :aria-label="t('nav.events')">
      <CalendarClock :size="18" :stroke-width="2" />
    </a>
    <a v-if="showCoOwnerLinks" class="nav-item" :class="{ active: incidentsActive }" href="#/incidents" :aria-label="t('nav.incidents')">
      <TriangleAlert :size="18" :stroke-width="2" />
    </a>
    <a v-if="showAdminLink" class="nav-item" :class="{ active: adminActive }" href="#/admin/users" :aria-label="t('nav.adminUsers')">
      <Shield :size="18" :stroke-width="2" />
    </a>
    <span class="nav-item nav-item-blank" aria-hidden="true"></span>
    <span class="nav-item nav-item-blank" aria-hidden="true"></span>
    <button class="nav-item nav-item-more" type="button" :aria-label="t('labels.openMore')" @click="$emit('openMore')">
      ...
    </button>
  </nav>
</template>

<style scoped>
.mobile-bottom-nav {
  position: fixed;
  right: 0;
  bottom: 0;
  left: 0;
  z-index: 60;
  display: grid;
  grid-template-columns: repeat(5, minmax(0, 1fr));
  gap: 0;
  border-top: 1px solid var(--border-color);
  background: var(--panel-bg);
  backdrop-filter: blur(12px);
}

.nav-item {
  display: grid;
  place-items: center;
  min-height: 58px;
  color: var(--muted-fg);
  text-decoration: none;
  border: 0;
  background: transparent;
}

.nav-item-more {
  color: var(--page-fg);
  font-size: 1.15rem;
  cursor: pointer;
}

.nav-item.active {
  color: #7fb3ff;
  background: rgba(72, 144, 255, 0.16);
  box-shadow: 0 0 0 1px rgba(72, 144, 255, 0.3) inset;
}

.nav-item-blank {
  background: rgba(0, 0, 0, 0.08);
}

html[data-theme="dark"] .nav-item-blank,
html[data-theme="system"][data-resolved-theme="dark"] .nav-item-blank {
  background: rgba(255, 255, 255, 0.04);
}

@media (min-width: 961px) {
  .mobile-bottom-nav {
    display: none;
  }
}
</style>
