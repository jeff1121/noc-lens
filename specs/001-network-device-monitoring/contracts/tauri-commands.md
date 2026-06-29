# 契約：Tauri IPC 指令（前端 ↔ Rust 後端）

**功能**：[../spec.md](../spec.md) ｜ **日期**：2026-06-26

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
| `device_import` | `{ content }` | `{ success: number, failed: { row, reason }[] }` | `FILE_ERROR`, `PARSE_ERROR` |

- `device_import` 由前端讀取 CSV 檔案後傳入文字內容（欄位至少 `ip_address,username,password,brand`，可含 `note`、`groups`）；逐列回報成功與失敗原因（FR-002）。

## 群組／標籤（Group）— 對應 US1 / FR-003,004

| 指令 | 輸入 | 輸出 | 主要錯誤碼 |
|------|------|------|-----------|
| `group_list` | `{}` | `Group[]` | `DB_ERROR` |
| `group_create` | `{ name }` | `Group` | `VALIDATION`, `DUPLICATE_NAME` |
| `group_delete` | `{ id }` | `void` | `NOT_FOUND` |
| `group_assign` | `{ device_id, group_ids: string[] }` | `void` | `NOT_FOUND` |
| `groups_for_device` | `{ device_id }` | `Group[]` | `NOT_FOUND` |

- `group_assign` 為覆寫式指派；呼叫端必須傳入該設備完整群組 id 清單。

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
| `schedule_run_now` | `{ id }` | `JobRun` | `NOT_FOUND`, `DB_ERROR` |
| `job_run_list` | `{ job_id }` | `JobRun[]` | `NOT_FOUND` |

- `schedule_update` 的 `interval_minutes` 與 `daily_time` 可傳 `null` 清空；固定間隔排程會清空 `daily_time`，每日排程會清空 `interval_minutes`。

## 歷史資料（History）— 對應 US3 / FR-016

| 指令 | 輸入 | 輸出 | 主要錯誤碼 |
|------|------|------|-----------|
| `snapshot_list` | `{ device_id, from?, to?, limit? }` | `StatusSnapshot[]`（依 collected_at 新到舊排序） | `NOT_FOUND`, `VALIDATION` |

- 未提供 `from/to` 時預設回傳最近 50 筆；提供期間且未提供 `limit` 時回傳整個期間資料。
- `from/to` 必須為 RFC3339 字串，後端會正規化為 UTC 後查詢。

## AI 報告（Report）— 對應 US4 / FR-018~022

| 指令 | 輸入 | 輸出 | 主要錯誤碼 |
|------|------|------|-----------|
| `report_generate` | `{ scope: { device_ids?, group_ids?, from?, to? }, title? }` | `Report` | `AI_UNAVAILABLE`, `AI_CONFIG_MISSING` |
| `report_list` | `{}` | `Report[]` | `DB_ERROR` |
| `report_export` | `{ id, out_path, format: "md"|"pdf" }` | `{ path }` | `NOT_FOUND`, `FILE_ERROR` |

- `report_generate` 會依 `scope.from/to` 查詢期間快照，並在送交 AI 前彙整 latest、trend、snapshot_count 與精簡 snapshots。
- `report_export` 由 Tauri 後端寫入指定路徑；`md` 保留完整 Markdown，`pdf` 產生基本 PDF 檔供離線保存。
- `AI_UNAVAILABLE`：AI 端點不可用時明確提示，且不影響既有狀態資料（FR-021）。

## 設定（Settings）— 對應 FR-027 / 安全

| 指令 | 輸入 | 輸出 | 主要錯誤碼 |
|------|------|------|-----------|
| `settings_get` | `{}` | `{ ai_base_url, ai_model, ssh_max_concurrency, ai_key_set: boolean }` | — |
| `settings_set` | `{ ai_base_url?, ai_model?, ssh_max_concurrency? }` | `void` | `VALIDATION` |
| `settings_set_ai_key` | `{ api_key }` | `void` | `KEYRING_ERROR` |

- AI 金鑰寫入 OS 金鑰庫；`settings_get` 僅回傳 `ai_key_set` 布林，不回傳金鑰本身（FR-024）。
