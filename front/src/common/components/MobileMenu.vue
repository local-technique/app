<script setup lang="ts">
import SidebarNav from "./SidebarNav.vue";

defineProps<{
  open: boolean;
  showCoOwnerLinks: boolean;
  showAdminLink: boolean;
  showAdminCategoryLink: boolean;
}>();

defineEmits<{
  close: [];
  navigate: [];
}>();
</script>

<template>
  <div v-if="open" class="mobile-menu-root">
    <button class="mobile-overlay" type="button" :aria-label="$t('controls.closeMenu')" @click="$emit('close')" />
    <div class="mobile-drawer" role="dialog" aria-modal="true" aria-label="More options">
      <header>
        <strong>CoPro</strong>
        <button type="button" :aria-label="$t('controls.closeMenu')" @click="$emit('close')">x</button>
      </header>

      <nav aria-label="Mobile menu">
        <SidebarNav
          :show-co-owner-links="showCoOwnerLinks"
          :show-admin-link="showAdminLink"
          :show-admin-category-link="showAdminCategoryLink"
          @navigate="$emit('navigate')"
        />
      </nav>
    </div>
  </div>
</template>

<style scoped>
.mobile-menu-root {
  position: fixed;
  inset: 0;
  z-index: 100;
}

.mobile-overlay {
  position: absolute;
  inset: 0;
  border: 0;
  background: var(--overlay-bg);
}

.mobile-drawer {
  position: absolute;
  right: 0;
  bottom: 0;
  left: 0;
  width: auto;
  max-height: 72vh;
  border-top-left-radius: 1rem;
  border-top-right-radius: 1rem;
  background: var(--panel-bg);
  border-top: 1px solid var(--border-color);
  box-shadow: 0 -18px 40px rgba(0, 0, 0, 0.26);
}

.mobile-drawer header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.9rem 1rem;
  border-bottom: 1px solid var(--border-color);
}

.mobile-drawer nav {
  padding-bottom: 0.85rem;
}

.mobile-drawer header button {
  border: 1px solid var(--control-border);
  border-radius: 0.5rem;
  padding: 0.25rem 0.5rem;
  background: var(--control-bg);
  color: var(--control-fg);
}

@media (min-width: 961px) {
  .mobile-menu-root {
    display: none;
  }
}
</style>
