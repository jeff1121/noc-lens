<script setup lang="ts">
import { onMounted, reactive, ref } from "vue";
import { api, type JobRun, type ScheduledJob } from "../api/tauri";
import { useDevicesStore } from "../stores/devices";
import { useGroupsStore } from "../stores/groups";

const devicesStore = useDevicesStore();
const groupsStore = useGroupsStore();

const jobs = ref<ScheduledJob[]>([]);
const error = ref<string | null>(null);
const runs = reactive<Record<string, JobRun[]>>({});

const form = reactive({
  name: "",
  target_type: "group" as "group" | "device",
  target_id: "",
  schedule_kind: "interval" as "interval" | "daily",
  interval_minutes: 60,
  daily_time: "08:00",
});

async function loadJobs() {
  error.value = null;
  try {
    jobs.value = await api.scheduleList();
    for (const j of jobs.value) runs[j.id] = await api.jobRunList(j.id);
  } catch (e: any) {
    error.value = e?.message ?? String(e);
  }
}

onMounted(async () => {
  await Promise.all([devicesStore.fetch(), groupsStore.fetch(), loadJobs()]);
});

async function create() {
  error.value = null;
  if (!form.name.trim() || !form.target_id) {
    error.value = "請填寫名稱並選擇目標";
    return;
  }
  try {
    await api.scheduleCreate({
      name: form.name.trim(),
      target_type: form.target_type,
      target_id: form.target_id,
      schedule_kind: form.schedule_kind,
      interval_minutes: form.schedule_kind === "interval" ? form.interval_minutes : null,
      daily_time: form.schedule_kind === "daily" ? form.daily_time : null,
    });
    form.name = "";
    await loadJobs();
  } catch (e: any) {
    error.value = e?.message ?? String(e);
  }
}

async function toggle(j: ScheduledJob) {
  await api.scheduleToggle(j.id, !j.enabled);
  await loadJobs();
}
async function remove(j: ScheduledJob) {
  if (confirm(`刪除排程「${j.name}」？`)) {
    await api.scheduleDelete(j.id);
    await loadJobs();
  }
}
async function runNow(j: ScheduledJob) {
  error.value = null;
  try {
    await api.scheduleRunNow(j.id);
    runs[j.id] = await api.jobRunList(j.id);
  } catch (e: any) {
    error.value = e?.message ?? String(e);
  }
}

const targetName = (j: ScheduledJob) => {
  if (j.target_type === "group")
    return groupsStore.groups.find((g) => g.id === j.target_id)?.name ?? "(群組)";
  return devicesStore.devices.find((d) => d.id === j.target_id)?.ip_address ?? "(設備)";
};
const scheduleDesc = (j: ScheduledJob) =>
  j.schedule_kind === "interval" ? `每 ${j.interval_minutes} 分鐘` : `每日 ${j.daily_time}`;
</script>

<template>
  <div class="flex flex-col h-full">
    <header class="px-6 py-4 border-b border-bg-border">
      <h2 class="text-xl font-semibold">排程監控</h2>
      <p class="text-sm text-ink-muted">對設備／群組設定自動查詢並保存歷史</p>
    </header>

    <div class="px-6 py-4 space-y-6">
      <p v-if="error" class="text-sm text-status-critical">{{ error }}</p>

      <!-- 建立排程 -->
      <section class="card p-5 space-y-3 max-w-3xl">
        <h3 class="font-semibold">新增排程</h3>
        <div class="grid grid-cols-2 gap-3">
          <input v-model="form.name" class="input" placeholder="排程名稱（如：每日巡檢）" />
          <select v-model="form.target_type" class="input">
            <option value="group">群組</option>
            <option value="device">單一設備</option>
          </select>
          <select v-model="form.target_id" class="input">
            <option value="">選擇目標…</option>
            <template v-if="form.target_type === 'group'">
              <option v-for="g in groupsStore.groups" :key="g.id" :value="g.id">{{ g.name }}</option>
            </template>
            <template v-else>
              <option v-for="d in devicesStore.devices" :key="d.id" :value="d.id">{{ d.ip_address }}</option>
            </template>
          </select>
          <select v-model="form.schedule_kind" class="input">
            <option value="interval">固定間隔</option>
            <option value="daily">每日固定時間</option>
          </select>
          <input
            v-if="form.schedule_kind === 'interval'"
            v-model.number="form.interval_minutes"
            type="number"
            min="1"
            class="input"
            placeholder="間隔（分鐘）"
          />
          <input v-else v-model="form.daily_time" type="time" class="input" />
        </div>
        <div class="flex justify-end">
          <button class="btn-primary" @click="create">新增排程</button>
        </div>
      </section>

      <!-- 排程清單 -->
      <section class="space-y-3 max-w-3xl">
        <div v-for="j in jobs" :key="j.id" class="card p-4">
          <div class="flex items-center justify-between">
            <div>
              <p class="font-medium">
                {{ j.name }}
                <span class="text-xs ml-2" :class="j.enabled ? 'text-status-normal' : 'text-ink-muted'">
                  {{ j.enabled ? "啟用中" : "已停用" }}
                </span>
              </p>
              <p class="text-sm text-ink-muted">{{ targetName(j) }} ・ {{ scheduleDesc(j) }}</p>
            </div>
            <div class="flex gap-2">
              <button class="btn-ghost text-xs" @click="runNow(j)">立即執行</button>
              <button class="btn-ghost text-xs" @click="toggle(j)">{{ j.enabled ? "停用" : "啟用" }}</button>
              <button class="text-xs text-ink-secondary hover:text-status-critical cursor-pointer" @click="remove(j)">刪除</button>
            </div>
          </div>
          <div v-if="runs[j.id]?.length" class="mt-3 text-xs text-ink-secondary space-y-1">
            <p class="text-ink-muted">最近執行：</p>
            <p v-for="r in runs[j.id].slice(0, 3)" :key="r.id">
              {{ r.started_at.slice(0, 16).replace("T", " ") }} ・
              成功 <span class="text-status-normal">{{ r.success_count }}</span> /
              失敗 <span class="text-status-critical">{{ r.failure_count }}</span>（共 {{ r.total }}）
            </p>
          </div>
        </div>
        <p v-if="!jobs.length" class="card p-8 text-center text-ink-muted text-sm">尚無排程</p>
      </section>
    </div>
  </div>
</template>
