<script setup lang="ts">
import { RouterLink, RouterView } from "vue-router";

const nav = [
  { to: "/devices", label: "設備清單", icon: "M3 5h18M3 12h18M3 19h18" },
  { to: "/groups", label: "群組／標籤", icon: "M4 6h16M4 12h10M4 18h7" },
  {
    to: "/schedules",
    label: "排程監控",
    icon: "M12 6v6l4 2M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20z",
  },
  { to: "/reports", label: "AI 報告", icon: "M9 12h6M9 16h6M9 8h6M5 3h14v18H5z" },
  { to: "/settings", label: "設定", icon: "M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z" },
];
</script>

<template>
  <div class="flex h-full">
    <!-- 側邊欄 -->
    <aside class="w-60 shrink-0 bg-bg-surface border-r border-bg-border flex flex-col">
      <div class="px-5 py-5 border-b border-bg-border">
        <h1 class="font-mono text-lg font-bold text-brand">noc-lens</h1>
        <p class="text-xs text-ink-muted mt-1">網路設備監控</p>
      </div>
      <nav class="flex-1 px-3 py-4 space-y-1" aria-label="主導覽">
        <RouterLink
          v-for="item in nav"
          :key="item.to"
          :to="item.to"
          class="flex items-center gap-3 rounded-md px-3 py-2 text-sm text-ink-secondary hover:bg-bg-raised hover:text-ink-primary transition-colors duration-150"
          active-class="bg-bg-raised text-ink-primary"
        >
          <svg
            class="w-4 h-4"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            aria-hidden="true"
          >
            <path :d="item.icon" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
          {{ item.label }}
        </RouterLink>
      </nav>
      <div class="px-5 py-3 text-xs text-ink-muted border-t border-bg-border">v0.1.0</div>
    </aside>

    <!-- 主內容 -->
    <main class="flex-1 overflow-auto">
      <RouterView />
    </main>
  </div>
</template>
