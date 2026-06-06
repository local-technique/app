<script setup lang="ts">
import { useI18n } from "vue-i18n";

const { t } = useI18n();
</script>

<template>
  <main class="page-wrap access-state">
    <h1 class="page-title">{{ t("labels.accessPendingTitle") }}</h1>
    <p class="empty-state">{{ t("labels.accessPendingMessage") }}</p>
  </main>
</template>

<style scoped>
.access-state {
  display: grid;
  gap: 0.6rem;
}
</style>
