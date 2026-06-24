<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import { api, BRANDS, type Device, type Group } from "../api/tauri";

const route = useRoute();
const router = useRouter();
const device = ref<Device | null>(null);
const groups = ref<Group[]>([]);
const error = ref<string | null>(null);

const brandLabel = (v: string) => BRANDS.find((b) => b.value === v)?.label ?? v;

onMounted(async () => {
  const id = route.params.id as string;
  try {
    const list = await api.deviceList();
    device.value = list.find((d) => d.id === id) ?? null;
    if (device.value) groups.value = await api.groupsForDevice(id);
  } catch (e: any) {
    error.value = e?.message ?? String(e);
  }
});
</script>

<template>
  <div class="flex flex-col h-full">
    <header class="px-6 py-4 border-b border-bg-border flex items-center gap-3">
      <button class="btn-ghost" @click="router.push('/devices')">← 返回</button>
      <h2 class="text-xl font-semibold font-mono">{{ device?.ip_address ?? "設備" }}</h2>
    </header>

    <div class="px-6 py-4 space-y-4">
      <p v-if="error" class="text-sm text-status-critical">{{ error }}</p>

      <div v-if="device" class="card p-5 grid grid-cols-2 gap-4 max-w-2xl">
        <div><span class="text-ink-muted text-sm">帳號</span><p>{{ device.username }}</p></div>
        <div><span class="text-ink-muted text-sm">品牌</span><p>{{ brandLabel(device.brand) }}</p></div>
        <div class="col-span-2"><span class="text-ink-muted text-sm">備註</span><p>{{ device.note || "—" }}</p></div>
        <div class="col-span-2">
          <span class="text-ink-muted text-sm">群組</span>
          <p class="flex flex-wrap gap-2 mt-1">
            <span v-for="g in groups" :key="g.id" class="rounded-full bg-bg-raised px-2 py-0.5 text-xs">{{ g.name }}</span>
            <span v-if="!groups.length" class="text-ink-muted">—</span>
          </p>
        </div>
      </div>

      <!-- US2：即時 SSH 查詢（尚未實作） -->
      <div class="card p-5 max-w-2xl border-dashed">
        <h3 class="font-semibold mb-1">即時狀態查詢</h3>
        <p class="text-sm text-ink-muted">
          透過 SSH 查詢 CPU／Memory／介面／流量等狀態與趨勢圖（US2，開發中）。
        </p>
      </div>
    </div>
  </div>
</template>
