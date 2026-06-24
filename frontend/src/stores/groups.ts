import { defineStore } from "pinia";
import { ref } from "vue";
import { api, type Group } from "../api/tauri";

export const useGroupsStore = defineStore("groups", () => {
  const groups = ref<Group[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function fetch() {
    loading.value = true;
    error.value = null;
    try {
      groups.value = await api.groupList();
    } catch (e: any) {
      error.value = e?.message ?? String(e);
    } finally {
      loading.value = false;
    }
  }

  async function create(name: string) {
    await api.groupCreate(name);
    await fetch();
  }

  async function remove(id: string) {
    await api.groupDelete(id);
    await fetch();
  }

  return { groups, loading, error, fetch, create, remove };
});
