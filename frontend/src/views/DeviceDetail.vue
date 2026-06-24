<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import {
  api,
  BRANDS,
  type Device,
  type Group,
  type QueryResult,
  type StatusSnapshot,
} from "../api/tauri";
import StatusBadge from "../components/StatusBadge.vue";

const route = useRoute();
const router = useRouter();
const id = route.params.id as string;

const device = ref<Device | null>(null);
const groups = ref<Group[]>([]);
const error = ref<string | null>(null);

const querying = ref(false);
const result = ref<QueryResult | null>(null);
const history = ref<StatusSnapshot[]>([]);

const brandLabel = (v: string) => BRANDS.find((b) => b.value === v)?.label ?? v;

const cpuPct = computed<number>(() => {
  const v = result.value?.metrics?.cpu?.usage_percent;
  return typeof v === "number" ? v : 0;
});
const memPct = computed<number>(() => {
  const v = result.value?.metrics?.memory?.usage_percent;
  return typeof v === "number" ? v : 0;
});

const badgeStatus = computed(() => {
  if (!result.value) return "unknown";
  if (result.value.status === "failed") return "critical";
  if (result.value.status === "partial") return "warning";
  return "normal";
});

function gauge(label: string, color: string) {
  return {
    chart: { type: "radialBar", sparkline: { enabled: true } },
    plotOptions: {
      radialBar: {
        hollow: { size: "55%" },
        dataLabels: {
          name: { color: "#94A3B8", fontSize: "12px", offsetY: 20 },
          value: { color: "#E2E8F0", fontSize: "22px", offsetY: -10 },
        },
      },
    },
    labels: [label],
    colors: [color],
    stroke: { lineCap: "round" },
  };
}

const historyChart = computed(() => {
  const points = [...history.value].reverse();
  return {
    options: {
      chart: { type: "line", toolbar: { show: false }, foreColor: "#94A3B8" },
      xaxis: {
        categories: points.map((s) => s.collected_at.slice(5, 16).replace("T", " ")),
        labels: { rotate: -45, style: { fontSize: "10px" } },
      },
      yaxis: { max: 100, min: 0, labels: { formatter: (v: number) => `${Math.round(v)}%` } },
      stroke: { curve: "smooth", width: 2 },
      colors: ["#3B82F6"],
      grid: { borderColor: "#293548" },
      dataLabels: { enabled: false },
      tooltip: { theme: "dark" },
    },
    series: [
      {
        name: "CPU 使用率",
        data: points.map((s) => {
          const v = s.metrics?.cpu?.usage_percent;
          return typeof v === "number" ? v : 0;
        }),
      },
    ],
  };
});

async function loadHistory() {
  try {
    history.value = await api.snapshotList(id, 30);
  } catch (e: any) {
    error.value = e?.message ?? String(e);
  }
}

async function runQuery() {
  error.value = null;
  querying.value = true;
  try {
    const results = await api.queryDevices([id]);
    result.value = results[0] ?? null;
    await loadHistory();
  } catch (e: any) {
    error.value = e?.message ?? String(e);
  } finally {
    querying.value = false;
  }
}

onMounted(async () => {
  try {
    const list = await api.deviceList();
    device.value = list.find((d) => d.id === id) ?? null;
    if (device.value) groups.value = await api.groupsForDevice(id);
    await loadHistory();
  } catch (e: any) {
    error.value = e?.message ?? String(e);
  }
});
</script>

<template>
  <div class="flex flex-col h-full">
    <header class="px-6 py-4 border-b border-bg-border flex items-center justify-between">
      <div class="flex items-center gap-3">
        <button class="btn-ghost" @click="router.push('/devices')">← 返回</button>
        <h2 class="text-xl font-semibold font-mono">{{ device?.ip_address ?? "設備" }}</h2>
      </div>
      <button class="btn-primary" :disabled="querying" @click="runQuery">
        {{ querying ? "查詢中…" : "即時查詢" }}
      </button>
    </header>

    <div class="px-6 py-4 space-y-4 overflow-auto">
      <p v-if="error" class="text-sm text-status-critical">{{ error }}</p>

      <!-- 設備資訊 -->
      <div v-if="device" class="card p-5 grid grid-cols-2 gap-4 max-w-2xl">
        <div>
          <span class="text-ink-muted text-sm">帳號</span>
          <p>{{ device.username }}</p>
        </div>
        <div>
          <span class="text-ink-muted text-sm">品牌</span>
          <p>{{ brandLabel(device.brand) }}</p>
        </div>
        <div class="col-span-2">
          <span class="text-ink-muted text-sm">備註</span>
          <p>{{ device.note || "—" }}</p>
        </div>
        <div class="col-span-2">
          <span class="text-ink-muted text-sm">群組</span>
          <p class="flex flex-wrap gap-2 mt-1">
            <span
              v-for="g in groups"
              :key="g.id"
              class="rounded-full bg-bg-raised px-2 py-0.5 text-xs"
              >{{ g.name }}</span
            >
            <span v-if="!groups.length" class="text-ink-muted">—</span>
          </p>
        </div>
      </div>

      <!-- 即時查詢結果 -->
      <div v-if="querying" class="card p-5 max-w-3xl">
        <div class="h-32 rounded bg-bg-raised animate-pulse"></div>
      </div>

      <div v-else-if="result" class="space-y-4">
        <div class="flex items-center gap-3">
          <h3 class="font-semibold">最新狀態</h3>
          <StatusBadge :status="badgeStatus" />
          <span v-if="result.error_message" class="text-sm text-status-critical">{{
            result.error_message
          }}</span>
        </div>

        <div
          v-if="result.status !== 'failed'"
          class="grid grid-cols-2 lg:grid-cols-4 gap-4 max-w-4xl"
        >
          <div class="card p-3">
            <apexchart
              type="radialBar"
              height="160"
              :options="gauge('CPU', '#3B82F6')"
              :series="[cpuPct]"
            />
          </div>
          <div class="card p-3">
            <apexchart
              type="radialBar"
              height="160"
              :options="gauge('Memory', '#22C55E')"
              :series="[memPct]"
            />
          </div>
          <div class="card p-4 col-span-2 grid grid-cols-2 gap-3 text-sm">
            <div>
              <span class="text-ink-muted">介面</span>
              <p>
                {{ result.metrics?.interface?.up ?? "—" }} up /
                {{ result.metrics?.interface?.down ?? "—" }} down
              </p>
            </div>
            <div>
              <span class="text-ink-muted">Module</span>
              <p>{{ result.metrics?.module === "n/a" ? "不適用" : "—" }}</p>
            </div>
            <div>
              <span class="text-ink-muted">Loading</span>
              <p>{{ (result.metrics?.loading?.samples_percent || []).join(" / ") || "不適用" }}</p>
            </div>
            <div>
              <span class="text-ink-muted">Traffic</span>
              <p>{{ result.metrics?.traffic === "n/a" ? "不適用" : "—" }}</p>
            </div>
          </div>
        </div>
      </div>

      <!-- 歷史趨勢 -->
      <div v-if="history.length" class="card p-5 max-w-4xl">
        <h3 class="font-semibold mb-3">CPU 歷史趨勢（最近 {{ history.length }} 次）</h3>
        <apexchart
          type="line"
          height="240"
          :options="historyChart.options"
          :series="historyChart.series"
        />
      </div>
      <p v-else class="text-sm text-ink-muted">尚無歷史資料，點「即時查詢」開始收集。</p>
    </div>
  </div>
</template>
