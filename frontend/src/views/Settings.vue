<script setup lang="ts">
import { onMounted, reactive, ref } from "vue";
import { api } from "../api/tauri";

const form = reactive({ ai_base_url: "", ai_model: "", ssh_max_concurrency: 10 });
const aiKeySet = ref(false);
const apiKey = ref("");
const saved = ref(false);
const keySaved = ref(false);
const error = ref<string | null>(null);

onMounted(async () => {
  try {
    const s = await api.settingsGet();
    form.ai_base_url = s.ai_base_url;
    form.ai_model = s.ai_model;
    form.ssh_max_concurrency = s.ssh_max_concurrency;
    aiKeySet.value = s.ai_key_set;
  } catch (e: any) {
    error.value = e?.message ?? String(e);
  }
});

async function save() {
  error.value = null;
  saved.value = false;
  try {
    await api.settingsSet({
      ai_base_url: form.ai_base_url,
      ai_model: form.ai_model,
      ssh_max_concurrency: form.ssh_max_concurrency,
    });
    saved.value = true;
  } catch (e: any) {
    error.value = e?.message ?? String(e);
  }
}

async function saveKey() {
  error.value = null;
  keySaved.value = false;
  if (!apiKey.value) return;
  try {
    await api.settingsSetAiKey(apiKey.value);
    apiKey.value = "";
    aiKeySet.value = true;
    keySaved.value = true;
  } catch (e: any) {
    error.value = e?.message ?? String(e);
  }
}
</script>

<template>
  <div class="flex flex-col h-full">
    <header class="px-6 py-4 border-b border-bg-border">
      <h2 class="text-xl font-semibold">設定</h2>
    </header>

    <div class="px-6 py-4 space-y-6 max-w-xl">
      <section class="card p-5 space-y-4">
        <h3 class="font-semibold">SSH</h3>
        <div>
          <label class="block text-sm text-ink-secondary mb-1" for="conc">同時連線上限</label>
          <input
            id="conc"
            v-model.number="form.ssh_max_concurrency"
            type="number"
            min="1"
            class="input w-32"
          />
        </div>
      </section>

      <section class="card p-5 space-y-4">
        <h3 class="font-semibold">AI（OpenAI 相容端點）</h3>
        <p class="text-xs text-status-warning">
          ⚠ 使用雲端端點時，設備狀態資料將離開本機；如需資料不離開本機，可指向本地服務（如
          Ollama／LM Studio）。
        </p>
        <div>
          <label class="block text-sm text-ink-secondary mb-1" for="url">Base URL</label>
          <input
            id="url"
            v-model="form.ai_base_url"
            class="input"
            placeholder="https://api.openai.com/v1"
          />
        </div>
        <div>
          <label class="block text-sm text-ink-secondary mb-1" for="model">模型</label>
          <input id="model" v-model="form.ai_model" class="input" placeholder="gpt-4o-mini" />
        </div>
        <div>
          <label class="block text-sm text-ink-secondary mb-1" for="key">
            API 金鑰（存於 OS 金鑰庫）
          </label>
          <div class="flex gap-2">
            <input
              id="key"
              v-model="apiKey"
              type="password"
              class="input"
              :placeholder="aiKeySet ? '已設定（輸入可覆寫）' : '輸入 API 金鑰'"
            />
            <button class="btn-ghost" @click="saveKey">儲存金鑰</button>
          </div>
          <p class="text-xs mt-1" :class="aiKeySet ? 'text-status-normal' : 'text-ink-muted'">
            狀態：{{ aiKeySet ? "已設定" : "未設定"
            }}<span v-if="keySaved" class="text-status-normal"> ・ 已儲存</span>
          </p>
        </div>
      </section>

      <div class="flex items-center gap-3">
        <button class="btn-primary" @click="save">儲存設定</button>
        <span v-if="saved" class="text-sm text-status-normal">已儲存</span>
        <span v-if="error" class="text-sm text-status-critical">{{ error }}</span>
      </div>
    </div>
  </div>
</template>
