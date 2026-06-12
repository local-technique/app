<script setup lang="ts">
import { useI18n } from "vue-i18n";

const { t } = useI18n();
</script>

<template>
  <div class="settings-page">
    <h1>{{ t("labels.settingsPageTitle") }}</h1>
  </div>
</template>

<style scoped>
.settings-page {
  padding: 1.5rem;
}
</style>
