<script setup lang="ts">
import { ref } from "vue";
import { api, type ImportResult } from "../api/tauri";

const emit = defineEmits<{ (e: "done"): void; (e: "cancel"): void }>();

const result = ref<ImportResult | null>(null);
const error = ref<string | null>(null);
const importing = ref(false);
const fileName = ref<string>("");

async function onFile(e: Event) {
  const input = e.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;
  fileName.value = file.name;
  error.value = null;
  result.value = null;
  importing.value = true;
  try {
    const content = await file.text();
    result.value = await api.deviceImport(content);
  } catch (err: any) {
    error.value = err?.message ?? String(err);
  } finally {
    importing.value = false;
  }
}
</script>

<template>
  <div class="space-y-4">
    <p class="text-sm text-ink-secondary">
      請選擇 CSV 檔（表頭需含
      <code class="font-mono text-ink-primary">ip_address,username,password,note,brand,groups</code
      >）。
    </p>

    <label class="btn-ghost cursor-pointer w-full">
      <input type="file" accept=".csv,text/csv" class="hidden" @change="onFile" />
      {{ fileName || "選擇 CSV 檔案…" }}
    </label>

    <p v-if="importing" class="text-sm text-ink-secondary">匯入中…</p>
    <p v-if="error" class="text-sm text-status-critical">{{ error }}</p>

    <div v-if="result" class="card p-4 space-y-2">
      <p class="text-sm">
        成功：<span class="text-status-normal font-semibold">{{ result.success }}</span> 筆，
        失敗：<span class="text-status-critical font-semibold">{{ result.failed.length }}</span> 筆
      </p>
      <ul
        v-if="result.failed.length"
        class="text-xs text-ink-secondary space-y-1 max-h-40 overflow-auto"
      >
        <li v-for="f in result.failed" :key="f.row">第 {{ f.row }} 列：{{ f.reason }}</li>
      </ul>
    </div>

    <div class="flex justify-end gap-2">
      <button class="btn-ghost" @click="emit('cancel')">關閉</button>
      <button v-if="result" class="btn-primary" @click="emit('done')">完成</button>
    </div>
  </div>
</template>
