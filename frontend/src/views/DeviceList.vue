<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import { useDevicesStore } from "../stores/devices";
import { useGroupsStore } from "../stores/groups";
import { api, BRANDS, type Device, type NewDevice, type QueryResult } from "../api/tauri";
import DeviceForm from "../components/DeviceForm.vue";
import ImportDialog from "../components/ImportDialog.vue";
import ModalDialog from "../components/ModalDialog.vue";

const router = useRouter();
const devicesStore = useDevicesStore();
const groupsStore = useGroupsStore();

const showForm = ref(false);
const showImport = ref(false);
const editing = ref<Device | null>(null);
const filterGroup = ref<string>("");
const selectedDeviceIds = ref<Set<string>>(new Set());
const queryResultsByDeviceId = ref<Record<string, QueryResult>>({});
const batchQuerying = ref(false);
const queryError = ref<string | null>(null);

const brandLabel = (v: string) => BRANDS.find((b) => b.value === v)?.label ?? v;
const selectedCount = computed(() => selectedDeviceIds.value.size);
const allVisibleSelected = computed(
  () =>
    devicesStore.devices.length > 0 &&
    devicesStore.devices.every((device) => selectedDeviceIds.value.has(device.id)),
);

onMounted(async () => {
  await Promise.all([groupsStore.fetch(), devicesStore.fetch()]);
});

const isEmpty = computed(() => !devicesStore.loading && devicesStore.devices.length === 0);

async function applyFilter() {
  await devicesStore.fetch(filterGroup.value || undefined);
  pruneSelection();
}

function openCreate() {
  editing.value = null;
  showForm.value = true;
}
function openEdit(d: Device) {
  editing.value = d;
  showForm.value = true;
}

async function onSubmit(payload: NewDevice) {
  if (editing.value) {
    const patch: any = { ...payload };
    if (!patch.password) delete patch.password; // 留空表示不變更
    await devicesStore.update(editing.value.id, patch);
  } else {
    await devicesStore.create(payload);
  }
  showForm.value = false;
}

async function onDelete(d: Device) {
  if (confirm(`確定刪除設備 ${d.ip_address}？`)) {
    await devicesStore.remove(d.id);
    removeSelection(d.id);
  }
}

async function onImportDone() {
  showImport.value = false;
  await devicesStore.fetch(filterGroup.value || undefined);
  pruneSelection();
}

watch(
  () => devicesStore.devices.map((device) => device.id).join("|"),
  () => pruneSelection(),
);

function isSelected(id: string) {
  return selectedDeviceIds.value.has(id);
}

function toggleSelect(id: string, checked: boolean) {
  const next = new Set(selectedDeviceIds.value);
  if (checked) next.add(id);
  else next.delete(id);
  selectedDeviceIds.value = next;
}

function toggleSelectFromEvent(id: string, event: Event) {
  const target = event.target as HTMLInputElement;
  toggleSelect(id, target.checked);
}

function toggleAllVisible(checked: boolean) {
  const next = new Set(selectedDeviceIds.value);
  for (const device of devicesStore.devices) {
    if (checked) next.add(device.id);
    else next.delete(device.id);
  }
  selectedDeviceIds.value = next;
}

function toggleAllVisibleFromEvent(event: Event) {
  const target = event.target as HTMLInputElement;
  toggleAllVisible(target.checked);
}

function clearSelection() {
  selectedDeviceIds.value = new Set();
}

function removeSelection(id: string) {
  const next = new Set(selectedDeviceIds.value);
  next.delete(id);
  selectedDeviceIds.value = next;
  const rest = { ...queryResultsByDeviceId.value };
  delete rest[id];
  queryResultsByDeviceId.value = rest;
}

function pruneSelection() {
  const visibleIds = new Set(devicesStore.devices.map((device) => device.id));
  selectedDeviceIds.value = new Set(
    [...selectedDeviceIds.value].filter((id) => visibleIds.has(id)),
  );
  queryResultsByDeviceId.value = Object.fromEntries(
    Object.entries(queryResultsByDeviceId.value).filter(([id]) => visibleIds.has(id)),
  );
}

async function querySelected() {
  if (!selectedCount.value) return;
  queryError.value = null;
  batchQuerying.value = true;
  try {
    const results = await api.queryDevices([...selectedDeviceIds.value]);
    queryResultsByDeviceId.value = {
      ...queryResultsByDeviceId.value,
      ...Object.fromEntries(results.map((result) => [result.device_id, result])),
    };
  } catch (e: any) {
    queryError.value = e?.message ?? String(e);
  } finally {
    batchQuerying.value = false;
  }
}

function resultLabel(result?: QueryResult) {
  if (!result) return "尚未查詢";
  if (result.status === "ok") return "正常";
  if (result.status === "partial") return "部分異常";
  return "失敗";
}

function resultClass(result?: QueryResult) {
  if (!result) return "text-ink-muted";
  if (result.status === "ok") return "text-status-normal";
  if (result.status === "partial") return "text-status-warning";
  return "text-status-critical";
}

function cpuSummary(result?: QueryResult) {
  const value = result?.metrics?.cpu?.usage_percent;
  return typeof value === "number" ? `CPU ${Math.round(value)}%` : "CPU —";
}
</script>

<template>
  <div class="flex flex-col h-full">
    <!-- 標題列 -->
    <header class="flex items-center justify-between px-6 py-4 border-b border-bg-border">
      <div>
        <h2 class="text-xl font-semibold">設備清單</h2>
        <p class="text-sm text-ink-muted">共 {{ devicesStore.devices.length }} 台設備</p>
      </div>
      <div class="flex items-center gap-2">
        <select v-model="filterGroup" class="input w-44" @change="applyFilter">
          <option value="">全部群組</option>
          <option v-for="g in groupsStore.groups" :key="g.id" :value="g.id">{{ g.name }}</option>
        </select>
        <button class="btn-ghost" @click="showImport = true">匯入 CSV</button>
        <button class="btn-primary" @click="openCreate">新增設備</button>
      </div>
    </header>

    <!-- 內容 -->
    <div class="flex-1 min-h-0 px-6 py-4">
      <p v-if="devicesStore.error" class="text-sm text-status-critical mb-3">
        {{ devicesStore.error }}
      </p>
      <p v-if="queryError" class="text-sm text-status-critical mb-3">{{ queryError }}</p>

      <!-- 載入骨架 -->
      <div v-if="devicesStore.loading" class="space-y-2" aria-busy="true">
        <div v-for="n in 6" :key="n" class="h-12 rounded-md bg-bg-surface animate-pulse"></div>
      </div>

      <!-- 空狀態 -->
      <div v-else-if="isEmpty" class="card p-10 text-center text-ink-secondary">
        <p class="mb-3">尚無設備</p>
        <button class="btn-primary" @click="openCreate">新增第一台設備</button>
      </div>

      <!-- 表格（虛擬捲動） -->
      <div v-else class="card flex flex-col h-full overflow-hidden">
        <div
          class="flex items-center justify-between gap-3 px-4 py-3 border-b border-bg-border text-sm"
        >
          <div class="text-ink-muted">已選取 {{ selectedCount }} 台</div>
          <div class="flex items-center gap-2">
            <button
              class="btn-ghost text-xs"
              :disabled="selectedCount === 0 || batchQuerying"
              @click="clearSelection"
            >
              清除選取
            </button>
            <button
              class="btn-primary text-xs"
              :disabled="selectedCount === 0 || batchQuerying"
              @click="querySelected"
            >
              {{ batchQuerying ? "查詢中…" : `查詢已選 ${selectedCount} 台` }}
            </button>
          </div>
        </div>
        <div
          class="grid grid-cols-[2rem_1.2fr_0.8fr_0.9fr_1.2fr_1fr_auto] gap-3 px-4 py-2 text-xs font-medium text-ink-muted border-b border-bg-border"
        >
          <input
            type="checkbox"
            class="h-4 w-4"
            :checked="allVisibleSelected"
            @change="toggleAllVisibleFromEvent"
          />
          <span>IP 位址</span>
          <span>帳號</span>
          <span>品牌</span>
          <span>備註</span>
          <span>查詢狀態</span>
          <span class="text-right">操作</span>
        </div>
        <RecycleScroller
          class="flex-1"
          :items="devicesStore.devices"
          :item-size="64"
          key-field="id"
          v-slot="{ item }"
        >
          <div
            class="grid grid-cols-[2rem_1.2fr_0.8fr_0.9fr_1.2fr_1fr_auto] gap-3 items-center px-4 h-[64px] border-b border-bg-border/50 hover:bg-bg-raised/60 transition-colors"
          >
            <input
              type="checkbox"
              class="h-4 w-4"
              :checked="isSelected(item.id)"
              @change="toggleSelectFromEvent(item.id, $event)"
            />
            <button
              class="font-mono text-brand-soft text-left hover:underline cursor-pointer"
              @click="router.push(`/devices/${item.id}`)"
            >
              {{ item.ip_address }}
            </button>
            <span class="text-ink-secondary truncate">{{ item.username }}</span>
            <span class="text-ink-secondary">{{ brandLabel(item.brand) }}</span>
            <span class="text-ink-muted truncate">{{ item.note || "—" }}</span>
            <span class="text-xs">
              <span :class="resultClass(queryResultsByDeviceId[item.id])">
                {{ resultLabel(queryResultsByDeviceId[item.id]) }}
              </span>
              <span class="block text-ink-muted">{{
                cpuSummary(queryResultsByDeviceId[item.id])
              }}</span>
            </span>
            <span class="flex justify-end gap-2">
              <button
                class="text-xs text-ink-secondary hover:text-brand cursor-pointer"
                @click="openEdit(item)"
              >
                編輯
              </button>
              <button
                class="text-xs text-ink-secondary hover:text-status-critical cursor-pointer"
                @click="onDelete(item)"
              >
                刪除
              </button>
            </span>
          </div>
        </RecycleScroller>
      </div>
    </div>

    <!-- 新增／編輯 -->
    <ModalDialog
      v-if="showForm"
      :title="editing ? '編輯設備' : '新增設備'"
      @close="showForm = false"
    >
      <DeviceForm :device="editing" @submit="onSubmit" @cancel="showForm = false" />
    </ModalDialog>

    <!-- 匯入 -->
    <ModalDialog v-if="showImport" title="匯入設備清單" @close="showImport = false">
      <ImportDialog @done="onImportDone" @cancel="showImport = false" />
    </ModalDialog>
  </div>
</template>
