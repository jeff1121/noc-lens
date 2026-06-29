import type {
  Brand,
  Device,
  ExportReportFormat,
  Group,
  ImportFailure,
  JobRun,
  NewDevice,
  NewScheduledJob,
  QueryResult,
  Report,
  ReportScope,
  ScheduledJob,
  StatusSnapshot,
  UpdateDevice,
  UpdateScheduledJob,
} from "../api/tauri";

interface MockSettings {
  ai_base_url: string;
  ai_model: string;
  ssh_max_concurrency: number;
  ai_key_set: boolean;
}

const supportedBrands = new Set(["cisco", "aruba", "fortigate_ngfw", "palo_alto"]);
let seq = 1;
const devices: Device[] = [];
const groups: Group[] = [];
const groupMembers = new Map<string, Set<string>>();
const snapshots: StatusSnapshot[] = [];
const jobs: ScheduledJob[] = [];
const runs: JobRun[] = [];
const reports: Report[] = [];
const settings: MockSettings = {
  ai_base_url: "",
  ai_model: "",
  ssh_max_concurrency: 10,
  ai_key_set: false,
};

export async function invoke<T>(command: string, args: Record<string, any> = {}): Promise<T> {
  switch (command) {
    case "device_list":
      return clone(listDevices(args.group_id)) as T;
    case "device_create":
      return clone(createDevice(args.input)) as T;
    case "device_update":
      return clone(updateDevice(args.id, args.patch)) as T;
    case "device_delete":
      deleteDevice(args.id);
      return undefined as T;
    case "device_import":
      return clone(importDevices(args.content)) as T;
    case "group_list":
      return clone(groups) as T;
    case "group_create":
      return clone(createGroup(args.name)) as T;
    case "group_delete":
      deleteGroup(args.id);
      return undefined as T;
    case "group_assign":
      assignGroups(args.device_id, args.group_ids ?? []);
      return undefined as T;
    case "groups_for_device":
      return clone(groupsForDevice(args.device_id)) as T;
    case "query_devices":
      return clone(queryDevices(args.device_ids ?? [])) as T;
    case "snapshot_list":
      return clone(listSnapshots(args.device_id, args.from, args.to, args.limit)) as T;
    case "schedule_list":
      return clone(jobs) as T;
    case "schedule_create":
      return clone(createSchedule(args.input)) as T;
    case "schedule_update":
      return clone(updateSchedule(args.id, args.patch)) as T;
    case "schedule_delete":
      deleteSchedule(args.id);
      return undefined as T;
    case "schedule_toggle":
      return clone(toggleSchedule(args.id, args.enabled)) as T;
    case "schedule_run_now":
      return clone(runScheduleNow(args.id)) as T;
    case "job_run_list":
      return clone(runs.filter((run) => run.job_id === args.job_id)) as T;
    case "settings_get":
      return clone(settings) as T;
    case "settings_set":
      updateSettings(args);
      return undefined as T;
    case "settings_set_ai_key":
      settings.ai_key_set = Boolean(args.api_key);
      return undefined as T;
    case "report_list":
      return clone(reports) as T;
    case "report_generate":
      return clone(generateReport(args.scope ?? {}, args.title ?? null)) as T;
    case "report_export":
      return clone(exportReport(args.id, args.out_path, args.format)) as T;
    default:
      throw appError("VALIDATION", `E2E mock 尚未支援指令：${command}`);
  }
}

function listDevices(groupId?: string | null) {
  if (!groupId) return devices;
  const members = groupMembers.get(groupId) ?? new Set<string>();
  return devices.filter((device) => members.has(device.id));
}

function createDevice(input: NewDevice): Device {
  if (!supportedBrands.has(input.brand)) {
    throw appError("UNSUPPORTED_BRAND", `不支援的品牌：${input.brand}`);
  }
  if (devices.some((device) => device.ip_address === input.ip_address)) {
    throw appError("DUPLICATE_IP", `IP 位址重複：${input.ip_address}`);
  }
  const now = nowIso();
  const device: Device = {
    id: nextId("dev"),
    ip_address: input.ip_address,
    username: input.username,
    note: input.note ?? null,
    brand: input.brand,
    created_at: now,
    updated_at: now,
  };
  devices.push(device);
  return device;
}

function updateDevice(id: string, patch: UpdateDevice): Device {
  const device = mustFind(devices, id, "設備");
  Object.assign(device, {
    ip_address: patch.ip_address ?? device.ip_address,
    username: patch.username ?? device.username,
    note: patch.note === undefined ? device.note : patch.note,
    brand: patch.brand ?? device.brand,
    updated_at: nowIso(),
  });
  return device;
}

function deleteDevice(id: string) {
  const index = devices.findIndex((device) => device.id === id);
  if (index === -1) throw appError("NOT_FOUND", `找不到資料：設備 ${id}`);
  devices.splice(index, 1);
  for (const members of groupMembers.values()) members.delete(id);
}

function importDevices(content: string) {
  const lines = content.trim().split(/\r?\n/);
  const headers =
    lines
      .shift()
      ?.split(",")
      .map((part) => part.trim()) ?? [];
  const col = (name: string) => headers.findIndex((header) => header.toLowerCase() === name);
  const failures: ImportFailure[] = [];
  let success = 0;

  for (const [index, line] of lines.entries()) {
    const row = line.split(",").map((part) => part.trim());
    const brand = row[col("brand")] as Brand;
    try {
      const device = createDevice({
        ip_address: row[col("ip_address")],
        username: row[col("username")],
        password: row[col("password")],
        note: row[col("note")] || null,
        brand,
      });
      const rawGroups = row[col("groups")] ?? "";
      const ids = rawGroups
        .split(";")
        .map((name) => name.trim())
        .filter(Boolean)
        .map((name) => findOrCreateGroup(name).id);
      assignGroups(device.id, ids);
      success += 1;
    } catch (error: any) {
      failures.push({ row: index + 1, reason: error?.message ?? String(error) });
    }
  }

  return { success, failed: failures };
}

function createGroup(name: string): Group {
  const trimmed = name.trim();
  if (!trimmed) throw appError("VALIDATION", "群組名稱不可為空");
  if (groups.some((group) => group.name === trimmed)) {
    throw appError("DUPLICATE_NAME", `名稱重複：${trimmed}`);
  }
  const group: Group = { id: nextId("grp"), name: trimmed, created_at: nowIso() };
  groups.push(group);
  groupMembers.set(group.id, new Set());
  return group;
}

function findOrCreateGroup(name: string) {
  return groups.find((group) => group.name === name) ?? createGroup(name);
}

function deleteGroup(id: string) {
  const index = groups.findIndex((group) => group.id === id);
  if (index === -1) throw appError("NOT_FOUND", `找不到資料：群組 ${id}`);
  groups.splice(index, 1);
  groupMembers.delete(id);
}

function assignGroups(deviceId: string, groupIds: string[]) {
  mustFind(devices, deviceId, "設備");
  for (const members of groupMembers.values()) members.delete(deviceId);
  for (const groupId of groupIds) {
    mustFind(groups, groupId, "群組");
    groupMembers.get(groupId)?.add(deviceId);
  }
}

function groupsForDevice(deviceId: string) {
  mustFind(devices, deviceId, "設備");
  return groups.filter((group) => groupMembers.get(group.id)?.has(deviceId));
}

function queryDevices(deviceIds: string[]): QueryResult[] {
  return deviceIds.map((id, index) => {
    mustFind(devices, id, "設備");
    const metrics = {
      cpu: { usage_percent: 25 + index * 5 },
      memory: { usage_percent: 44 + index * 4 },
      interface: { up: 12, down: 0 },
      loading: { samples_percent: [12, 16, 20] },
      traffic: "n/a",
      module: "n/a",
    };
    const snapshotId = addSnapshot(id, null, "ok", null, metrics);
    return {
      device_id: id,
      status: "ok",
      error_message: null,
      metrics,
      snapshot_id: snapshotId,
    };
  });
}

function listSnapshots(
  deviceId: string,
  from?: string | null,
  to?: string | null,
  limit?: number | null,
) {
  mustFind(devices, deviceId, "設備");
  let rows = snapshots
    .filter((snapshot) => snapshot.device_id === deviceId)
    .filter((snapshot) => !from || snapshot.collected_at >= from)
    .filter((snapshot) => !to || snapshot.collected_at <= to)
    .sort((a, b) => b.collected_at.localeCompare(a.collected_at));
  if (limit) rows = rows.slice(0, limit);
  return rows;
}

function createSchedule(input: NewScheduledJob): ScheduledJob {
  const job = normalizeSchedule({
    id: nextId("job"),
    name: input.name,
    target_type: input.target_type,
    target_id: input.target_id,
    schedule_kind: input.schedule_kind,
    interval_minutes: input.interval_minutes ?? null,
    daily_time: input.daily_time ?? null,
    enabled: true,
    created_at: nowIso(),
  });
  jobs.unshift(job);
  return job;
}

function updateSchedule(id: string, patch: UpdateScheduledJob): ScheduledJob {
  const job = mustFind(jobs, id, "排程");
  return Object.assign(
    job,
    normalizeSchedule({
      ...job,
      ...patch,
      interval_minutes:
        patch.interval_minutes === undefined ? job.interval_minutes : patch.interval_minutes,
      daily_time: patch.daily_time === undefined ? job.daily_time : patch.daily_time,
    }),
  );
}

function toggleSchedule(id: string, enabled: boolean): ScheduledJob {
  const job = mustFind(jobs, id, "排程");
  job.enabled = enabled;
  return job;
}

function deleteSchedule(id: string) {
  const index = jobs.findIndex((job) => job.id === id);
  if (index === -1) throw appError("NOT_FOUND", `找不到資料：排程 ${id}`);
  jobs.splice(index, 1);
}

function runScheduleNow(id: string): JobRun {
  const job = mustFind(jobs, id, "排程");
  const targetDevices =
    job.target_type === "device"
      ? [mustFind(devices, job.target_id, "設備")]
      : listDevices(job.target_id);
  const startedAt = nowIso();
  for (const device of targetDevices) {
    addSnapshot(device.id, id, "ok", null, {
      cpu: { usage_percent: 35 },
      memory: { usage_percent: 50 },
      interface: { up: 12, down: 0 },
    });
  }
  const run: JobRun = {
    id: nextId("run"),
    job_id: id,
    started_at: startedAt,
    finished_at: nowIso(),
    total: targetDevices.length,
    success_count: targetDevices.length,
    failure_count: 0,
  };
  runs.unshift(run);
  return run;
}

function normalizeSchedule(job: ScheduledJob): ScheduledJob {
  if (!job.name.trim()) throw appError("VALIDATION", "排程名稱不可為空");
  if (!job.target_id.trim()) throw appError("VALIDATION", "排程目標不可為空");
  if (job.schedule_kind === "interval") {
    if (!job.interval_minutes || job.interval_minutes <= 0) {
      throw appError("VALIDATION", "interval_minutes 須大於 0");
    }
    job.daily_time = null;
  } else {
    if (!job.daily_time) throw appError("VALIDATION", "daily_time 須為 HH:mm");
    job.interval_minutes = null;
  }
  return job;
}

function updateSettings(args: Record<string, any>) {
  settings.ai_base_url = args.ai_base_url ?? settings.ai_base_url;
  settings.ai_model = args.ai_model ?? settings.ai_model;
  settings.ssh_max_concurrency = args.ssh_max_concurrency ?? settings.ssh_max_concurrency;
}

function generateReport(scope: ReportScope, title?: string | null): Report {
  if (!settings.ai_base_url || !settings.ai_key_set) {
    throw appError("AI_CONFIG_MISSING", "請先於設定填入 AI 端點與金鑰");
  }
  const ids = resolveScopeDeviceIds(scope);
  const report: Report = {
    id: nextId("rpt"),
    title: title || "設備健康摘要報告",
    scope_json: JSON.stringify({
      range: { from: scope.from, to: scope.to },
      device_count: ids.length,
    }),
    summary_md: `## 設備健康摘要\n\n整體：${ids.length} 台正常。\n\n### 趨勢觀察\nCPU 與記憶體維持穩定。`,
    generated_at: nowIso(),
    model_endpoint: settings.ai_base_url,
  };
  reports.unshift(report);
  return report;
}

function exportReport(id: string, outPath: string, format: ExportReportFormat) {
  mustFind(reports, id, "報告");
  if (!["md", "pdf"].includes(format)) throw appError("VALIDATION", "format 須為 md 或 pdf");
  return { path: outPath };
}

function resolveScopeDeviceIds(scope: ReportScope) {
  const ids = new Set<string>();
  for (const id of scope.device_ids ?? []) ids.add(id);
  for (const groupId of scope.group_ids ?? []) {
    for (const device of listDevices(groupId)) ids.add(device.id);
  }
  if (!ids.size) for (const device of devices) ids.add(device.id);
  return [...ids];
}

function addSnapshot(
  deviceId: string,
  jobRunId: string | null,
  status: string,
  errorMessage: string | null,
  metrics: Record<string, any>,
) {
  const id = nextId("snap");
  snapshots.unshift({
    id,
    device_id: deviceId,
    job_run_id: jobRunId,
    collected_at: nowIso(),
    status,
    error_message: errorMessage,
    metrics,
  });
  return id;
}

function mustFind<T extends { id: string }>(items: T[], id: string, label: string): T {
  const item = items.find((entry) => entry.id === id);
  if (!item) throw appError("NOT_FOUND", `找不到資料：${label} ${id}`);
  return item;
}

function appError(code: string, message: string) {
  return { code, message };
}

function nextId(prefix: string) {
  return `${prefix}-${seq++}`;
}

function nowIso() {
  return new Date().toISOString();
}

function clone<T>(value: T): T {
  return value === undefined ? value : JSON.parse(JSON.stringify(value));
}
