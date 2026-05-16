<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import { exchangeCallbackCode } from "./session";

const route = useRoute();
const router = useRouter();
const message = ref("Completing sign-in...");
const showBackToLogin = ref(false);
let redirectTimeoutId: number | null = null;

async function goToLogin(): Promise<void> {
  await router.replace({ path: "/login" });
}

function scheduleFallbackRedirect(): void {
  if (redirectTimeoutId !== null) {
    window.clearTimeout(redirectTimeoutId);
  }
  redirectTimeoutId = window.setTimeout(() => {
    void goToLogin();
  }, 3000);
}

onMounted(async () => {
  const code = typeof route.query.code === "string" ? route.query.code : "";
  if (!code) {
    message.value = "Sign-in failed. Please try again.";
    showBackToLogin.value = true;
    scheduleFallbackRedirect();
    return;
  }

  const result = await exchangeCallbackCode(code);
  if (!result.ok) {
    message.value = "Sign-in failed. Please try again.";
    showBackToLogin.value = true;
    scheduleFallbackRedirect();
    return;
  }

  await router.replace(result.redirect);
});

onBeforeUnmount(() => {
  if (redirectTimeoutId !== null) {
    window.clearTimeout(redirectTimeoutId);
  }
});
</script>

<template>
  <main class="page-wrap">
    <p class="timeline-meta">{{ message }}</p>
    <p v-if="showBackToLogin">
      <RouterLink to="/login">Back to login</RouterLink>
    </p>
  </main>
</template>
