<script setup lang="ts">
import { reactive, ref } from "vue";
import { BRANDS, type Brand, type Device, type NewDevice } from "../api/tauri";

const props = defineProps<{ device?: Device | null }>();
const emit = defineEmits<{
  (e: "submit", payload: NewDevice): void;
  (e: "cancel"): void;
}>();

const form = reactive({
  ip_address: props.device?.ip_address ?? "",
  username: props.device?.username ?? "",
  password: "",
  note: props.device?.note ?? "",
  brand: (props.device?.brand ?? "cisco") as Brand,
});

const error = ref<string | null>(null);

function submit() {
  error.value = null;
  if (!form.ip_address.trim()) {
    error.value = "請輸入 IP 位址";
    return;
  }
  if (!form.username.trim()) {
    error.value = "請輸入登入帳號";
    return;
  }
  if (!props.device && !form.password) {
    error.value = "請輸入登入密碼";
    return;
  }
  emit("submit", {
    ip_address: form.ip_address.trim(),
    username: form.username.trim(),
    password: form.password,
    note: form.note?.trim() || null,
    brand: form.brand,
  });
}
</script>

<template>
  <form class="space-y-4" @submit.prevent="submit">
    <div>
      <label class="block text-sm text-ink-secondary mb-1" for="ip">IP 位址</label>
      <input id="ip" v-model="form.ip_address" class="input" placeholder="10.0.0.1" />
    </div>
    <div>
      <label class="block text-sm text-ink-secondary mb-1" for="user">登入帳號</label>
      <input id="user" v-model="form.username" class="input" placeholder="admin" />
    </div>
    <div>
      <label class="block text-sm text-ink-secondary mb-1" for="pw">
        登入密碼<span v-if="device" class="text-ink-muted">（留空表示不變更）</span>
      </label>
      <input id="pw" v-model="form.password" type="password" class="input" placeholder="••••••" />
    </div>
    <div>
      <label class="block text-sm text-ink-secondary mb-1" for="brand">品牌</label>
      <select id="brand" v-model="form.brand" class="input">
        <option v-for="b in BRANDS" :key="b.value" :value="b.value">{{ b.label }}</option>
      </select>
    </div>
    <div>
      <label class="block text-sm text-ink-secondary mb-1" for="note">備註</label>
      <input id="note" v-model="form.note" class="input" placeholder="例：核心交換器" />
    </div>

    <p v-if="error" class="text-sm text-status-critical">{{ error }}</p>

    <div class="flex justify-end gap-2 pt-2">
      <button type="button" class="btn-ghost" @click="emit('cancel')">取消</button>
      <button type="submit" class="btn-primary">儲存</button>
    </div>
  </form>
</template>
