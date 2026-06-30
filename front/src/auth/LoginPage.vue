<script setup lang="ts">
import { computed, onMounted } from "vue";
import { useRoute } from "vue-router";
import { sanitizeRedirectPath } from "./redirect";
import { initOAuthSession, providerStartUrl } from "./session";

const route = useRoute();

const redirectPath = computed(() => {
  const queryValue = route.query.redirect;
  if (typeof queryValue !== "string") {
    return "/events";
  }
  return sanitizeRedirectPath(queryValue);
});

const googleHref = computed(() => providerStartUrl("google", redirectPath.value));
const microsoftHref = computed(() => providerStartUrl("microsoft", redirectPath.value));
onMounted(() => {
  initOAuthSession();
});
</script>

<template>
  <main class="page-wrap login-wrap">
    <h1 class="page-title">Sign in</h1>
    <p class="login-subtitle">Please sign in to continue.</p>

    <div class="login-actions">
      <a class="login-button" :href="googleHref">Continue with Google</a>
      <a class="login-button" :href="microsoftHref">Continue with Microsoft</a>
    </div>
  </main>
</template>

<style scoped>
.login-wrap {
  max-width: 30rem;
}

.login-subtitle {
  margin-top: 0.5rem;
  color: var(--muted-fg);
}

.login-actions {
  display: grid;
  gap: 0.8rem;
  margin-top: 1.2rem;
}

.login-button {
  display: inline-flex;
  justify-content: center;
  align-items: center;
  min-height: 2.7rem;
  border: 1px solid var(--control-border);
  border-radius: 0.8rem;
  background: var(--control-bg);
  color: var(--control-fg);
  text-decoration: none;
  font-weight: 700;
}

.login-button:hover {
  border-color: #4f9fff;
}
</style>
