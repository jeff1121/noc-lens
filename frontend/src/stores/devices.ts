import { defineStore } from "pinia";
import { ref } from "vue";
import { api, type Device, type NewDevice, type UpdateDevice } from "../api/tauri";

export const useDevicesStore = defineStore("devices", () => {
  const devices = ref<Device[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const activeGroupId = ref<string | null>(null);

  async function fetch(groupId?: string) {
    loading.value = true;
    error.value = null;
    activeGroupId.value = groupId ?? null;
    try {
      devices.value = await api.deviceList(groupId);
    } catch (e: any) {
      error.value = e?.message ?? String(e);
    } finally {
      loading.value = false;
    }
  }

  async function create(input: NewDevice) {
    await api.deviceCreate(input);
    await fetch(activeGroupId.value ?? undefined);
  }

  async function update(id: string, patch: UpdateDevice) {
    await api.deviceUpdate(id, patch);
    await fetch(activeGroupId.value ?? undefined);
  }

  async function remove(id: string) {
    await api.deviceDelete(id);
    await fetch(activeGroupId.value ?? undefined);
  }

  return { devices, loading, error, activeGroupId, fetch, create, update, remove };
});
