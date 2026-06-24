# 契約：Tauri IPC 指令（前端 ↔ Rust 後端）

**功能**：[../spec.md](../spec.md) ｜ **日期**：2026-06-24

前端透過 Tauri `invoke(<command>, <args>)` 呼叫後端。下列為對外介面契約：指令名稱、輸入、輸出與錯誤。所有指令回傳 `Result<T, AppError>`；`AppError` 含 `code`（字串列舉）與 `message`（zh-TW）。

> 慣例：時間為 ISO8601 字串；id 為 UUID 字串。密碼僅於「寫入」時傳入明文，**永不**於任何讀取指令回傳。

---

## 設備（Device）— 對應 US1 / FR-001~005

| 指令 | 輸入 | 輸出 | 主要錯誤碼 |
|------|------|------|-----------|
| `device_list` | `{ group_id?: string }` | `Device[]`（不含密碼） | `DB_ERROR` |
| `device_create` | `{ ip_address, username, password, note?, brand }` | `Device` | `VALIDATION`, `DUPLICATE_IP`, `UNSUPPORTED_BRAND` |
| `device_update` | `{ id, ...patch }` | `Device` | `NOT_FOUND`, `VALIDATION` |
| `device_delete` | `{ id }` | `void` | `NOT_FOUND` |
| `device_import` | `{ csv_path }` | `{ success: number, failed: { row, reason }[] }` | `FILE_ERROR`, `PARSE_ERROR` |

- `device_import` 解析 CSV（欄位至少 `ip_address,username,password,note`，可含 `brand`、`groups`）；逐列回報成功與失敗原因（FR-002）。

## 群組／標籤（Group）— 對應 US1 / FR-003,004

| 指令 | 輸入 | 輸出 | 主要錯誤碼 |
|------|------|------|-----------|
| `group_list` | `{}` | `Group[]` | `DB_ERROR` |
| `group_create` | `{ name }` | `Group` | `VALIDATION`, `DUPLICATE_NAME` |
| `group_delete` | `{ id }` | `void` | `NOT_FOUND` |
| `group_assign` | `{ device_id, group_ids: string[] }` | `void` | `NOT_FOUND` |

## 即時查詢（SSH Query）— 對應 US2 / FR-006~012

| 指令 | 輸入 | 輸出 | 主要錯誤碼 |
|------|------|------|-----------|
| `query_devices` | `{ device_ids: string[] }` | `QueryResult[]` | `DB_ERROR` |

- `QueryResult`：`{ device_id, status: "ok"|"partial"|"failed", error_message?, metrics?, snapshot_id }`。
- 逐台回報；單台 `failed`（如 `SSH_AUTH_FAILED`、`SSH_TIMEOUT`、`SSH_UNREACHABLE`）不影響其他台（FR-011）。
- 不適用指標於 `metrics` 內以 `"n/a"` 標示（FR-010）。
- 結果同時寫入 StatusSnapshot（job_run_id 為空）。

## 排程（Schedule）— 對應 US3 / FR-013~017

| 指令 | 輸入 | 輸出 | 主要錯誤碼 |
|------|------|------|-----------|
| `schedule_list` | `{}` | `ScheduledJob[]` | `DB_ERROR` |
| `schedule_create` | `{ name, target_type, target_id, schedule_kind, interval_minutes?, daily_time? }` | `ScheduledJob` | `VALIDATION` |
| `schedule_update` | `{ id, ...patch }` | `ScheduledJob` | `NOT_FOUND`, `VALIDATION` |
| `schedule_delete` | `{ id }` | `void` | `NOT_FOUND` |
| `schedule_toggle` | `{ id, enabled }` | `ScheduledJob` | `NOT_FOUND` |
| `job_run_list` | `{ job_id }` | `JobRun[]` | `NOT_FOUND` |

## 歷史資料（History）— 對應 US3 / FR-016

| 指令 | 輸入 | 輸出 | 主要錯誤碼 |
|------|------|------|-----------|
| `snapshot_list` | `{ device_id, from?, to? }` | `StatusSnapshot[]`（依 collected_at 排序） | `NOT_FOUND` |

## AI 報告（Report）— 對應 US4 / FR-018~022

| 指令 | 輸入 | 輸出 | 主要錯誤碼 |
|------|------|------|-----------|
| `report_generate` | `{ scope: { device_ids?, group_ids?, from?, to? }, title? }` | `Report` | `AI_UNAVAILABLE`, `AI_CONFIG_MISSING` |
| `report_list` | `{}` | `Report[]` | `DB_ERROR` |
| `report_export` | `{ id, out_path, format: "md"|"pdf" }` | `{ path }` | `NOT_FOUND`, `FILE_ERROR` |

- `AI_UNAVAILABLE`：AI 端點不可用時明確提示，且不影響既有狀態資料（FR-021）。

## 設定（Settings）— 對應 FR-027 / 安全

| 指令 | 輸入 | 輸出 | 主要錯誤碼 |
|------|------|------|-----------|
| `settings_get` | `{}` | `{ ai_base_url, ai_model, ssh_max_concurrency, ai_key_set: boolean }` | — |
| `settings_set` | `{ ai_base_url?, ai_model?, ssh_max_concurrency? }` | `void` | `VALIDATION` |
| `settings_set_ai_key` | `{ api_key }` | `void` | `KEYRING_ERROR` |

- AI 金鑰寫入 OS 金鑰庫；`settings_get` 僅回傳 `ai_key_set` 布林，不回傳金鑰本身（FR-024）。
