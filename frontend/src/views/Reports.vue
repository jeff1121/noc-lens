<script setup lang="ts">
import { onMounted, ref } from "vue";
import { api, type Report } from "../api/tauri";

const reports = ref<Report[]>([]);
const selected = ref<Report | null>(null);
const generating = ref(false);
const error = ref<string | null>(null);

async function load() {
  try {
    reports.value = await api.reportList();
    if (!selected.value && reports.value.length) selected.value = reports.value[0];
  } catch (e: any) {
    error.value = e?.message ?? String(e);
  }
}

onMounted(load);

async function generate() {
  error.value = null;
  generating.value = true;
  try {
    const rpt = await api.reportGenerate({}, undefined);
    await load();
    selected.value = rpt;
  } catch (e: any) {
    error.value = e?.message ?? String(e);
  } finally {
    generating.value = false;
  }
}

function exportMd(r: Report) {
  const blob = new Blob([r.summary_md], { type: "text/markdown;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `${r.title}-${r.generated_at.slice(0, 10)}.md`;
  a.click();
  URL.revokeObjectURL(url);
}
</script>

<template>
  <div class="flex flex-col h-full">
    <header class="px-6 py-4 border-b border-bg-border flex items-center justify-between">
      <div>
        <h2 class="text-xl font-semibold">AI 報告</h2>
        <p class="text-sm text-ink-muted">由 AI 依狀態資料產生摘要與維護報告</p>
      </div>
      <button class="btn-primary" :disabled="generating" @click="generate">
        {{ generating ? "產生中…" : "產生報告" }}
      </button>
    </header>

    <div class="flex flex-1 min-h-0">
      <!-- 報告清單 -->
      <aside class="w-64 shrink-0 border-r border-bg-border overflow-auto">
        <button
          v-for="r in reports"
          :key="r.id"
          class="w-full text-left px-4 py-3 border-b border-bg-border/50 hover:bg-bg-raised cursor-pointer"
          :class="selected?.id === r.id ? 'bg-bg-raised' : ''"
          @click="selected = r"
        >
          <p class="text-sm font-medium truncate">{{ r.title }}</p>
          <p class="text-xs text-ink-muted">{{ r.generated_at.slice(0, 16).replace("T", " ") }}</p>
        </button>
        <p v-if="!reports.length" class="p-4 text-sm text-ink-muted">尚無報告</p>
      </aside>

      <!-- 報告內容 -->
      <section class="flex-1 overflow-auto p-6">
        <p v-if="error" class="text-sm text-status-critical mb-3">{{ error }}</p>
        <div v-if="selected" class="max-w-3xl">
          <div class="flex items-center justify-between mb-4">
            <h3 class="text-lg font-semibold">{{ selected.title }}</h3>
            <button class="btn-ghost text-sm" @click="exportMd(selected)">匯出 Markdown</button>
          </div>
          <div class="card p-5 whitespace-pre-wrap text-sm leading-relaxed font-sans">
            {{ selected.summary_md }}
          </div>
        </div>
        <p v-else class="text-ink-muted text-sm">
          點右上「產生報告」開始（需先於設定填入 AI 端點與金鑰）。
        </p>
      </section>
    </div>
  </div>
</template>
