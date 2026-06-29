<script setup lang="ts">
import { onMounted, reactive, ref } from "vue";
import { api, type ExportReportFormat, type Report, type ReportScope } from "../api/tauri";
import { useDevicesStore } from "../stores/devices";
import { useGroupsStore } from "../stores/groups";

type ReportScopeMode = "all" | "devices" | "groups";

const devicesStore = useDevicesStore();
const groupsStore = useGroupsStore();
const reports = ref<Report[]>([]);
const selected = ref<Report | null>(null);
const generating = ref(false);
const exporting = ref(false);
const error = ref<string | null>(null);
const exportMessage = ref<string | null>(null);
const form = reactive({
  title: "",
  mode: "all" as ReportScopeMode,
  deviceIds: [] as string[],
  groupIds: [] as string[],
  from: "",
  to: "",
});

async function load() {
  try {
    reports.value = await api.reportList();
    if (!selected.value && reports.value.length) selected.value = reports.value[0];
  } catch (e: any) {
    error.value = e?.message ?? String(e);
  }
}

onMounted(async () => {
  await Promise.all([devicesStore.fetch(), groupsStore.fetch(), load()]);
});

function buildScope(): ReportScope | null {
  const from = normalizeDatetime(form.from, "起始時間");
  const to = normalizeDatetime(form.to, "結束時間");
  if (from === false || to === false) return null;
  if (from && to && from > to) {
    error.value = "起始時間不可晚於結束時間";
    return null;
  }
  if (form.mode === "devices" && !form.deviceIds.length) {
    error.value = "請至少選擇一台設備";
    return null;
  }
  if (form.mode === "groups" && !form.groupIds.length) {
    error.value = "請至少選擇一個群組";
    return null;
  }

  return {
    device_ids: form.mode === "devices" ? form.deviceIds : null,
    group_ids: form.mode === "groups" ? form.groupIds : null,
    from,
    to,
  };
}

function normalizeDatetime(value: string, label: string): string | null | false {
  if (!value) return null;
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    error.value = `${label}格式不正確`;
    return false;
  }
  return date.toISOString();
}

async function generate() {
  error.value = null;
  exportMessage.value = null;
  const scope = buildScope();
  if (!scope) return;
  generating.value = true;
  try {
    const title = form.title.trim() || undefined;
    const rpt = await api.reportGenerate(scope, title);
    await load();
    selected.value = rpt;
  } catch (e: any) {
    error.value = e?.message ?? String(e);
  } finally {
    generating.value = false;
  }
}

async function exportReport(r: Report, format: ExportReportFormat) {
  error.value = null;
  exportMessage.value = null;
  const outPath = prompt("請輸入匯出檔案完整路徑", defaultExportPath(r, format));
  if (!outPath) return;
  exporting.value = true;
  try {
    const result = await api.reportExport(r.id, outPath, format);
    exportMessage.value = `已匯出：${result.path}`;
  } catch (e: any) {
    error.value = e?.message ?? String(e);
  } finally {
    exporting.value = false;
  }
}

function defaultExportPath(r: Report, format: ExportReportFormat) {
  const safeTitle = r.title.replace(/[^\p{L}\p{N}_-]+/gu, "-").replace(/^-|-$/g, "");
  return `${safeTitle || "report"}-${r.generated_at.slice(0, 10)}.${format}`;
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
        <p v-if="exportMessage" class="text-sm text-status-normal mb-3">{{ exportMessage }}</p>

        <section class="card p-5 mb-5 max-w-3xl space-y-4">
          <div class="grid grid-cols-2 gap-3">
            <input v-model="form.title" class="input" placeholder="報告標題（選填）" />
            <select v-model="form.mode" class="input">
              <option value="all">全部設備</option>
              <option value="devices">指定設備</option>
              <option value="groups">指定群組</option>
            </select>
            <label class="text-sm">
              <span class="block text-ink-muted mb-1">起始時間</span>
              <input v-model="form.from" type="datetime-local" class="input w-full" />
            </label>
            <label class="text-sm">
              <span class="block text-ink-muted mb-1">結束時間</span>
              <input v-model="form.to" type="datetime-local" class="input w-full" />
            </label>
          </div>

          <div v-if="form.mode === 'devices'" class="grid grid-cols-2 gap-2 text-sm">
            <label
              v-for="device in devicesStore.devices"
              :key="device.id"
              class="flex items-center gap-2"
            >
              <input v-model="form.deviceIds" type="checkbox" :value="device.id" />
              <span class="font-mono">{{ device.ip_address }}</span>
            </label>
          </div>

          <div v-if="form.mode === 'groups'" class="grid grid-cols-2 gap-2 text-sm">
            <label
              v-for="group in groupsStore.groups"
              :key="group.id"
              class="flex items-center gap-2"
            >
              <input v-model="form.groupIds" type="checkbox" :value="group.id" />
              <span>{{ group.name }}</span>
            </label>
          </div>
        </section>

        <div v-if="selected" class="max-w-3xl">
          <div class="flex items-center justify-between mb-4">
            <h3 class="text-lg font-semibold">{{ selected.title }}</h3>
            <div class="flex gap-2">
              <button
                class="btn-ghost text-sm"
                :disabled="exporting"
                @click="exportReport(selected, 'md')"
              >
                匯出 Markdown
              </button>
              <button
                class="btn-ghost text-sm"
                :disabled="exporting"
                @click="exportReport(selected, 'pdf')"
              >
                匯出 PDF
              </button>
            </div>
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
