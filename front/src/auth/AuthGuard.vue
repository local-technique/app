<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { sanitizeRedirectPath } from "./redirect";
import { ensureAuthenticated } from "./session";

const route = useRoute();
const router = useRouter();
const checking = ref(false);

const requiresAuth = computed(() => route.matched.some((entry) => entry.meta.requiresAuth === true));

watch(
  () => route.fullPath,
  async () => {
    if (!requiresAuth.value) {
      return;
    }

    checking.value = true;
    const ok = await ensureAuthenticated();
    checking.value = false;

    if (!ok) {
      const redirect = sanitizeRedirectPath(route.fullPath);
      await router.replace({
        path: "/login",
        query: { redirect },
      });
    }
  },
  { immediate: true },
);
</script>

<template>
  <div v-if="checking" class="auth-guard-loading">Authenticating...</div>
  <slot v-else />
</template>

<style scoped>
.auth-guard-loading {
  color: var(--muted-fg);
  font-size: 0.95rem;
}
</style>
