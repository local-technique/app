<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { getAccessToken } from "../../auth/session";
import { ApiReference } from "@scalar/api-reference";
import "@scalar/api-reference/style.css";

const { t } = useI18n();

const spec = ref<Record<string, unknown> | null>(null);
const error = ref(false);

onMounted(async () => {
  try {
    const token = getAccessToken();
    const res = await fetch(
      `${import.meta.env.VITE_API_BASE_URL ?? "http://localhost:8080"}/openapi.json`,
      { headers: token ? { Authorization: `Bearer ${token}` } : undefined },
    );
    if (!res.ok) throw new Error("failed to fetch spec");
    spec.value = await res.json();
  } catch {
    error.value = true;
  }
});
</script>

<template>
  <main class="page-wrap">
    <h1 class="page-title">{{ t("labels.apiDocumentation") }}</h1>
    <p v-if="error" class="error-message">{{ t("labels.specLoadFailed") }}</p>
    <div v-else-if="!spec" class="empty-state">...</div>
    <div v-else class="scalar-wrapper">
      <ApiReference
        :configuration="{
          spec: { content: spec },
          hideDownloadButton: true,
          hideTestRequestButton: true,
          hideModels: true,
          agent: { disabled: true },
          pathRouting: { basePath: '#/settings/api-doc' },
        }" />
    </div>
  </main>
</template>

<style>
/* Scalar theme overrides — must be global (not scoped) to reach shadow DOM */
.scalar-wrapper {
  margin-top: 1rem;
  border: 1px solid var(--border-color);
  border-radius: 0.9rem;
  overflow: hidden;
  --scalar-color-1: var(--page-fg);
  --scalar-color-2: var(--page-fg);
  --scalar-color-3: var(--muted-fg);
  --scalar-color-accent: #4f9fff;
  --scalar-background-1: var(--page-bg);
  --scalar-background-2: var(--panel-bg);
  --scalar-background-3: var(--page-bg);
  --scalar-border-color: var(--border-color);
  --scalar-color-disabled: var(--muted-fg);
  --scalar-sidebar-background-1: var(--panel-bg);
}

.error-message {
  color: #f35a67;
  font-weight: 700;
}

.empty-state {
  margin-top: 1.2rem;
  color: var(--muted-fg);
}
</style>
