<script setup lang="ts">
import { useI18n } from "vue-i18n";

const { t } = useI18n();
</script>

<template>
  <main class="page-wrap">
    <h1 class="page-title">{{ t("labels.pageNotFound") }}</h1>
    <p class="timeline-meta">{{ t("labels.routeMissing") }}</p>
    <p><a href="#/events">{{ t("labels.backToEvents") }}</a></p>
  </main>
</template>
