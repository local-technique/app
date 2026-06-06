<script setup lang="ts">
import { computed } from "vue";
import * as icons from "@lucide/vue";

const props = defineProps<{
  name: string;
  size?: number;
}>();

function toPascalCase(value: string): string {
  return value
    .split(/[-_\s]+/)
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join("");
}

const iconComponent = computed(() => {
  const key = toPascalCase(props.name || "tag");
  const resolved = (icons as Record<string, unknown>)[key];
  return resolved ?? icons.Tag;
});
</script>

<template>
  <component :is="iconComponent" :size="size ?? 18" :stroke-width="2" aria-hidden="true" />
</template>
