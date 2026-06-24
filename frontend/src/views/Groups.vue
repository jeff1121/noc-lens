<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useGroupsStore } from "../stores/groups";

const groupsStore = useGroupsStore();
const newName = ref("");
const error = ref<string | null>(null);

onMounted(() => groupsStore.fetch());

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
  }
}
</script>

<template>
  <div class="flex flex-col h-full">
    <header class="px-6 py-4 border-b border-bg-border">
      <h2 class="text-xl font-semibold">群組／標籤</h2>
      <p class="text-sm text-ink-muted">以地點或用途分類設備（例：高雄三民區、高雄高中）</p>
    </header>

    <div class="px-6 py-4 space-y-4 max-w-xl">
      <div class="flex gap-2">
        <input
          v-model="newName"
          class="input"
          placeholder="新增群組名稱…"
          @keyup.enter="add"
        />
        <button class="btn-primary" @click="add">新增</button>
      </div>
      <p v-if="error" class="text-sm text-status-critical">{{ error }}</p>

      <div class="card divide-y divide-bg-border">
        <div
          v-for="g in groupsStore.groups"
          :key="g.id"
          class="flex items-center justify-between px-4 py-3"
        >
          <span>{{ g.name }}</span>
          <button class="text-xs text-ink-secondary hover:text-status-critical cursor-pointer" @click="remove(g.id, g.name)">
            刪除
          </button>
        </div>
        <p v-if="!groupsStore.groups.length" class="px-4 py-6 text-center text-ink-muted text-sm">
          尚無群組
        </p>
      </div>
    </div>
  </div>
</template>
