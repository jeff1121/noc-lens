<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{ status: "normal" | "warning" | "critical" | "updating" | "unknown" }>();

const map = {
  normal: { label: "正常", cls: "bg-status-normal/15 text-status-normal" },
  warning: { label: "警告", cls: "bg-status-warning/15 text-status-warning" },
  critical: { label: "嚴重", cls: "bg-status-critical/15 text-status-critical" },
  updating: { label: "更新中", cls: "bg-status-updating/15 text-status-updating" },
  unknown: { label: "未知", cls: "bg-bg-raised text-ink-muted" },
} as const;

const view = computed(() => map[props.status] ?? map.unknown);
</script>

<template>
  <span
    class="inline-flex items-center gap-1.5 rounded-full px-2 py-0.5 text-xs font-medium"
    :class="view.cls"
  >
    <span class="w-1.5 h-1.5 rounded-full bg-current"></span>
    {{ view.label }}
  </span>
</template>
