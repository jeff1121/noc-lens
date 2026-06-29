<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { useDevicesStore } from "../stores/devices";
import { useGroupsStore } from "../stores/groups";

const groupsStore = useGroupsStore();
const devicesStore = useDevicesStore();
const newName = ref("");
const error = ref<string | null>(null);
const selectedDeviceId = ref("");
const selectedGroupIds = ref<string[]>([]);
const assignmentLoading = ref(false);
const assignmentSaving = ref(false);
const assignmentMessage = ref<string | null>(null);

onMounted(async () => {
  await Promise.all([groupsStore.fetch(), devicesStore.fetch()]);
  selectedDeviceId.value = devicesStore.devices[0]?.id ?? "";
});

watch(selectedDeviceId, () => {
  loadAssignedGroups();
});

async function add() {
  error.value = null;
  if (!newName.value.trim()) return;
  try {
    await groupsStore.create(newName.value.trim());
    newName.value = "";
  } catch (e: any) {
    error.value = e?.message ?? String(e);
  }
}

async function remove(id: string, name: string) {
  if (confirm(`確定刪除群組「${name}」？`)) {
    await groupsStore.remove(id);
    selectedGroupIds.value = selectedGroupIds.value.filter((groupId) => groupId !== id);
  }
}

async function loadAssignedGroups() {
  assignmentMessage.value = null;
  if (!selectedDeviceId.value) {
    selectedGroupIds.value = [];
    return;
  }
  assignmentLoading.value = true;
  try {
    const assigned = await groupsStore.fetchForDevice(selectedDeviceId.value);
    selectedGroupIds.value = assigned.map((group) => group.id);
  } catch (e: any) {
    error.value = e?.message ?? String(e);
  } finally {
    assignmentLoading.value = false;
  }
}

async function saveAssignment() {
  assignmentMessage.value = null;
  error.value = null;
  if (!selectedDeviceId.value) {
    error.value = "請先選擇設備";
    return;
  }
  assignmentSaving.value = true;
  try {
    await groupsStore.assignDevice(selectedDeviceId.value, selectedGroupIds.value);
    assignmentMessage.value = "群組指派已儲存";
  } catch (e: any) {
    error.value = e?.message ?? String(e);
  } finally {
    assignmentSaving.value = false;
  }
}
</script>

<template>
  <div class="flex flex-col h-full">
    <header class="px-6 py-4 border-b border-bg-border">
      <h2 class="text-xl font-semibold">群組／標籤</h2>
      <p class="text-sm text-ink-muted">以地點或用途分類設備（例：高雄三民區、高雄高中）</p>
    </header>

    <div class="px-6 py-4 space-y-6 max-w-3xl">
      <p v-if="error" class="text-sm text-status-critical">{{ error }}</p>

      <section class="space-y-3">
        <h3 class="font-semibold">群組清單</h3>
        <div class="flex gap-2">
          <input v-model="newName" class="input" placeholder="新增群組名稱…" @keyup.enter="add" />
          <button class="btn-primary" @click="add">新增</button>
        </div>

        <div class="card divide-y divide-bg-border">
          <div
            v-for="g in groupsStore.groups"
            :key="g.id"
            class="flex items-center justify-between px-4 py-3"
          >
            <span>{{ g.name }}</span>
            <button
              class="text-xs text-ink-secondary hover:text-status-critical cursor-pointer"
              @click="remove(g.id, g.name)"
            >
              刪除
            </button>
          </div>
          <p v-if="!groupsStore.groups.length" class="px-4 py-6 text-center text-ink-muted text-sm">
            尚無群組
          </p>
        </div>
      </section>

      <section class="card p-5 space-y-4">
        <div>
          <h3 class="font-semibold">設備指派</h3>
          <p class="text-sm text-ink-muted">選擇設備後勾選此設備所屬群組</p>
        </div>

        <select v-model="selectedDeviceId" class="input w-full">
          <option value="">選擇設備…</option>
          <option v-for="device in devicesStore.devices" :key="device.id" :value="device.id">
            {{ device.ip_address }}（{{ device.username }}）
          </option>
        </select>

        <div v-if="selectedDeviceId" class="space-y-2">
          <p v-if="assignmentLoading" class="text-sm text-ink-muted">讀取指派中…</p>
          <label
            v-for="group in groupsStore.groups"
            :key="group.id"
            class="flex items-center gap-2 text-sm"
          >
            <input
              v-model="selectedGroupIds"
              type="checkbox"
              class="h-4 w-4"
              :value="group.id"
              :disabled="assignmentLoading || assignmentSaving"
            />
            <span>{{ group.name }}</span>
          </label>
          <p v-if="!groupsStore.groups.length" class="text-sm text-ink-muted">尚無可指派群組</p>
        </div>
        <p v-else class="text-sm text-ink-muted">尚無設備可指派</p>

        <div class="flex items-center justify-between">
          <p class="text-sm text-status-normal">{{ assignmentMessage }}</p>
          <button
            class="btn-primary"
            :disabled="!selectedDeviceId || assignmentLoading || assignmentSaving"
            @click="saveAssignment"
          >
            {{ assignmentSaving ? "儲存中…" : "儲存指派" }}
          </button>
        </div>
      </section>
    </div>
  </div>
</template>
