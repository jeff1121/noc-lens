<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { useDevicesStore } from "../stores/devices";
import { useGroupsStore } from "../stores/groups";
import { BRANDS, type Device, type NewDevice } from "../api/tauri";
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

const brandLabel = (v: string) => BRANDS.find((b) => b.value === v)?.label ?? v;

onMounted(async () => {
  await Promise.all([groupsStore.fetch(), devicesStore.fetch()]);
});

const isEmpty = computed(() => !devicesStore.loading && devicesStore.devices.length === 0);

async function applyFilter() {
  await devicesStore.fetch(filterGroup.value || undefined);
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
  }
}

async function onImportDone() {
  showImport.value = false;
  await devicesStore.fetch(filterGroup.value || undefined);
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
          class="grid grid-cols-[1.4fr_1fr_1fr_1.4fr_auto] gap-3 px-4 py-2 text-xs font-medium text-ink-muted border-b border-bg-border"
        >
          <span>IP 位址</span>
          <span>帳號</span>
          <span>品牌</span>
          <span>備註</span>
          <span class="text-right">操作</span>
        </div>
        <RecycleScroller
          class="flex-1"
          :items="devicesStore.devices"
          :item-size="52"
          key-field="id"
          v-slot="{ item }"
        >
          <div
            class="grid grid-cols-[1.4fr_1fr_1fr_1.4fr_auto] gap-3 items-center px-4 h-[52px] border-b border-bg-border/50 hover:bg-bg-raised/60 transition-colors"
          >
            <button
              class="font-mono text-brand-soft text-left hover:underline cursor-pointer"
              @click="router.push(`/devices/${item.id}`)"
            >
              {{ item.ip_address }}
            </button>
            <span class="text-ink-secondary truncate">{{ item.username }}</span>
            <span class="text-ink-secondary">{{ brandLabel(item.brand) }}</span>
            <span class="text-ink-muted truncate">{{ item.note || "—" }}</span>
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
