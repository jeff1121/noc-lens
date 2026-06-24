/**
 * Tauri IPC 封裝層（對應 contracts/tauri-commands.md）。
 * 前端統一透過 `api` 呼叫後端指令。
 */
import { invoke } from "@tauri-apps/api/core";

export type Brand = "cisco" | "aruba" | "fortigate_ngfw" | "palo_alto";

export const BRANDS: { value: Brand; label: string }[] = [
  { value: "cisco", label: "Cisco" },
  { value: "aruba", label: "Aruba" },
  { value: "fortigate_ngfw", label: "Fortigate-NGFW" },
  { value: "palo_alto", label: "Palo Alto Networks" },
];

export interface Device {
  id: string;
  ip_address: string;
  username: string;
  note?: string | null;
  brand: Brand;
  created_at: string;
  updated_at: string;
}

export interface NewDevice {
  ip_address: string;
  username: string;
  password: string;
  note?: string | null;
  brand: Brand;
}

export interface UpdateDevice {
  ip_address?: string;
  username?: string;
  password?: string;
  note?: string | null;
  brand?: Brand;
}

export interface Group {
  id: string;
  name: string;
  created_at: string;
}

export interface ImportFailure {
  row: number;
  reason: string;
}
export interface ImportResult {
  success: number;
  failed: ImportFailure[];
}

export interface Settings {
  ai_base_url: string;
  ai_model: string;
  ssh_max_concurrency: number;
  ai_key_set: boolean;
}

export type Metrics = Record<string, any>;

export interface QueryResult {
  device_id: string;
  status: "ok" | "partial" | "failed";
  error_message?: string | null;
  metrics?: Metrics | null;
  snapshot_id?: string | null;
}

export interface StatusSnapshot {
  id: string;
  device_id: string;
  job_run_id?: string | null;
  collected_at: string;
  status: string;
  error_message?: string | null;
  metrics: Metrics;
}

/** 後端錯誤格式（AppError 序列化）。 */
export interface AppError {
  code: string;
  message: string;
}

export const api = {
  // 設備
  deviceList: (groupId?: string) =>
    invoke<Device[]>("device_list", { group_id: groupId ?? null }),
  deviceCreate: (input: NewDevice) => invoke<Device>("device_create", { input }),
  deviceUpdate: (id: string, patch: UpdateDevice) =>
    invoke<Device>("device_update", { id, patch }),
  deviceDelete: (id: string) => invoke<void>("device_delete", { id }),
  deviceImport: (content: string) =>
    invoke<ImportResult>("device_import", { content }),

  // 群組
  groupList: () => invoke<Group[]>("group_list"),
  groupCreate: (name: string) => invoke<Group>("group_create", { name }),
  groupDelete: (id: string) => invoke<void>("group_delete", { id }),
  groupAssign: (deviceId: string, groupIds: string[]) =>
    invoke<void>("group_assign", { device_id: deviceId, group_ids: groupIds }),
  groupsForDevice: (deviceId: string) =>
    invoke<Group[]>("groups_for_device", { device_id: deviceId }),

  // 即時查詢與歷史
  queryDevices: (deviceIds: string[]) =>
    invoke<QueryResult[]>("query_devices", { device_ids: deviceIds }),
  snapshotList: (deviceId: string, limit?: number) =>
    invoke<StatusSnapshot[]>("snapshot_list", { device_id: deviceId, limit: limit ?? null }),

  // 設定
  settingsGet: () => invoke<Settings>("settings_get"),
  settingsSet: (p: {
    ai_base_url?: string;
    ai_model?: string;
    ssh_max_concurrency?: number;
  }) => invoke<void>("settings_set", p),
};
