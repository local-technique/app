<script setup lang="ts">
import { computed } from "vue";
import { BriefcaseBusiness, CalendarClock, FolderTree, Settings, Shield, TriangleAlert } from "@lucide/vue";
import { useI18n } from "vue-i18n";
import { useRoute } from "vue-router";

defineEmits<{
  openMore: [];
}>();

const props = defineProps<{
  showCoOwnerLinks: boolean;
  showAdminLink: boolean;
  showAdminCategoryLink: boolean;
}>();

const { t } = useI18n();
const route = useRoute();

type NavItem = {
  key: string;
  isActive: boolean;
  label: string;
  href: string;
  icon: any;
};

const allItems = computed<NavItem[]>(() => {
  const items: NavItem[] = [];

  if (props.showCoOwnerLinks) {
    items.push(
      {
        key: "events",
        isActive: route.path.startsWith("/events"),
        label: t("nav.events"),
        href: "#/events",
        icon: CalendarClock,
      },
      {
        key: "incidents",
        isActive: route.path.startsWith("/incidents"),
        label: t("nav.incidents"),
        href: "#/incidents",
        icon: TriangleAlert,
      },
      {
        key: "projects",
        isActive: route.path.startsWith("/projects"),
        label: t("nav.projects"),
        href: "#/projects",
        icon: BriefcaseBusiness,
      },
    );
  }

  if (props.showAdminLink) {
    items.push({
      key: "users",
      isActive: route.path.startsWith("/admin/users"),
      label: t("nav.adminUsers"),
      href: "#/admin/users",
      icon: Shield,
    });
  }

  if (props.showAdminCategoryLink) {
    items.push({
      key: "categories",
      isActive: route.path.startsWith("/admin/categories"),
      label: t("nav.adminCategories"),
      href: "#/admin/categories",
      icon: FolderTree,
    });
  }

  items.push({
    key: "settings",
    isActive: route.path.startsWith("/settings"),
    label: t("nav.settings"),
    href: "#/settings",
    icon: Settings,
  });

  return items;
});

const visibleItems = computed(() => allItems.value.slice(0, 5));
const spacerCount = computed(() => 5 - visibleItems.value.length);
</script>

<template>
  <nav class="mobile-bottom-nav" aria-label="Mobile primary navigation">
    <a
      v-for="item in visibleItems"
      :key="item.key"
      class="nav-item"
      :class="{ active: item.isActive }"
      :href="item.href"
      :aria-label="item.label"
    >
      <component :is="item.icon" :size="18" :stroke-width="2" />
    </a>
    <span v-for="n in spacerCount" :key="'spacer-' + n" class="nav-item nav-item-spacer" aria-hidden="true"></span>
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
  grid-template-columns: repeat(6, minmax(0, 1fr));
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

.nav-item-spacer {
  background: rgba(0, 0, 0, 0.08);
}

html[data-theme="dark"] .nav-item-spacer,
html[data-theme="system"][data-resolved-theme="dark"] .nav-item-spacer {
  background: rgba(255, 255, 255, 0.04);
}

.nav-item.active {
  color: #7fb3ff;
  background: rgba(72, 144, 255, 0.16);
  box-shadow: 0 0 0 1px rgba(72, 144, 255, 0.3) inset;
}

@media (min-width: 961px) {
  .mobile-bottom-nav {
    display: none;
  }
}
</style>
